use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use agent_server::api::AppState;
use agent_server::build_router;
use agent_server::config::Config;
use agent_server::eth::EthClient;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("agent_server=info".parse()?))
        .init();

    let cfg = Config::from_env()?;

    let eth = EthClient::new(
        &cfg.rpc_url,
        cfg.chain_id,
        &cfg.agent_private_key,
        cfg.sync_contract_proxy,
    )
    .await?;

    tracing::info!(
        bind_addr = %cfg.bind_addr,
        chain_id = cfg.chain_id,
        agent = %format!("{:#x}", eth.agent),
        proxy = %format!("{:#x}", cfg.sync_contract_proxy),
        "agent rating server starting"
    );

    let state = AppState { eth: Arc::new(eth) };

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(cfg.bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}


