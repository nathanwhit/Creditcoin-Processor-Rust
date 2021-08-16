#![cfg(feature = "integration-testing")]
mod common;
use common::*;

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces,
    dead_code
)]
fn send_funds_success() {
    use ccprocessor_rust::ext::*;
    use ccprocessor_rust::handler::types::*;
    use ccprocessor_rust::handler::*;
    use prost::Message as _;
    use protobuf::Message as _;
    use rug::Integer;
    use std::convert::TryFrom as _;
    use std::str::FromStr as _;
    setup_logs();
    integration_test(|ports| {
        let my_sighash_signer =
            signer_with_secret("3d0b15c6f4503b5d2749d6e2f3a298a08f2546d9b0fa3303bbdfab658c782631");
        let my_sighash = SigHash::from(&my_sighash_signer);
        let fundraiser_sighash_signer =
            signer_with_secret("f359a7392056fa14bc90c079cc70789883e8a0836ff72ce9d1fc9a2d3a93a800");
        let fundraiser_sighash = SigHash::from(&fundraiser_sighash_signer);
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest {
            tip: 3,
            ..Default::default()
        };
        let mut command = SendFunds {
            amount: 1.into(),
            sighash: fundraiser_sighash.clone().into(),
        };
        let command_guid_ = Guid::from(make_nonce());
        let mut amount_needed = command.amount.clone() + tx_fee;
        {
            let amount = amount_needed.clone();
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "oIsSD5WFqFWlwsD".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &my_sighash_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let my_sighash_wallet_id_ = WalletId::from(&my_sighash);
        {
            let amount = 0;
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "U4llqtGjIJT6Oml".into(),
            };
            let response =
                send_command_with_signer(collect_coins, ports, None, &fundraiser_sighash_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let fundraiser_sighash_wallet_id_ = WalletId::from(&fundraiser_sighash);
        let command_guid_ = Guid::from(make_nonce());
        let mut guid = command_guid_.clone();
        execute_success(
            command,
            ports,
            Some(Nonce::from(command_guid_.clone())),
            &my_sighash_signer,
        );
        expect_set_state_entries(
            ports,
            vec![
                (
                    my_sighash_wallet_id_.clone().to_string(),
                    wallet_with(Some(0)).unwrap().into(),
                ),
                (
                    fundraiser_sighash_wallet_id_.clone().to_string(),
                    wallet_with(Some(1)).unwrap().into(),
                ),
                (
                    Address::with_prefix_key(
                        ccprocessor_rust::handler::constants::FEE,
                        guid.as_str(),
                    )
                    .to_string(),
                    ccprocessor_rust::protos::Fee {
                        sighash: my_sighash.clone().into(),
                        block: 2u64.to_string(),
                    }
                    .to_bytes()
                    .into(),
                ),
            ],
        )
        .unwrap();
    });
}

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces,
    dead_code
)]
fn send_funds_cannot_afford_fee() {
    use ccprocessor_rust::ext::*;
    use ccprocessor_rust::handler::types::*;
    use ccprocessor_rust::handler::*;
    use prost::Message as _;
    use protobuf::Message as _;
    use rug::Integer;
    use std::convert::TryFrom as _;
    use std::str::FromStr as _;
    setup_logs();
    integration_test(|ports| {
        let my_sighash_signer =
            signer_with_secret("1f00d0f7ee401140d110df2b30af9f4b984f10b807a5b1448e36dba22f271a10");
        let my_sighash = SigHash::from(&my_sighash_signer);
        let fundraiser_sighash_signer =
            signer_with_secret("9c7a4f040b2f95065a43c0b79ad97c96542cc8146fce0378b9ba076ea75e194c");
        let fundraiser_sighash = SigHash::from(&fundraiser_sighash_signer);
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest::default();
        let mut command = SendFunds {
            amount: 1.into(),
            sighash: fundraiser_sighash.clone().into(),
        };
        let command_guid_ = Guid::from(make_nonce());
        {
            let amount = 1;
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "AUIbuQCUXFctnB5".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &my_sighash_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let my_sighash_wallet_id_ = WalletId::from(&my_sighash);
        let command_guid_ = Guid::from(make_nonce());
        execute_failure(
            command,
            "Insufficient funds",
            ports,
            Some(Nonce::from(command_guid_.clone())),
            &my_sighash_signer,
        );
    });
}

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces,
    dead_code
)]
fn send_funds_cannot_afford_amount() {
    use ccprocessor_rust::ext::*;
    use ccprocessor_rust::handler::types::*;
    use ccprocessor_rust::handler::*;
    use prost::Message as _;
    use protobuf::Message as _;
    use rug::Integer;
    use std::convert::TryFrom as _;
    use std::str::FromStr as _;
    setup_logs();
    integration_test(|ports| {
        let my_sighash_signer =
            signer_with_secret("c87941c01aa3ac8735c6d99b728e227962b927426b2f71afe7369158ba634f78");
        let my_sighash = SigHash::from(&my_sighash_signer);
        let fundraiser_sighash_signer =
            signer_with_secret("522966d38f57792d3090ab788ccbaf410c5c51bd6d36ebdebe3dfd4443c65ed6");
        let fundraiser_sighash = SigHash::from(&fundraiser_sighash_signer);
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest::default();
        let mut command = SendFunds {
            amount: 1.into(),
            sighash: fundraiser_sighash.clone().into(),
        };
        let command_guid_ = Guid::from(make_nonce());
        {
            let amount = tx_fee.clone();
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "bM3lFKKd8NTw54s".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &my_sighash_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let my_sighash_wallet_id_ = WalletId::from(&my_sighash);
        let command_guid_ = Guid::from(make_nonce());
        execute_failure(
            command,
            "Insufficient funds",
            ports,
            Some(Nonce::from(command_guid_.clone())),
            &my_sighash_signer,
        );
    });
}

#[test]
#[allow(
    unused_variables,
    unused_parens,
    unused_imports,
    unused_mut,
    unused_braces,
    dead_code
)]
fn send_funds_to_self() {
    use ccprocessor_rust::ext::*;
    use ccprocessor_rust::handler::types::*;
    use ccprocessor_rust::handler::*;
    use prost::Message as _;
    use protobuf::Message as _;
    use rug::Integer;
    use std::convert::TryFrom as _;
    use std::str::FromStr as _;
    setup_logs();
    integration_test(|ports| {
        let my_sighash_signer =
            signer_with_secret("b3605f7348d4fa9d955aa64aa625954c1bd9709622dab2f821cc644a37594f74");
        let my_sighash = SigHash::from(&my_sighash_signer);
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest::default();
        let mut command = SendFunds {
            amount: 1.into(),
            sighash: my_sighash.clone().into(),
        };
        let command_guid_ = Guid::from(make_nonce());
        let command_guid_ = Guid::from(make_nonce());
        execute_failure(
            command,
            "Invalid destination",
            ports,
            Some(Nonce::from(command_guid_.clone())),
            &my_sighash_signer,
        );
    });
}
