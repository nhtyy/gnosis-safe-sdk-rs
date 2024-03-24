pub mod client;
pub use client::*;

pub mod types;
pub mod wrappers;

#[macro_export]
/// Make a GET request sending and expecting JSON.
/// if JSON deser fails, emit a `WARN` level tracing event
macro_rules! json_post {
    ($client:expr, $url:expr, $params:expr,) => {
        json_post!($client, $url, $params)
    };

    ($client:expr, $url:expr, $params:expr) => {
    {
        let url = $url;
        tracing::debug!(body = serde_json::to_string(&$params).unwrap().as_str());

        let resp = $client.post(url.clone()).json(&$params).send().await?;
        let status = resp.status();

        // json deser fails
        if !status.is_success() {
            tracing::warn!(
                method = "POST",
                url = %url,
                params = serde_json::to_string(&$params).unwrap().as_str(),
                response = resp.text().await?.as_str(),
                status = ?status,
                "Unexpected response from server"
            );

            return Err(::anyhow::anyhow!("Unexpected response from server"));
        } else {
            Ok(())
        }
    }
}}

#[macro_export]
/// Make a GET request sending and expecting JSON.
/// if JSON deser fails, emit a `WARN` level tracing event
macro_rules! json_get {
    ($client:expr, $url:expr, $expected:ty,) => {
        json_get!($client, $url, $expected)
    };
    ($client:expr, $url:expr, $expected:ty) => {{
        let empty = std::collections::HashMap::<&'static str, &'static str>::default();
        json_get!($client, $url, $expected, empty)
    }};
    ($client:expr, $url:expr, $expected:ty, $query:expr,) => {
        json_get!($client, $url, $expected, $query)
    };
    ($client:expr, $url:expr, $expected:ty, $query:expr) => {{
        let mut url = $url.clone();
        url.query_pairs_mut().extend_pairs($query);
        tracing::debug!(url = url.as_str(), "Dispatching api request");
        let resp = $client.get($url).send().await?;

        if !resp.status().is_success() {
            tracing::warn!(
                method = "GET",
                url = %url,
                response = resp.text().await?.as_str(),
                "Unexpected response from server"
            );

            return Err(::anyhow::anyhow!("Unexpected response from server"));
        }


        Ok(resp.json::<$expected>().await?)
    }};
}

// impl<T: Transactionable> From<SignedSafePayload<T>> for MultisigTransactionRequest {
//     /// consumes a signed transaction and returns a MultisigTransactionRequest
//     fn from(
//         SignedSafePayload {
//             payload,
//             signature,
//             sender,
//         }: SignedSafePayload<T>,
//     ) -> Self {
//         let hash = payload.encode_eip712().unwrap();
//         let inner = payload.tx;
//         Self {
//             to: to_checksum(&inner.to(), None),
//             // todo check encoding
//             value: inner.value().to_string(),
//             data: Option::Some("0x".to_owned() + &bytes_to_hex_string(inner.calldata().unwrap())),
//             operation: payload.operation,
//             safe_tx_gas: payload.safe_tx_gas.to_string(),
//             base_gas: payload.base_gas.to_string(),
//             gas_price: payload.gas_price.to_string(),
//             gas_token: to_checksum(&payload.gas_token, None),
//             refund_receiver: Some(to_checksum(&payload.refund_receiver, None)),
//             nonce: payload.nonce.to_string(),
//             signature: Option::Some("0x".to_owned() + &signature.to_string()),
//             safe_tx_hash: "0x".to_owned() + &bytes_to_hex_string(hash),
//             sender: to_checksum(&sender, None),
//             origin: Option::None,
//         }
//     }
// }

// returns the first pending transactions that matches this calldata
// pub async fn match_calldata<T: Transactionable>(
//     tx: &T,
//     safe_address: Address,
//     chain_id: u64,
// ) -> anyhow::Result<Option<TransactionDetails>> {
//     let calldata = tx.calldata()?;
//     let tx_details = super::api::queued_details(chain_id, safe_address).await?;
//     Ok(tx_details.into_iter().find(|transaction_details| {
//         let TransactionDetails {
//                 tx_data: Some(TransactionData {
//                     hex_data: Some(data)
//                     ,..
//                 })
//                 ,..
//             } = transaction_details else {
//                 return false;
//             };

//         *data == "0x".to_owned() + &bytes_to_hex_string(&calldata)
//     }))
// }

// pub fn extract_sigs_from_details<T: Transactionable>(details: &TransactionDetails) -> String {
//     let confirms = match details.detailed_execution_info.clone() {
//         Some(tx_type) => match tx_type {
//             DetailedExecutionInfo::Multisig(multisig) => multisig.confirmations,
//             _ => {
//                 return "".to_string();
//             }
//         },
//         None => {
//             return "".to_string();
//         }
//     };

//     SafeTransaction::<T>::sort_and_join_sigs(
//         &confirms
//             .into_iter()
//             .filter_map(|c| match c.signer.value.parse::<ethers::types::Address>() {
//                 Ok(address) => match c.signature {
//                     Some(sig) => Some((address, sig)),
//                     None => None,
//                 },
//                 Err(_) => {
//                     debug!("could not parse address {}", c.signer.value);
//                     None
//                 }
//             })
//             .collect(),
//     )
// }

// pub fn is_signed(details: &TransactionDetails, signer: Address) -> bool {
//     match details.detailed_execution_info.clone() {
//         Some(info) => match info {
//             DetailedExecutionInfo::Multisig(multisig_info) => {
//                 let all_signers = multisig_info
//                     .confirmations
//                     .into_iter()
//                     .map(|confirm| confirm.signer.value.parse().unwrap())
//                     .collect::<Vec<Address>>();

//                 println!("all signers: {:?}", all_signers);

//                 all_signers.contains(&signer)
//             }
//             _ => false,
//         },
//         None => false,
//     }
// }
