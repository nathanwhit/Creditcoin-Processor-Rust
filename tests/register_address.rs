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
fn register_address_success() {
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
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest {
            tip: 2,
            ..::core::default::Default::default()
        };
        let mut command = RegisterAddress {
            blockchain: "ethereum".into(),
            address: "myaddress".into(),
            network: "rinkeby".into(),
        };
        let command_guid_ = Guid::from(make_nonce());
        {
            let amount = tx_fee.clone();
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "ncp3CpqlvPLhtOw".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &my_sighash_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let my_sighash_wallet_id_ = WalletId::from(&my_sighash);
        let command_guid_ = Guid::from(make_nonce());
        let mut address_proto = ccprocessor_rust::protos::Address {
            blockchain: command.blockchain.clone(),
            value: command.address.clone(),
            network: command.network.clone(),
            sighash: my_sighash.to_string(),
        };
        let mut address = Address::with_prefix_key(
            ccprocessor_rust::handler::constants::ADDR,
            &string!("ethereum", "myaddress", "rinkeby"),
        );
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
                    address.clone().to_string().to_string(),
                    address_proto.clone().to_bytes().into(),
                ),
                (
                    my_sighash_wallet_id_.clone().to_string(),
                    wallet_with(Some(0)).unwrap().into(),
                ),
                make_fee(&guid, &my_sighash, Some(1)),
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
fn register_address_taken() {
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
            signer_with_secret("edb3e8e44d4a1f0050ce03a729b2da887b644e95ec6bf6a0cfdbf0f40bf47d91");
        let my_sighash = SigHash::from(&my_sighash_signer);
        let other_sighash_signer =
            signer_with_secret("7586793549de011ef43bfac7cee149feb1f1de9a5f558c75ef46714b544f4fe3");
        let other_sighash = SigHash::from(&other_sighash_signer);
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest {
            tip: 2,
            ..::core::default::Default::default()
        };
        let mut command = RegisterAddress {
            blockchain: "ethereum".into(),
            address: "myaddress".into(),
            network: "rinkeby".into(),
        };
        let command_guid_ = Guid::from(make_nonce());
        let mut address_proto = ccprocessor_rust::protos::Address {
            blockchain: command.clone().blockchain.clone(),
            value: command.clone().address.clone(),
            network: command.clone().network.clone(),
            sighash: my_sighash.clone().to_string(),
        };
        let mut address = Address::with_prefix_key(
            ccprocessor_rust::handler::constants::ADDR,
            &string!("ethereum", "myaddress", "rinkeby"),
        );
        {
            let amount = tx_fee.clone();
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "4MrUslkrXe5bAHt".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &my_sighash_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let my_sighash_wallet_id_ = WalletId::from(&my_sighash);
        {
            let amount = tx_fee.clone();
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "qrXc4V46SL8cHOL".into(),
            };
            let response =
                send_command_with_signer(collect_coins, ports, None, &other_sighash_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let other_sighash_wallet_id_ = WalletId::from(&other_sighash);
        let command_guid_ = Guid::from(make_nonce());
        {
            let tx = command.clone().clone();
            let response = send_command_with_signer(tx, ports, None, &other_sighash_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        execute_failure(
            command,
            "The address has been already registered",
            ports,
            Some(Nonce::from(command_guid_.clone())),
            &my_sighash_signer,
        );
    });
}
