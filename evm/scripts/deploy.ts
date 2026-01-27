import hre from "hardhat";

const { ethers, network } = hre;

type DeployResult = {
  mode: "full" | "upgrade";
  network: string;
  chainId: number;
  deployer: string;
  admin: string;
  token: string;
  implementation: string;
  proxy: string;
};

function envOr(name: string, fallback?: string): string | undefined {
  const v = process.env[name];
  return v && v.trim().length > 0 ? v.trim() : fallback;
}

function mustEnv(name: string): string {
  const v = envOr(name);
  if (!v) throw new Error(`${name} is required`);
  return v;
}

async function main() {
  const [deployer] = await ethers.getSigners();
  const deployerAddr = await deployer.getAddress();

  const mode = (envOr("DEPLOY_MODE", "full") as "full" | "upgrade") || "full";

  const admin = envOr("SYNC_ADMIN", deployerAddr)!;
  const tokenName = envOr("TOKEN_NAME", "Strova Credits")!;
  const tokenSymbol = envOr("TOKEN_SYMBOL", "Strova")!;
  const tokenDecimals = Number(envOr("TOKEN_DECIMALS", "9")!);

  if (!Number.isInteger(tokenDecimals) || tokenDecimals < 0 || tokenDecimals > 255) {
    throw new Error("TOKEN_DECIMALS must be an integer between 0 and 255");
  }

  const chainId = Number((await deployer.provider!.getNetwork()).chainId);

  console.log(`network=${network.name} chainId=${chainId}`);
  console.log(`deployer=${deployerAddr}`);
  console.log(`mode=${mode}`);

  // -------------------------
  // UPGRADE-ONLY MODE
  // -------------------------
  // Deploy a new implementation and upgrade the existing proxy via UUPS `upgradeToAndCall`.
  // Keeps proxy address unchanged.
  if (mode === "upgrade") {
    const proxyAddr = mustEnv("SYNC_CONTRACT_PROXY");
    const upgradeCalldata = envOr("UPGRADE_CALLDATA", "0x")!;

    const Impl = await ethers.getContractFactory("SyncContract");
    const impl = await Impl.deploy();
    await impl.waitForDeployment();
    const implAddr = await impl.getAddress();
    console.log(`newImplementation=${implAddr}`);

    const sync = await ethers.getContractAt("SyncContract", proxyAddr, deployer);
    // NOTE: caller must be current admin (authorizeUpgrade).
    const tx = await sync.upgradeToAndCall(implAddr, upgradeCalldata);
    console.log(`upgradeTxHash=${tx.hash}`);
    await tx.wait();

    // Optional verification: verify the new implementation only.
    if (process.env.VERIFY === "true") {
      try {
        await hre.run("verify:verify", { address: implAddr, constructorArguments: [] });
        console.log(`verified=${implAddr}`);
      } catch (e: any) {
        const msg = (e?.message || String(e)) as string;
        if (msg.toLowerCase().includes("already verified")) {
          console.log(`alreadyVerified=${implAddr}`);
        } else {
          console.warn(`verifyFailed=${implAddr} error=${msg}`);
        }
      }
    }

    const out: DeployResult = {
      mode,
      network: network.name,
      chainId,
      deployer: deployerAddr,
      admin: await sync.admin(),
      token: await sync.creditToken(),
      implementation: implAddr,
      proxy: proxyAddr
    };

    console.log("deployResult=", JSON.stringify(out, null, 2));
    return;
  }

  // -------------------------
  // FULL DEPLOY MODE
  // -------------------------
  console.log(`admin=${admin}`);

  // 1) Deploy ERC20 token with deployer as initial mint authority (then transfer to proxy).
  const Token = await ethers.getContractFactory("StrovaCreditToken");
  const token = await Token.deploy(tokenName, tokenSymbol, tokenDecimals, deployerAddr);
  await token.waitForDeployment();
  const tokenAddr = await token.getAddress();
  console.log(`token=${tokenAddr}`);

  // 2) Deploy SyncContract implementation (UUPS impl, but we deploy via ERC1967 proxy wrapper).
  const Impl = await ethers.getContractFactory("SyncContract");
  const impl = await Impl.deploy();
  await impl.waitForDeployment();
  const implAddr = await impl.getAddress();
  console.log(`implementation=${implAddr}`);

  // 3) Deploy proxy + initialize(storage) in one step.
  const initData = impl.interface.encodeFunctionData("initialize", [admin, tokenAddr]);
  const Proxy = await ethers.getContractFactory("SyncContractProxy");
  const proxy = await Proxy.deploy(implAddr, initData);
  await proxy.waitForDeployment();
  const proxyAddr = await proxy.getAddress();
  console.log(`proxy=${proxyAddr}`);

  // 4) Transfer mint authority to proxy so `rateData(... sendTokensImmediately=true)` / `claimCredits()`
  // can mint. (This mirrors Solana where the program state PDA is mint authority.)
  const tx = await token.transferMintAuthority(proxyAddr);
  await tx.wait();
  console.log(`mintAuthorityTransferredTo=${proxyAddr}`);

  // 5) Optional verification (Basescan). Requires BASESCAN_API_KEY + a public RPC URL.
  if (process.env.VERIFY === "true") {
    const verify = async (address: string, constructorArguments: unknown[]) => {
      try {
        await hre.run("verify:verify", { address, constructorArguments });
        console.log(`verified=${address}`);
      } catch (e: any) {
        const msg = (e?.message || String(e)) as string;
        if (msg.toLowerCase().includes("already verified")) {
          console.log(`alreadyVerified=${address}`);
          return;
        }
        console.warn(`verifyFailed=${address} error=${msg}`);
      }
    };

    // Token (constructor args)
    await verify(tokenAddr, [tokenName, tokenSymbol, tokenDecimals, deployerAddr]);

    // Implementation has no constructor args
    await verify(implAddr, []);

    // Proxy constructor args: (implementation, initData)
    await verify(proxyAddr, [implAddr, initData]);
  }

  const out: DeployResult = {
    mode,
    network: network.name,
    chainId,
    deployer: deployerAddr,
    admin,
    token: tokenAddr,
    implementation: implAddr,
    proxy: proxyAddr
  };

  console.log("deployResult=", JSON.stringify(out, null, 2));
}

main().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});


