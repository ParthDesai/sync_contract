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

### Base Mainnet

```bash
export BASE_MAINNET_RPC_URL="https://..."
export DEPLOYER_PRIVATE_KEY="0x..."
npx hardhat run scripts/deploy.ts --network base
```

## Transfer admin

The `SyncContract` stores an `admin` address in contract storage. You can transfer it via `transferAdmin`.

```bash
cd evm
export BASE_SEPOLIA_RPC_URL="https://..."
export DEPLOYER_PRIVATE_KEY="0x..."          # must be current admin
export SYNC_CONTRACT_PROXY="0x..."
export NEW_ADMIN="0x..."

npx hardhat run scripts/transferAdmin.ts --network baseSepolia
```


