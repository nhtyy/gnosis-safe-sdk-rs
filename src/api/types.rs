use super::wrappers::{ChecksumAddress, Hash};
use crate::{safe::SignedSafePayload, transaction::Transactionable};
use ethers::types::{transaction::eip712::Eip712, Address, Bytes};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum Operation {
    CALL = 0,
    DELEGATE = 1,
}

/// SAFE Info tracked by the API
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SafeInfoResponse {
    /// The Safe's address
    #[serde(rename = "address")]
    pub safe_address: Address,
    /// The current on-chain nonce (not counting any pending txns)
    pub nonce: u64,
    /// The number of required signers
    pub threshold: u32,
    /// A list of the Owners
    pub owners: Vec<Address>,
    /// The implementation address this safe proxies
    pub master_copy: Address,
    /// Any modules this safe uses
    pub modules: Vec<String>,
    /// The fallback handler for this safe (0 if none)
    pub fallback_handler: Address,
    /// The guard for this safe (0 if none)
    pub guard: Address,
    /// The safe version string
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProposeRequest {
    pub safe: ChecksumAddress,
    pub to: ChecksumAddress,
    pub value: u128,
    pub data: Bytes,
    pub operation: Operation,
    pub gas_token: ChecksumAddress,
    pub safe_tx_gas: u128,
    pub base_gas: u128,
    pub nonce: u128,
    pub contract_transaction_hash: Hash,
    pub sender: ChecksumAddress,
    pub signature: String,
}

impl<T: Transactionable> TryFrom<SignedSafePayload<T>> for ProposeRequest {
    type Error = anyhow::Error;

    fn try_from(
        SignedSafePayload {
            payload,
            signature,
            sender,
        }: SignedSafePayload<T>,
    ) -> Result<Self, Self::Error> {
        let hash = payload.encode_eip712()?;
        let inner = payload.tx;

        Ok(Self {
            to: inner.to().into(),
            value: inner.value().as_u128(),
            data: inner.calldata().map(Into::into).unwrap_or_default(),
            operation: payload.operation,
            safe_tx_gas: payload.safe_tx_gas.as_u128(),
            base_gas: payload.base_gas.as_u128(),
            gas_token: payload.gas_token.into(),
            nonce: payload.nonce.as_u128(),
            contract_transaction_hash: hash.into(),
            sender: sender.into(),
            signature: signature.to_string(),
            safe: payload.safe_address.into(),
        })
    }
}
