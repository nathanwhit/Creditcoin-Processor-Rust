#![cfg(any(test, feature = "integration-testing"))]

use crate::ext::MessageExt;
use crate::handler::types::{Address, Guid, SigHash};
use crate::handler::*;
use sawtooth_sdk::signing::{create_context, secp256k1::Secp256k1PrivateKey, Signer};
use serde::Serialize;

macro_rules! command {
    ($num: ident $(,)? $($param: ident),*) => {
        paste::paste! {
            #[derive(Serialize, PartialEq, Clone)]
            pub struct [<$num ArgCommand>] {
                pub v: String,
                $(
                    pub [<$param:lower>] : String
                ),*
            }

            impl [<$num ArgCommand>] {
                pub fn new<$($param : serde::Serialize + std::fmt::Display),*>(v: impl Into<String>, $([<$param:lower>]: $param),*) -> Self {
                    Self {
                        v: v.into(),
                        $([<$param:lower>] : [<$param:lower>].to_string()),*
                    }
                }
            }

            impl<$($param : serde::Serialize + std::fmt::Display),* > From<(&str, $($param),*)> for [<$num ArgCommand>] {
                fn from((v, $([<$param:lower>]),*): (&str, $($param),*)) -> Self {
                    Self {
                        v: v.into(),
                        $([<$param:lower>] : [<$param:lower>].to_string()),*
                    }
                }
            }
        }
    };
}

command!(Zero);
command!(One, P1);
command!(Two, P1, P2);
command!(Three, P1, P2, P3);
command!(Four, P1, P2, P3, P4);
command!(Five, P1, P2, P3, P4, P5);
command!(Six, P1, P2, P3, P4, P5, P6);

pub trait ToGenericCommand {
    type GenericCommand: Serialize;
    fn to_generic_command(self) -> Self::GenericCommand;
}

impl ToGenericCommand for SendFunds {
    type GenericCommand = TwoArgCommand;
    fn to_generic_command(self) -> <Self as ToGenericCommand>::GenericCommand {
        let SendFunds { amount, sighash } = self;
        TwoArgCommand::new("SendFunds", amount.to_string(), sighash.0)
    }
}

impl ToGenericCommand for RegisterAddress {
    type GenericCommand = ThreeArgCommand;
    fn to_generic_command(self) -> ThreeArgCommand {
        let RegisterAddress {
            blockchain,
            address,
            network,
        } = self;
        ThreeArgCommand::new("RegisterAddress", blockchain, address, network)
    }
}
impl ToGenericCommand for RegisterTransfer {
    type GenericCommand = ThreeArgCommand;
    fn to_generic_command(self) -> ThreeArgCommand {
        let RegisterTransfer {
            gain,
            order_id,
            blockchain_tx_id,
        } = self;
        ThreeArgCommand::new(
            "RegisterTransfer",
            gain.to_string(),
            order_id,
            blockchain_tx_id,
        )
    }
}
impl ToGenericCommand for AddAskOrder {
    type GenericCommand = SixArgCommand;
    fn to_generic_command(self) -> SixArgCommand {
        let AddAskOrder {
            address_id,
            amount_str,
            interest,
            maturity,
            fee_str,
            expiration,
        } = self;
        SixArgCommand::new(
            "AddAskOrder",
            address_id,
            amount_str,
            interest,
            maturity,
            fee_str,
            expiration,
        )
    }
}
impl ToGenericCommand for AddBidOrder {
    type GenericCommand = SixArgCommand;
    fn to_generic_command(self) -> SixArgCommand {
        let AddBidOrder {
            address_id,
            amount_str,
            interest,
            maturity,
            fee_str,
            expiration,
        } = self;
        SixArgCommand::new(
            "AddBidOrder",
            address_id,
            amount_str,
            interest,
            maturity,
            fee_str,
            expiration,
        )
    }
}
impl ToGenericCommand for AddOffer {
    type GenericCommand = ThreeArgCommand;
    fn to_generic_command(self) -> ThreeArgCommand {
        let AddOffer {
            ask_order_id,
            bid_order_id,
            expiration,
        } = self;
        ThreeArgCommand::new("AddOffer", ask_order_id, bid_order_id, expiration)
    }
}
impl ToGenericCommand for AddDealOrder {
    type GenericCommand = TwoArgCommand;
    fn to_generic_command(self) -> TwoArgCommand {
        let AddDealOrder {
            offer_id,
            expiration,
        } = self;
        TwoArgCommand::new("AddDealOrder", offer_id, expiration)
    }
}
impl ToGenericCommand for CompleteDealOrder {
    type GenericCommand = TwoArgCommand;
    fn to_generic_command(self) -> TwoArgCommand {
        let CompleteDealOrder {
            deal_order_id,
            transfer_id,
        } = self;
        TwoArgCommand::new("CompleteDealOrder", deal_order_id, transfer_id)
    }
}
impl ToGenericCommand for LockDealOrder {
    type GenericCommand = OneArgCommand;
    fn to_generic_command(self) -> OneArgCommand {
        let LockDealOrder { deal_order_id } = self;
        OneArgCommand::new("LockDealOrder", deal_order_id)
    }
}
impl ToGenericCommand for CloseDealOrder {
    type GenericCommand = TwoArgCommand;
    fn to_generic_command(self) -> TwoArgCommand {
        let CloseDealOrder {
            deal_order_id,
            transfer_id,
        } = self;
        TwoArgCommand::new("CloseDealOrder", deal_order_id, transfer_id)
    }
}
impl ToGenericCommand for Exempt {
    type GenericCommand = TwoArgCommand;
    fn to_generic_command(self) -> TwoArgCommand {
        let Exempt {
            deal_order_id,
            transfer_id,
        } = self;
        TwoArgCommand::new("Exempt", deal_order_id, transfer_id)
    }
}
impl ToGenericCommand for AddRepaymentOrder {
    type GenericCommand = FourArgCommand;
    fn to_generic_command(self) -> FourArgCommand {
        let AddRepaymentOrder {
            deal_order_id,
            address_id,
            amount_str,
            expiration,
        } = self;
        FourArgCommand::new(
            "AddRepaymentOrder",
            deal_order_id,
            address_id,
            amount_str,
            expiration,
        )
    }
}
impl ToGenericCommand for CompleteRepaymentOrder {
    type GenericCommand = OneArgCommand;
    fn to_generic_command(self) -> OneArgCommand {
        let CompleteRepaymentOrder { repayment_order_id } = self;
        OneArgCommand::new("CompleteRepaymentOrder", repayment_order_id)
    }
}
impl ToGenericCommand for CloseRepaymentOrder {
    type GenericCommand = TwoArgCommand;
    fn to_generic_command(self) -> TwoArgCommand {
        let CloseRepaymentOrder {
            repayment_order_id,
            transfer_id,
        } = self;
        TwoArgCommand::new("CloseRepaymentOrder", repayment_order_id, transfer_id)
    }
}
impl ToGenericCommand for CollectCoins {
    type GenericCommand = ThreeArgCommand;
    fn to_generic_command(self) -> ThreeArgCommand {
        let CollectCoins {
            eth_address,
            amount,
            blockchain_tx_id,
        } = self;
        ThreeArgCommand::new(
            "CollectCoins",
            eth_address,
            amount.to_string(),
            blockchain_tx_id,
        )
    }
}

pub fn signer_with_secret(secret: &str) -> Signer<'static> {
    let private_key = Secp256k1PrivateKey::from_hex(secret.trim()).unwrap();

    let context = create_context("secp256k1").unwrap();
    Signer::new_boxed(context, Box::new(private_key))
}

pub fn make_fee(guid: &Guid, sighash: &SigHash, block: Option<u64>) -> (String, Vec<u8>) {
    let fee_id = Address::with_prefix_key(crate::handler::constants::FEE, guid.as_str());
    let fee = crate::protos::Fee {
        sighash: sighash.clone().into(),
        block: block.unwrap_or_default().to_string(),
    };
    (fee_id.to_string(), fee.to_bytes())
}

pub type Nonce = [u8; 16];

pub fn make_nonce() -> Nonce {
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use std::convert::TryInto;
    let nonce: Vec<_> = rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(16)
        .collect();
    nonce[..16].try_into().unwrap()
}

pub fn address_id_for(address: &str) -> Address {
    Address::with_prefix_key(
        crate::handler::constants::ADDR,
        &crate::string!("ethereum", address, "rinkeby"),
    )
}

pub fn register_address_for(value: &str) -> RegisterAddress {
    RegisterAddress {
        blockchain: "ethereum".into(),
        address: value.into(),
        network: "rinkeby".into(),
    }
}

pub fn address_for(value: &str, sighash: &SigHash) -> crate::protos::Address {
    crate::protos::Address {
        blockchain: "ethereum".into(),
        value: value.into(),
        network: "rinkeby".into(),
        sighash: sighash.to_string(),
    }
}
