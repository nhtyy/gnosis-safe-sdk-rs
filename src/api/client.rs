use super::types::{Paged, ProposeRequest, SafeInfoResponse, SafeTransactionResponse};
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

        let nonce = this.next_nonce().await?;
        tracing::debug!("setting nonce to {}", nonce);

        // Store the next nonce so we can give it out
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
    #[tracing::instrument(level = tracing::Level::DEBUG, skip(self), ret)]
    pub async fn safe_info(&self) -> anyhow::Result<SafeInfoResponse> {
        json_get!(
            self.client,
            BASE_URL.join(&format!("v1/safes/{}/", self.safe_address))?,
            SafeInfoResponse
        )
    }

    #[tracing::instrument(level = tracing::Level::DEBUG, skip(self))]
    pub async fn propose(&self, tx: ProposeRequest) -> anyhow::Result<()> {
        json_post!(
            self.client,
            BASE_URL.join(&format!(
                "v1/safes/{}/multisig-transactions/",
                self.safe_address
            ))?,
            tx
        )
    }

    /// Gets the most recent tx for the safe
    #[tracing::instrument(level = tracing::Level::DEBUG, skip(self))]
    pub async fn next_nonce(&self) -> anyhow::Result<u64> {
        let reported_next = self.safe_info().await?.nonce;
        let pending = self.pending().await?;

        if !pending.results.is_empty() {
            return Ok(pending
                .results
                .into_iter()
                .map(|tx| tx.nonce)
                .max()
                .expect("to not check an empty array") + 1);
        }

        Ok(reported_next)
    }

    #[tracing::instrument(level = tracing::Level::DEBUG, skip(self))]
    pub async fn pending(&self) -> anyhow::Result<Paged<SafeTransactionResponse>> {
        debug!("getting pending txs for safe {}", self.safe_address);

        let nonce = self.safe_info().await?.nonce;

        json_get!(
            self.client,
            BASE_URL.join(&format!(
                "v1/safes/{}/multisig-transactions/?nonce__gte={nonce}",
                self.safe_address
            ))?,
            Paged<SafeTransactionResponse>
        )
    }
}
