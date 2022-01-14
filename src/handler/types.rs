use anyhow::Context;
use derive_more::{Add, AddAssign, Display, Div, Mul, Sub, SubAssign};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::ops::{Add, Sub};
use std::ops::{AddAssign, Deref};
use std::ops::{Div, Mul};

use derive_more::{From, Into};
use rug::integer::SmallInteger;
use rug::{Assign, Integer};

use sawtooth_sdk::processor::handler::ApplyError;
use sawtooth_sdk::processor::handler::ContextError;

use crate::ext::IntegerExt;
use crate::handler::constants::*;
use crate::handler::utils::sha512_id;
use crate::{bail_transaction, protos, string};

use super::utils;

pub type TxnResult<T, E = anyhow::Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum CCApplyError {
    InvalidTransaction(String),
    InternalError(String),
}

impl fmt::Display for CCApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CCApplyError::InvalidTransaction(e) => write!(f, "{}", e),
            CCApplyError::InternalError(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl From<CCApplyError> for ApplyError {
    fn from(err: CCApplyError) -> Self {
        match err {
            CCApplyError::InvalidTransaction(e) => ApplyError::InvalidTransaction(e),
            CCApplyError::InternalError(e) => ApplyError::InternalError(e),
        }
    }
}

impl std::error::Error for CCApplyError {}

impl From<ContextError> for CCApplyError {
    fn from(context_error: ContextError) -> Self {
        match context_error {
            ContextError::TransactionReceiptError(..) => {
                CCApplyError::InternalError(format!("{}", context_error))
            }
            _ => CCApplyError::InvalidTransaction(format!("{}", context_error)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into, Default)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct SigHash(pub String);

impl From<&str> for SigHash {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl SigHash {
    pub fn to_wallet_id(&self) -> WalletId {
        let wallet_id = string!(NAMESPACE_PREFIX, WALLET, self.to_lowercase());
        wallet_id.into()
    }
    pub fn from_public_key(key: &str) -> TxnResult<SigHash> {
        let compressed = utils::compress(key)?;
        let hash = sha512_id(compressed.as_bytes());
        Ok(SigHash(hash))
    }
}

#[test]
fn sighash_to_wallet_id_always_returns_lowercase() {
    let sighash = SigHash::from("-InvestoR-SighasH");
    assert_eq!(
        sighash.to_wallet_id().to_string(),
        "8a1a040000-investor-sighash"
    );
}

impl Deref for SigHash {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for SigHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into, Default)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct WalletId(pub String);

impl From<&SigHash> for WalletId {
    fn from(sig: &SigHash) -> Self {
        let buf = string!(NAMESPACE_PREFIX, WALLET, sig.as_str());
        WalletId(buf)
    }
}

impl From<SigHash> for WalletId {
    fn from(sig: SigHash) -> Self {
        let buf = string!(NAMESPACE_PREFIX, WALLET, sig.as_str());
        WalletId(buf)
    }
}

impl Deref for WalletId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for WalletId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into, Default)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Guid(pub String);

impl From<&str> for Guid {
    fn from(s: &str) -> Self {
        Guid(s.to_string())
    }
}

#[cfg(all(feature = "integration-testing"))]
impl From<Guid> for [u8; 16] {
    fn from(guid: Guid) -> Self {
        assert_eq!(guid.as_str().len(), 16);
        guid.as_str().as_bytes().try_into().unwrap()
    }
}

#[cfg(all(feature = "integration-testing"))]
impl From<[u8; 16]> for Guid {
    fn from(arr: [u8; 16]) -> Self {
        let arr = arr.to_vec();
        let s = String::from_utf8(arr).unwrap();
        Guid(s)
    }
}

impl Deref for Guid {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into, Default)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct Address(pub String);

impl Address {
    pub fn with_prefix_key(prefix: &str, key: &str) -> Self {
        let id = sha512_id(key);
        let addr = string!(NAMESPACE_PREFIX, prefix, &id);
        assert_eq!(addr.len(), MERKLE_ADDRESS_LENGTH);
        Self(addr)
    }
    pub fn checked_from(address: &str, prefix: &str) -> TxnResult<Self> {
        Address::validate(address, prefix)?;
        Ok(Address(address.to_owned()))
    }
    pub fn validate(address: &str, expected_prefix: &str) -> TxnResult<()> {
        if !address.starts_with(&*NAMESPACE_PREFIX) {
            bail_transaction!(
                "Invalid id",
                context = "the id {:?} must start with {}",
                address,
                { &*NAMESPACE_PREFIX }
            );
        } else if !address[NAMESPACE_PREFIX.len()..].starts_with(expected_prefix) {
            bail_transaction!(
                "Invalid id",
                context = "the id {:?} must be be under the sub-namespace {}",
                address,
                expected_prefix
            );
        } else if address.len() != 70 {
            println!("Bad {:?}", address);

            bail_transaction!(
                "Invalid id",
                context = "the id {:?} must be 70 characters long, but it is {}",
                address,
                { address.len() }
            );
        } else if !address.chars().all(|c| c.is_ascii_hexdigit()) {
            bail_transaction!(
                "Invalid id",
                context = "the id {:?} must consist only of hexadecimal characters",
                address
            );
        }
        Ok(())
    }
}

impl Deref for Address {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Address {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&Address> for String {
    fn from(address: &Address) -> String {
        address.0.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into, Default)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct State(pub Vec<u8>);

impl Deref for State {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for State {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

pub type StateVec = Vec<(String, Vec<u8>)>;

#[derive(
    Debug,
    Display,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    From,
    Add,
    Sub,
    Mul,
    Div,
    Default,
    AddAssign,
    SubAssign,
)]
#[cfg_attr(any(test, feature = "integration-testing"), derive(serde::Serialize))]
#[cfg_attr(any(test, feature = "integration-testing"), serde(into = "String"))]
pub struct Credo(pub Integer);

impl Credo {
    pub fn new() -> Self {
        Self(Integer::new())
    }
    pub fn try_parse<S: AsRef<str>>(s: S) -> TxnResult<Self> {
        Ok(Credo(<Integer as IntegerExt>::try_parse(s)?))
    }

    pub fn try_parse_signed<S: AsRef<str>>(s: S) -> TxnResult<Self> {
        Ok(Credo(<Integer as IntegerExt>::try_parse_signed(s)?))
    }

    pub fn from_wallet(wallet: &protos::Wallet) -> TxnResult<Self> {
        Credo::try_parse(&wallet.amount).context("Failed to parse wallet balance from string")
    }
}

impl<'a> PartialEq<&'a Credo> for Credo {
    fn eq(&self, other: &&'a Credo) -> bool {
        self.0 == other.0
    }
}
impl<'a> PartialOrd<&'a Credo> for Credo {
    fn partial_cmp(&self, other: &&'a Credo) -> Option<std::cmp::Ordering> {
        self.partial_cmp(*other)
    }
}

impl PartialEq<i64> for Credo {
    fn eq(&self, other: &i64) -> bool {
        self.0.eq(other)
    }
}
impl PartialOrd<i64> for Credo {
    fn partial_cmp(&self, other: &i64) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}
impl<'a> Add<&'a Credo> for Credo {
    type Output = Credo;

    fn add(self, rhs: &'a Credo) -> Self::Output {
        Credo(self.0 + &rhs.0)
    }
}
impl<'a> Sub<&'a Credo> for Credo {
    type Output = Credo;

    fn sub(self, rhs: &'a Credo) -> Self::Output {
        Credo(self.0 - &rhs.0)
    }
}
impl<'a> Add<u64> for Credo {
    type Output = Credo;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl<'a> AddAssign<&'a Credo> for Credo {
    fn add_assign(&mut self, rhs: &'a Credo) {
        self.0 += &rhs.0;
    }
}
impl Assign for Credo {
    fn assign(&mut self, src: Self) {
        self.0.assign(src.0)
    }
}
impl<'a> Assign<&'a Credo> for Credo {
    fn assign(&mut self, src: &'a Credo) {
        self.0.assign(&src.0)
    }
}
impl From<Credo> for Integer {
    fn from(v: Credo) -> Self {
        v.0
    }
}
impl From<i64> for Credo {
    fn from(v: i64) -> Self {
        Self(Integer::from(v))
    }
}
impl From<Credo> for String {
    fn from(val: Credo) -> String {
        val.to_string()
    }
}
#[derive(
    Debug,
    Display,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    From,
    Add,
    Sub,
    Mul,
    Div,
    Default,
    AddAssign,
    SubAssign,
)]
#[cfg_attr(any(test, feature = "integration-testing"), derive(serde::Serialize))]
#[cfg_attr(any(test, feature = "integration-testing"), serde(into = "String"))]
pub struct CurrencyAmount(pub Integer);

impl CurrencyAmount {
    pub fn try_parse<S: AsRef<str>>(s: S) -> TxnResult<Self> {
        Ok(CurrencyAmount(<Integer as IntegerExt>::try_parse(s)?))
    }

    pub fn try_parse_signed<S: AsRef<str>>(s: S) -> TxnResult<Self> {
        Ok(CurrencyAmount(<Integer as IntegerExt>::try_parse_signed(
            s,
        )?))
    }
}

impl PartialEq<i64> for CurrencyAmount {
    fn eq(&self, other: &i64) -> bool {
        self.0.eq(other)
    }
}

impl PartialOrd<i64> for CurrencyAmount {
    fn partial_cmp(&self, other: &i64) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl<'a> Mul<&'a CurrencyAmount> for CurrencyAmount {
    type Output = CurrencyAmount;

    fn mul(self, rhs: &'a CurrencyAmount) -> Self::Output {
        Self(self.0 * &rhs.0)
    }
}

impl From<CurrencyAmount> for Integer {
    fn from(v: CurrencyAmount) -> Self {
        v.0
    }
}
impl From<i64> for CurrencyAmount {
    fn from(v: i64) -> Self {
        Self(Integer::from(v))
    }
}
impl Add<i64> for CurrencyAmount {
    type Output = CurrencyAmount;

    fn add(self, rhs: i64) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl From<CurrencyAmount> for String {
    fn from(val: CurrencyAmount) -> String {
        val.to_string()
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From, Default, Mul, Div, Add, AddAssign,
)]
#[cfg_attr(any(test, feature = "integration-testing"), derive(serde::Serialize))]

pub struct BlockNum(pub u64);

impl BlockNum {
    pub fn new() -> Self {
        Self(0)
    }
}

impl fmt::Display for BlockNum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<&str> for BlockNum {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.contains('-') {
            error!(
                "Negative value: {:?} found while parsing to BlockNum",
                value,
            );
            return Err(CCApplyError::InvalidTransaction(NEGATIVE_NUMBER_ERR.into()))?;
        }
        Ok(BlockNum(value.parse::<u64>().map_err(|e| {
            error!(
                "Failed parsing value: {:?} into BlockNum with error {}",
                value, e
            );
            anyhow::Error::from(CCApplyError::InvalidTransaction(
                INVALID_NUMBER_FORMAT_ERR.into(),
            ))
        })?))
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, From, Default, Mul, Div, Add, AddAssign,
)]
pub struct BlockInterval(pub i128);

impl Div<BlockInterval> for BlockInterval {
    type Output = BlockInterval;

    fn div(self, rhs: BlockInterval) -> Self::Output {
        BlockInterval(self.0 / rhs.0)
    }
}

impl PartialEq<u64> for BlockInterval {
    fn eq(&self, other: &u64) -> bool {
        self.0.eq(&i128::from(*other))
    }
}

impl PartialOrd<u64> for BlockInterval {
    fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&i128::from(*other))
    }
}

impl PartialEq<BlockInterval> for u64 {
    fn eq(&self, other: &BlockInterval) -> bool {
        i128::from(*self).eq(&other.0)
    }
}

impl PartialOrd<BlockInterval> for u64 {
    fn partial_cmp(&self, other: &BlockInterval) -> Option<std::cmp::Ordering> {
        i128::from(*self).partial_cmp(&other.0)
    }
}

impl From<u64> for BlockInterval {
    fn from(val: u64) -> Self {
        Self(val.into())
    }
}

impl BlockInterval {
    pub fn from_blocknum(num: BlockNum) -> Self {
        Self(num.0.into())
    }
}

impl fmt::Display for BlockInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<BlockNum> for BlockInterval {
    fn eq(&self, other: &BlockNum) -> bool {
        self.0 == i128::from(other.0)
    }
}

impl PartialOrd<BlockNum> for BlockInterval {
    fn partial_cmp(&self, other: &BlockNum) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&i128::from(other.0))
    }
}

impl PartialEq<BlockInterval> for BlockNum {
    fn eq(&self, other: &BlockInterval) -> bool {
        i128::from(self.0) == other.0
    }
}

impl PartialOrd<BlockInterval> for BlockNum {
    fn partial_cmp(&self, other: &BlockInterval) -> Option<std::cmp::Ordering> {
        i128::from(self.0).partial_cmp(&other.0)
    }
}

#[test]
fn try_from_str_for_blocknum_works_as_expected() {
    assert_eq!(BlockNum::try_from("8").unwrap(), BlockNum(8));

    let result = BlockNum::try_from("-5").unwrap_err();

    match result.downcast_ref::<CCApplyError>() {
        Some(CCApplyError::InvalidTransaction(s)) => {
            assert_eq!(s, NEGATIVE_NUMBER_ERR);
        }
        _ => panic!("unexpected error"),
    };
}

impl TryFrom<&String> for BlockNum {
    type Error = anyhow::Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        <Self as TryFrom<&str>>::try_from(&*value)
    }
}

impl Add<u64> for BlockNum {
    type Output = BlockNum;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl Sub<u64> for BlockNum {
    type Output = BlockInterval;

    fn sub(self, rhs: u64) -> Self::Output {
        let rhs = BlockNum(rhs);
        self.sub(rhs)
    }
}

impl Sub<BlockNum> for BlockNum {
    type Output = BlockInterval;

    fn sub(self, rhs: BlockNum) -> Self::Output {
        let diff = i128::from(self.0) - i128::from(rhs.0);
        BlockInterval(diff)
    }
}

impl Mul<BlockNum> for u64 {
    type Output = BlockNum;

    fn mul(self, rhs: BlockNum) -> Self::Output {
        BlockNum(self * rhs.0)
    }
}

impl PartialEq<u64> for BlockNum {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl PartialEq<BlockNum> for u64 {
    fn eq(&self, other: &BlockNum) -> bool {
        *self == other.0
    }
}

impl PartialOrd<u64> for BlockNum {
    fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}
impl PartialOrd<BlockNum> for u64 {
    fn partial_cmp(&self, other: &BlockNum) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}
impl Div<BlockNum> for BlockNum {
    type Output = BlockNum;

    fn div(self, rhs: BlockNum) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}
impl From<BlockNum> for SmallInteger {
    fn from(value: BlockNum) -> Self {
        SmallInteger::from(value.0)
    }
}
impl From<BlockNum> for u64 {
    fn from(value: BlockNum) -> Self {
        value.0
    }
}
impl From<BlockNum> for Integer {
    fn from(value: BlockNum) -> Self {
        Integer::from(value.0)
    }
}
impl From<BlockNum> for i32 {
    fn from(value: BlockNum) -> Self {
        value.0 as i32
    }
}
impl From<BlockNum> for String {
    fn from(value: BlockNum) -> Self {
        value.0.to_string()
    }
}
