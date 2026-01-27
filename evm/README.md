# SyncContract (Base / EVM)

This folder contains the Solidity/Base (EVM) port of the Solana Anchor program in `programs/sync_contract/`.

## What’s included

- `contracts/SyncContract.sol`: upgradeable (UUPS) implementation intended to run **behind an ERC1967 proxy**
- `contracts/SyncoraCreditToken.sol`: standard ERC20 with a single `mintAuthority` that can be transferred
- `test/SyncContract.wholeFlow.ts`: ports the “whole flow” test behavior from `tests/src/test_whole_flow.rs`

## Install (suggested)

This repo’s root `package.json` is Solana-focused. Keep EVM deps isolated in this folder.

From `evm/`:

- Install packages (not run automatically by this agent):
  - `hardhat`
  - `@nomicfoundation/hardhat-toolbox`
  - `@openzeppelin/contracts`
  - `@openzeppelin/contracts-upgradeable`

## Run tests (suggested)

From `evm/`:

```bash
npm test
```

## Deploy (Base Sepolia / Base)

The deploy script deploys:
- `SyncoraCreditToken`
- `SyncContract` implementation
- `SyncContractProxy` (ERC1967 proxy) and runs `initialize(admin, token)`
- transfers token `mintAuthority` to the proxy

### `scripts/deploy.ts` (what it does)

`scripts/deploy.ts` performs the following, in order:

- Deploy `StrovaCreditToken` (ERC20) with **deployer** as initial `mintAuthority`
- Deploy `SyncContract` implementation
- Deploy `SyncContractProxy` (ERC1967 proxy) and call `initialize(admin, token)` via proxy constructor init data
- Transfer token `mintAuthority` to the **proxy** so the Sync contract can mint credits
- Optionally verify contracts on Basescan when `VERIFY=true`

### Base Sepolia

From `evm/`:

```bash
export BASE_SEPOLIA_RPC_URL="https://..."
export DEPLOYER_PRIVATE_KEY="0x..."
export BASESCAN_API_KEY="..."
# optional:
export VERIFY="true"              # run Basescan verification
# optional:
export SYNC_ADMIN="0x..."          # defaults to deployer
export TOKEN_NAME="Syncora Credit" # defaults shown
export TOKEN_SYMBOL="SYNCRED"
export TOKEN_DECIMALS="9"

npx hardhat run scripts/deploy.ts --network baseSepolia
```

### `scripts/deploy.ts` environment variables

- **`DEPLOY_MODE`**: `full` (default) or `upgrade`
- **`BASE_SEPOLIA_RPC_URL` / `BASE_MAINNET_RPC_URL`**: RPC URL for the chosen network
- **`DEPLOYER_PRIVATE_KEY`**: EOA used to deploy + pay gas
- **`SYNC_ADMIN`** (optional): initial admin stored in `SyncContract` (defaults to deployer)
- **`TOKEN_NAME`** (optional): token name (defaults to `Syncora Credit`)
- **`TOKEN_SYMBOL`** (optional): token symbol (defaults to `SYNCRED`)
- **`TOKEN_DECIMALS`** (optional): token decimals (defaults to `9`)
- **`BASESCAN_API_KEY`** (optional): required only if verifying
- **`VERIFY`** (optional): set to `true` to attempt verification (`verify:verify`) for token, implementation, and proxy

#### Upgrade-only env vars

When `DEPLOY_MODE=upgrade`:

- **`SYNC_CONTRACT_PROXY`**: proxy address to upgrade (**required**)
- **`UPGRADE_CALLDATA`** (optional): calldata hex to pass to `upgradeToAndCall` (defaults to `0x`)

### `scripts/deploy.ts` CLI usage

- Run via Hardhat:

```bash
npx hardhat run scripts/deploy.ts --network baseSepolia
```

- Networks available (see `hardhat.config.ts`):
  - `baseSepolia`
  - `base`

### Base Mainnet

```bash
export BASE_MAINNET_RPC_URL="https://..."
export DEPLOYER_PRIVATE_KEY="0x..."
npx hardhat run scripts/deploy.ts --network base
```

### Upgrade only (keep proxy address)

```bash
cd evm
export BASE_SEPOLIA_RPC_URL="https://..."
export DEPLOYER_PRIVATE_KEY="0x..."      # must be current admin
export DEPLOY_MODE="upgrade"
export SYNC_CONTRACT_PROXY="0x..."
# optional:
export UPGRADE_CALLDATA="0x"
export VERIFY="true"

npx hardhat run scripts/deploy.ts --network baseSepolia
```

## Transfer admin

The `SyncContract` stores an `admin` address in contract storage. You can transfer it via `transferAdmin`.

### `scripts/transferAdmin.ts` (what it does)

`scripts/transferAdmin.ts` calls `transferAdmin(newAdmin)` on the **proxy** contract. The transaction must be sent by the **current admin**.

```bash
cd evm
export BASE_SEPOLIA_RPC_URL="https://..."
export DEPLOYER_PRIVATE_KEY="0x..."          # must be current admin
export SYNC_CONTRACT_PROXY="0x..."
export NEW_ADMIN="0x..."

npx hardhat run scripts/transferAdmin.ts --network baseSepolia
```

### `scripts/transferAdmin.ts` environment variables

- **`BASE_SEPOLIA_RPC_URL` / `BASE_MAINNET_RPC_URL`**: RPC URL for the chosen network
- **`DEPLOYER_PRIVATE_KEY`**: private key of the **current admin** (pays gas)
- **`SYNC_CONTRACT_PROXY`**: address of the deployed proxy
- **`NEW_ADMIN`**: address to set as the new admin

### `scripts/transferAdmin.ts` CLI usage

```bash
npx hardhat run scripts/transferAdmin.ts --network baseSepolia
```

## SyncContract error selectors (quick decode)

When a tx reverts with **custom error data**, the first 4 bytes are the error selector. These are the current selectors for `SyncContract.sol`:

| Selector | Custom error |
|---:|---|
| `0x7d88852e` | `ErrAgentAlreadyCreated()` |
| `0xefad4bb4` | `ErrAgentIsNotEnabled()` |
| `0xbcdf64fb` | `ErrAgentNotCreated()` |
| `0x9c42e801` | `ErrAlreadyInitializedSubmission()` |
| `0x0c2446f3` | `ErrCallerNotAdmin()` |
| `0xd803bbaa` | `ErrDataAlreadyRated()` |
| `0x46e4a5bd` | `ErrDataLinkEmpty()` |
| `0xc3b01dc7` | `ErrDataLinkTooLarge()` |
| `0x2f6a83e0` | `ErrInvalidDataLink()` |
| `0xc93cadc8` | `ErrInvalidRating()` |
| `0x5042b1cb` | `ErrPrimaryCategoryTooLarge()` |
| `0xae6b030e` | `ErrSecondaryCategoryTooLarge()` |
| `0x85ac61de` | `ErrSeedMustbeDeletedBeforeRating()` |
| `0xecc6fdf0` | `ErrZeroAddress()` |


