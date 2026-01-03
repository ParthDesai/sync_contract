import hre from "hardhat";

const { ethers, network } = hre;

function mustEnv(name: string): string {
  const v = process.env[name];
  if (!v || v.trim().length === 0) throw new Error(`${name} is required`);
  return v.trim();
}

async function main() {
  const proxy = mustEnv("SYNC_CONTRACT_PROXY");
  const newAdmin = mustEnv("NEW_ADMIN");

  const [signer] = await ethers.getSigners();
  const signerAddr = await signer.getAddress();
  const chainId = Number((await signer.provider!.getNetwork()).chainId);

  console.log(`network=${network.name} chainId=${chainId}`);
  console.log(`signer=${signerAddr}`);
  console.log(`proxy=${proxy}`);
  console.log(`newAdmin=${newAdmin}`);

  const sync = await ethers.getContractAt("SyncContract", proxy, signer);
  const currentAdmin: string = await sync.admin();
  console.log(`currentAdmin=${currentAdmin}`);

  const tx = await sync.transferAdmin(newAdmin);
  console.log(`txHash=${tx.hash}`);
  await tx.wait();

  const updatedAdmin: string = await sync.admin();
  console.log(`updatedAdmin=${updatedAdmin}`);
}

main().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});


