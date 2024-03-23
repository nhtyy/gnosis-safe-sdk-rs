use ethers::abi::ethereum_types::FromDecStrErr;
use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

/// An address wrapper that ensures checksum encoding
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct ChecksumAddress(pub Address);

impl std::ops::Deref for ChecksumAddress {
    type Target = Address;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Address> for ChecksumAddress {
    fn from(addr: Address) -> Self {
        Self(addr)
    }
}

impl From<ChecksumAddress> for Address {
    fn from(val: ChecksumAddress) -> Self {
        val.0
    }
}

impl serde::Serialize for ChecksumAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ethers::utils::to_checksum(self, None).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ChecksumAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Address::deserialize(deserializer)?.into())
    }
}

impl std::fmt::Debug for ChecksumAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ethers::utils::to_checksum(self, None))
    }
}

impl std::fmt::Display for ChecksumAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ethers::utils::to_checksum(self, None))
    }
}

impl FromStr for ChecksumAddress {
    type Err = <Address as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Address>().map(Into::into)
    }
}

impl ethers::abi::Tokenizable for ChecksumAddress {
    fn from_token(token: ethers::abi::Token) -> Result<Self, ethers::abi::InvalidOutputType>
    where
        Self: Sized,
    {
        Address::from_token(token).map(Into::into)
    }

    fn into_token(self) -> ethers::abi::Token {
        self.0.into_token()
    }
}

/// A U256 wrapper that ensures decimal string encoding
#[derive(Debug, Clone, Copy, Default)]
pub struct DecimalU256(U256);

impl Display for DecimalU256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl std::ops::Deref for DecimalU256 {
    type Target = U256;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<U256> for DecimalU256 {
    fn from(i: U256) -> Self {
        Self(i)
    }
}

impl From<DecimalU256> for U256 {
    fn from(i: DecimalU256) -> Self {
        i.0
    }
}

impl FromStr for DecimalU256 {
    type Err = FromDecStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(U256::from_dec_str(s)?.into())
    }
}

impl serde::Serialize for DecimalU256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{}", self.0).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for DecimalU256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

pub(crate) mod dec_u256_ser {
    use super::*;

    #[allow(dead_code)]
    pub(crate) fn serialize<S>(u: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{}", DecimalU256::from(*u)).serialize(serializer)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DecimalU256::deserialize(deserializer).map(Into::into)
    }
}

pub(crate) mod dec_u256_opt_ser {
    use super::*;

    #[allow(dead_code)]
    pub(crate) fn serialize<S>(u: &Option<U256>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        u.map(Into::<DecimalU256>::into).serialize(serializer)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Option<U256>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Option::<DecimalU256>::deserialize(deserializer)?.map(Into::into))
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Hash(#[serde(serialize_with = "hex_encode_hash")] [u8; 32]);

fn hex_encode_hash<S>(hash: &[u8; 32], s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&format!("0x{}", hex::encode(hash)))
}

impl std::ops::Deref for Hash {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<[u8; 32]> for Hash {
    fn from(hash: [u8; 32]) -> Self {
        Self(hash)
    }
}
