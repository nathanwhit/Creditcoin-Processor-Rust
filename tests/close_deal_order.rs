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
fn close_deal_order_success() {
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
            signer_with_secret("9cca303722f0f8be60ec75049db74f1125f140d24893eea375c97ebf40b4965b");
        let investor = SigHash::from(&investor_signer);
        let fundraiser_signer =
            signer_with_secret("6ea10e10487997543b2bc38ff1edae5be7886df51c2dabe7902bb89f9c28d5dc");
        let fundraiser = SigHash::from(&fundraiser_signer);
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest {
            tip: 13,
            ..::core::default::Default::default()
        };
        let mut investor_address_id = address_id_for("investoraddress");
        let mut fundraiser_address_id = address_id_for("fundraiseraddress");
        let mut register_address_investor = register_address_for("investoraddress");
        let mut register_address_fundraiser = register_address_for("fundraiseraddress");
        let mut investor_address = address_for("investoraddress", &investor.clone());
        let mut fundraiser_address = address_for("fundraiseraddress", &fundraiser.clone());
        let mut add_ask_order = AddAskOrder {
            address_id: investor_address_id.clone().into(),
            amount_str: "1000".into(),
            interest: "100".into(),
            maturity: "10".into(),
            fee_str: "1".into(),
            expiration: 10000.into(),
        };
        let mut add_ask_order_guid = Guid::from(make_nonce());
        let mut ask_order_id =
            Address::with_prefix_key(ASK_ORDER.clone(), add_ask_order_guid.clone().as_str());
        let mut ask_order = ccprocessor_rust::protos::AskOrder {
            blockchain: investor_address.clone().blockchain.clone(),
            address: add_ask_order.clone().address_id.clone(),
            amount: add_ask_order.clone().amount_str.clone(),
            interest: add_ask_order.clone().interest.clone(),
            maturity: add_ask_order.clone().maturity.clone(),
            fee: add_ask_order.clone().fee_str.clone(),
            expiration: add_ask_order.clone().expiration.into(),
            block: (request.tip - 9).to_string(),
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
        let mut add_bid_order_guid = Guid::from(make_nonce());
        let mut bid_order_id =
            Address::with_prefix_key(BID_ORDER.clone(), add_bid_order_guid.clone().as_str());
        let mut bid_order = ccprocessor_rust::protos::BidOrder {
            blockchain: fundraiser_address.clone().blockchain.clone(),
            address: fundraiser_address_id.clone().into(),
            amount: add_bid_order.clone().amount_str.clone(),
            interest: add_bid_order.clone().interest.clone(),
            maturity: add_bid_order.clone().maturity.clone(),
            fee: add_bid_order.clone().fee_str.clone(),
            expiration: add_bid_order.clone().expiration.into(),
            block: (request.tip - 8).to_string(),
            sighash: fundraiser.clone().into(),
        };
        let mut add_offer = AddOffer {
            ask_order_id: ask_order_id.clone().into(),
            bid_order_id: bid_order_id.clone().into(),
            expiration: 10000.into(),
        };
        let mut add_offer_guid = Guid::from(make_nonce());
        let mut offer_id =
            Address::with_prefix_key(OFFER.clone(), &string!(&ask_order_id, &bid_order_id));
        let mut offer = ccprocessor_rust::protos::Offer {
            blockchain: investor_address.clone().blockchain.clone(),
            ask_order: ask_order_id.clone().into(),
            bid_order: bid_order_id.clone().into(),
            expiration: add_offer.clone().expiration.into(),
            block: (request.tip - 7).to_string(),
            sighash: investor.clone().to_string(),
        };
        let mut add_deal_order = AddDealOrder {
            offer_id: offer_id.clone().into(),
            expiration: 10000.into(),
        };
        let mut deal_order_id = Address::with_prefix_key(DEAL_ORDER.clone(), &offer_id.clone());
        let mut deal_order = ccprocessor_rust::protos::DealOrder {
            blockchain: offer.clone().blockchain,
            src_address: ask_order.clone().address,
            dst_address: bid_order.clone().address,
            amount: bid_order.clone().amount,
            interest: bid_order.clone().interest,
            maturity: bid_order.clone().maturity,
            fee: bid_order.clone().fee,
            expiration: add_ask_order.clone().expiration.into(),
            sighash: fundraiser.clone().to_string(),
            block: (request.tip - 6).to_string(),
            ..::core::default::Default::default()
        };
        let mut register_transfer_invest = RegisterTransfer {
            gain: 0.into(),
            order_id: deal_order_id.clone().into(),
            blockchain_tx_id: String::from("blockchaintxid"),
        };
        let mut invest_transfer_id = Address::with_prefix_key(
            TRANSFER.clone(),
            &string!(
                &investor_address.blockchain,
                &register_transfer_invest.blockchain_tx_id,
                &investor_address.network
            ),
        );
        let mut invest_transfer = ccprocessor_rust::protos::Transfer {
            blockchain: investor_address.clone().blockchain.clone(),
            dst_address: fundraiser_address_id.clone().to_string(),
            src_address: investor_address_id.clone().to_string(),
            order: register_transfer_invest.clone().order_id.clone(),
            amount: deal_order.clone().amount,
            tx: register_transfer_invest.clone().blockchain_tx_id.clone(),
            sighash: investor.clone().to_string(),
            block: (request.tip - 5).to_string(),
            processed: false,
        };
        let mut complete_deal_order = CompleteDealOrder {
            deal_order_id: deal_order_id.clone().into(),
            transfer_id: invest_transfer_id.clone().into(),
        };
        let mut updated_deal_order = ccprocessor_rust::protos::DealOrder {
            loan_transfer: invest_transfer_id.clone().into(),
            block: (request.tip - 4).to_string(),
            ..deal_order.clone()
        };
        let mut updated_transfer = ccprocessor_rust::protos::Transfer {
            processed: true,
            ..invest_transfer.clone()
        };
        let mut lock_deal_order = LockDealOrder {
            deal_order_id: deal_order_id.clone().into(),
        };
        let mut locked_deal_order = ccprocessor_rust::protos::DealOrder {
            lock: fundraiser.clone().to_string(),
            ..updated_deal_order.clone()
        };
        let mut ticks = BlockNum::from(
            (((request.tip - 1) - u64::from_str(&updated_deal_order.block).unwrap())
                + u64::from_str(&updated_deal_order.maturity).unwrap())
                / u64::from_str(&updated_deal_order.maturity).unwrap(),
        );
        let mut gain_amount = ccprocessor_rust::handler::utils::calc_interest(
            &CurrencyAmount::try_parse(&deal_order.clone().amount).unwrap(),
            ticks.clone().into(),
            &CurrencyAmount::try_parse(&deal_order.clone().interest).unwrap(),
        );
        let mut register_transfer_repayment = RegisterTransfer {
            gain: gain_amount.clone().into(),
            order_id: deal_order_id.clone().into(),
            blockchain_tx_id: String::from("repaymenttxid"),
        };
        let mut repayment_transfer_id = Address::with_prefix_key(
            TRANSFER.clone(),
            &string!(
                &investor_address.blockchain,
                &register_transfer_repayment.blockchain_tx_id,
                &investor_address.network
            ),
        );
        let mut amount = CurrencyAmount::try_parse(deal_order.amount).unwrap() + gain_amount;
        let mut repayment_transfer = ccprocessor_rust::protos::Transfer {
            blockchain: investor_address.clone().blockchain.clone(),
            dst_address: investor_address_id.clone().to_string(),
            src_address: fundraiser_address_id.clone().to_string(),
            order: register_transfer_repayment.clone().order_id.clone(),
            amount: amount.clone().into(),
            tx: register_transfer_repayment.clone().blockchain_tx_id.clone(),
            sighash: fundraiser.clone().to_string(),
            block: (request.tip - 2).to_string(),
            processed: false,
        };
        let mut updated_repayment_transfer = ccprocessor_rust::protos::Transfer {
            processed: true,
            ..repayment_transfer.clone()
        };
        let mut closed_deal_order = ccprocessor_rust::protos::DealOrder {
            repayment_transfer: repayment_transfer_id.clone().into(),
            ..locked_deal_order.clone()
        };
        let mut command = CloseDealOrder {
            deal_order_id: deal_order_id.clone().into(),
            transfer_id: repayment_transfer_id.clone().into(),
        };
        let command_guid_ = Guid::from(make_nonce());
        {
            let amount = tx_fee.clone() * 5;
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "inzrDQmA593rwIN".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &investor_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let investor_wallet_id_ = WalletId::from(&investor);
        {
            let amount = tx_fee.clone() * 6 + 1;
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "2AcOPXI5FdJhgRe".into(),
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
            let tx = register_address_investor.clone();
            let response = send_command_with_signer(tx, ports, None, &investor_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        {
            let tx = register_address_fundraiser.clone();
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
        {
            let tx = add_deal_order.clone();
            let response = send_command_with_signer(tx, ports, None, &fundraiser_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        {
            let tx = register_transfer_invest.clone();
            let response = send_command_with_signer(tx, ports, None, &investor_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        {
            let tx = complete_deal_order.clone();
            let response = send_command_with_signer(tx, ports, None, &investor_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        {
            let tx = lock_deal_order.clone();
            let response = send_command_with_signer(tx, ports, None, &fundraiser_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        {
            let tx = register_transfer_repayment.clone();
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
                    deal_order_id.clone().to_string(),
                    closed_deal_order.clone().to_bytes().into(),
                ),
                (
                    repayment_transfer_id.clone().to_string(),
                    updated_repayment_transfer.clone().to_bytes().into(),
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
    });
}
