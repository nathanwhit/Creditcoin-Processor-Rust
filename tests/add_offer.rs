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
fn add_offer_success() {
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
        let fundraiser_signer =
            signer_with_secret("48b0ae97607427a8550e4da5edc8da0a04617adde25c98a405a0c47114cdf69e");
        let fundraiser = SigHash::from(&fundraiser_signer);
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest {
            tip: 7,
            ..::core::default::Default::default()
        };
        let mut investor_address_id = address_id_for("investoraddress");
        let mut fundraiser_address_id = address_id_for("fundraiseraddress");
        let mut add_ask_order_guid = Guid::from(make_nonce());
        let mut add_bid_order_guid = Guid::from(make_nonce());
        let mut ask_order_id =
            Address::with_prefix_key(ASK_ORDER.clone(), add_ask_order_guid.clone().as_str());
        let mut bid_order_id =
            Address::with_prefix_key(BID_ORDER.clone(), add_bid_order_guid.clone().as_str());
        let mut command = AddOffer {
            ask_order_id: ask_order_id.clone().into(),
            bid_order_id: bid_order_id.clone().into(),
            expiration: 10000.into(),
        };
        let command_guid_ = Guid::from(make_nonce());
        let mut investor_address_proto = address_for("investoraddress", &investor.clone());
        let mut fundraiser_address_proto = address_for("fundraiseraddress", &fundraiser.clone());
        let mut add_ask_order = AddAskOrder {
            address_id: investor_address_id.clone().into(),
            amount_str: "1000".into(),
            interest: "100".into(),
            maturity: "10".into(),
            fee_str: "1".into(),
            expiration: 10000.into(),
        };
        let mut ask_order = ccprocessor_rust::protos::AskOrder {
            blockchain: investor_address_proto.clone().blockchain.clone(),
            address: add_ask_order.clone().address_id.clone(),
            amount: add_ask_order.clone().amount_str.clone(),
            interest: add_ask_order.clone().interest.clone(),
            maturity: add_ask_order.clone().maturity.clone(),
            fee: add_ask_order.clone().fee_str.clone(),
            expiration: add_ask_order.clone().expiration.into(),
            block: (request.tip - 1).to_string(),
            sighash: investor.clone().into(),
        };
        let mut add_bid_order = AddBidOrder {
            address_id: fundraiser_address_id.clone().into(),
            amount_str: "1000".into(),
            interest: "100".into(),
            maturity: "10".into(),
            fee_str: "1".into(),
            expiration: 10000.into(),
        };
        let mut bid_order = ccprocessor_rust::protos::BidOrder {
            blockchain: fundraiser_address_proto.clone().blockchain.clone(),
            address: fundraiser_address_id.clone().into(),
            amount: add_bid_order.clone().amount_str.clone(),
            interest: add_bid_order.clone().interest.clone(),
            maturity: add_bid_order.clone().maturity.clone(),
            fee: add_bid_order.clone().fee_str.clone(),
            expiration: add_bid_order.clone().expiration.into(),
            block: (request.tip - 1).to_string(),
            sighash: fundraiser.clone().into(),
        };
        {
            let amount = tx_fee.clone() * 3;
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "vPLhtOwk9olipA5".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &investor_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let investor_wallet_id_ = WalletId::from(&investor);
        {
            let amount = tx_fee.clone() * 2;
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "4SdmxHN4MrUslkr".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &fundraiser_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let fundraiser_wallet_id_ = WalletId::from(&fundraiser);
        let add_ask_order_guid_ = Guid::from(make_nonce());
        let add_bid_order_guid_ = Guid::from(make_nonce());
        {
            let tx = register_address_for("investoraddress");
            let response = send_command_with_signer(tx, ports, None, &investor_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        {
            let tx = register_address_for("fundraiseraddress");
            let response = send_command_with_signer(tx, ports, None, &fundraiser_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        {
            let tx = add_ask_order.clone();
            let response = send_command_with_signer(
                tx,
                ports,
                Some(Nonce::from(add_ask_order_guid.clone())),
                &investor_signer,
            );
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        {
            let tx = add_bid_order.clone();
            let response = send_command_with_signer(
                tx,
                ports,
                Some(Nonce::from(add_bid_order_guid.clone())),
                &fundraiser_signer,
            );
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let mut offer_address_id = Address::with_prefix_key(
            OFFER.clone(),
            &string!(&command.ask_order_id, &command.bid_order_id),
        );
        let mut offer = ccprocessor_rust::protos::Offer {
            blockchain: investor_address_proto.clone().blockchain.clone(),
            ask_order: command.clone().ask_order_id.clone(),
            bid_order: command.clone().bid_order_id.clone(),
            expiration: command.clone().expiration.into(),
            block: (request.tip - 1).to_string(),
            sighash: investor.clone().to_string(),
        };
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
                    offer_address_id.clone().to_string(),
                    offer.clone().to_bytes().into(),
                ),
                (
                    investor_wallet_id_.clone().to_string(),
                    wallet_with(Some(0)).unwrap().into(),
                ),
                make_fee(
                    &command_guid_.clone(),
                    &investor.clone(),
                    Some(request.tip - 1),
                ),
            ],
        )
        .unwrap();
    });
}
