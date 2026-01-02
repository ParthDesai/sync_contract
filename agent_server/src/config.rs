use anyhow::{anyhow, Context, Result};
use ethers::types::{Address, U64};
use std::env;
use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub rpc_url: String,
    pub chain_id: u64,
    pub sync_contract_proxy: Address,
    pub agent_private_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Load .env if present (safe default; env vars still override)
        let _ = dotenvy::dotenv();

        let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let bind_addr = SocketAddr::from_str(&bind_addr).context("invalid BIND_ADDR")?;

        let rpc_url = env::var("RPC_URL").context("RPC_URL is required")?;
        let sync_contract_proxy =
            env::var("SYNC_CONTRACT_PROXY").context("SYNC_CONTRACT_PROXY is required")?;
        let sync_contract_proxy = Address::from_str(&sync_contract_proxy)
            .map_err(|_| anyhow!("invalid SYNC_CONTRACT_PROXY address"))?;

        let agent_private_key =
            env::var("AGENT_PRIVATE_KEY").context("AGENT_PRIVATE_KEY is required")?;

        // Base Sepolia default. Use 8453 for Base mainnet.
        let chain_id = env::var("CHAIN_ID")
            .ok()
            .and_then(|v| U64::from_dec_str(&v).ok())
            .map(|v| v.as_u64())
            .unwrap_or(84532);

        Ok(Self {
            bind_addr,
            rpc_url,
            chain_id,
            sync_contract_proxy,
            agent_private_key,
        })
    }
}


