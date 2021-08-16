#![cfg(test)]
#![allow(non_snake_case, non_upper_case_globals)]

mod execution;
#[cfg(feature = "mock")]
pub mod mocked;

use crate::test_utils::*;
use mocked::{MockSettings, MockTransactionContext};
use sawtooth_sdk::processor::handler::ApplyError;
use sawtooth_sdk::processor::TransactionProcessor;
use sawtooth_sdk::signing::secp256k1::Secp256k1PrivateKey;
use sawtooth_sdk::signing::{create_context, Context, CryptoFactory};
use serde::Serialize;
use serde_cbor::{value, Value};

use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;
use std::sync::Once;

use enclose::enclose;
use itertools::Itertools;
use mockall::lazy_static;
use mockall::predicate;
use openssl::sha::sha512;
use prost::Message;
use protobuf::Message as ProtobufMessage;
use protobuf::RepeatedField;
use rand::{thread_rng, Rng};
use rug::Integer;
use sawtooth_sdk::messages::batch::{Batch, BatchHeader, BatchList};
use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::messages::transaction::{Transaction, TransactionHeader};
use sawtooth_sdk::processor::handler::TransactionContext;
use sawtooth_sdk::signing::Signer;

use crate::ext::{IntegerExt, MessageExt};
use crate::handler::constants::*;
use crate::handler::types::{CCApplyError, SigHash};
use crate::handler::types::{Guid, WalletId};
use crate::handler::utils::{calc_interest, sha512_id};
use crate::{protos, string};

use super::context::mocked::MockHandlerContext;
use super::types::{Address, BlockNum, Credo, CurrencyAmount, TxnResult};
use super::AddBidOrder;
use super::AddDealOrder;
use super::AddOffer;
use super::AddRepaymentOrder;
use super::CloseDealOrder;
use super::CloseRepaymentOrder;
use super::CollectCoins;
use super::CompleteDealOrder;
use super::CompleteRepaymentOrder;
use super::Exempt;
use super::LockDealOrder;
use super::RegisterAddress;
use super::RegisterTransfer;
use super::SendFunds;
use super::{CCCommand, CCTransaction, CCTransactionHandler, Housekeeping};

use once_cell::sync::Lazy;

// TEST UTILS

static INIT_LOGS: Once = Once::new();
lazy_static! {
    static ref INVESTOR_SIGHASH: SigHash = SigHash::from("investor");
    static ref FUNDRAISER_SIGHASH: SigHash = SigHash::from("fundraiser");
}

fn init_logs() {
    INIT_LOGS.call_once(|| {
        // UNCOMMENT TO COLLECT LOGS
        // crate::setup_logs(3).unwrap();
    })
}

fn wallet_with(balance: Option<impl Into<Integer> + Clone>) -> Option<Vec<u8>> {
    balance.map(|b| {
        let wallet = crate::protos::Wallet {
            amount: b.into().to_string(),
        };
        let mut buf = Vec::with_capacity(wallet.encoded_len());
        wallet.encode(&mut buf).unwrap();
        buf
    })
}

macro_rules! expect {
    ($id: ident, $fun: ident where $c: expr, returning $ret: expr, $count: literal times) => {

        paste::paste! {
                #[allow(unused_variables)]
                $id.[<expect_ $fun>]()
                .times($count)
                .withf($c)
                .return_once($ret)
            };

    };
    ($id: ident, $fun: ident where $c: expr, returning $ret: expr) => {
        expect!($id, $fun where $c, returning $ret, 1 times)
    };
    ($id: ident, $fun: ident ($($arg: pat),* $(,)?), returning $ret: expr) => {
        expect!($id, $fun where { |$($arg),*| true}, returning $ret, 1 times)
    };
    ($id: ident, $fun: ident ($($arg: pat if $e: expr),* $(,)?) -> $ret: expr , $count:literal times) => {
        expect!($id, $fun where {
            move |$($arg),*| {
                $($e)&&*
            }
        }, returning {
            move |$($arg),*| {
                $ret
            }
        }, 1 times)
    };
    ($id: ident, $fun: ident ($($arg: pat),* $(,)?) -> $ret: expr , $count:literal times) => {
        expect!($id, $fun where { |$($arg),*| true}, returning {move |$($arg),*| {
            $ret
        }}, $count times)
    };
    ($id: ident, $fun: ident ($($arg: pat),* $(,)?) -> $ret: expr ) => {
        expect!($id, $fun ($($arg),*) -> $ret , 1 times)
    };
    ($id: ident, $fun: ident ($($arg: pat if $e: expr),*  $(,)?) -> $ret: expr) => {
       expect!($id, $fun ($($arg if $e),*) -> $ret , 1 times)
    };
    ($id: ident, get balance at $w: ident -> $ret: expr) => {
        expect!($id, get_state_entry where {
            enclose!(($w) move |_w| {
                _w == $w.as_str()
            })
        }, returning {
            move |_| Ok(wallet_with($ret))
        }, 1 times)
    };
    ($id: ident, get balance at $w: ident, returning $ret: expr) => {
        expect!($id, get_state_entry where {
            enclose!(($w) move |_w| {
                _w == $w.as_str()
            })
        }, returning $ret, 1 times)
    };
    ($id: ident, set balance at $w: ident to $amt: ident) => {
        {
            expect!($id, set_state_entry where {
                let $amt = $amt.clone();
                let _wallet = wallet_with(Some($amt)).unwrap();
                enclose!(($w) move |_w, _a| {
                    _w == $w.as_str() && _a == &_wallet
                })
            }, returning {
                |_,_| Ok(())
            }, 1 times);
            wallet_with(Some($amt.clone())).unwrap()
        }
    };
    ($id: ident, set balance at $w: ident to ($amt: expr)) => {
        {
            expect!($id, set_state_entry where {
                enclose!(($w) move |_w, _a| {
                    _w == $w.as_str() && _a == &wallet_with(Some($amt.clone())).unwrap()
                })
            }, returning {
                |_,_| Ok(())
            }, 1 times);
            wallet_with(Some($amt.clone())).unwrap()
        }
    };
    ($id: ident, sighash -> $sig: ident) => {
        expect!($id, sighash where {
            |_| true
        }, returning {
            enclose!(($sig) move |_| Ok($sig))
        })
    };
    ($id: ident, sighash -> $sig: expr) => {
        expect!($id, sighash where {
            |_| true
        }, returning {
            enclose!(($sig) move |_| Ok(crate::handler::types::SigHash($sig.to_string())))
        })
    };
    ($id: ident, guid -> $guid: ident) => {
        expect!($id, guid where {
            |_| true
        }, returning {
            enclose!(($guid) move |_| $guid)
        })
    };
    ($id: ident, guid -> $guid: literal) => {
        expect!($id, guid where {
            |_| true
        }, returning {
            move |_| crate::handler::types::Guid($guid.to_string())
        })
    };
}

static PROCESSED_BLOCK_IDX: Lazy<String> = Lazy::new(|| {
    string!(
        NAMESPACE_PREFIX.as_str(),
        PROCESSED_BLOCK,
        PROCESSED_BLOCK_ID,
    )
});

// ----- COMMAND DESERIALIZATION TESTS -----

#[track_caller]
fn deserialize_success(value: impl Serialize, expected: impl Into<CCCommand>) {
    let value = value::to_value(value).unwrap();
    let expected = expected.into();
    let result = CCCommand::try_from(value).unwrap();
    assert_eq!(result, expected);
}

#[track_caller]
fn deserialize_failure(value: impl Serialize, expected_err: &str) {
    let value = value::to_value(value).unwrap();
    let result = CCCommand::try_from(value).unwrap_err();
    match result.downcast_ref::<CCApplyError>() {
        Some(CCApplyError::InvalidTransaction(s)) => {
            assert_eq!(s, expected_err);
        }
        _ => panic!("Expected an InvalidTransaction error"),
    };
}
// SendFunds

#[test]
fn send_funds_accept() {
    deserialize_success(
        TwoArgCommand::new("SendFunds", 1, "foo"),
        SendFunds {
            amount: 1.into(),
            sighash: SigHash("foo".into()),
        },
    )
}

#[test]
fn send_funds_case_insensitive() {
    deserialize_success(
        TwoArgCommand::new("SeNdfUnDs", 1, "foo"),
        SendFunds {
            amount: 1.into(),
            sighash: SigHash("foo".into()),
        },
    );

    let value = value::to_value(TwoArgCommand::new("SendFunds", 1, "Foo-Bar")).unwrap();

    let result = CCCommand::try_from(value).unwrap();
    assert_eq!(
        result,
        SendFunds {
            amount: 1.into(),
            sighash: SigHash("foo-bar".into()),
        }
        .into(),
    );
}

#[test]
fn send_funds_rejects_negative() {
    deserialize_failure(
        TwoArgCommand::new("SendFunds", -1, "foo"),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn send_funds_rejects_non_integer() {
    deserialize_failure(
        TwoArgCommand::new("SendFunds", "bad", "foo"),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn send_funds_rejects_missing_arg() {
    deserialize_failure(OneArgCommand::new("SendFunds", 1), "Expecting sighash");
    deserialize_failure(ZeroArgCommand::new("SendFunds"), "Expecting amount");
}

// RegisterAddress

#[test]
fn register_address_accept() {
    deserialize_success(
        ThreeArgCommand::new("RegisterAddress", "blockchain", "address", "network"),
        RegisterAddress {
            blockchain: "blockchain".into(),
            address: "address".into(),
            network: "network".into(),
        },
    )
}

#[test]
fn register_address_case_insensitive() {
    deserialize_success(
        ThreeArgCommand::new("ReGiStErAdDrEsS", "blockchain", "address", "network"),
        RegisterAddress {
            blockchain: "blockchain".into(),
            address: "address".into(),
            network: "network".into(),
        },
    )
}

#[test]
fn register_address_missing_arg() {
    deserialize_failure(
        TwoArgCommand::new("RegisterAddress", "blockchain", "address"),
        "Expecting network",
    );
    deserialize_failure(
        OneArgCommand::new("RegisterAddress", "blockchain"),
        "Expecting address",
    );
    deserialize_failure(
        ZeroArgCommand::new("RegisterAddress"),
        "Expecting blockchain",
    );
}

// RegisterTransfer

#[test]
fn register_transfer_accept() {
    deserialize_success(
        ThreeArgCommand::new("RegisterTransfer", 1, "orderid", "txid"),
        RegisterTransfer {
            gain: 1.into(),
            order_id: "orderid".into(),
            blockchain_tx_id: "txid".into(),
        },
    );
}

#[test]
fn register_transfer_case_insensitive() {
    deserialize_success(
        ThreeArgCommand::new("ReGiStErTrAnSfEr", 1, "orderid", "txid"),
        RegisterTransfer {
            gain: 1.into(),
            order_id: "orderid".into(),
            blockchain_tx_id: "txid".into(),
        },
    );
}

#[test]
fn register_transfer_accepts_negative_gain() {
    deserialize_success(
        ThreeArgCommand::new("RegisterTransfer", -1, "orderid", "txid"),
        RegisterTransfer {
            gain: (-1).into(),
            order_id: "orderid".into(),
            blockchain_tx_id: "txid".into(),
        },
    );
}

#[test]
fn register_transfer_invalid_gain() {
    deserialize_failure(
        ThreeArgCommand::new("RegisterTransfer", "invalid", "orderid", "txid"),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn register_transfer_missing_arg() {
    deserialize_failure(
        TwoArgCommand::new("RegisterTransfer", 1, "orderid"),
        "Expecting blockchainTxId",
    );
    deserialize_failure(
        OneArgCommand::new("RegisterTransfer", 1),
        "Expecting orderID",
    );
    deserialize_failure(ZeroArgCommand::new("RegisterTransfer"), "Expecting gain");
}

// AddAskOrder

#[test]
fn add_ask_order_accept() {
    let args = SixArgCommand::new("AddAskOrder", "addressid", 1, 2, 3, 4, 5);
    let args_uppercase = SixArgCommand {
        p1: "ADDRESSID".into(),
        ..args.clone()
    };
    let expected = AddAskOrder {
        address_id: "addressid".into(),
        amount_str: 1.to_string(),
        interest: 2.to_string(),
        maturity: 3.to_string(),
        fee_str: 4.to_string(),
        expiration: 5.into(),
    };
    deserialize_success(args, expected.clone());
    deserialize_success(args_uppercase, expected);
}

#[test]
fn add_ask_order_case_insensitive() {
    let args = SixArgCommand::new("AdDAsKoRdEr", "addressid", 1, 2, 3, 4, 5);
    let expected = AddAskOrder {
        address_id: "addressid".into(),
        amount_str: 1.to_string(),
        interest: 2.to_string(),
        maturity: 3.to_string(),
        fee_str: 4.to_string(),
        expiration: 5.into(),
    };
    deserialize_success(args, expected);
}

#[test]
fn add_ask_order_negative_amount() {
    deserialize_failure(
        SixArgCommand::new("AddAskOrder", "addressid", -1, 2, 3, 4, 5),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn add_ask_order_invalid_amount() {
    deserialize_failure(
        SixArgCommand::new("AddAskOrder", "addressid", "bad", 2, 3, 4, 5),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn add_ask_order_negative_interest() {
    deserialize_failure(
        SixArgCommand::new("AddAskOrder", "addressid", 1, -2, 3, 4, 5),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn add_ask_order_invalid_interest() {
    deserialize_failure(
        SixArgCommand::new("AddAskOrder", "addressid", 1, "BAD", 3, 4, 5),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn add_ask_order_negative_maturity() {
    deserialize_failure(
        SixArgCommand::new("AddAskOrder", "addressid", 1, 2, -3, 4, 5),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn add_ask_order_invalid_maturity() {
    deserialize_failure(
        SixArgCommand::new("AddAskOrder", "addressid", 1, 2, "BAD", 4, 5),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn add_ask_order_negative_fee() {
    deserialize_failure(
        SixArgCommand::new("AddAskOrder", "addressid", 1, 2, 3, -4, 5),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn add_ask_order_invalid_fee() {
    deserialize_failure(
        SixArgCommand::new("AddAskOrder", "addressid", 1, 2, 3, "BAD", 5),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn add_ask_order_negative_expiration() {
    deserialize_failure(
        SixArgCommand::new("AddAskOrder", "addressid", 1, 2, 3, 4, -5),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn add_ask_order_invalid_expiration() {
    deserialize_failure(
        SixArgCommand::new("AddAskOrder", "addressid", 1, 2, 3, 4, "BAD"),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn add_ask_order_missing_arg() {
    deserialize_failure(
        FiveArgCommand::new("AddAskOrder", "addressid", 1, 2, 3, 4),
        "Expecting expiration",
    );
    deserialize_failure(
        FourArgCommand::new("AddAskOrder", "addressid", 1, 2, 3),
        "Expecting fee",
    );
    deserialize_failure(
        ThreeArgCommand::new("AddAskOrder", "addressid", 1, 2),
        "Expecting maturity",
    );
    deserialize_failure(
        TwoArgCommand::new("AddAskOrder", "addressid", 1),
        "Expecting interest",
    );
    deserialize_failure(
        OneArgCommand::new("AddAskOrder", "addressid"),
        "Expecting amount",
    );
    deserialize_failure(ZeroArgCommand::new("AddAskOrder"), "Expecting addressId");
}

// AddBidOrder

#[test]
fn add_bid_order_accept() {
    let args = SixArgCommand::new("AddBidOrder", "addressid", 1, 2, 3, 4, 5);
    let args_uppercase = SixArgCommand {
        p1: "ADDRESSID".into(),
        ..args.clone()
    };
    let expected = AddBidOrder {
        address_id: "addressid".into(),
        amount_str: 1.to_string(),
        interest: 2.to_string(),
        maturity: 3.to_string(),
        fee_str: 4.to_string(),
        expiration: 5.into(),
    };
    deserialize_success(args, expected.clone());
    deserialize_success(args_uppercase, expected);
}

#[test]
fn add_bid_order_case_insensitive() {
    let args = SixArgCommand::new("AdDbIdOrDeR", "addressid", 1, 2, 3, 4, 5);
    let expected = AddBidOrder {
        address_id: "addressid".into(),
        amount_str: 1.to_string(),
        interest: 2.to_string(),
        maturity: 3.to_string(),
        fee_str: 4.to_string(),
        expiration: 5.into(),
    };
    deserialize_success(args, expected);
}

#[test]
fn add_bid_order_negative_amount() {
    deserialize_failure(
        SixArgCommand::new("AddBidOrder", "addressid", -1, 2, 3, 4, 5),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn add_bid_order_invalid_amount() {
    deserialize_failure(
        SixArgCommand::new("AddBidOrder", "addressid", "bad", 2, 3, 4, 5),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn add_bid_order_negative_interest() {
    deserialize_failure(
        SixArgCommand::new("AddBidOrder", "addressid", 1, -2, 3, 4, 5),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn add_bid_order_invalid_interest() {
    deserialize_failure(
        SixArgCommand::new("AddBidOrder", "addressid", 1, "BAD", 3, 4, 5),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn add_bid_order_negative_maturity() {
    deserialize_failure(
        SixArgCommand::new("AddBidOrder", "addressid", 1, 2, -3, 4, 5),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn add_bid_order_invalid_maturity() {
    deserialize_failure(
        SixArgCommand::new("AddBidOrder", "addressid", 1, 2, "BAD", 4, 5),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn add_bid_order_negative_fee() {
    deserialize_failure(
        SixArgCommand::new("AddBidOrder", "addressid", 1, 2, 3, -4, 5),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn add_bid_order_invalid_fee() {
    deserialize_failure(
        SixArgCommand::new("AddBidOrder", "addressid", 1, 2, 3, "BAD", 5),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn add_bid_order_negative_expiration() {
    deserialize_failure(
        SixArgCommand::new("AddBidOrder", "addressid", 1, 2, 3, 4, -5),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn add_bid_order_invalid_expiration() {
    deserialize_failure(
        SixArgCommand::new("AddBidOrder", "addressid", 1, 2, 3, 4, "BAD"),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn add_bid_order_missing_arg() {
    deserialize_failure(
        FiveArgCommand::new("AddBidOrder", "addressid", 1, 2, 3, 4),
        "Expecting expiration",
    );
    deserialize_failure(
        FourArgCommand::new("AddBidOrder", "addressid", 1, 2, 3),
        "Expecting fee",
    );
    deserialize_failure(
        ThreeArgCommand::new("AddBidOrder", "addressid", 1, 2),
        "Expecting maturity",
    );
    deserialize_failure(
        TwoArgCommand::new("AddBidOrder", "addressid", 1),
        "Expecting interest",
    );
    deserialize_failure(
        OneArgCommand::new("AddBidOrder", "addressid"),
        "Expecting amount",
    );
    deserialize_failure(ZeroArgCommand::new("AddBidOrder"), "Expecting addressId");
}

// AddOffer

#[test]
fn add_offer_accept() {
    let args = ThreeArgCommand::new("AddOffer", "askorder", "bidorder", 1);
    let args_upper = ThreeArgCommand {
        p1: "ASKORDER".into(),
        p2: "BIDORDER".into(),
        ..args.clone()
    };
    let expected = AddOffer {
        ask_order_id: "askorder".into(),
        bid_order_id: "bidorder".into(),
        expiration: 1.into(),
    };
    deserialize_success(args, expected.clone());
    deserialize_success(args_upper, expected);
}

#[test]
fn add_offer_case_insensitive() {
    let args = ThreeArgCommand::new("AdDoFfEr", "askorder", "bidorder", 1);
    let expected = AddOffer {
        ask_order_id: "askorder".into(),
        bid_order_id: "bidorder".into(),
        expiration: 1.into(),
    };
    deserialize_success(args, expected);
}

#[test]
fn add_offer_negative_expiration() {
    deserialize_failure(
        ThreeArgCommand::new("AddOffer", "ask", "bid", -2),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn add_offer_invalid_expiration() {
    deserialize_failure(
        ThreeArgCommand::new("AddOffer", "ask", "bid", "BAD"),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn add_offer_missing_arg() {
    deserialize_failure(
        TwoArgCommand::new("AddOffer", "ask", "bid"),
        "Expecting expiration",
    );
    deserialize_failure(
        OneArgCommand::new("AddOffer", "ask"),
        "Expecting bidOrderId",
    );
    deserialize_failure(ZeroArgCommand::new("AddOffer"), "Expecting askOrderId");
}

// AddDealOrder

#[test]
fn add_deal_order_accept() {
    let expected = AddDealOrder {
        offer_id: "offerid".into(),
        expiration: 1.into(),
    };
    deserialize_success(
        TwoArgCommand::new("AddDealOrder", "offerid", 1),
        expected.clone(),
    );
    deserialize_success(TwoArgCommand::new("AddDealOrder", "OFFERID", 1), expected);
}

#[test]
fn add_deal_order_case_insensitive() {
    let expected = AddDealOrder {
        offer_id: "offerid".into(),
        expiration: 1.into(),
    };
    deserialize_success(TwoArgCommand::new("AdDdEaLoRdEr", "offerid", 1), expected);
}

#[test]
fn add_deal_order_negative_expiration() {
    deserialize_failure(
        TwoArgCommand::new("AddDealOrder", "offerid", -1),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn add_deal_order_invalid_expiration() {
    deserialize_failure(
        TwoArgCommand::new("AddDealOrder", "offerid", "BAD"),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn add_deal_order_missing_arg() {
    deserialize_failure(
        OneArgCommand::new("AddDealOrder", "offerid"),
        "Expecting expiration",
    );
    deserialize_failure(ZeroArgCommand::new("AddDealOrder"), "Expecting offerId");
}

// CompleteDealOrder

#[test]
fn complete_deal_order_accept() {
    let expected = CompleteDealOrder {
        deal_order_id: "orderid".into(),
        transfer_id: "transferid".into(),
    };
    deserialize_success(
        TwoArgCommand::new("CompleteDealOrder", "orderid", "transferid"),
        expected.clone(),
    );
    deserialize_success(
        TwoArgCommand::new("CompleteDealOrder", "ORDERID", "TRANSFERID"),
        expected,
    );
}

#[test]
fn complete_deal_order_case_insensitive() {
    let expected = CompleteDealOrder {
        deal_order_id: "orderid".into(),
        transfer_id: "transferid".into(),
    };
    deserialize_success(
        TwoArgCommand::new("CoMpLeTeDeAlOrDer", "orderid", "transferid"),
        expected,
    );
}

#[test]
fn complete_deal_order_missing_arg() {
    deserialize_failure(
        OneArgCommand::new("CompleteDealOrder", "orderid"),
        "Expecting transferId",
    );
    deserialize_failure(
        ZeroArgCommand::new("CompleteDealOrder"),
        "Expecting dealOrderId",
    );
}

// LockDealOrder

#[test]
fn lock_deal_order_accept() {
    let expected = LockDealOrder {
        deal_order_id: "orderid".into(),
    };
    deserialize_success(
        OneArgCommand::new("LockDealOrder", "orderid"),
        expected.clone(),
    );
    deserialize_success(OneArgCommand::new("LockDealOrder", "ORDERID"), expected);
}

#[test]
fn lock_deal_order_case_insensitive() {
    let expected = LockDealOrder {
        deal_order_id: "orderid".into(),
    };
    deserialize_success(OneArgCommand::new("LoCkDeAlOrDeR", "orderid"), expected);
}

#[test]
fn lock_deal_order_missing_arg() {
    deserialize_failure(
        ZeroArgCommand::new("LockDealOrder"),
        "Expecting dealOrderId",
    );
}

// CloseDealOrder

#[test]
fn close_deal_order_accept() {
    let expected = CloseDealOrder {
        deal_order_id: "orderid".into(),
        transfer_id: "transferid".into(),
    };
    deserialize_success(
        TwoArgCommand::new("CloseDealOrder", "orderid", "transferid"),
        expected.clone(),
    );
    deserialize_success(
        TwoArgCommand::new("CloseDealOrder", "ORDERID", "TRANSFERID"),
        expected,
    );
}

#[test]
fn close_deal_order_case_insensitive() {
    let expected = CloseDealOrder {
        deal_order_id: "orderid".into(),
        transfer_id: "transferid".into(),
    };
    deserialize_success(
        TwoArgCommand::new("ClOsEdEaLoRdEr", "orderid", "transferid"),
        expected,
    );
}

#[test]
fn close_deal_order_missing_arg() {
    deserialize_failure(
        OneArgCommand::new("CloseDealOrder", "orderid"),
        "Expecting transferId",
    );
    deserialize_failure(
        ZeroArgCommand::new("CloseDealOrder"),
        "Expecting dealOrderId",
    );
}

// Exempt

#[test]
fn exempt_accept() {
    let expected = Exempt {
        deal_order_id: "orderid".into(),
        transfer_id: "transferid".into(),
    };
    deserialize_success(
        TwoArgCommand::new("Exempt", "orderid", "transferid"),
        expected.clone(),
    );
    deserialize_success(
        TwoArgCommand::new("Exempt", "ORDERID", "TRANSFERID"),
        expected,
    );
}

#[test]
fn exempt_case_insensitive() {
    let expected = Exempt {
        deal_order_id: "orderid".into(),
        transfer_id: "transferid".into(),
    };
    deserialize_success(
        TwoArgCommand::new("ExEmPt", "orderid", "transferid"),
        expected,
    );
}

#[test]
fn exempt_missing_arg() {
    deserialize_failure(
        OneArgCommand::new("Exempt", "orderid"),
        "Expecting transferId",
    );
    deserialize_failure(ZeroArgCommand::new("Exempt"), "Expecting dealOrderId");
}

// AddRepaymentOrder

#[test]
fn add_repayment_order_accept() {
    let expected = AddRepaymentOrder {
        deal_order_id: "orderid".into(),
        address_id: "addressid".into(),
        amount_str: "1".into(),
        expiration: 2.into(),
    };
    deserialize_success(
        FourArgCommand::new("AddRepaymentOrder", "orderid", "addressid", 1, 2),
        expected.clone(),
    );
    deserialize_success(
        FourArgCommand::new("AddRepaymentOrder", "ORDERID", "ADDRESSID", 1, 2),
        expected,
    );
}

#[test]
fn add_repayment_order_case_insensitive() {
    let expected = AddRepaymentOrder {
        deal_order_id: "orderid".into(),
        address_id: "addressid".into(),
        amount_str: "1".into(),
        expiration: 2.into(),
    };
    deserialize_success(
        FourArgCommand::new("AdDrEpAyMeNtOrDeR", "orderid", "addressid", 1, 2),
        expected,
    );
}

#[test]
fn add_repayment_order_negative_amount() {
    deserialize_failure(
        FourArgCommand::new("AddRepaymentOrder", "orderid", "addressid", -1, 2),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn add_repayment_order_invalid_amount() {
    deserialize_failure(
        FourArgCommand::new("AddRepaymentOrder", "orderid", "addressid", "BAD", 2),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn add_repayment_order_invalid_expiration() {
    deserialize_failure(
        FourArgCommand::new("AddRepaymentOrder", "orderid", "addressid", 1, "BAD"),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn add_repayment_order_missing_arg() {
    deserialize_failure(
        ThreeArgCommand::new("AddRepaymentOrder", "orderid", "addressid", 1),
        "Expecting expiration",
    );
    deserialize_failure(
        TwoArgCommand::new("AddRepaymentOrder", "orderid", "addressid"),
        "Expecting amount",
    );
    deserialize_failure(
        OneArgCommand::new("AddRepaymentOrder", "orderid"),
        "Expecting addressId",
    );
    deserialize_failure(
        ZeroArgCommand::new("AddRepaymentOrder"),
        "Expecting dealOrderId",
    );
}

// CompleteRepaymentOrder

#[test]
fn complete_repayment_order_accept() {
    let expected = CompleteRepaymentOrder {
        repayment_order_id: "repaymentid".into(),
    };
    deserialize_success(
        OneArgCommand::new("CompleteRepaymentOrder", "repaymentid"),
        expected.clone(),
    );
    deserialize_success(
        OneArgCommand::new("CompleteRepaymentOrder", "REPAYMENTID"),
        expected,
    );
}

#[test]
fn complete_repayment_order_case_insensitive() {
    let expected = CompleteRepaymentOrder {
        repayment_order_id: "repaymentid".into(),
    };
    deserialize_success(
        OneArgCommand::new("CoMpLeTeRePaYmEnToRdEr", "repaymentid"),
        expected,
    );
}

#[test]
fn complete_repayment_order_missing_arg() {
    deserialize_failure(
        ZeroArgCommand::new("CompleteRepaymentOrder"),
        "Expecting repaymentOrderId",
    );
}

// CloseRepaymentOrder

#[test]
fn close_repayment_order_accept() {
    let expected = CloseRepaymentOrder {
        repayment_order_id: "repaymentid".into(),
        transfer_id: "transferid".into(),
    };
    deserialize_success(
        TwoArgCommand::new("CloseRepaymentOrder", "repaymentid", "transferid"),
        expected.clone(),
    );
    deserialize_success(
        TwoArgCommand::new("CloseRepaymentOrder", "REPAYMENTID", "TRANSFERID"),
        expected,
    );
}

#[test]
fn close_repayment_order_case_insensitive() {
    let expected = CloseRepaymentOrder {
        repayment_order_id: "repaymentid".into(),
        transfer_id: "transferid".into(),
    };
    deserialize_success(
        TwoArgCommand::new("ClOsErEpAyMeNtOrDeR", "repaymentid", "transferid"),
        expected,
    );
}

#[test]
fn close_repayment_order_missing_arg() {
    deserialize_failure(
        OneArgCommand::new("CloseRepaymentOrder", "repaymentid"),
        "Expecting transferId",
    );
    deserialize_failure(
        ZeroArgCommand::new("CloseRepaymentOrder"),
        "Expecting repaymentOrderId",
    );
}

// CollectCoins

#[test]
fn collect_coins_accept() {
    let expected = CollectCoins {
        eth_address: "ethaddress".into(),
        amount: 1.into(),
        blockchain_tx_id: "blockchainid".into(),
    };

    deserialize_success(
        ThreeArgCommand::new("CollectCoins", "ethaddress", 1, "blockchainid"),
        expected.clone(),
    );
    deserialize_success(
        ThreeArgCommand::new("CollectCoins", "ETHADDRESS", 1, "BLOCKCHAINID"),
        expected,
    );
}

#[test]
fn collect_coins_case_insensitive() {
    let expected = CollectCoins {
        eth_address: "ethaddress".into(),
        amount: 1.into(),
        blockchain_tx_id: "blockchainid".into(),
    };

    deserialize_success(
        ThreeArgCommand::new("CoLlEcTcOiNs", "ethaddress", 1, "blockchainid"),
        expected,
    );
}

#[test]
fn collect_coins_negative_amount() {
    deserialize_failure(
        ThreeArgCommand::new("CollectCoins", "ethaddress", -1, "blockchainid"),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn collect_coins_invalid_amount() {
    deserialize_failure(
        ThreeArgCommand::new("CollectCoins", "ethaddress", "BAD", "blockchainid"),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn collect_coins_missing_arg() {
    deserialize_failure(
        TwoArgCommand::new("CollectCoins", "ethaddress", 1),
        "Expecting blockchainTxId",
    );
    deserialize_failure(
        OneArgCommand::new("CollectCoins", "ethaddress"),
        "Expecting amount",
    );
    deserialize_failure(ZeroArgCommand::new("CollectCoins"), "Expecting ethAddress");
}

// Housekeeping

#[test]
fn housekeeping_accept() {
    deserialize_success(
        OneArgCommand::new("Housekeeping", 1),
        CCCommand::Housekeeping(Housekeeping {
            block_idx: 1.into(),
        }),
    )
}

#[test]
fn housekeeping_case_insensitive() {
    deserialize_success(
        OneArgCommand::new("HoUsEkEePiNg", 1),
        CCCommand::Housekeeping(Housekeeping {
            block_idx: 1.into(),
        }),
    )
}

#[test]
fn housekeeping_negative_block_idx() {
    deserialize_failure(OneArgCommand::new("Housekeeping", -1), NEGATIVE_NUMBER_ERR);
}

#[test]
fn housekeeping_invalid_block_idx() {
    deserialize_failure(
        OneArgCommand::new("Housekeeping", "BAD"),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn housekeeping_rejects_missing_arg() {
    deserialize_failure(ZeroArgCommand::new("Housekeeping"), "Expecting blockIdx");
}

// RegisterDealOrder

#[test]
fn register_deal_order_accept() {
    deserialize_success(
        ElevenArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "1",
            "1",
            "1",
            "1",
            "signature",
            "pubkey",
            "dealorderid",
            "txid",
        ),
        CCCommand::RegisterDealOrder(RegisterDealOrder {
            ask_address_id: "askaddress".into(),
            bid_address_id: "bidaddress".into(),
            amount_str: "1".into(),
            interest: "1".into(),
            maturity: "1".into(),
            fee_str: "1".into(),
            expiration: 1.into(),
            fundraiser_signature: "signature".into(),
            fundraiser_public_key: "pubkey".into(),
            deal_order_id: "dealorderid".into(),
            blockchain_tx_id: "txid".into(),
        }),
    );
}

#[test]
fn register_deal_order_negative_amount_str() {
    deserialize_failure(
        ElevenArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "-1",
            "1",
            "1",
            "1",
            "1",
            "signature",
            "pubkey",
            "dealorderid",
            "txid",
        ),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn register_deal_order_invalid_amount_str() {
    deserialize_failure(
        ElevenArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "BAD",
            "1",
            "1",
            "1",
            "1",
            "signature",
            "pubkey",
            "dealorderid",
            "txid",
        ),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn register_deal_order_negative_interest() {
    deserialize_failure(
        ElevenArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "-1",
            "1",
            "1",
            "1",
            "signature",
            "pubkey",
            "dealorderid",
            "txid",
        ),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn register_deal_order_invalid_interest() {
    deserialize_failure(
        ElevenArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "BAD",
            "1",
            "1",
            "1",
            "signature",
            "pubkey",
            "dealorderid",
            "txid",
        ),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn register_deal_order_negative_maturity() {
    deserialize_failure(
        ElevenArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "1",
            "-1",
            "1",
            "1",
            "signature",
            "pubkey",
            "dealorderid",
            "txid",
        ),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn register_deal_order_invalid_maturity() {
    deserialize_failure(
        ElevenArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "1",
            "BAD",
            "1",
            "1",
            "signature",
            "pubkey",
            "dealorderid",
            "txid",
        ),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn register_deal_order_negative_fee_str() {
    deserialize_failure(
        ElevenArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "1",
            "1",
            "-1",
            "1",
            "signature",
            "pubkey",
            "dealorderid",
            "txid",
        ),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn register_deal_order_invalid_fee_str() {
    deserialize_failure(
        ElevenArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "1",
            "1",
            "BAD",
            "1",
            "signature",
            "pubkey",
            "dealorderid",
            "txid",
        ),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn register_deal_order_negative_expiration() {
    deserialize_failure(
        ElevenArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "1",
            "1",
            "1",
            "-1",
            "signature",
            "pubkey",
            "dealorderid",
            "txid",
        ),
        NEGATIVE_NUMBER_ERR,
    );
}

#[test]
fn register_deal_order_invalid_expiration() {
    deserialize_failure(
        ElevenArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "1",
            "1",
            "1",
            "BAD",
            "signature",
            "pubkey",
            "dealorderid",
            "txid",
        ),
        INVALID_NUMBER_FORMAT_ERR,
    );
}

#[test]
fn register_deal_order_rejects_missing_arg() {
    deserialize_failure(
        TenArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "1",
            "1",
            "1",
            "1",
            "signature",
            "pubkey",
            "dealorderid",
        ),
        "Expecting blockchainTxId",
    );
    deserialize_failure(
        NineArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "1",
            "1",
            "1",
            "1",
            "signature",
            "pubkey",
        ),
        "Expecting dealOrderId",
    );
    deserialize_failure(
        EightArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "1",
            "1",
            "1",
            "1",
            "signature",
        ),
        "Expecting fundraiserPublicKey",
    );
    deserialize_failure(
        SevenArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "1",
            "1",
            "1",
            "1",
        ),
        "Expecting fundraiserSignature",
    );
    deserialize_failure(
        SixArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "1",
            "1",
            "1",
        ),
        "Expecting expiration",
    );
    deserialize_failure(
        FiveArgCommand::new(
            "RegisterDealOrder",
            "askaddress",
            "bidaddress",
            "1",
            "1",
            "1",
        ),
        "Expecting fee",
    );
    deserialize_failure(
        FourArgCommand::new("RegisterDealOrder", "askaddress", "bidaddress", "1", "1"),
        "Expecting maturity",
    );
    deserialize_failure(
        ThreeArgCommand::new("RegisterDealOrder", "askaddress", "bidaddress", "1"),
        "Expecting interest",
    );
    deserialize_failure(
        TwoArgCommand::new("RegisterDealOrder", "askaddress", "bidaddress"),
        "Expecting amount",
    );
    deserialize_failure(
        OneArgCommand::new("RegisterDealOrder", "askaddress"),
        "Expecting bidAddressId",
    );
    deserialize_failure(
        ZeroArgCommand::new("RegisterDealOrder"),
        "Expecting askAddressId",
    );
}

fn make_fee(guid: &Guid, sighash: &SigHash, block: Option<u64>) -> (String, Vec<u8>) {
    let fee_id = Address::with_prefix_key(super::constants::FEE, guid.as_str());
    let fee = crate::protos::Fee {
        sighash: sighash.clone().into(),
        block: block.unwrap_or_default().to_string(),
    };
    (fee_id.to_string(), fee.to_bytes())
}

fn expect_set_state_entries(tx_ctx: &mut MockTransactionContext, entries: Vec<(String, Vec<u8>)>) {
    expect!(tx_ctx, set_state_entries where {
            let entries = entries.into_iter().sorted().collect_vec();
            move |e| {
                let s = itertools::sorted(e.clone()).collect_vec();
                for (entry, other) in entries.iter().zip(&s) {
                    if entry != other {
                        println!("Not equal! Expected {:?} -- Found {:?}", entry, other);
                        return false;
                    }
                }
                if entries.len() != s.len() {
                    println!("Unequal lengths! Expected {:?} -- Found {:?}", entries.len(), s.len());
                    return false;
                }
                true
            }
        }, returning |_| Ok(()));
}

fn expect_delete_state_entries(tx_ctx: &mut MockTransactionContext, entries: Vec<String>) {
    tx_ctx
        .expect_delete_state_entries()
        .once()
        .withf({
            let entries = entries.into_iter().sorted().collect_vec();
            move |e| {
                let s = itertools::sorted(e.clone()).collect_vec();
                for (entry, &other) in entries.iter().zip(&s) {
                    if entry != other {
                        println!("Not equal! Expected {:?} -- Found {:?}", entry, other);
                        return false;
                    }
                }
                if entries.len() != s.len() {
                    println!(
                        "Unequal lengths! Expected {:?} -- Found {:?}",
                        entries.len(),
                        s.len()
                    );
                    return false;
                }
                true
            }
        })
        .returning(|_| Ok(Vec::new()));
}

// ----- COMMAND EXECUTION TESTS -----

impl Default for CCTransactionHandler {
    fn default() -> Self {
        Self::new("")
    }
}

// TODO: replace with hex::encode() after PR#6 is merged
fn to_hex_string(bytes: &Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    strs.join("")
}

#[track_caller]
fn execute_success(
    command: impl CCTransaction,
    request: &TpProcessRequest,
    tx_ctx: &MockTransactionContext,
    ctx: &mut MockHandlerContext,
) {
    command.execute(request, tx_ctx, ctx).unwrap();
}

#[track_caller]
fn execute_failure(
    command: impl CCTransaction + ToGenericCommand,
    request: &TpProcessRequest,
    tx_ctx: &MockTransactionContext,
    ctx: &mut MockHandlerContext,
    expected_err: &str,
) {
    let result = command.execute(request, tx_ctx, ctx).unwrap_err();
    match result.downcast_ref::<CCApplyError>() {
        Some(CCApplyError::InvalidTransaction(s)) => {
            assert_eq!(s, expected_err);
        }
        _ => panic!("Expected an InvalidTransaction error"),
    };
}

fn charge_fee(tx_ctx: &mut MockTransactionContext, sighash: &SigHash) {
    let wallet_id = WalletId::from(sighash);
    let fee = TX_FEE.clone();
    expect!(tx_ctx, get balance at wallet_id -> Some(fee));
}

fn expect_get_state_entry(
    tx_ctx: &mut MockTransactionContext,
    id: impl Into<String>,
    ret: Option<impl Message + Default>,
    times: Option<usize>,
) {
    let id = id.into();
    let ret = ret.map(|m| m.to_bytes());
    tx_ctx
        .expect_get_state_entry()
        .times(times.unwrap_or(1))
        .withf(move |m| m == &id)
        .return_once({
            let ret = ret;
            |_| Ok(ret)
        });
}

// --- Housekeeping ---

#[test]
fn housekeeping_reward_in_chain() {
    init_logs();

    // Housekeeeping with block idx = 0
    let command = Housekeeping {
        block_idx: BlockNum(0),
    };

    // Chain tip is far ahead
    let request = TpProcessRequest {
        tip: u64::from((CONFIRMATION_COUNT * 2 + BLOCK_REWARD_PROCESSING_COUNT) * 4),
        ..Default::default()
    };
    let mut tx_ctx = MockTransactionContext::default();

    // get_state_entry should be called on the processed_block_idx address, and we will return
    // CONFIRMATION_COUNT * 2 + BLOCK_REWARD_PROCESSING_COUNT, which will force housekeeping to run
    expect!(tx_ctx,
        get_state_entry(k if k == PROCESSED_BLOCK_IDX.as_str())
        -> Ok(Some(
            Integer::from(CONFIRMATION_COUNT * 2 + BLOCK_REWARD_PROCESSING_COUNT).to_string().into_bytes()
        ))
    );

    // pretend update1 is not set
    let mut ctx = MockHandlerContext::default();
    expect!(ctx,
        get_setting(k if k == "sawtooth.validator.update1") -> Ok(None)
    );

    let height_start = CONFIRMATION_COUNT * 2 + BLOCK_REWARD_PROCESSING_COUNT + 1;
    let height_end = height_start + BLOCK_REWARD_PROCESSING_COUNT;

    let mut signers = vec![];

    // housekeeping tries to get the signatures for the blocks
    // from height_start to height_end in order to issue mining rewards
    // return a dummy signer
    for height in height_start.0..height_end.0 {
        let signer = format!("signer{}", height);
        signers.push(signer.clone());
        expect!(tx_ctx,
            get_sig_by_num(h if *h == height) -> Ok(signer)
        );
    }

    let reward_amount = REWARD_AMOUNT.clone();

    for (idx, signer) in signers.clone().into_iter().enumerate() {
        let wallet_id = WalletId::from(&SigHash(sha512_id(signer.as_bytes())));

        // the first signer has no wallet, the rest have an existing balance of `idx`
        let balance = if idx == 0 { None } else { Some(idx as u64) };

        log::info!("starting balance = {:?}", balance);

        // housekeeping should try to fetch the current wallet for each signer
        // return the balance above
        expect!(tx_ctx, get balance at wallet_id -> balance);

        // we expect the wallet to have an updated balance of reward_amount + old balance
        let amount_expected = reward_amount.clone() + balance.unwrap_or(0);

        log::info!("expect end wallet = {:?}", amount_expected);
        // housekeeping should try to set the state to update
        // the wallet balance with the reward added
        expect!(
            tx_ctx,
            set balance at wallet_id to amount_expected
        );
    }

    // housekeeping should then set the processed_block_idx to the last processed block height
    // which in this case is height_end - 1
    expect!(tx_ctx, set_state_entry(
            addr if addr == PROCESSED_BLOCK_IDX.as_str(),
            state if state == &(height_end - 1).unwrap().to_string().into_bytes()
        ) -> Ok(())
    );

    // run housekeeping
    execute_success(command, &request, &tx_ctx, &mut ctx);
}

#[test]
fn housekeeping_reward_fork() {
    init_logs();

    // Housekeeeping with block idx = 0
    let command = Housekeeping {
        block_idx: BlockNum(0),
    };

    let last_processed = CONFIRMATION_COUNT * 2 + BLOCK_REWARD_PROCESSING_COUNT;

    // Chain tip is far ahead
    let request = TpProcessRequest {
        tip: u64::from(last_processed * 4),
        block_signature: "headblocksig".into(),
        ..Default::default()
    };
    let mut tx_ctx = MockTransactionContext::default();

    // get_state_entry should be called on the processed_block_idx address, and we will return
    // CONFIRMATION_COUNT * 2 + BLOCK_REWARD_PROCESSING_COUNT, which will force housekeeping to run
    expect!(tx_ctx,
        get_state_entry(k if k == PROCESSED_BLOCK_IDX.as_str())
        -> Ok(Some(
            Integer::from(last_processed).to_string().into_bytes()
        ))
    );

    // pretend update1 is not set
    let mut ctx = MockHandlerContext::default();
    expect!(ctx,
        get_setting(k if k == "sawtooth.validator.update1") -> Ok(None)
    );

    // the get_reward_block_signatures path iterates in reverse inclusively, so if last_processed = 5
    // and BLOCK_REWARD_PROCESSING_COUNT = 5, then the bounds
    // should be [10, 6] i.e. [last_processed + BLOCK_REWARD_PROCESSING_COUNT, last_processed+1]
    let last_pred = last_processed + 1;
    let first_pred = last_processed + BLOCK_REWARD_PROCESSING_COUNT;

    log::warn!("{}..{}", last_pred, first_pred);

    let signers: Vec<String> = (last_pred.0..first_pred.0)
        .map(|i| format!("signer{}", i))
        .collect();

    let signers_ = signers.clone();

    // housekeeping tries to get the signatures for the blocks
    // iterating backwards from first_pred to last_pred
    expect!(tx_ctx,
        get_reward_block_signatures(id if id == "headblocksig", first if *first == first_pred, last if *last == last_pred) -> Ok(
            signers_
        )
    );

    let reward_amount = REWARD_AMOUNT.clone();

    for (idx, signer) in signers.into_iter().enumerate() {
        let wallet_id = WalletId::from(&SigHash(sha512_id(signer.as_bytes())));
        let wallet_id_ = wallet_id.clone();

        // the first signer has no wallet, the rest have an existing balance of `idx`
        let balance = if idx == 0 { None } else { Some(idx as u64) };

        log::info!("starting balance = {:?}", balance);

        // housekeeping should try to fetch the current wallet for each signer
        // return the balance above
        expect!(
            tx_ctx,
            get_state_entry(k if k == wallet_id.as_str()) -> Ok(wallet_with(balance))
        );

        // we expect the wallet to have an updated balance of reward_amount + old balance
        let wallet_expected = crate::protos::Wallet {
            amount: (reward_amount.clone() + balance.unwrap_or(0)).to_string(),
        };
        let state_expected = wallet_expected.to_bytes();

        log::info!("expect end wallet = {:?}", wallet_expected);
        // housekeeping should try to set the state to update
        // the wallet balance with the reward added
        expect!(
            tx_ctx,
            set_state_entry(
                addr if addr == wallet_id_.as_str(),
                state if state == &state_expected
            ) -> Ok(())
        );
    }

    // housekeeping should then set the processed_block_idx to the last processed block height
    // which in this case is height_end - 1
    expect!(tx_ctx, set_state_entry(
            addr if addr == PROCESSED_BLOCK_IDX.as_str(),
            state if state == &(first_pred).to_string().into_bytes()
        ) -> Ok(())
    );

    // run housekeeping
    execute_success(command, &request, &tx_ctx, &mut ctx);
}

#[test]
fn housekeeping_not_enough_confirmations() {
    init_logs();

    // Housekeeeping with block idx = 0
    let command = Housekeeping {
        block_idx: BlockNum(0),
    };

    // no blocks have been processed
    let last_processed = 0;

    // Chain tip is not quite at the threshold for running because
    // the blocks have not yet gotten enough confirmations
    let request = TpProcessRequest {
        tip: u64::from(BLOCK_REWARD_PROCESSING_COUNT + 1),
        block_signature: "headblocksig".into(),
        ..Default::default()
    };
    let mut tx_ctx = MockTransactionContext::default();

    expect!(tx_ctx,
        get_state_entry(k if k == PROCESSED_BLOCK_IDX.as_str())
        -> Ok(Some(
            Integer::from(last_processed).to_string().into_bytes()
        ))
    );

    let mut ctx = MockHandlerContext::default();

    // execute housekeeping
    execute_success(command, &request, &tx_ctx, &mut ctx);
}

fn expect_get_last_processed_block(
    tx_ctx: &mut MockTransactionContext,
    last_processed: impl Into<Integer> + Send + 'static,
) {
    expect!(tx_ctx,
        get_state_entry(k if k == PROCESSED_BLOCK_IDX.as_str())
        -> Ok(Some(
            last_processed.into().to_string().into_bytes()
        ))
    );
}

fn expect_set_last_processed_block(
    tx_ctx: &mut MockTransactionContext,
    last_processed: impl Into<Integer> + Send + 'static,
) {
    let last_processed = last_processed.into().to_string().into_bytes();
    tx_ctx
        .expect_set_state_entry()
        .once()
        .withf(move |address, value| {
            address == PROCESSED_BLOCK_IDX.as_str() && value == &last_processed
        })
        .returning(|_, _| Ok(()));
}

fn expect_get_state_entries_by_prefix<M: MessageExt<M> + Send>(
    tx_ctx: &mut MockTransactionContext,
    tip_number: u64,
    prefix: &str,
    entries: Vec<(&str, M)>,
) {
    let tip_id = string!("#", (tip_number - 1).to_string());
    let prefix = prefix.to_string();
    let ret = entries
        .into_iter()
        .map(|(suffix, proto)| {
            let address = string!(&prefix, suffix);
            (address, proto.to_bytes())
        })
        .collect();
    tx_ctx
        .expect_get_state_entries_by_prefix()
        .with(
            mockall::predicate::function(move |tip: &str| tip.starts_with(&tip_id)),
            mockall::predicate::function(move |address| address == prefix.clone()),
        )
        .once()
        .return_once(|_, _| Ok(ret));
}

fn expect_get_setting(ctx: &mut MockHandlerContext, key: &str, value: Option<&str>) {
    let key = String::from(key);
    let value = value.map(String::from);

    ctx.expect_get_setting()
        .withf(move |k| k == key)
        .once()
        .return_once(move |_| Ok(value));
}

#[test]
fn housekeeping_within_block_reward_count() {
    use std::convert::TryInto;
    init_logs();

    // Housekeeeping with block idx = 0
    let command = Housekeeping {
        block_idx: BlockNum(0),
    };

    // pretend we've issued some rewards already
    let last_processed = 4 * CONFIRMATION_COUNT + BLOCK_REWARD_PROCESSING_COUNT;

    // Chain tip is not quite at the threshold for running because
    // fewer than BLOCK_REWARD_PROCESSING_COUNT additional blocks have been processed
    let request = TpProcessRequest {
        tip: (last_processed + BLOCK_REWARD_PROCESSING_COUNT.0 - 1)
            .0
            .try_into()
            .unwrap(),
        block_signature: "headblocksig".into(),
        ..Default::default()
    };
    let mut tx_ctx = MockTransactionContext::default();

    // Housekeeping should check the last processed block, then bail out
    expect_get_last_processed_block(&mut tx_ctx, last_processed);

    let mut ctx = MockHandlerContext::default();

    // execute housekeeping
    execute_success(command, &request, &tx_ctx, &mut ctx);
}

#[test]
fn housekeeping_removes_expired_entries() {
    init_logs();

    let command = Housekeeping {
        block_idx: BlockNum(200),
    };

    let sighash = SigHash::from("sighash");

    let last_processed = BlockNum(130);

    let request = TpProcessRequest {
        tip: 250,
        block_signature: "headblocksig".into(),
        ..Default::default()
    };

    let mut tx_ctx = MockTransactionContext::default();

    let ask = string!(NAMESPACE_PREFIX, ASK_ORDER);
    let bid = string!(NAMESPACE_PREFIX, BID_ORDER);
    let offer = string!(NAMESPACE_PREFIX, OFFER);
    let deal = string!(NAMESPACE_PREFIX, DEAL_ORDER);
    let repay = string!(NAMESPACE_PREFIX, REPAYMENT_ORDER);
    let fee = string!(NAMESPACE_PREFIX, FEE);

    expect_get_last_processed_block(&mut tx_ctx, last_processed);

    {
        expect_get_state_entries_by_prefix(
            &mut tx_ctx,
            request.tip,
            &ask,
            vec![
                (
                    "shouldbedeleted",
                    protos::AskOrder {
                        expiration: 50,
                        block: "100".into(),
                        ..Default::default()
                    },
                ),
                (
                    "shouldnotbedeleted",
                    protos::AskOrder {
                        expiration: 500,
                        block: "220".into(),
                        ..Default::default()
                    },
                ),
            ],
        );

        expect_delete_state_entry(&mut tx_ctx, string!(&ask, "shouldbedeleted"));
    }

    {
        expect_get_state_entries_by_prefix(
            &mut tx_ctx,
            request.tip,
            &bid,
            vec![
                (
                    "shouldbedeleted",
                    protos::BidOrder {
                        block: "110".into(),
                        expiration: 30,
                        ..Default::default()
                    },
                ),
                (
                    "shouldnotbedeleted",
                    protos::BidOrder {
                        block: "220".into(),
                        expiration: 300,
                        ..Default::default()
                    },
                ),
            ],
        );

        expect_delete_state_entry(&mut tx_ctx, string!(&bid, "shouldbedeleted"));
    }

    {
        expect_get_state_entries_by_prefix(
            &mut tx_ctx,
            request.tip,
            &offer,
            vec![
                (
                    "shouldbedeleted",
                    protos::Offer {
                        block: "110".into(),
                        expiration: 30,
                        ..Default::default()
                    },
                ),
                (
                    "shouldnotbedeleted",
                    protos::Offer {
                        block: "220".into(),
                        expiration: 300,
                        ..Default::default()
                    },
                ),
            ],
        );

        expect_delete_state_entry(&mut tx_ctx, string!(&offer, "shouldbedeleted"));
    }

    {
        expect_get_state_entries_by_prefix(
            &mut tx_ctx,
            request.tip,
            &deal,
            vec![
                (
                    "shouldbedeleted",
                    protos::DealOrder {
                        block: "110".into(),
                        expiration: 30,
                        ..Default::default()
                    },
                ),
                (
                    "shouldnotbedeleted",
                    protos::DealOrder {
                        block: "220".into(),
                        expiration: 300,
                        ..Default::default()
                    },
                ),
            ],
        );

        expect_delete_state_entry(&mut tx_ctx, string!(&deal, "shouldbedeleted"));
    }

    {
        expect_get_state_entries_by_prefix(
            &mut tx_ctx,
            request.tip,
            &repay,
            vec![
                (
                    "shouldbedeleted",
                    protos::RepaymentOrder {
                        block: "110".into(),
                        expiration: 30,
                        ..Default::default()
                    },
                ),
                (
                    "shouldnotbedeleted",
                    protos::RepaymentOrder {
                        block: "220".into(),
                        expiration: 300,
                        ..Default::default()
                    },
                ),
            ],
        );

        expect_delete_state_entry(&mut tx_ctx, string!(&repay, "shouldbedeleted"));
    }

    {
        expect_get_state_entries_by_prefix::<protos::Fee>(&mut tx_ctx, request.tip, &fee, vec![]);
    }

    tx_ctx
        .expect_get_reward_block_signatures()
        .once()
        .return_once(|_, _, _| Ok(vec![]));

    expect_set_last_processed_block(&mut tx_ctx, command.block_idx);

    let mut ctx = MockHandlerContext::default();
    expect!(
        ctx, sighash -> sighash
    );

    expect_get_setting(&mut ctx, "sawtooth.gateway.sighash", Some(sighash.as_str()));

    command.execute(&request, &tx_ctx, &mut ctx).unwrap();
}

const NONE: Option<protos::Wallet> = None;

#[test]
fn register_deal_order_success() {
    init_logs();
    let key = Box::new(
        sawtooth_sdk::signing::secp256k1::Secp256k1PrivateKey::from_hex(
            "b508a759f375ae5bf72662e3ccb67d5d83dbc2290346dc7c560fe35017380f64",
        )
        .unwrap(),
    );

    let context = Box::new(sawtooth_sdk::signing::secp256k1::Secp256k1Context::new());
    let signer = sawtooth_sdk::signing::Signer::new_boxed(context, key);
    let fundraiser_sighash =
        SigHash::from_public_key(&signer.get_public_key().unwrap().as_hex()).unwrap();
    let investor_sighash = SigHash::from("investorsighash");

    let ask_address_id = "askaddress".to_owned();
    let bid_address_id = "bidaddress".to_owned();
    let amount_str = "1".to_owned();
    let interest = "1".to_owned();
    let maturity = "1".to_owned();
    let fee_str = "1".to_owned();
    let expiration: BlockNum = 1000.into();

    let first_three = [
        ask_address_id.clone(),
        bid_address_id.clone(),
        amount_str.clone(),
    ]
    .join("");
    let signature = [
        first_three,
        interest.clone(),
        maturity.clone(),
        fee_str.clone(),
        expiration.to_string(),
    ]
    .join(",");

    let fundraiser_signature = signer.sign(&signature.as_bytes()).unwrap();

    let deal_order_id = [
        NAMESPACE_PREFIX.clone().as_str(),
        DEAL_ORDER,
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefbeef",
    ]
    .join("");

    let command = RegisterDealOrder {
        ask_address_id: ask_address_id.clone(),
        bid_address_id: bid_address_id.clone(),
        amount_str: amount_str.clone(),
        interest: interest.clone(),
        maturity: maturity.clone(),
        fee_str: fee_str.clone(),
        expiration: expiration.clone(),
        fundraiser_signature,
        fundraiser_public_key: signer.get_public_key().unwrap().as_hex(),
        deal_order_id: deal_order_id.clone(),
        blockchain_tx_id: "txid".into(),
    };

    let ask_address = protos::Address {
        blockchain: "ethereum".into(),
        value: "investor_address".into(),
        network: "rinkeby".into(),
        sighash: investor_sighash.clone().into(),
    };

    let bid_address = protos::Address {
        sighash: fundraiser_sighash.clone().into(),
        ..ask_address.clone()
    };

    let request = TpProcessRequest {
        tip: 2,
        ..Default::default()
    };

    let mut tx_ctx = MockTransactionContext::default();
    let mut ctx = MockHandlerContext::default();

    expect_get_state_entry(&mut tx_ctx, command.deal_order_id.as_str(), NONE, None);

    expect_get_state_entry(
        &mut tx_ctx,
        command.ask_address_id.clone(),
        Some(ask_address.clone()),
        None,
    );

    expect_get_state_entry(
        &mut tx_ctx,
        command.bid_address_id.clone(),
        Some(bid_address.clone()),
        None,
    );

    let transfer_id_key = [
        ask_address.blockchain.clone(),
        command.blockchain_tx_id.clone(),
        ask_address.network.clone(),
    ]
    .join("");

    let transfer_id = Address::with_prefix_key(TRANSFER, &transfer_id_key);

    expect_get_state_entry(&mut tx_ctx, transfer_id.clone(), NONE, None);

    let guid = Guid::from("guid");
    expect!(ctx, sighash -> investor_sighash);
    expect!(ctx, guid -> guid);

    let wallet_id = investor_sighash.to_wallet_id();

    let fundraiser_wallet_id = fundraiser_sighash.to_wallet_id();

    let fee = TX_FEE.clone();

    expect!(tx_ctx, get balance at wallet_id -> Some(fee));
    expect!(tx_ctx, get balance at fundraiser_wallet_id -> Some(1));
    expect!(ctx, verify(_) -> Ok(()));

    let deal_order = protos::DealOrder {
        blockchain: ask_address.blockchain.clone(),
        src_address: ask_address_id.clone(),
        dst_address: bid_address_id.clone(),
        amount: amount_str.clone(),
        interest: interest.clone(),
        maturity: maturity.clone(),
        fee: fee_str.clone(),
        expiration: 1000,
        block: 1.to_string(),
        sighash: fundraiser_sighash.clone().into(),
        ..Default::default()
    };
    let transfer = protos::Transfer {
        blockchain: deal_order.blockchain.clone(),
        src_address: deal_order.src_address.clone(),
        dst_address: deal_order.dst_address.clone(),
        order: deal_order_id.clone().into(),
        amount: amount_str.clone(),
        processed: false,
        block: 1.to_string(),
        sighash: investor_sighash.clone().into(),
        tx: command.blockchain_tx_id.clone(),
    };

    expect_set_state_entries(
        &mut tx_ctx,
        vec![
            (wallet_id.to_string(), wallet_with(Some(1)).unwrap()),
            (
                fundraiser_wallet_id.to_string(),
                wallet_with(Some(0)).unwrap(),
            ),
            (deal_order_id.to_string(), deal_order.to_bytes()),
            (transfer_id.to_string(), transfer.to_bytes()),
            make_fee(&guid, &investor_sighash, Some(1)),
        ],
    );

    execute_success(command, &request, &tx_ctx, &mut ctx);
}
