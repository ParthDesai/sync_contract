# Agent Rating Server (Rust)

Small HTTP service that lets an “agent backend” submit `rateData` transactions to the **SyncContract proxy** (Base / Base Sepolia).

This service:
- holds the **agent private key** (from an env var)
- exposes a small HTTP API
- signs and submits transactions to the configured RPC

## Configuration

The server reads config from **environment variables**:

- **`RPC_URL` (required)**: RPC URL (Base Sepolia / Base)
- **`SYNC_CONTRACT_PROXY` (required)**: proxy address of `SyncContract`
- **`AGENT_PRIVATE_KEY` (required)**: hex private key (with or without `0x`)
- **`CHAIN_ID` (optional)**: defaults to **`84532`** (Base Sepolia). Use `8453` for Base mainnet.
- **`BIND_ADDR` (optional)**: defaults to `0.0.0.0:8080`

### Security notes

- **Never commit** `AGENT_PRIVATE_KEY` to git.
- Prefer injecting secrets via your infra (Docker/k8s/CI secrets).
- Treat this server as a signing service; in production you likely want **auth** (API key/JWT/mTLS) and stricter CORS.

## CORS

Browser CORS is restricted to localhost origins:

- `http://localhost` (any port)
- `http://127.0.0.1` (any port)

Requests without an `Origin` header (typical server-to-server) are unaffected.

## API

### `GET /health`

Returns `200 ok` when the server is up.

### `POST /create-agent`

Creates the on-chain agent config for the server’s agent key (calls `createAgent()`).

Response:

```json
{
  "txHash": "0x...",
  "agent": "0x..."
}
```

### `POST /rate`

Submits a `rateData` transaction as the agent.

Body:

```json
{
  "dataLink": "ipfs://bafy...",
  "isValid": true,
  "rating": 85
}
```

Optional fields (defaults shown):

```json
{
  "isSeedDeleted": true,
  "syntheticDataLink": null,
  "sendTokensImmediately": false
}
```

Response:

```json
{
  "txHash": "0x...",
  "dataKey": "0x...",
  "agent": "0x..."
}
```

Notes:
- `rating` is optional. If omitted, the agent still marks `isValid`, but no credits are granted unless a rating is provided.

### `GET /users/:user/submissions?start=&limit=`

Read-only query for a user’s submissions (paginated). Anyone can call it.

- `start` (optional): default `0`
- `limit` (optional): default `50`, max `200`

Example:

```bash
curl -sS "http://127.0.0.1:8080/users/0x1111111111111111111111111111111111111111/submissions?start=0&limit=10"
```

## Install / Build / Run

Per repo rule, this folder includes a `Cargo.toml.template`. Copy it to `Cargo.toml` locally.

### Compile

```bash
cd agent_server
cp Cargo.toml.template Cargo.toml
cargo build --release
```

### Start

```bash
cd agent_server
cp Cargo.toml.template Cargo.toml

export RPC_URL="https://..."
export SYNC_CONTRACT_PROXY="0x..."
export AGENT_PRIVATE_KEY="0x..."
# optional:
export CHAIN_ID="84532"
export BIND_ADDR="0.0.0.0:8080"

cargo run --release
```

## Quick test with curl

### Health

```bash
curl -sS http://127.0.0.1:8080/health
```

### Create agent on-chain

```bash
curl -sS -X POST http://127.0.0.1:8080/create-agent
```

### Rate data

```bash
curl -sS -X POST http://127.0.0.1:8080/rate \
  -H 'content-type: application/json' \
  -d '{"dataLink":"ipfs://bafy...","isValid":true,"rating":55}'
```

## E2E test (Hardhat node + Rust server)

There’s an end-to-end test that:
- starts a local Hardhat JSON-RPC node
- deploys the proxy + token
- starts this Rust server (using `Cargo.toml.template` via `--manifest-path`)
- calls `POST /create-agent`, then `POST /rate`, and asserts on-chain state

Run from `evm/`:

```bash
cd evm
npx hardhat compile
npm test -- test/AgentServer.e2e.ts
```


