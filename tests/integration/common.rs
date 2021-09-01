#![cfg(feature = "integration-testing")]
#![allow(dead_code, unused_imports)]

pub use anyhow::Result;
pub use bollard::container::{Config, CreateContainerOptions, RemoveContainerOptions};
pub use bollard::image::CreateImageOptions;
pub use bollard::models::{HostConfig, PortBinding};
pub use bollard::network::CreateNetworkOptions;
pub use bollard::Docker;
pub use ccprocessor_rust::ext::MessageExt;
pub use ccprocessor_rust::handler::constants::*;
pub use ccprocessor_rust::handler::types::*;
pub use ccprocessor_rust::handler::utils::to_hex_string;
use ccprocessor_rust::handler::RegisterAddress;
pub use ccprocessor_rust::handler::{CCTransaction, CCTransactionHandler, CollectCoins};
pub use ccprocessor_rust::{string, test_utils::*};
pub use derive_more::{Deref, DerefMut};
pub use futures_lite::{Future, StreamExt};

pub use itertools::Itertools;
pub use maplit::hashmap;
use once_cell::sync::Lazy;
pub use openssl::sha::sha512;
pub use protobuf::{Message, RepeatedField};
pub use rand::distributions::Alphanumeric;
pub use rand::{thread_rng, Rng};
pub use sawtooth_sdk::messages::batch::{Batch, BatchHeader, BatchList};
pub use sawtooth_sdk::messages::transaction::Transaction;

pub use sawtooth_sdk::signing::secp256k1::Secp256k1PublicKey;
pub use sawtooth_sdk::{
    messages::transaction::TransactionHeader,
    signing::{create_context, secp256k1::Secp256k1PrivateKey, Signer},
};
pub use serde::Deserialize;
use std::collections::HashSet;
pub use std::convert::TryInto;
pub use std::net::Ipv4Addr;
use std::sync::Mutex;

pub use assert_matches::assert_matches;
pub use pretty_assertions::{assert_eq, assert_ne};
pub use sawtooth_sdk::messages::processor::TpProcessRequest;
pub use std::panic::{catch_unwind, AssertUnwindSafe};
pub use std::sync::atomic::AtomicBool;
pub use std::sync::{Arc, Once};
pub use std::time::{Duration, Instant};
pub use std::{fs::File, io::Read};
pub use tokio::runtime::Runtime;

pub trait Ext {
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
        Ok(_image) => {
            log::info!("got image");
        }
        Err(_e) => {
            while let Some(Ok(_response)) = docker
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
    use std::iter;
    let mut rng = thread_rng();
    let chars: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(10)
        .collect();
    format!("{}_{}", base, chars)
}

static TAKEN_PORTS: Lazy<Mutex<HashSet<u16>>> = Lazy::new(|| Mutex::new(HashSet::new()));

#[derive(Deref, DerefMut)]
pub struct DockerClient {
    #[deref]
    #[deref_mut]
    pub client: Docker,
    cleanup_containers: Vec<String>,
    cleanup_networks: Vec<String>,
    validator_component_port: u16,
    validator_endpoint_port: u16,
    rest_api_port: u16,
    gateway_port: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct PortConfig {
    validator_component: u16,
    validator_endpoint: u16,
    rest_api: u16,
    gateway: u16,
}

fn pick_port() -> u16 {
    let mut tries = 0;
    while tries < 25 {
        tries += 1;
        let port = portpicker::pick_unused_port().unwrap();
        if TAKEN_PORTS.lock().unwrap().insert(port) {
            return port;
        }
    }
    panic!("Failed to find an unused port in {} tries", tries);
}

impl DockerClient {
    pub fn new() -> Self {
        Self {
            client: Docker::connect_with_local_defaults()
                .expect("Docker daemon not found, it must be running for integration tests"),
            validator_component_port: pick_port(),
            validator_endpoint_port: pick_port(),
            rest_api_port: pick_port(),
            gateway_port: pick_port(),
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

    ensure_image_present(docker, "hyperledger/sawtooth-settings-tp", "1.0").await?;
    ensure_image_present(docker, "gluwa/creditcoin-validator", "1.7.1").await?;
    ensure_image_present(docker, "gluwa/sawtooth-rest-api", "latest").await?;

    let name = random_name("creditcoin-validator");

    let comp_port = docker.validator_component_port;
    let endpoint_port = docker.validator_endpoint_port;

    let comp_port_str = format!("{}/tcp", docker.validator_component_port);
    let endpoint_port_str = format!("{}/tcp", docker.validator_endpoint_port);

    let validator = docker
        .create_container(
            Some(CreateContainerOptions { name }),
            Config {
                exposed_ports: Some(hashmap! {
                    comp_port_str.as_str() => hashmap! {},
                    endpoint_port_str.as_str() => hashmap! {}
                }),
                cmd: Some(vec![
                    "bash", "-c",
                    &format!(r#"if [[ ! -f /etc/sawtooth/keys/validator.pub ]]; then echo "First run"; sawadm keygen; sawset genesis --key /etc/sawtooth/keys/validator.priv; sawset proposal create --key /etc/sawtooth/keys/validator.priv sawtooth-consensus.algorithm.name=PoW sawtooth.consensus.algorithm.version=0.1 -o config.batch; sawadm genesis config-genesis.batch config.batch; fi; sawtooth-validator -vv --endpoint tcp://173.66.57.160:{endp} --bind component:tcp://0.0.0.0:{comp} --bind network:tcp://0.0.0.0:{endp}"#, endp=endpoint_port, comp=comp_port),
                ]),
                image: Some("gluwa/creditcoin-validator:1.7.1"),
                host_config: Some(HostConfig {
                    network_mode: Some(network_id.clone()),
                    port_bindings: Some(hashmap! {
                        comp_port_str.clone() => Some(vec![PortBinding {
                            host_port: Some(docker.validator_component_port.to_string()),
                            host_ip: None,
                        }]),
                        endpoint_port_str.clone() => Some(vec![PortBinding {
                            host_port: Some(docker.validator_endpoint_port.to_string()),
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
                    comp_port_str.as_str() => hashmap! {}
                }),
                cmd: Some(vec![
                    "bash",
                    "-c",
                    &format!(
                        "settings-tp -vv -C tcp://{}:{}",
                        &validator_ip, docker.validator_component_port
                    ),
                ]),
                image: Some("hyperledger/sawtooth-settings-tp:1.0"),
                host_config: Some(HostConfig {
                    network_mode: Some(network_id.clone()),

                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;

    let settings_id = settings.id;

    docker.cleanup_containers.push(settings_id.clone());

    let name = random_name("rest-api");

    let rest_port = docker.rest_api_port.to_string();
    let rest_api = docker
        .create_container(
            Some(CreateContainerOptions { name }),
            Config {
                exposed_ports: Some(hashmap! {
                    rest_port.as_str() => hashmap! {}
                }),
                cmd: Some(vec![
                    "bash",
                    "-c",
                    &format!(
                        "sawtooth-rest-api -vv -C tcp://{}:{} --bind 0.0.0.0:{rest}",
                        &validator_ip,
                        docker.validator_component_port,
                        rest = docker.rest_api_port
                    ),
                ]),
                image: Some("gluwa/sawtooth-rest-api:latest"),
                host_config: Some(HostConfig {
                    network_mode: Some(network_id.clone()),
                    port_bindings: Some(hashmap! {
                        rest_port.clone() => Some(vec![PortBinding {
                            host_port: Some(rest_port.clone()),
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

pub fn signer_from_file(profile: &str) -> Signer {
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
pub struct BatchSendResponse {
    pub link: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawBatchStatusResponse {
    data: Vec<RawBatchStatus>,
    link: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawBatchStatus {
    id: String,
    invalid_transactions: Vec<FailedTransaction>,
    status: String,
}

#[derive(Clone, Debug)]
pub enum BatchStatus {
    Committed,
    Invalid(Vec<FailedTransaction>),
    Pending,
    Unknown,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FailedTransaction {
    id: String,
    message: String,
}

impl From<RawBatchStatus> for BatchStatus {
    fn from(raw: RawBatchStatus) -> Self {
        let _status = raw.status.clone();
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
        matches!(self, Self::Committed | Self::Invalid(_))
    }
}

pub fn send_command_with_signer(
    command: impl CCTransaction + ToGenericCommand,
    ports: PortConfig,
    nonce: Option<Nonce>,
    signer: &Signer,
) -> BatchSendResponse {
    let command = command.to_generic_command();
    let payload_bytes = serde_cbor::to_vec(&command).unwrap();
    // build transaction
    let mut txn_header = TransactionHeader::new();
    txn_header.set_family_name(CCTransactionHandler::family_name());

    let family_vers = CCTransactionHandler::family_versions();
    let last_version = family_vers.last().unwrap();
    txn_header.set_family_version(last_version.to_string());

    // Generate a random 128 bit number to use as a nonce

    let nonce = to_hex_string(&nonce.unwrap_or_else(make_nonce).to_vec());
    txn_header.set_nonce(nonce);

    let mut input_vec = CCTransactionHandler::namespaces();
    input_vec.push(ccprocessor_rust::handler::constants::SETTINGS_NAMESPACE.into());
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
    let response = ureq::post(&format!("http://localhost:{}/batches", ports.rest_api))
        .set("Content-Type", "application/octet-stream")
        .timeout(Duration::from_secs(30))
        .send_bytes(&batch_list_bytes)
        .unwrap();

    let response: BatchSendResponse = serde_json::from_reader(response.into_reader()).unwrap();

    response
}

pub fn new_secret() -> String {
    use libsecp256k1::SecretKey;
    let mut rng = old_rand::thread_rng();
    let secret = SecretKey::random(&mut rng);
    format!("{:x}", secret)
}

pub fn send_command(
    command: impl CCTransaction + ToGenericCommand,
    ports: PortConfig,
    nonce: Option<Nonce>,
) -> BatchSendResponse {
    let signer = signer_with_secret(&new_secret());
    send_command_with_signer(command, ports, nonce, &signer)
}

pub fn check_status(link: &str) -> BatchStatus {
    let status = ureq::get(link).call().unwrap().into_string().unwrap();
    let response: RawBatchStatusResponse = serde_json::from_str(&status).unwrap();
    response.data[0].clone().into()
}

pub fn complete_batch(link: &str, timeout: Option<Duration>) -> Option<BatchStatus> {
    let mut status = check_status(link);
    let start = Instant::now();
    let timeout = timeout.unwrap_or_else(|| Duration::from_secs(60));
    let mut timed_out = false;
    while !status.is_complete() {
        if start.elapsed() > timeout {
            timed_out = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
        status = check_status(link);
    }
    if timed_out && !status.is_complete() {
        None
    } else {
        Some(status)
    }
}

pub fn execute_success(
    command: impl CCTransaction + ToGenericCommand,
    ports: PortConfig,
    nonce: Option<Nonce>,
    signer: &Signer,
) {
    let response = send_command_with_signer(command, ports, nonce, signer);

    let status = complete_batch(&response.link, None).unwrap();

    assert!(
        matches!(status, BatchStatus::Committed),
        "status was {:?}",
        status
    );
}

pub fn execute_failure(
    command: impl CCTransaction + ToGenericCommand,
    expected_err: &str,
    ports: PortConfig,
    nonce: Option<Nonce>,
    signer: &Signer,
) {
    let response = send_command_with_signer(command, ports, nonce, signer);

    let status = complete_batch(&response.link, None).unwrap();

    match status {
        BatchStatus::Committed => panic!("Expected failure but the transaction was accepted"),
        BatchStatus::Invalid(v) => {
            let f = v.first().unwrap();
            assert_eq!(f.message, expected_err);
        }
        BatchStatus::Pending => panic!("Transaction never finished executing"),
        BatchStatus::Unknown => panic!("Unknown error"),
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Paging {
    limit: Option<u64>,
    start: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StateEntry {
    address: String,
    data: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RawStateResponse {
    data: Vec<StateEntry>,
    head: String,
    link: String,
    paging: Paging,
}

pub fn expect_delete_state_entries(ports: PortConfig, entries: Vec<String>) -> Result<()> {
    let url = format!("http://localhost:{}/state", ports.rest_api);

    for address in entries {
        let response = ureq::get(&url).query("address", &address).call()?;
        let response: RawStateResponse = serde_json::from_reader(response.into_reader())?;
        assert_eq!(response.data.len(), 0);
    }
    Ok(())
}

pub fn expect_set_state_entries(ports: PortConfig, entries: Vec<(String, Vec<u8>)>) -> Result<()> {
    let url = format!("http://localhost:{}/state", ports.rest_api);

    for (address, value) in entries {
        let response = ureq::get(&url).query("address", &address).call()?;
        let response: RawStateResponse = serde_json::from_reader(response.into_reader())?;
        assert_eq!(response.data.len(), 1);
        let entry = &response.data[0];
        assert_eq!(entry.address, address);
        let data_decoded = base64::decode(&entry.data)?;

        if data_decoded != value {
            ccprocessor_rust::assert_state_data_eq!(address, data_decoded, value, ccprocessor_rust);
        }
    }
    Ok(())
}

pub fn expect_set_state_entry(ports: PortConfig, address: String, value: Vec<u8>) -> Result<()> {
    expect_set_state_entries(ports, vec![(address, value)])
}

pub fn integration_test(func: impl FnOnce(PortConfig) + Send + 'static) {
    let test = DockerClient::new();
    let ports = PortConfig {
        validator_component: test.validator_component_port,
        validator_endpoint: test.validator_endpoint_port,
        rest_api: test.rest_api_port,
        gateway: test.gateway_port,
    };
    test.run(|_client| async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let binary_path = env!("CARGO_BIN_EXE_ccprocessor-rust");
        let mut sub = std::process::Command::new(binary_path)
            .arg("-E")
            .arg(&format!("tcp://localhost:{}", ports.validator_component))
            .arg("-G")
            .arg(&format!("tcp://localhost:{}", ports.gateway))
            .spawn()
            .expect("Failed to spawn ccprocessor-rust");

        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();
        let _thread = std::thread::spawn(move || {
            let stop = stop_thread;
            let context = zmq::Context::new();
            let gateway_sock = context.socket(zmq::REP).unwrap();
            gateway_sock.set_rcvtimeo(1000).unwrap();
            gateway_sock.set_sndtimeo(1000).unwrap();
            gateway_sock
                .bind(&format!("tcp://0.0.0.0:{}", ports.gateway))
                .expect(&format!("Gateway port already in use: {}", ports.gateway));
            while !stop.load(std::sync::atomic::Ordering::SeqCst) {
                while let Ok(Ok(req)) = gateway_sock.recv_string(0) {
                    log::debug!("Gateway got request: {}", req);
                    gateway_sock.send("good", 0).unwrap();
                }
            }
            log::info!("Gateway stopping");
        });

        let res = catch_unwind(AssertUnwindSafe(|| func(ports)));

        stop.store(true, std::sync::atomic::Ordering::SeqCst);
        sub.kill().unwrap();
        match res {
            Ok(res) => res,
            Err(e) => std::panic::resume_unwind(e),
        }
    });
}

pub fn setup_logs() {
    static LOGS: Once = Once::new();
    LOGS.call_once(|| {
        ccprocessor_rust::setup_logs(0).unwrap();
    });
}
