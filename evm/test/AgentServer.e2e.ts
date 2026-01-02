import { expect } from "chai";
import { spawn, type ChildProcess } from "node:child_process";
import fs from "node:fs/promises";
import net from "node:net";
import path from "node:path";
import { ethers } from "ethers";

const HARDHAT_MNEMONIC = "test test test test test test test test test test test junk";

function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

async function getFreePort(): Promise<number> {
  return await new Promise((resolve, reject) => {
    const srv = net.createServer();
    srv.on("error", reject);
    srv.listen(0, "127.0.0.1", () => {
      const addr = srv.address();
      srv.close(() => {
        if (addr && typeof addr === "object") resolve(addr.port);
        else reject(new Error("failed to get free port"));
      });
    });
  });
}

async function waitForHttpOk(url: string, timeoutMs = 20_000) {
  const start = Date.now();
  // eslint-disable-next-line no-constant-condition
  while (true) {
    try {
      const res = await fetch(url);
      if (res.ok) return;
    } catch {
      // ignore
    }
    if (Date.now() - start > timeoutMs) throw new Error(`timeout waiting for ${url}`);
    await sleep(200);
  }
}

function spawnHardhatNode(port: number): ChildProcess {
  // run from evm/ so hardhat picks up config & contracts
  const cwd = path.resolve(__dirname, "..");
  const child = spawn("npx", ["hardhat", "node", "--hostname", "127.0.0.1", "--port", String(port)], {
    cwd,
    env: { ...process.env },
    stdio: ["ignore", "pipe", "pipe"]
  });
  return child;
}

function spawnAgentServer(bindPort: number, rpcUrl: string, proxy: string, agentPk: string): ChildProcess {
  const cwd = path.resolve(__dirname, "..", "..", "agent_server");
  const manifest = path.resolve(cwd, "Cargo.toml");

  const child = spawn(
    "cargo",
    ["run", "--quiet", "--manifest-path", manifest],
    {
      cwd,
      env: {
        ...process.env,
        BIND_ADDR: `127.0.0.1:${bindPort}`,
        RPC_URL: rpcUrl,
        SYNC_CONTRACT_PROXY: proxy,
        AGENT_PRIVATE_KEY: agentPk,
        CHAIN_ID: "31337",
        RUST_LOG: "info"
      },
      stdio: ["ignore", "pipe", "pipe"]
    }
  );
  return child;
}

type Token = ethers.BaseContract & {
  transferMintAuthority(newAuthority: string): Promise<ethers.ContractTransactionResponse>;
};

type Sync = ethers.BaseContract & {
  createAgent(): Promise<ethers.ContractTransactionResponse>;
  allowAgent(agent: string): Promise<ethers.ContractTransactionResponse>;
  submitData(
    dataLink: string,
    primaryCategory: string,
    secondaryCategory: string,
    domain: string,
    dataType: string,
    dataFormat: string,
    fileSizeInKB: bigint
  ): Promise<ethers.ContractTransactionResponse>;
  accumulatedCredits(user: string): Promise<bigint>;
};

function asToken(c: ethers.BaseContract): Token {
  return c as unknown as Token;
}

function asSync(c: ethers.BaseContract): Sync {
  return c as unknown as Sync;
}

async function deployContracts(_provider: ethers.JsonRpcProvider, admin: ethers.Signer) {
  const evmDir = path.resolve(__dirname, "..");
  const adminAddr = await admin.getAddress();

  // Load Hardhat artifacts (compiled output) so we can deploy without Hardhat runtime.
  async function loadArtifact(p: string): Promise<{ abi: ethers.InterfaceAbi; bytecode: string }> {
    const raw = await fs.readFile(p, "utf8");
    const j = JSON.parse(raw) as { abi: ethers.InterfaceAbi; bytecode: string };
    return { abi: j.abi, bytecode: j.bytecode };
  }

  const tokenArtifact = await loadArtifact(
    path.join(evmDir, "artifacts/contracts/SyncoraCreditToken.sol/SyncoraCreditToken.json")
  );

  const implArtifact = await loadArtifact(path.join(evmDir, "artifacts/contracts/SyncContract.sol/SyncContract.json"));

  const proxyArtifact = await loadArtifact(
    path.join(evmDir, "artifacts/contracts/SyncContractProxy.sol/SyncContractProxy.json")
  );

  const TokenFactory = new ethers.ContractFactory(
    tokenArtifact.abi,
    tokenArtifact.bytecode,
    admin
  );
  const token = (await TokenFactory.deploy("Syncora Credit", "SYNCRED", 9, adminAddr)) as unknown as ethers.Contract;
  await token.waitForDeployment();

  const ImplFactory = new ethers.ContractFactory(
    implArtifact.abi,
    implArtifact.bytecode,
    admin
  );
  const impl = await ImplFactory.deploy();
  await impl.waitForDeployment();

  const initData = new ethers.Interface(implArtifact.abi).encodeFunctionData("initialize", [
    adminAddr,
    await token.getAddress()
  ]);

  const ProxyFactory = new ethers.ContractFactory(
    proxyArtifact.abi,
    proxyArtifact.bytecode,
    admin
  );
  const proxy = await ProxyFactory.deploy(await impl.getAddress(), initData);
  await proxy.waitForDeployment();

  // Make proxy the mint authority
  await (await asToken(token).transferMintAuthority(await proxy.getAddress())).wait();

  const sync = new ethers.Contract(await proxy.getAddress(), implArtifact.abi, admin);
  return { token, sync, proxy };
}

describe("agent_server e2e", function () {
  this.timeout(180_000);

  it("starts node, deploys proxy, starts agent_server, POST /rate updates chain state", async () => {
    const nodePort = await getFreePort();
    const serverPort = await getFreePort();
    const rpcUrl = `http://127.0.0.1:${nodePort}`;

    const node = spawnHardhatNode(nodePort);

    try {
      await waitForHttpOk(rpcUrl, 30_000);

      const provider = new ethers.JsonRpcProvider(rpcUrl);
      const admin0 = ethers
        .HDNodeWallet.fromPhrase(HARDHAT_MNEMONIC, undefined, "m/44'/60'/0'/0/0")
        .connect(provider);
      const admin = new ethers.NonceManager(admin0);

      const user0 = ethers
        .HDNodeWallet.fromPhrase(HARDHAT_MNEMONIC, undefined, "m/44'/60'/0'/0/2")
        .connect(provider);
      const user = new ethers.NonceManager(user0);
      const userAddr = await user0.getAddress();

      // Compile contracts once (Hardhat artifacts must exist)
      // If artifacts are missing, this test will fail with a helpful module-not-found error.
      const { sync } = await deployContracts(provider, admin);
      const syncTyped = asSync(sync);

      // Create a dedicated agent key that only the Rust server will use.
      const agent = ethers.Wallet.createRandom().connect(provider);
      const agentAddr = await agent.getAddress();

      // Fund agent (so server can pay gas)
      await (await admin.sendTransaction({ to: agentAddr, value: ethers.parseEther("1.0") })).wait();

      // User submits a data link
      const dataLink = "ipfs://bafy-agent-e2e";
      await (
        await asSync(syncTyped.connect(user)).submitData(
          dataLink,
          "Healthcare",
          "PatientData",
          "ipfs",
          "image",
          "png",
          123n
        )
      ).wait();

      // Start Rust agent_server wired to this node and agent wallet
      const agentPk = agent.privateKey;
      const proxyAddr = await sync.getAddress();
      const srv = spawnAgentServer(serverPort, rpcUrl, proxyAddr, agentPk);

      try {
        await waitForHttpOk(`http://127.0.0.1:${serverPort}/health`, 30_000);

        // Ask server to create its agent config on-chain.
        const createRes = await fetch(`http://127.0.0.1:${serverPort}/create-agent`, {
          method: "POST",
          headers: { origin: "http://localhost:3000" }
        });
        expect(createRes.status).to.equal(200);
        const createBody = await createRes.json();
        expect(createBody.txHash).to.match(/^0x[0-9a-fA-F]{64}$/);
        expect(createBody.agent.toLowerCase()).to.equal(agentAddr.toLowerCase());

        // Admin allows agent (admin key is NOT used by server)
        await (await asSync(syncTyped.connect(admin)).allowAgent(agentAddr)).wait();

        // Call /rate
        const res = await fetch(`http://127.0.0.1:${serverPort}/rate`, {
          method: "POST",
          headers: { "content-type": "application/json", origin: "http://localhost:3000" },
          body: JSON.stringify({ dataLink, isValid: true, rating: 55 })
        });
        expect(res.status).to.equal(200);
        const body = await res.json();
        expect(body.txHash).to.match(/^0x[0-9a-fA-F]{64}$/);
        expect(body.dataKey).to.match(/^0x[0-9a-fA-F]{64}$/);
        expect(body.agent.toLowerCase()).to.equal(agentAddr.toLowerCase());

        console.log("body", body);

        // Assert on-chain effect: accumulatedCredits(user) increased by 55
        const credits: bigint = await syncTyped.accumulatedCredits(userAddr);
        expect(credits).to.equal(55n);
      } finally {
        srv.kill("SIGKILL");
      }
    } finally {
      node.kill("SIGKILL");
    }
  });
});



