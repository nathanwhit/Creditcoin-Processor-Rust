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
fn add_deal_order_success() {
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
            signer_with_secret("a7b02b5f64b5dfc1707f7a1d5ba36e86eed1db01f3b3be2d67be26fa97347352");
        let investor = SigHash::from(&investor_signer);
        let fundraiser_signer =
            signer_with_secret("d3e004d33ca25e188841da501a0ebb156928c10cf74e5cd9ee08424ff46a6f30");
        let fundraiser = SigHash::from(&fundraiser_signer);
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest {
            tip: 8,
            ..::core::default::Default::default()
        };
        let mut investor_address_id = address_id_for("investoraddress");
        let mut fundraiser_address_id = address_id_for("fundraiseraddress");
        let mut add_ask_order_guid = Guid::from(make_nonce());
        let mut add_bid_order_guid = Guid::from(make_nonce());
        let mut add_offer_guid = Guid::from(make_nonce());
        let mut ask_order_id =
            Address::with_prefix_key(ASK_ORDER.clone(), add_ask_order_guid.clone().as_str());
        let mut bid_order_id =
            Address::with_prefix_key(BID_ORDER.clone(), add_bid_order_guid.clone().as_str());
        let mut offer_id =
            Address::with_prefix_key(OFFER.clone(), &string!(&ask_order_id, &bid_order_id));
        let mut command = AddDealOrder {
            offer_id: offer_id.clone().into(),
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
            block: (request.tip - 4).to_string(),
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
            block: (request.tip - 3).to_string(),
            sighash: fundraiser.clone().into(),
        };
        let mut add_offer = AddOffer {
            ask_order_id: ask_order_id.clone().into(),
            bid_order_id: bid_order_id.clone().into(),
            expiration: 10000.into(),
        };
        let mut offer = ccprocessor_rust::protos::Offer {
            blockchain: investor_address_proto.clone().blockchain.clone(),
            ask_order: ask_order_id.clone().into(),
            bid_order: bid_order_id.clone().into(),
            expiration: command.clone().expiration.into(),
            block: (request.tip - 2).to_string(),
            sighash: investor.clone().to_string(),
        };
        {
            let amount = tx_fee.clone() * 3;
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "DFhHR7jM4Lkt3GQ".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &investor_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let investor_wallet_id_ = WalletId::from(&investor);
        {
            let amount = tx_fee.clone() * 3 + 1;
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "F73egbIzFc7XB0Z".into(),
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
        {
            let tx = add_offer.clone();
            let response = send_command_with_signer(
                tx,
                ports,
                Some(Nonce::from(add_offer_guid.clone())),
                &investor_signer,
            );
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let mut deal_order_id =
            Address::with_prefix_key(DEAL_ORDER.clone(), &command.clone().offer_id);
        let mut deal_order = ccprocessor_rust::protos::DealOrder {
            blockchain: offer.clone().blockchain,
            src_address: ask_order.clone().address,
            dst_address: bid_order.clone().address,
            amount: bid_order.clone().amount,
            interest: bid_order.clone().interest,
            maturity: bid_order.clone().maturity,
            fee: bid_order.clone().fee,
            expiration: command.clone().expiration.into(),
            sighash: fundraiser.clone().to_string(),
            block: (request.tip - 1).to_string(),
            ..::core::default::Default::default()
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
                    deal_order_id.clone().to_string(),
                    deal_order.clone().to_bytes().into(),
                ),
                (
                    fundraiser_wallet_id_.clone().to_string(),
                    wallet_with(Some(0)).unwrap().into(),
                ),
                make_fee(
                    &command_guid_.clone(),
                    &fundraiser.clone(),
                    Some(request.tip - 1),
                ),
            ],
        )
        .unwrap();
        expect_delete_state_entries(
            ports,
            vec![
                offer.clone().ask_order.clone().to_string(),
                offer.clone().bid_order.clone().to_string(),
                offer_id.clone().clone().to_string(),
            ],
        )
        .unwrap();
    });
}
