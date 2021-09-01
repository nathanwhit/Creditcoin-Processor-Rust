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
fn add_bid_order_success() {
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
        let fundraiser_signer =
            signer_with_secret("827c39480011a29fa972ed8b671ee5a69edd13e24b5442ee2694514e56d15d88");
        let fundraiser = SigHash::from(&fundraiser_signer);
        let mut tse = ToStateEntryCtx::new(2u64);
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest {
            tip: 3,
            ..::core::default::Default::default()
        };
        let mut register_address = register_address_for("fundraiseraddress");
        let (mut address_id, mut address_proto) =
            tse.state_entry_from(register_address.clone(), fundraiser.clone());
        let mut add_bid_order_guid = Guid::from(make_nonce());
        let mut add_bid_order = AddBidOrder {
            address_id: address_id.clone().into(),
            amount_str: "1000".into(),
            interest: "100".into(),
            maturity: "10".into(),
            fee_str: "1".into(),
            expiration: 10000.into(),
        };
        let (mut bid_order_id, mut bid_order) = tse.state_entry_from(
            add_bid_order.clone(),
            AddBidOrderArgs {
                guid: add_bid_order_guid.clone(),
                address: address_proto.clone(),
                sighash: fundraiser.clone(),
            },
        );
        let mut command_guid_ = add_bid_order_guid.clone();
        let mut command = add_bid_order.clone();
        let mut bid_order_id =
            AddressId::with_prefix_key(BID_ORDER.clone(), command_guid_.clone().as_str());
        let mut address_proto = address_for("fundraiseraddress", &fundraiser.clone());
        let mut bid_order = ccprocessor_rust::protos::BidOrder {
            blockchain: address_proto.blockchain.clone(),
            address: command.address_id.clone(),
            amount: command.amount_str.clone(),
            interest: command.interest.clone(),
            maturity: command.maturity.clone(),
            fee: command.fee_str.clone(),
            expiration: command.expiration.clone().into(),
            block: (request.tip - 1).to_string(),
            sighash: fundraiser.clone().into(),
        };
        {
            let amount = tx_fee * 2;
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "ncp3CpqlvPLhtOw".into(),
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
        execute_success(
            command,
            ports,
            Some(Nonce::from(command_guid_.clone())),
            &fundraiser_signer,
        );
        expect_set_state_entries(
            ports,
            vec![
                (
                    fundraiser_wallet_id_.clone().to_string(),
                    wallet_with(Some(0)).unwrap().into(),
                ),
                (
                    bid_order_id.clone().to_string().to_string(),
                    bid_order.clone().to_bytes().into(),
                ),
                make_fee(&command_guid_.clone(), &fundraiser.clone(), Some(2)),
            ],
        )
        .unwrap();
    });
}
