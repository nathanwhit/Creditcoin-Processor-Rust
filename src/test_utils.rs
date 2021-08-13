#![cfg(any(test, feature = "integration-testing"))]
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use crate::ext::MessageExt;
use crate::handler::constants::{ADDR, TRANSFER};
use crate::handler::types::{Address, Credo, CurrencyAmount, Guid, SigHash, State};
use crate::{handler::*, protos, string};
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
    let private_key = Secp256k1PrivateKey::from_hex(&secret.trim()).unwrap();

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Investor,
    Fundraiser,
    Collector,
    Any,
}

#[derive(Clone, PartialEq)]
pub struct Identity {
    pub secret: String,
    pub sighash: SigHash,
}

impl Identity {
    pub fn signer(&self) -> Signer<'static> {
        signer_with_secret(&self.secret)
    }
}

impl fmt::Debug for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Identity")
            .field("sighash", &self.sighash)
            .finish_non_exhaustive()
    }
}

impl Identity {
    pub fn new(secret: &str) -> Self {
        let secret = secret.to_owned();
        let signer = signer_with_secret(&secret);
        let sighash = SigHash::from(&signer);
        Self { secret, sighash }
    }
}

#[derive(Debug, Clone)]
pub struct TestInfo {
    pub investor: Option<Identity>,
    pub fundraiser: Option<Identity>,
    pub collector: Option<Identity>,
    pub primary: Identity,
    pub tx_fee: Credo,
    pub state: HashMap<Address, State>,
}

impl TestInfo {
    pub fn identity_for(&self, role: Role) -> Option<Identity> {
        match role {
            Role::Investor => self.investor.clone(),
            Role::Fundraiser => self.fundraiser.clone(),
            Role::Collector => self.collector.clone(),
            Role::Any => Some(self.primary.clone()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TestCommand {
    pub identity: Identity,
    pub guid: Guid,
    pub command: CCCommand,
    pub role: Role,
}

impl TestCommand {
    pub fn new(command: CCCommand, role: Role, identity: Identity) -> Self {
        let guid = Guid::from(make_nonce());
        Self {
            identity,
            command,
            guid,
            role,
        }
    }
}

pub trait HasDependencies {
    fn dependencies(&self, info: &TestInfo) -> Option<Vec<TestCommand>>;
}

impl HasDependencies for SendFunds {
    fn dependencies(&self, info: &TestInfo) -> Option<Vec<TestCommand>> {
        None
    }
}

impl HasDependencies for RegisterAddress {
    fn dependencies(&self, info: &TestInfo) -> Option<Vec<TestCommand>> {
        None
    }
}

impl HasDependencies for RegisterTransfer {
    fn dependencies(&self, info: &TestInfo) -> Option<Vec<TestCommand>> {
        todo!()
    }
}

pub trait HasState {
    fn resulting_state(&self, info: &TestInfo) -> (Address, State);
}

impl HasState for TestCommand {
    fn resulting_state(&self, info: &TestInfo) -> (Address, State) {
        match &self.command {
            CCCommand::SendFunds(_) => todo!(),
            CCCommand::RegisterAddress(RegisterAddress {
                blockchain,
                address,
                network,
            }) => {
                let address_id = Address::with_prefix_key(
                    ADDR,
                    &crate::string!(&blockchain, &address, &network),
                );
                let state = protos::Address {
                    blockchain: blockchain.clone(),
                    value: address.clone(),
                    network: network.clone(),
                    sighash: self.identity.sighash.to_string(),
                };
                (address_id, state.to_bytes().into())
            }
            CCCommand::RegisterTransfer(RegisterTransfer {
                gain,
                order_id,
                blockchain_tx_id,
            }) => {
                let s = info.state.get(&Address::from(order_id.clone())).unwrap();
                let order = protos::DealOrder::try_parse(&s).unwrap();
                let address = info
                    .state
                    .get(&Address::from(order.src_address.clone()))
                    .unwrap();
                let address = protos::Address::try_parse(&address).unwrap();
                let address_id = Address::with_prefix_key(
                    TRANSFER,
                    &string!(&order.blockchain, &blockchain_tx_id, &address.network),
                );
                let state = protos::Transfer {
                    blockchain: order.blockchain,
                    src_address: order.src_address,
                    dst_address: order.dst_address,
                    order: order_id.clone(),
                    amount: (CurrencyAmount::try_parse(order.amount).unwrap() + gain.clone())
                        .to_string(),
                    tx: blockchain_tx_id.clone(),
                    block: (u64::from_str(&order.block).unwrap() + 1).to_string(),
                    processed: false,
                    sighash: info.identity_for(Role::Investor).unwrap().sighash.into(),
                };
                (address_id, state.to_bytes().into())
            }
            CCCommand::AddAskOrder(_) => todo!(),
            CCCommand::AddBidOrder(_) => todo!(),
            CCCommand::AddOffer(_) => todo!(),
            CCCommand::AddDealOrder(_) => todo!(),
            CCCommand::CompleteDealOrder(_) => todo!(),
            CCCommand::LockDealOrder(_) => todo!(),
            CCCommand::CloseDealOrder(_) => todo!(),
            CCCommand::Exempt(_) => todo!(),
            CCCommand::AddRepaymentOrder(_) => todo!(),
            CCCommand::CompleteRepaymentOrder(_) => todo!(),
            CCCommand::CloseRepaymentOrder(_) => todo!(),
            CCCommand::CollectCoins(_) => todo!(),
            CCCommand::Housekeeping(_) => todo!(),
        }
    }
}
