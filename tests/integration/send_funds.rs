#![cfg(feature = "integration-testing")]
use super::common::*;

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
            signer_with_secret("827c39480011a29fa972ed8b671ee5a69edd13e24b5442ee2694514e56d15d88");
        let my_sighash = SigHash::from(&my_sighash_signer);
        let fundraiser_sighash_signer =
            signer_with_secret("48b0ae97607427a8550e4da5edc8da0a04617adde25c98a405a0c47114cdf69e");
        let fundraiser_sighash = SigHash::from(&fundraiser_sighash_signer);
        let mut tse = ToStateEntryCtx::new(4u64);
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
                blockchain_tx_id: "vPLhtOwk9olipA5".into(),
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
                blockchain_tx_id: "4SdmxHN4MrUslkr".into(),
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
                    AddressId::with_prefix_key(
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
            signer_with_secret("bc7c40aa9628e0b771d0bc1ecd7975011f18d46dab673de62997297b1f40985f");
        let my_sighash = SigHash::from(&my_sighash_signer);
        let fundraiser_sighash_signer =
            signer_with_secret("a84f62499ffd6aeacb118de215f07557c3a961e36177a672c1ebb25ff8d953af");
        let fundraiser_sighash = SigHash::from(&fundraiser_sighash_signer);
        let mut tse = ToStateEntryCtx::new(3u64);
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
                blockchain_tx_id: "L8cHOLuIHD6ymiC".into(),
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
            signer_with_secret("e67a6eae65ba742a692d3240bafcb87931133e83170a755bb65cd5b348c0a216");
        let my_sighash = SigHash::from(&my_sighash_signer);
        let fundraiser_sighash_signer =
            signer_with_secret("2c0f5b934d46ce41042be387ed7f78648dfa926ac38c91f13a8396910b21d285");
        let fundraiser_sighash = SigHash::from(&fundraiser_sighash_signer);
        let mut tse = ToStateEntryCtx::new(3u64);
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
                blockchain_tx_id: "eAma7TfWs0J6s1x".into(),
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
            signer_with_secret("d8ca9043781be77d563924e5de78406249c97ede8cc6811f32c731ac62199995");
        let my_sighash = SigHash::from(&my_sighash_signer);
        let mut tse = ToStateEntryCtx::new(2u64);
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
