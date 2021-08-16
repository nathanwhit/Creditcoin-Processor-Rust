#![cfg(all(test, feature = "mock"))]

mod add_ask_order;
mod add_bid_order;
mod add_deal_order;
mod add_offer;
mod add_repayment_order;
mod close_deal_order;
mod close_repayment_order;
mod complete_deal_order;
mod complete_repayment_order;
mod exempt;
mod lock_deal_order;
mod register_address;
mod register_transfer;
mod send_funds;

use super::mocked::{MockSettings, MockTransactionContext};
use crate::test_utils::*;
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
pub use crate::handler::types::{Guid, WalletId};
pub use crate::handler::utils::{calc_interest, sha512_id};
pub use crate::{protos, string};

use super::*;
use crate::handler::context::mocked::MockHandlerContext;
use crate::handler::types::{Address, BlockNum, Credo, CurrencyAmount, TxnResult};
use crate::handler::AddAskOrder;
use crate::handler::AddBidOrder;
use crate::handler::AddDealOrder;
use crate::handler::AddOffer;
use crate::handler::AddRepaymentOrder;
use crate::handler::CloseDealOrder;
use crate::handler::CloseRepaymentOrder;
use crate::handler::CollectCoins;
use crate::handler::CompleteDealOrder;
use crate::handler::CompleteRepaymentOrder;
use crate::handler::Exempt;
use crate::handler::LockDealOrder;
use crate::handler::RegisterAddress;
use crate::handler::RegisterTransfer;
use crate::handler::SendFunds;
use crate::handler::{CCCommand, CCTransaction, CCTransactionHandler, Housekeeping};
