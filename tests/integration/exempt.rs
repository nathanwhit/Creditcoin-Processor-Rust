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
fn exempt_success() {
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
        let mut tse = ToStateEntryCtx::new(3u64);
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest {
            tip: u64::from(tse.tip()),
            ..Default::default()
        };
        let mut add_ask_order_guid = Guid::random();
        let mut add_bid_order_guid = Guid::random();
        let mut add_offer_guid = Guid::random();
        let mut add_deal_order_guid = Guid::random();
        let mut register_address_investor = register_address_for("investoraddress");
        let (mut investor_address_id, mut investor_address) =
            tse.state_entry_from(register_address_investor.clone(), investor.clone());
        let mut register_address_fundraiser = register_address_for("fundraiseraddress");
        let (mut fundraiser_address_id, mut fundraiser_address) =
            tse.state_entry_from(register_address_fundraiser.clone(), fundraiser.clone());
        let mut add_ask_order = AddAskOrder {
            address_id: investor_address_id.clone().into(),
            amount_str: "1000".into(),
            interest: "100".into(),
            maturity: "10".into(),
            fee_str: "1".into(),
            expiration: 10000.into(),
        };
        let (mut ask_order_id, mut ask_order) = tse.state_entry_from(
            add_ask_order.clone(),
            AddAskOrderArgs {
                guid: add_ask_order_guid.clone(),
                address: investor_address.clone(),
                sighash: investor.clone(),
            },
        );
        let mut add_bid_order = AddBidOrder {
            address_id: fundraiser_address_id.clone().into(),
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
                address: fundraiser_address.clone(),
                sighash: fundraiser.clone(),
            },
        );
        let mut add_offer = AddOffer {
            ask_order_id: ask_order_id.clone().into(),
            bid_order_id: bid_order_id.clone().into(),
            expiration: 10000.into(),
        };
        let (mut offer_id, mut offer) = tse.state_entry_from(
            add_offer.clone(),
            AddOfferArgs {
                src_address: investor_address.clone(),
                sighash: investor.clone(),
            },
        );
        let mut add_deal_order = AddDealOrder {
            offer_id: offer_id.clone().into(),
            expiration: 10000.into(),
        };
        let (mut deal_order_id, mut deal_order) = tse.state_entry_from(
            add_deal_order.clone(),
            AddDealOrderArgs {
                bid_order: bid_order.clone().clone(),
                ask_order: ask_order.clone().clone(),
                offer: offer.clone().clone(),
                sighash: fundraiser.clone(),
            },
        );
        let mut register_transfer = RegisterTransfer {
            gain: 0.into(),
            order_id: deal_order_id.clone().into(),
            blockchain_tx_id: String::from("blockchaintxid"),
        };
        let (mut transfer_id, mut transfer) = tse.state_entry_from(
            register_transfer.clone(),
            RegisterTransferArgs {
                kind: TransferKind::DealOrder(deal_order.clone()),
                src_address: investor_address.clone(),
                src_sighash: investor.clone(),
            },
        );
        let mut exempt = Exempt {
            deal_order_id: deal_order_id.clone().into(),
            transfer_id: transfer_id.clone().into(),
        };
        let mut updated_deal_order = ccprocessor_rust::protos::DealOrder {
            repayment_transfer: transfer_id.clone().into(),
            ..deal_order.clone()
        };
        let mut updated_transfer = ccprocessor_rust::protos::Transfer {
            processed: true,
            ..transfer.clone()
        };
        tse.inc_tip();
        let mut command = exempt.clone();
        let command_guid_ = Guid::from(make_nonce());
        {
            let amount = tx_fee.clone() * 5;
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
            let amount = tx_fee.clone() * 3 + 1;
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
            let tx = register_transfer.clone();
            let response = send_command_with_signer(tx, ports, None, &investor_signer);
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
                    deal_order_id.clone().to_string(),
                    updated_deal_order.clone().to_bytes().into(),
                ),
                (
                    transfer_id.clone().to_string(),
                    updated_transfer.clone().to_bytes().into(),
                ),
                (
                    investor_wallet_id_.clone().to_string(),
                    wallet_with(Some(0)).unwrap().into(),
                ),
                make_fee(
                    &command_guid_.clone(),
                    &investor.clone(),
                    Some(tse.tip() - 1),
                ),
            ],
        )
        .unwrap();
    });
}
