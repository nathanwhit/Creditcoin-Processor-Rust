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
fn add_ask_order_success() {
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
        let mut register_address = register_address_for("investoraddress");
        let mut register_address_guid = Guid::from(make_nonce());
        let (mut address_id, mut address) =
            tse.state_entry_from(register_address.clone(), investor.clone());
        let mut add_ask_order = AddAskOrder {
            address_id: address_id.clone().into(),
            amount_str: "1000".into(),
            interest: "100".into(),
            maturity: "10".into(),
            fee_str: "1".into(),
            expiration: 10000.into(),
        };
        let mut add_ask_order_guid = Guid::from(make_nonce());
        let (mut ask_order_id, mut ask_order) = tse.state_entry_from(
            add_ask_order.clone(),
            AddAskOrderArgs {
                guid: add_ask_order_guid.clone(),
                sighash: investor.clone(),
                address: address.clone().clone(),
            },
        );
        let mut command_guid_ = add_ask_order_guid.clone();
        let mut command = add_ask_order.clone();
        {
            let amount = tx_fee * 2;
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
        {
            let tx = register_address.clone();
            let response = send_command_with_signer(
                tx,
                ports,
                Some(Nonce::from(register_address_guid.clone())),
                &investor_signer,
            );
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
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
                    investor_wallet_id_.clone().to_string(),
                    wallet_with(Some(0)).unwrap().into(),
                ),
                (
                    ask_order_id.clone().to_string().to_string(),
                    ask_order.clone().to_bytes().into(),
                ),
                make_fee(&command_guid_.clone(), &investor.clone(), Some(2)),
            ],
        )
        .unwrap();
    });
}
