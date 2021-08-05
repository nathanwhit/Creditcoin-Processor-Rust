#![cfg(feature = "integration-testing")]
use anyhow::Result;
use bollard::container::{
    Config, CreateContainerOptions, InspectContainerOptions, NetworkingConfig,
    RemoveContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::models::{EndpointSettings, HostConfig, PortBinding};
use bollard::network::CreateNetworkOptions;
use bollard::Docker;
use ccprocessor_rust::handler::types::SigHash;
use ccprocessor_rust::handler::utils::{sha512_id, to_hex_string};
use ccprocessor_rust::handler::{CCTransaction, CCTransactionHandler, CollectCoins, SendFunds};
use ccprocessor_rust::{test_utils::*, DEFAULT_ENDPOINT, DEFAULT_GATEWAY};
use derive_more::{Deref, DerefMut};
use futures_lite::{Future, Stream, StreamExt};
use itertools::Itertools;
use maplit::hashmap;
use openssl::sha::sha512;
use protobuf::{Message, RepeatedField};
use rand::{thread_rng, Rng};
use sawtooth_sdk::messages::batch::{Batch, BatchHeader, BatchList};
use sawtooth_sdk::messages::transaction::Transaction;
use sawtooth_sdk::processor::TransactionProcessor;
use sawtooth_sdk::{
    messages::transaction::TransactionHeader,
    signing::{create_context, secp256k1::Secp256k1PrivateKey, Signer},
};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::panic::{catch_unwind, panic_any};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use std::{fs::File, io::Read, sync::Once};
use tokio::runtime::Runtime;
use ureq::Response;
// use test_env_log::test;

trait Ext {
    fn into_strings(self) -> Vec<String>;
}

impl<T> Ext for Vec<T>
where
    T: Into<String>,
{
    fn into_strings(self) -> Vec<String> {
        self.into_iter().map(Into::into).collect()
    }
}

pub async fn ensure_image_present(docker: &Docker, image: &str, tag: &str) -> Result<()> {
    let image_name = format!("{}:{}", image, tag);
    match docker.inspect_image(&image_name).await {
        Ok(image) => {
            println!("got image");
        }
        Err(e) => {
            while let Some(Ok(response)) = docker
                .create_image(
                    Some(CreateImageOptions {
                        from_image: image,
                        tag,
                        ..Default::default()
                    }),
                    None,
                    None,
                )
                .next()
                .await
            {
                println!("Pulling image");
            }
        }
    }
    Ok(())
}

pub fn random_name(base: &str) -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::iter;
    let mut rng = thread_rng();
    let chars: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(10)
        .collect();
    format!("{}_{}", base, chars)
}

#[derive(Deref, DerefMut)]
pub struct DockerClient {
    #[deref]
    #[deref_mut]
    pub client: Docker,
    cleanup_containers: Vec<String>,
    cleanup_networks: Vec<String>,
    // validator_
}

impl DockerClient {
    pub fn new() -> Self {
        Self {
            client: Docker::connect_with_local_defaults().unwrap(),
            cleanup_containers: vec![],
            cleanup_networks: vec![],
        }
    }
    pub fn run<T: Future<Output = ()> + Send + 'static>(self, func: impl FnOnce(&Self) -> T) {
        let rt = match Runtime::new() {
            Ok(r) => r,
            Err(e) => panic!("Failed to start tokio runtime: {}", e),
        };

        match rt.block_on(self.run_internal(func)) {
            Ok(_) => log::info!("Success!"),
            Err(e) => {
                log::error!("Sadboi hours: {}", e);
                panic!("{}", e);
            }
        }
    }

    async fn run_internal<T: Future<Output = ()> + Send + 'static>(
        mut self,
        test: impl FnOnce(&Self) -> T,
    ) -> Result<()> {
        setup(&mut self).await?;
        let result: Result<(), Option<Box<dyn std::any::Any + Send + 'static>>> =
            match tokio::spawn(test(&self)).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    log::error!("Test body failed (panicked: {})", e.is_panic());
                    Err(e.try_into_panic().ok())
                }
            };

        self.cleanup();

        if let Err(Some(panic)) = result {
            std::panic::resume_unwind(panic);
        }
        Ok(())
    }

    fn cleanup(&mut self) {
        for container in self.cleanup_containers.drain(..) {
            pollster::block_on(self.client.remove_container(
                &container,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            ))
            .unwrap();
        }
        for net in self.cleanup_networks.drain(..) {
            pollster::block_on(self.client.remove_network(&net)).unwrap();
        }
    }

    async fn get_container_ip(&self, network: &str, container: &str) -> Result<Ipv4Addr> {
        let details = self.inspect_network::<&str>(network, None).await?;
        log::warn!("Details = {:?}", details);
        if let Some(containers) = details.containers {
            if let Some(info) = containers.get(container) {
                if let Some(ip) = &info.ipv4_address {
                    let ip_addr = ip.split('/').next().unwrap().parse()?;
                    return Ok(ip_addr);
                }
            }
        }
        Ok(Ipv4Addr::UNSPECIFIED)
    }
}

impl Drop for DockerClient {
    fn drop(&mut self) {
        self.cleanup();
    }
}

pub async fn setup(docker: &mut DockerClient) -> Result<()> {
    let network_name = random_name("cc-net");
    let network = docker
        .create_network(CreateNetworkOptions::<&str> {
            name: &network_name,
            internal: false,
            driver: "bridge",
            ..Default::default()
        })
        .await?;
    let network_id = network.id.unwrap();

    docker.cleanup_networks.push(network_id.clone());

    ensure_image_present(&docker, "hyperledger/sawtooth-settings-tp", "1.0").await?;
    ensure_image_present(&docker, "gluwa/creditcoin-validator", "1.7.1").await?;
    ensure_image_present(&docker, "gluwa/sawtooth-rest-api", "latest").await?;

    let name = random_name("creditcoin-validator");

    let validator = docker
        .create_container(
            Some(CreateContainerOptions { name }),
            Config {
                exposed_ports: Some(hashmap! {
                    "4004/tcp" => hashmap! {},
                    "8800/tcp" => hashmap! {}
                }),
                cmd: Some(vec![
                    "bash", "-c",
                    r#"if [[ ! -f /etc/sawtooth/keys/validator.pub ]]; then echo "First run"; sawadm keygen; sawset genesis --key /etc/sawtooth/keys/validator.priv; sawset proposal create --key /etc/sawtooth/keys/validator.priv sawtooth-consensus.algorithm.name=PoW sawtooth.consensus.algorithm.version=0.1 -o config.batch; sawadm genesis config-genesis.batch config.batch; fi; sawtooth-validator -vv --endpoint tcp://173.66.57.160:8800 --bind component:tcp://0.0.0.0:4004 --bind network:tcp://0.0.0.0:8800"#,
                ]),
                image: Some("gluwa/creditcoin-validator:1.7.1"),
                host_config: Some(HostConfig {
                    network_mode: Some(network_id.clone()),
                    port_bindings: Some(hashmap! {
                        "4004/tcp".into() => Some(vec![PortBinding {
                            host_port: Some("4004".into()),
                            host_ip: None,
                        }]),
                        "8800/tcp".into() => Some(vec![PortBinding {
                            host_port: Some("8800".into()),
                            host_ip: None,
                        }])
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;
    let validator_id = validator.id;

    docker.cleanup_containers.push(validator_id.clone());

    docker.start_container::<&str>(&validator_id, None).await?;

    let validator_ip = docker.get_container_ip(&network_id, &validator_id).await?;

    let name = random_name("settings-tp");
    let settings = docker
        .create_container(
            Some(CreateContainerOptions { name }),
            Config {
                exposed_ports: Some(hashmap! {
                    "4004/tcp" => hashmap! {}
                }),
                cmd: Some(vec![
                    "bash",
                    "-c",
                    &format!("settings-tp -vv -C tcp://{}:4004", &validator_ip),
                ]),
                image: Some("hyperledger/sawtooth-settings-tp:1.0"),
                host_config: Some(HostConfig {
                    network_mode: Some(network_id.clone()),

                    ..Default::default()
                }),

                // networking_config: Some(NetworkingConfig {
                //     endpoints_config: Some(hashmap! {
                //         &network_id => EndpointSettings {

                //         }
                //     }),
                // }),
                ..Default::default()
            },
        )
        .await?;

    log::warn!("Settings = {:?}", settings);
    let settings_id = settings.id;

    docker.cleanup_containers.push(settings_id.clone());

    let name = random_name("rest-api");
    let rest_api = docker
        .create_container(
            Some(CreateContainerOptions { name }),
            Config {
                exposed_ports: Some(hashmap! {
                    "8008" => hashmap! {}
                }),
                cmd: Some(vec![
                    "bash",
                    "-c",
                    &format!(
                        "sawtooth-rest-api -vv -C tcp://{}:4004 --bind 0.0.0.0:8008",
                        &validator_ip
                    ),
                ]),
                image: Some("gluwa/sawtooth-rest-api:latest"),
                host_config: Some(HostConfig {
                    network_mode: Some(network_id.clone()),
                    port_bindings: Some(hashmap! {
                        "8008".into() => Some(vec![PortBinding {
                            host_port: Some("8008".into()),
                            host_ip: Some("0.0.0.0".into()),
                        }]),
                    }),
                    ..Default::default()
                }),

                // networking_config: Some(NetworkingConfig {
                //     endpoints_config: Some(hashmap! {
                //         &network_id => EndpointSettings {

                //         }
                //     }),
                // }),
                ..Default::default()
            },
        )
        .await?;

    let rest_api_id = rest_api.id;

    docker.cleanup_containers.push(rest_api_id.clone());

    docker.start_container::<&str>(&rest_api_id, None).await?;
    docker.start_container::<&str>(&settings_id, None).await?;
    // let settings =
    // let docker

    // tokio::time::sleep(Duration::from_secs(30)).await;

    // panic!();
    Ok(())
}

fn signer_from_file(profile: &str) -> Signer {
    let mut private_key_file_name = dirs::home_dir().unwrap();
    private_key_file_name.push(format!(".sawtooth/keys/{}.priv", profile));

    // TODO: read keys via command line args
    let mut private_key_file = File::open(private_key_file_name).unwrap();
    let mut private_key_hex = String::new();
    private_key_file
        .read_to_string(&mut private_key_hex)
        .unwrap();

    let private_key = Secp256k1PrivateKey::from_hex(private_key_hex.trim()).unwrap();
    let signing_context = create_context("secp256k1").unwrap();
    Signer::new_boxed(signing_context, Box::new(private_key))
}

#[derive(Deserialize, Clone, Debug)]
struct BatchSendResponse {
    link: String,
}

#[derive(Deserialize, Clone, Debug)]
struct RawBatchStatusResponse {
    data: Vec<RawBatchStatus>,
    link: String,
}

#[derive(Deserialize, Clone, Debug)]
struct RawBatchStatus {
    id: String,
    invalid_transactions: Vec<String>,
    status: String,
}

#[derive(Clone, Debug)]
enum BatchStatus {
    Committed,
    Invalid(Vec<String>),
    Pending,
    Unknown,
}

impl From<RawBatchStatus> for BatchStatus {
    fn from(raw: RawBatchStatus) -> Self {
        let status = raw.status.clone();
        match raw.status.as_str() {
            "COMMITTED" => Self::Committed,
            "PENDING" => Self::Pending,
            "UNKNOWN" => Self::Unknown,
            "INVALID" => Self::Invalid(raw.invalid_transactions),
            other => unreachable!("Got unexpected status : {}", other),
        }
    }
}

impl BatchStatus {
    fn is_complete(&self) -> bool {
        match self {
            Self::Committed | Self::Invalid(_) => true,
            _ => false,
        }
    }
    fn is_successful(&self) -> bool {
        match self {
            Self::Committed => true,
            _ => false,
        }
    }
}

fn check_status(link: &str) -> BatchStatus {
    let status = ureq::get(link).call().unwrap().into_string().unwrap();
    let response: RawBatchStatusResponse = serde_json::from_str(&status).unwrap();
    response.data[0].clone().into()
}

fn send_command(command: impl CCTransaction + ToGenericCommand) -> Response {
    let command = command.to_generic_command();
    let payload_bytes = serde_cbor::to_vec(&command).unwrap();
    let signer = signer_from_file("my_key");

    // build transaction
    let mut txn_header = TransactionHeader::new();
    txn_header.set_family_name(CCTransactionHandler::family_name());

    let family_vers = CCTransactionHandler::family_versions();
    let last_version = family_vers.last().unwrap();
    txn_header.set_family_version(last_version.to_string());

    // Generate a random 128 bit number to use as a nonce
    let mut nonce = [0u8; 16];
    thread_rng()
        .try_fill(&mut nonce[..])
        .expect("Error generating random nonce");
    txn_header.set_nonce(to_hex_string(&nonce.to_vec()));

    let input_vec = CCTransactionHandler::namespaces();
    let output_vec = input_vec.clone();

    txn_header.set_inputs(RepeatedField::from_vec(input_vec));
    txn_header.set_outputs(RepeatedField::from_vec(output_vec));
    txn_header.set_signer_public_key(
        signer
            .get_public_key()
            .expect("Error retrieving Public Key")
            .as_hex(),
    );
    txn_header.set_batcher_public_key(
        signer
            .get_public_key()
            .expect("Error retrieving Public Key")
            .as_hex(),
    );

    txn_header.set_payload_sha512(to_hex_string(&sha512(&payload_bytes).to_vec()));
    let txn_header_bytes = txn_header
        .write_to_bytes()
        .expect("Error converting transaction header to bytes");

    // sign transaction
    let signature = signer
        .sign(&txn_header_bytes)
        .expect("Error signing the transaction header");

    let mut txn = Transaction::new();
    txn.set_header(txn_header_bytes.to_vec());
    txn.set_header_signature(signature);
    txn.set_payload(payload_bytes);

    // batch header
    let mut batch_header = BatchHeader::new();

    batch_header.set_signer_public_key(
        signer
            .get_public_key()
            .expect("Error retrieving Public Key")
            .as_hex(),
    );

    let transaction_ids = vec![txn.clone()]
        .iter()
        .map(|trans| String::from(trans.get_header_signature()))
        .collect();

    batch_header.set_transaction_ids(RepeatedField::from_vec(transaction_ids));

    let batch_header_bytes = batch_header
        .write_to_bytes()
        .expect("Error converting batch header to bytes");

    let signature = signer
        .sign(&batch_header_bytes)
        .expect("Error signing the batch header");

    let mut batch = Batch::new();

    batch.set_header(batch_header_bytes);
    batch.set_header_signature(signature);
    batch.set_transactions(RepeatedField::from_vec(vec![txn]));

    let mut batch_list = BatchList::new();
    batch_list.set_batches(RepeatedField::from_vec(vec![batch]));
    let batch_list_bytes = batch_list
        .write_to_bytes()
        .expect("Error converting batch list to bytes");

    // TODO: specify address via cli flags
    let response = ureq::post("http://localhost:8008/batches")
        .set("Content-Type", "application/octet-stream")
        .timeout(Duration::from_secs(30))
        .send_bytes(&batch_list_bytes)
        .unwrap();

    response
}

#[track_caller]
fn execute_success(command: impl CCTransaction + ToGenericCommand) {
    use std::time::Duration;

    let response = send_command(command);
    let body = response.into_string().unwrap();

    let response: BatchSendResponse = serde_json::from_str(&body).unwrap();
    println!("***** DEBUG, response={:?}", response);

    let mut status = check_status(&response.link);
    while !status.is_complete() {
        status = check_status(&response.link);
    }

    log::warn!("status = {:?}", status);

    assert!(matches!(status, BatchStatus::Committed));
    // TODO: send to REST API and expect failure!
}

#[track_caller]
fn execute_failure(command: impl CCTransaction + ToGenericCommand, expected_err: &str) {
    use std::time::Duration;

    let response = send_command(command);
    let body = response.into_string().unwrap();

    let response: BatchSendResponse = serde_json::from_str(&body).unwrap();

    println!("***** DEBUG, body={:?}", body);

    // TODO: send to REST API and expect failure!
    println!("DEBUG: expected failure {}", expected_err);
}

fn integration_test(func: impl FnOnce() + Send + 'static) {
    let mut test = DockerClient::new();
    test.run(|client| async move {
        let binary_path = env!("CARGO_BIN_EXE_ccprocessor-rust");
        let mut sub = std::process::Command::new(binary_path)
            .arg("-vvv")
            .spawn()
            .expect("Failed to spawn ccprocessor-rust");

        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();
        let thread = std::thread::spawn(move || {
            let stop = stop_thread;
            let context = zmq::Context::new();
            let gateway_sock = context.socket(zmq::REP).unwrap();
            gateway_sock.set_rcvtimeo(1000).unwrap();
            gateway_sock.set_sndtimeo(1000).unwrap();
            gateway_sock.bind("tcp://0.0.0.0:55555").unwrap();
            while !stop.load(std::sync::atomic::Ordering::SeqCst) {
                // if let gateway_sock.recv(msg, flags)
                while let Ok(Ok(req)) = gateway_sock.recv_string(0) {
                    log::warn!("Gateway got request: {}", req);
                    gateway_sock.send("good", 0).unwrap();
                }
            }
            log::warn!("Gateway stopping");
        });

        func();

        stop.store(true, std::sync::atomic::Ordering::SeqCst);
        sub.kill().unwrap();
    });
}

#[test]
fn foo() {
    ccprocessor_rust::setup_logs(2).unwrap();
    integration_test(|| {
        let my_sighash = SigHash::from("my_sighash");

        let command = CollectCoins {
            amount: 1000000000.into(),
            eth_address: "asdfasdf".into(),
            blockchain_tx_id: "unused_if_hacked".into(),
        };
        println!("cool kids only");
        assert!(true == true);

        println!("Executing failure");
        execute_success(command);
        std::thread::sleep(Duration::from_secs(15));
    });
}
