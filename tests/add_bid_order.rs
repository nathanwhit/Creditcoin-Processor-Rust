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
            signer_with_secret("d2378ed388553b5c3ec5f8f82a6caa3de92a698d0633ef8b8a8d65a228e4f664");
        let fundraiser = SigHash::from(&fundraiser_signer);
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest {
            tip: 3,
            ..::core::default::Default::default()
        };
        let mut address_id = address_id_for("fundraiseraddress");
        let mut command = AddBidOrder {
            address_id: address_id.clone().into(),
            amount_str: "1000".into(),
            interest: "100".into(),
            maturity: "10".into(),
            fee_str: "1".into(),
            expiration: 10000.into(),
        };
        let command_guid_ = Guid::from(make_nonce());
        let mut bid_order_id =
            Address::with_prefix_key(BID_ORDER.clone(), command_guid_.clone().as_str());
        {
            let amount = tx_fee * 2;
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "WRSriWDXitc4nIv".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &fundraiser_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let fundraiser_wallet_id_ = WalletId::from(&fundraiser);
        {
            let tx = register_address_for("fundraiseraddress");
            let response = send_command_with_signer(tx, ports, None, &fundraiser_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let mut address_proto = address_for("fundraiseraddress", &fundraiser.clone());
        let mut bid_order = ccprocessor_rust::protos::BidOrder {
            blockchain: address_proto.clone().blockchain.clone(),
            address: command.clone().address_id.clone(),
            amount: command.clone().amount_str.clone(),
            interest: command.clone().interest.clone(),
            maturity: command.clone().maturity.clone(),
            fee: command.clone().fee_str.clone(),
            expiration: command.clone().expiration.into(),
            block: (request.tip - 1).to_string(),
            sighash: fundraiser.clone().into(),
        };
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
