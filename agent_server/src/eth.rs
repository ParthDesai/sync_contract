use anyhow::{Context, Result};
use ethers::prelude::*;
use ethers::types::{Address, TxHash, U256};
use std::sync::Arc;
use tokio::sync::Mutex;

// Minimal ABI for SyncContract.rateData(...) and getSubmissionKey(...)
abigen!(
    SyncContract,
    r#"[
        function createAgent() external
        function rateData(string dataLink, bool isSeedDeleted, bool isValid, bool hasRating, uint8 rating, bool hasSyntheticDataLink, string syntheticDataLink, bool sendTokensImmediately) external
        function getSubmissionKey(string dataLink) external view returns (bytes32)
    ]"#,
);

#[async_trait::async_trait]
pub trait EthApi: Send + Sync {
    fn agent(&self) -> Address;

    async fn submission_key(&self, data_link: &str) -> anyhow::Result<[u8; 32]>;

    async fn create_agent(&self) -> anyhow::Result<TxHash>;

    async fn rate(
        &self,
        data_link: String,
        is_seed_deleted: bool,
        is_valid: bool,
        rating: Option<u8>,
        synthetic_data_link: Option<String>,
        send_tokens_immediately: bool,
    ) -> anyhow::Result<TxHash>;
}

#[derive(Clone)]
pub struct EthClient {
    pub agent: Address,
    contract: SyncContract<
        NonceManagerMiddleware<SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>>,
    >,
    /// Serialize tx submission from this process (helps if multiple HTTP requests arrive concurrently).
    send_lock: Arc<Mutex<()>>,
}

#[async_trait::async_trait]
impl EthApi for EthClient {
    async fn submission_key(&self, data_link: &str) -> Result<[u8; 32]> {
        let k: [u8; 32] = self
            .contract
            .get_submission_key(data_link.to_string())
            .call()
            .await
            .context("getSubmissionKey call failed")?;
        Ok(k)
    }

    async fn create_agent(&self) -> Result<TxHash> {
        let _g = self.send_lock.lock().await;

        let contract_call = self
        .contract
        .create_agent();

        let pending = contract_call
            .send()
            .await
            .context("failed to send createAgent tx")?;
        let receipt = pending
            .await
            .context("failed waiting for createAgent receipt")?
            .context("createAgent tx dropped from mempool")?;
        Ok(receipt.transaction_hash)
    }

    async fn rate(
        &self,
        data_link: String,
        is_seed_deleted: bool,
        is_valid: bool,
        rating: Option<u8>,
        synthetic_data_link: Option<String>,
        send_tokens_immediately: bool,
    ) -> Result<TxHash> {
        let _g = self.send_lock.lock().await;
        let has_rating = rating.is_some();
        let rating_value = rating.unwrap_or(0);

        let has_synth = synthetic_data_link.is_some();
        let synth_value = synthetic_data_link.unwrap_or_default();

        let contract_call = self
        .contract
        .rate_data(
            data_link,
            is_seed_deleted,
            is_valid,
            has_rating,
            rating_value,
            has_synth,
            synth_value,
            send_tokens_immediately,
        );

        let pending = contract_call
            .send()
            .await
            .context("failed to send rateData tx")?;
        let receipt = pending
            .await
            .context("failed waiting for rateData receipt")?
            .context("rateData tx dropped from mempool")?;
        Ok(receipt.transaction_hash)
    }

    fn agent(&self) -> Address {
        self.agent
    }
}

impl EthClient {
    pub async fn new(rpc_url: &str, chain_id: u64, private_key_hex: &str, proxy: Address) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .context("failed to create provider from RPC_URL")?
            .interval(std::time::Duration::from_millis(200));

        let wallet: LocalWallet = private_key_hex
            .trim()
            .strip_prefix("0x")
            .unwrap_or(private_key_hex.trim())
            .parse::<LocalWallet>()
            .context("failed to parse AGENT_PRIVATE_KEY")?
            .with_chain_id(chain_id);

        let agent = wallet.address();
        let signer = SignerMiddleware::new(provider, wallet);
        // Prevents "nonce has already been used" when the same key is used across processes
        // (e.g., test setup txs + this server txs).
        let client = NonceManagerMiddleware::new(signer, agent);
        let client = Arc::new(client);

        let contract = SyncContract::new(proxy, client);

        Ok(Self {
            agent,
            contract,
            send_lock: Arc::new(Mutex::new(())),
        })
    }
}




