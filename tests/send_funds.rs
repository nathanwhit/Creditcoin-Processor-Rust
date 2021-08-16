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
            signer_with_secret("08c121ca505e4e1e51da99767eda5f11f054561a64c43e63ea9ef86e2e30d8cb");
        let my_sighash = SigHash::from(&my_sighash_signer);
        let fundraiser_sighash_signer =
            signer_with_secret("73c70c6b077d9e66994174ba8c8aa5980e9541e835ae9393d006c521d7d0d773");
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
                blockchain_tx_id: "xG5HZ1UcrcXtw02".into(),
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
                blockchain_tx_id: "5jcMcUw9UVVbfom".into(),
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
            signer_with_secret("54a800864e9c5cc451204aaaa8ca1cba4a14a9b663d54056cb7974329baeb991");
        let my_sighash = SigHash::from(&my_sighash_signer);
        let fundraiser_sighash_signer =
            signer_with_secret("fdadd7f948d87be09cf35aa6381ef541b462ce27e9681427dd5abdfdc4b0d359");
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
                blockchain_tx_id: "O4ROOwYXR6pploS".into(),
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
            signer_with_secret("335256b094cc7ab7c09060eb45c313f17da19e5953bbc619c46d4d586cc11486");
        let my_sighash = SigHash::from(&my_sighash_signer);
        let fundraiser_sighash_signer =
            signer_with_secret("4097ac407278c0b1d196fe8f40e801944ad35a3cec268a9039713cbedceb1007");
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
                blockchain_tx_id: "sDl4fIPJqab8nPs".into(),
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
            signer_with_secret("4fc9897e1f3dd6f29d1c6f87975feaeca0e896ffce733519aa2dae4ace59fbde");
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
