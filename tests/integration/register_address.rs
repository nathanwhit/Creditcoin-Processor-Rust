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
        let investor_signer =
            signer_with_secret("827c39480011a29fa972ed8b671ee5a69edd13e24b5442ee2694514e56d15d88");
        let investor = SigHash::from(&investor_signer);
        let mut tse = ToStateEntryCtx::new(2u64);
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest {
            tip: u64::from(tse.tip()),
            ..Default::default()
        };
        let mut register_address = RegisterAddress {
            blockchain: "ethereum".into(),
            address: "myaddress".into(),
            network: "rinkeby".into(),
        };
        let (mut address_id, mut address) =
            tse.state_entry_from(register_address.clone(), investor.clone());
        let mut command = register_address.clone();
        let command_guid_ = Guid::from(make_nonce());
        {
            let amount = tx_fee.clone();
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "ncp3CpqlvPLhtOw".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &investor_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let investor_wallet_id_ = WalletId::from(&investor);
        let mut guid = command_guid_.clone();
        execute_success(
            command,
            ports,
            Some(Nonce::from(command_guid_.clone())),
            &investor_signer,
        );
        expect_set_state_entries(
            ports,
            vec![
                (
                    address_id.clone().to_string().to_string(),
                    address.clone().to_bytes().into(),
                ),
                (
                    investor_wallet_id_.clone().to_string(),
                    wallet_with(Some(0)).unwrap().into(),
                ),
                make_fee(&guid.clone(), &investor.clone(), Some(tse.tip() - 1)),
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
        let investor_signer =
            signer_with_secret("edb3e8e44d4a1f0050ce03a729b2da887b644e95ec6bf6a0cfdbf0f40bf47d91");
        let investor = SigHash::from(&investor_signer);
        let fundraiser_signer =
            signer_with_secret("7586793549de011ef43bfac7cee149feb1f1de9a5f558c75ef46714b544f4fe3");
        let fundraiser = SigHash::from(&fundraiser_signer);
        let mut tse = ToStateEntryCtx::new(3u64);
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest {
            tip: u64::from(tse.tip()),
            ..Default::default()
        };
        let mut register_address = RegisterAddress {
            blockchain: "ethereum".into(),
            address: "myaddress".into(),
            network: "rinkeby".into(),
        };
        let (mut address_id, mut address) =
            tse.state_entry_from(register_address.clone(), fundraiser.clone());
        let mut command = register_address.clone();
        let command_guid_ = Guid::from(make_nonce());
        {
            let amount = tx_fee.clone();
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "4MrUslkrXe5bAHt".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &investor_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let investor_wallet_id_ = WalletId::from(&investor);
        {
            let amount = tx_fee.clone();
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "qrXc4V46SL8cHOL".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &fundraiser_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let fundraiser_wallet_id_ = WalletId::from(&fundraiser);
        {
            let tx = register_address.clone();
            let response = send_command_with_signer(tx, ports, None, &fundraiser_signer);
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
            &investor_signer,
        );
    });
}
