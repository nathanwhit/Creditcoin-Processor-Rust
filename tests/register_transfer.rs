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
fn register_transfer_success() {
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
            signer_with_secret("f7fcee59e5e0562f106e9ad863a652c51aea9726b082fb7e7ddcf2c143dfea97");
        let investor = SigHash::from(&investor_signer);
        let fundraiser_signer =
            signer_with_secret("bb6f0666ece8216757847eab5d23285ebdb24c0036471c627f3b89ddcf3c3f6b");
        let fundraiser = SigHash::from(&fundraiser_signer);
        let mut tx_fee = ccprocessor_rust::handler::constants::TX_FEE.clone();
        let mut request = TpProcessRequest {
            tip: 9,
            ..::core::default::Default::default()
        };
        let mut investor_address_id = address_id_for("myaddress");
        let mut fundraiser_address_id = address_id_for("otheraddress");
        let mut bid_order_guid = Guid::from(make_nonce());
        let mut ask_order_guid = Guid::from(make_nonce());
        let mut offer_guid = Guid::from(make_nonce());
        let mut ask_order_id = Address::with_prefix_key(ASK_ORDER, ask_order_guid.as_str());
        let mut bid_order_id = Address::with_prefix_key(BID_ORDER, bid_order_guid.as_str());
        let mut offer_id = Address::with_prefix_key(
            OFFER,
            &string!(ask_order_id.as_str(), bid_order_id.as_str()),
        );
        let mut deal_order_guid = Guid::from(make_nonce());
        let mut deal_order_id = Address::with_prefix_key(DEAL_ORDER, offer_id.as_str());
        let mut command = RegisterTransfer {
            gain: 1.into(),
            order_id: deal_order_id.clone().into(),
            blockchain_tx_id: String::from("blockchaintxid"),
        };
        let command_guid_ = Guid::from(make_nonce());
        {
            let amount = Integer::from(Credo::from(4) * tx_fee.clone() + 2);
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "WRkKA0WtuVzFPd7".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &investor_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let investor_wallet_id_ = WalletId::from(&investor);
        {
            let amount = Integer::from(Credo::from(4) * tx_fee.clone());
            let collect_coins = ccprocessor_rust::handler::CollectCoins {
                amount: amount.into(),
                eth_address: "dummy".into(),
                blockchain_tx_id: "0Khh4HBD7PhG4eQ".into(),
            };
            let response = send_command_with_signer(collect_coins, ports, None, &fundraiser_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let fundraiser_wallet_id_ = WalletId::from(&fundraiser);
        {
            let tx = RegisterAddress {
                blockchain: String::from("ethereum"),
                address: String::from("myaddress"),
                network: String::from("rinkeby"),
            };
            let response = send_command_with_signer(tx, ports, None, &investor_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        {
            let tx = RegisterAddress {
                blockchain: String::from("ethereum"),
                address: String::from("otheraddress"),
                network: String::from("rinkeby"),
            };
            let response = send_command_with_signer(tx, ports, None, &fundraiser_signer);
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        {
            let tx = AddBidOrder {
                address_id: investor_address_id.clone().to_string(),
                amount_str: String::from("1"),
                interest: String::from("0"),
                maturity: String::from("1"),
                fee_str: String::from("2"),
                expiration: BlockNum(100),
            };
            let response = send_command_with_signer(
                tx,
                ports,
                Some(Nonce::from(bid_order_guid.clone())),
                &investor_signer,
            );
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        {
            let tx = AddAskOrder {
                address_id: fundraiser_address_id.clone().to_string(),
                amount_str: String::from("1"),
                interest: String::from("0"),
                maturity: String::from("1"),
                fee_str: String::from("2"),
                expiration: BlockNum(100),
            };
            let response = send_command_with_signer(
                tx,
                ports,
                Some(Nonce::from(ask_order_guid.clone())),
                &fundraiser_signer,
            );
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        {
            let tx = AddOffer {
                ask_order_id: ask_order_id.clone().into(),
                bid_order_id: bid_order_id.clone().into(),
                expiration: BlockNum(100),
            };
            let response = send_command_with_signer(
                tx,
                ports,
                Some(Nonce::from(offer_guid.clone())),
                &fundraiser_signer,
            );
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        {
            let tx = AddDealOrder {
                offer_id: offer_id.clone().into(),
                expiration: 100.into(),
            };
            let response = send_command_with_signer(
                tx,
                ports,
                Some(Nonce::from(deal_order_guid.clone())),
                &investor_signer,
            );
            assert_matches!(
                complete_batch(&response.link, None),
                Some(BatchStatus::Committed)
            );
        }
        let mut deal_order = ccprocessor_rust::protos::DealOrder {
            blockchain: String::from("ethereum"),
            dst_address: investor_address_id.clone().into(),
            src_address: fundraiser_address_id.clone().into(),
            amount: String::from("1"),
            sighash: investor.clone().to_string(),
            ..::core::default::Default::default()
        };
        let mut investor_address = ccprocessor_rust::protos::Address {
            blockchain: String::from("ethereum"),
            value: String::from("myaddress"),
            network: String::from("rinkeby"),
            sighash: investor.clone().to_string(),
        };
        let mut fundraiser_address = ccprocessor_rust::protos::Address {
            blockchain: String::from("ethereum"),
            value: String::from("otheraddress"),
            network: String::from("rinkeby"),
            sighash: fundraiser.clone().to_string(),
        };
        let mut transfer_id = Address::with_prefix_key(
            TRANSFER,
            &string!(
                &investor_address.blockchain,
                &command.blockchain_tx_id,
                &investor_address.network
            ),
        );
        let mut transfer = ccprocessor_rust::protos::Transfer {
            blockchain: investor_address.clone().blockchain.clone(),
            dst_address: fundraiser_address_id.clone().to_string(),
            src_address: investor_address_id.clone().to_string(),
            order: command.clone().order_id.clone(),
            amount: (command.gain.clone() + 1).to_string(),
            tx: command.clone().blockchain_tx_id.clone(),
            sighash: investor.clone().to_string(),
            block: 8.to_string(),
            processed: false,
        };
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
                    transfer_id.clone().to_string().to_string(),
                    transfer.clone().to_bytes().into(),
                ),
                (
                    investor_wallet_id_.clone().to_string(),
                    wallet_with(Some(0)).unwrap().into(),
                ),
                make_fee(&guid, &investor, Some(8)),
            ],
        )
        .unwrap();
    });
}
