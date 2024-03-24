use super::types::{ProposeRequest, SafeInfoResponse};
use super::wrappers::ChecksumAddress;
use crate::encoding::bytes_to_hex_string;
use crate::safe::SafeTransaction;
use crate::safe::{SafeTransactionBuilder, SignedSafePayload};
use crate::transaction::Transactionable;
use crate::{json_get, json_post};
use core::fmt::Debug;
use ethers::signers::Signer;
use ethers::types::transaction::eip712::Eip712;
use ethers::types::Address;
use ethers::utils::to_checksum;
use lazy_static::lazy_static;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64};
use tracing::debug;

/// Mainnet only
const _BASE_URL: &str = "https://safe-transaction-mainnet.safe.global/api/";

lazy_static! {
    static ref MAINNET_CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .default_headers({
            reqwest::header::HeaderMap::from_iter(
                [("cache-control", "no-cache")]
                    .iter()
                    .map(|(k, v)| (HeaderName::from_static(k), HeaderValue::from_static(v))),
            )
        })
        .build()
        .unwrap();
    static ref BASE_URL: Url = Url::parse(_BASE_URL).expect("Can parse BASE_URL");
}

pub struct SafeClient {
    safe_address: ChecksumAddress,
    client: reqwest::Client,
    nonce: AtomicU64,
}

impl SafeClient {
    pub async fn new(safe_address: Address) -> anyhow::Result<Self> {
        let this = SafeClient {
            safe_address: safe_address.into(),
            client: MAINNET_CLIENT.clone(),
            nonce: AtomicU64::new(0),
        };

        let nonce = this.safe_info().await?.nonce;

        this.nonce
            .store(nonce, std::sync::atomic::Ordering::Relaxed);

        Ok(this)
    }

    /// Increments the nonce and returns a builder with the nonce set
    pub fn safe_tx_builder<T: Transactionable>(&self, tx: T) -> SafeTransactionBuilder<T> {
        SafeTransactionBuilder::new(tx, self.chain_id(), self.safe_address.into()).nonce(
            self.nonce
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                .into(),
        )
    }

    const fn chain_id(&self) -> u64 {
        1
    }
}

impl SafeClient {
    pub async fn safe_info(&self) -> anyhow::Result<SafeInfoResponse> {
        debug!("getting safe {}", self.safe_address);

        json_get!(
            self.client,
            BASE_URL.join(&format!("v1/safes/{}/", self.safe_address))?,
            SafeInfoResponse
        )
    }

    pub async fn propose(&self, tx: ProposeRequest) -> anyhow::Result<()> {
        debug!("proposing tx for safe {}", self.safe_address);

        json_post!(
            self.client,
            BASE_URL.join(&format!(
                "v1/safes/{}/multisig-transactions/",
                self.safe_address
            ))?,
            tx
        )
    }
}
