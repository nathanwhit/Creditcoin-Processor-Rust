#![cfg(any(test, feature = "integration-testing"))]

use crate::ext::MessageExt;
use crate::handler::constants;
use crate::handler::types::{AddressId, BlockNum, CurrencyAmount, Guid, SigHash, WalletId};
use crate::{handler::*, protos, string};
use prost::Message;
use rug::Integer;
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
    let fee_id = AddressId::with_prefix_key(constants::FEE, guid.as_str());
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

pub fn address_id_for(address: &str) -> AddressId {
    AddressId::with_prefix_key(constants::ADDR, &string!("ethereum", address, "rinkeby"))
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

pub fn wallet_with(balance: Option<impl Into<Integer> + Clone>) -> Option<Vec<u8>> {
    balance.map(|b| {
        let wallet = crate::protos::Wallet {
            amount: b.into().to_string(),
        };
        let mut buf = Vec::with_capacity(wallet.encoded_len());
        wallet.encode(&mut buf).unwrap();
        buf
    })
}

#[macro_export]
macro_rules! assert_state_data_eq {
    ($address: expr, $actual_data: expr, $expected_data: expr, $root: ident) => {{
        let ns = $address.strip_prefix(&*NAMESPACE_PREFIX).unwrap();
        let data_decoded = $actual_data;
        let value = $expected_data;
        if ns.starts_with(ADDR) {
            if let Ok(actual) = $root::protos::Address::try_parse(&data_decoded) {
                if let Ok(expected) = $root::protos::Address::try_parse(&value) {
                    assert_eq!(actual, expected);
                }
            }
        } else if ns.starts_with(ASK_ORDER) {
            if let Ok(actual) = $root::protos::AskOrder::try_parse(&data_decoded) {
                if let Ok(expected) = $root::protos::AskOrder::try_parse(&value) {
                    assert_eq!(actual, expected);
                }
            }
        } else if ns.starts_with(BID_ORDER) {
            if let Ok(actual) = $root::protos::BidOrder::try_parse(&data_decoded) {
                if let Ok(expected) = $root::protos::BidOrder::try_parse(&value) {
                    assert_eq!(actual, expected);
                }
            }
        } else if ns.starts_with(DEAL_ORDER) {
            if let Ok(actual) = $root::protos::DealOrder::try_parse(&data_decoded) {
                if let Ok(expected) = $root::protos::DealOrder::try_parse(&value) {
                    assert_eq!(actual, expected);
                }
            }
        } else if ns.starts_with(FEE) {
            if let Ok(actual) = $root::protos::Fee::try_parse(&data_decoded) {
                if let Ok(expected) = $root::protos::Fee::try_parse(&value) {
                    assert_eq!(actual, expected);
                }
            }
        } else if ns.starts_with(OFFER) {
            if let Ok(actual) = $root::protos::Offer::try_parse(&data_decoded) {
                if let Ok(expected) = $root::protos::Offer::try_parse(&value) {
                    assert_eq!(actual, expected);
                }
            }
        } else if ns.starts_with(REPAYMENT_ORDER) {
            if let Ok(actual) = $root::protos::RepaymentOrder::try_parse(&data_decoded) {
                if let Ok(expected) = $root::protos::RepaymentOrder::try_parse(&value) {
                    assert_eq!(actual, expected);
                }
            }
        } else if ns.starts_with(TRANSFER) {
            if let Ok(actual) = $root::protos::Transfer::try_parse(&data_decoded) {
                if let Ok(expected) = $root::protos::Transfer::try_parse(&value) {
                    assert_eq!(actual, expected);
                }
            }
        } else if ns.starts_with(WALLET) {
            if let Ok(actual) = $root::protos::Transfer::try_parse(&data_decoded) {
                if let Ok(expected) = $root::protos::Transfer::try_parse(&value) {
                    assert_eq!(actual, expected);
                }
            }
        }
    }};
}

impl Guid {
    pub fn random() -> Guid {
        Guid::from(make_nonce())
    }
    pub fn new() -> Guid {
        Guid::random()
    }
}

pub trait ToAddressId: Sized {
    type Args;
    fn to_address_id(&self, args: Self::Args) -> AddressId;
}

impl protos::Address {
    pub fn to_address_id(&self) -> AddressId {
        <Self as ToAddressId>::to_address_id(self, ())
    }
}

impl ToAddressId for protos::Address {
    type Args = ();

    fn to_address_id(&self, _args: ()) -> AddressId {
        let key = string!(&self.blockchain, &self.value.to_lowercase(), &self.network);
        AddressId::with_prefix_key(constants::ADDR, &key)
    }
}

pub struct AskOrderIdArgs {
    pub add_ask_order_guid: Guid,
}

impl ToAddressId for protos::AskOrder {
    type Args = AskOrderIdArgs;

    fn to_address_id(&self, args: Self::Args) -> AddressId {
        AddressId::with_prefix_key(constants::ASK_ORDER, args.add_ask_order_guid.as_str())
    }
}

pub struct BidOrderIdArgs {
    pub add_bid_order_guid: Guid,
}

impl ToAddressId for protos::BidOrder {
    type Args = BidOrderIdArgs;

    fn to_address_id(&self, args: Self::Args) -> AddressId {
        AddressId::with_prefix_key(constants::BID_ORDER, args.add_bid_order_guid.as_str())
    }
}

pub struct DealOrderIdArgs {
    pub offer_id: AddressId,
}

impl ToAddressId for protos::DealOrder {
    type Args = DealOrderIdArgs;

    fn to_address_id(&self, args: Self::Args) -> AddressId {
        AddressId::with_prefix_key(constants::DEAL_ORDER, args.offer_id.as_str())
    }
}

pub struct FeeIdArgs {
    pub txn_guid: Guid,
}

impl ToAddressId for protos::Fee {
    type Args = FeeIdArgs;

    fn to_address_id(&self, args: Self::Args) -> AddressId {
        AddressId::with_prefix_key(constants::FEE, args.txn_guid)
    }
}

pub struct OfferIdArgs {
    pub ask_order_id: AddressId,
    pub bid_order_id: AddressId,
}

impl ToAddressId for protos::Offer {
    type Args = OfferIdArgs;

    fn to_address_id(&self, args: Self::Args) -> AddressId {
        let OfferIdArgs {
            ask_order_id,
            bid_order_id,
        } = args;
        let key = string!(ask_order_id, bid_order_id);
        AddressId::with_prefix_key(constants::OFFER, &key)
    }
}

pub struct RepaymentOrderIdArgs {
    pub add_repayment_order_guid: Guid,
}

impl ToAddressId for protos::RepaymentOrder {
    type Args = RepaymentOrderIdArgs;

    fn to_address_id(&self, args: Self::Args) -> AddressId {
        AddressId::with_prefix_key(constants::REPAYMENT_ORDER, args.add_repayment_order_guid)
    }
}

pub struct TransferIdArgs {
    pub network: String,
}

impl ToAddressId for protos::Transfer {
    type Args = TransferIdArgs;

    fn to_address_id(&self, args: Self::Args) -> AddressId {
        let key = string!(&self.blockchain, &self.tx, &args.network);
        AddressId::with_prefix_key(constants::TRANSFER, &key)
    }
}

pub enum WalletIdArg {
    WalletId(WalletId),
    SigHash(SigHash),
}

impl ToAddressId for protos::Wallet {
    type Args = WalletIdArg;

    fn to_address_id(&self, args: Self::Args) -> AddressId {
        match args {
            WalletIdArg::WalletId(id) => id.into(),
            WalletIdArg::SigHash(sig) => sig.to_wallet_id().into(),
        }
    }
}

pub type StateData = Vec<u8>;

#[derive(Clone, Debug)]
pub struct ToStateEntryCtx {
    block: BlockNum,
}

impl ToStateEntryCtx {
    pub fn tip(&self) -> u64 {
        (self.block - 1).unwrap().into()
    }
    pub fn new(block: impl Into<BlockNum>) -> Self {
        Self {
            block: block.into(),
        }
    }
    pub fn inc_tip(&mut self) {
        self.block += BlockNum(1);
    }
    pub fn state_entry_from<T: ToStateEntry>(
        &mut self,
        tx: T,
        args: <T as ToStateEntry>::Args,
    ) -> (AddressId, <T as ToStateEntry>::Output) {
        let res = tx.to_state_entry(args, &*self);
        self.inc_tip();
        res
    }
}

impl Default for ToStateEntryCtx {
    fn default() -> Self {
        Self::new(1)
    }
}

pub trait ToStateEntry {
    type Args;
    type Output: prost::Message + Default;

    fn to_state_entry(&self, args: Self::Args, ctx: &ToStateEntryCtx) -> (AddressId, Self::Output);

    fn to_state(&self, args: Self::Args, ctx: &ToStateEntryCtx) -> (AddressId, StateData) {
        let (addr, state) = self.to_state_entry(args, ctx);
        (addr, state.to_bytes())
    }
}

impl ToStateEntry for RegisterAddress {
    type Args = SigHash;

    type Output = protos::Address;

    fn to_state_entry(
        &self,
        sighash: SigHash,
        _ctx: &ToStateEntryCtx,
    ) -> (AddressId, Self::Output) {
        let address = protos::Address {
            blockchain: self.blockchain.clone(),
            value: self.address.clone(),
            network: self.network.clone(),
            sighash: sighash.into(),
        };

        (address.to_address_id(), address)
    }
}

pub enum TransferKind {
    DealOrder(protos::DealOrder),
    RepaymentOrder(protos::RepaymentOrder),
}

pub struct RegisterTransferArgs {
    pub kind: TransferKind,
    pub src_address: protos::Address,
    pub src_sighash: SigHash,
}

impl ToStateEntry for RegisterTransfer {
    type Args = RegisterTransferArgs;

    type Output = protos::Transfer;

    fn to_state_entry(&self, args: Self::Args, ctx: &ToStateEntryCtx) -> (AddressId, Self::Output) {
        let (src_address_id, dest_address_id, amount) = match args.kind {
            TransferKind::DealOrder(order) if self.gain == 0 => {
                (order.src_address, order.dst_address, order.amount)
            }
            TransferKind::DealOrder(order) => (
                order.dst_address,
                order.src_address,
                (CurrencyAmount::try_parse(order.amount).unwrap() + self.gain.clone()).to_string(),
            ),
            TransferKind::RepaymentOrder(order) => {
                (order.src_address, order.dst_address, order.amount)
            }
        };
        let transfer = protos::Transfer {
            blockchain: args.src_address.blockchain,
            src_address: src_address_id,
            dst_address: dest_address_id,
            order: self.order_id.clone(),
            amount,
            tx: self.blockchain_tx_id.clone(),
            block: ctx.tip().to_string(),
            processed: false,
            sighash: args.src_sighash.into(),
        };
        (
            transfer.to_address_id(TransferIdArgs {
                network: args.src_address.network,
            }),
            transfer,
        )
    }
}

pub struct AddAskOrderArgs {
    pub guid: Guid,
    pub address: protos::Address,
    pub sighash: SigHash,
}

impl ToStateEntry for AddAskOrder {
    type Args = AddAskOrderArgs;

    type Output = protos::AskOrder;

    fn to_state_entry(&self, args: Self::Args, ctx: &ToStateEntryCtx) -> (AddressId, Self::Output) {
        let AddAskOrder {
            address_id,
            amount_str,
            interest,
            maturity,
            fee_str,
            expiration,
        } = self.clone();

        let ask_order = protos::AskOrder {
            address: address_id,
            blockchain: args.address.blockchain,
            amount: amount_str,
            interest,
            maturity,
            fee: fee_str,
            expiration: expiration.into(),
            block: ctx.tip().to_string(),
            sighash: args.sighash.into(),
        };

        (
            ask_order.to_address_id(AskOrderIdArgs {
                add_ask_order_guid: args.guid,
            }),
            ask_order,
        )
    }
}

pub struct AddBidOrderArgs {
    pub guid: Guid,
    pub address: protos::Address,
    pub sighash: SigHash,
}

impl ToStateEntry for AddBidOrder {
    type Args = AddBidOrderArgs;

    type Output = protos::BidOrder;

    fn to_state_entry(&self, args: Self::Args, ctx: &ToStateEntryCtx) -> (AddressId, Self::Output) {
        let AddBidOrder {
            address_id,
            amount_str,
            interest,
            maturity,
            fee_str,
            expiration,
        } = self.clone();

        let bid_order = protos::BidOrder {
            address: address_id,
            blockchain: args.address.blockchain,
            amount: amount_str,
            interest,
            maturity,
            fee: fee_str,
            expiration: expiration.into(),
            block: ctx.tip().to_string(),
            sighash: args.sighash.into(),
        };

        (
            bid_order.to_address_id(BidOrderIdArgs {
                add_bid_order_guid: args.guid,
            }),
            bid_order,
        )
    }
}

pub struct AddOfferArgs {
    pub src_address: protos::Address,
    pub sighash: SigHash,
}

impl ToStateEntry for AddOffer {
    type Args = AddOfferArgs;

    type Output = protos::Offer;

    fn to_state_entry(&self, args: Self::Args, ctx: &ToStateEntryCtx) -> (AddressId, Self::Output) {
        let AddOffer {
            ask_order_id,
            bid_order_id,
            expiration,
        } = self.clone();

        let offer = protos::Offer {
            blockchain: args.src_address.blockchain,
            ask_order: ask_order_id.clone(),
            bid_order: bid_order_id.clone(),
            expiration: expiration.into(),
            block: ctx.tip().to_string(),
            sighash: args.sighash.into(),
        };

        (
            offer.to_address_id(OfferIdArgs {
                ask_order_id: ask_order_id.into(),
                bid_order_id: bid_order_id.into(),
            }),
            offer,
        )
    }
}

pub struct AddDealOrderArgs {
    pub bid_order: protos::BidOrder,
    pub ask_order: protos::AskOrder,
    pub offer: protos::Offer,
    pub sighash: SigHash,
}

impl ToStateEntry for AddDealOrder {
    type Args = AddDealOrderArgs;

    type Output = protos::DealOrder;

    fn to_state_entry(&self, args: Self::Args, ctx: &ToStateEntryCtx) -> (AddressId, Self::Output) {
        // let deal_order_id =
        let AddDealOrder {
            offer_id: _,
            expiration,
        } = self;
        let AddDealOrderArgs {
            bid_order,
            ask_order,
            offer,
            sighash,
        } = args;

        let deal_order = protos::DealOrder {
            blockchain: offer.blockchain.clone(),
            src_address: ask_order.address,
            dst_address: bid_order.address,
            amount: bid_order.amount,
            interest: bid_order.interest,
            maturity: bid_order.maturity,
            fee: bid_order.fee,
            expiration: (*expiration).into(),
            block: ctx.tip().to_string(),
            sighash: sighash.into(),
            ..Default::default()
        };

        let offer_id = offer.to_address_id(OfferIdArgs {
            ask_order_id: AddressId::from(&offer.ask_order),
            bid_order_id: AddressId::from(&offer.bid_order),
        });

        (
            deal_order.to_address_id(DealOrderIdArgs { offer_id }),
            deal_order,
        )
    }
}

pub struct CompleteDealOrderArgs {
    pub deal_order: protos::DealOrder,
}

impl ToStateEntry for CompleteDealOrder {
    type Args = CompleteDealOrderArgs;

    type Output = protos::DealOrder;

    fn to_state_entry(&self, args: Self::Args, ctx: &ToStateEntryCtx) -> (AddressId, Self::Output) {
        let CompleteDealOrder {
            deal_order_id,
            transfer_id,
        } = self;

        (
            deal_order_id.into(),
            protos::DealOrder {
                loan_transfer: transfer_id.into(),
                block: ctx.tip().to_string(),
                ..args.deal_order
            },
        )
    }
}

pub struct AddRepaymentOrderArgs {
    pub guid: Guid,
    pub src_address: protos::Address,
    pub deal_order: protos::DealOrder,
    pub sighash: SigHash,
}

impl ToStateEntry for AddRepaymentOrder {
    type Args = AddRepaymentOrderArgs;

    type Output = protos::RepaymentOrder;

    fn to_state_entry(&self, args: Self::Args, ctx: &ToStateEntryCtx) -> (AddressId, Self::Output) {
        let AddRepaymentOrder {
            deal_order_id,
            address_id,
            amount_str,
            expiration,
        } = self.clone();

        let repayment_order = protos::RepaymentOrder {
            blockchain: args.src_address.blockchain,
            src_address: address_id,
            dst_address: args.deal_order.src_address,
            amount: amount_str,
            expiration: expiration.into(),
            block: ctx.tip().to_string(),
            deal: deal_order_id,
            sighash: args.sighash.into(),
            ..Default::default()
        };

        (
            repayment_order.to_address_id(RepaymentOrderIdArgs {
                add_repayment_order_guid: args.guid,
            }),
            repayment_order,
        )
    }
}
