import { expect } from "chai";
import { ethers } from "hardhat";
import type { BaseContract, ContractTransactionResponse } from "ethers";

function bytes32RightPadUtf8(s: string): string {
  const b = ethers.toUtf8Bytes(s);
  if (b.length > 32) throw new Error("string too long");
  const zeros = new Uint8Array(32 - b.length);
  return ethers.hexlify(ethers.concat([b, zeros]));
}

type Token = BaseContract & {
  mintAuthority(): Promise<string>;
  transferMintAuthority(newAuthority: string): Promise<ContractTransactionResponse>;
  balanceOf(owner: string): Promise<bigint>;
};

type AgentCfg = { exists: boolean; isEnabled: boolean };

type AgentResponse = {
  agent: string;
  isValid: boolean;
  isSeedDeleted: boolean;
  hasRating: boolean;
  rating: number;
  calculatedCredits: bigint;
  hasSyntheticDataLink: boolean;
};

type Submission = {
  exists: boolean;
  user: string;
  timestamp: bigint;
  domain: string;
  dataType: string;
  dataFormat: string;
  fileSizeInKB: bigint;
  dataLink: string;
  header: { primaryCategory: string; secondaryCategory: string };
  isRated: boolean;
  response: AgentResponse;
};

type Sync = BaseContract & {
  createAgent(): Promise<ContractTransactionResponse>;
  allowAgent(agent: string): Promise<ContractTransactionResponse>;
  submitData(
    dataLink: string,
    primaryCategory: string,
    secondaryCategory: string,
    domain: string,
    dataType: string,
    dataFormat: string,
    fileSizeInKB: bigint
  ): Promise<ContractTransactionResponse>;
  rateData(
    dataLink: string,
    isSeedDeleted: boolean,
    isValid: boolean,
    hasRating: boolean,
    rating: number,
    hasSyntheticDataLink: boolean,
    syntheticDataLink: string,
    sendTokensImmediately: boolean
  ): Promise<ContractTransactionResponse>;
  claimCredits(): Promise<ContractTransactionResponse>;
  transferMintAuthority(newAuthority: string): Promise<ContractTransactionResponse>;
  getSubmissionKey(dataLink: string): Promise<string>;
  getSubmission(dataKey: string): Promise<Submission>;
  userSubmissionCount(user: string): Promise<bigint>;
  getUserSubmissionKeys(user: string, start: bigint, limit: bigint): Promise<string[]>;
  getUserSubmissionSummaries(
    user: string,
    start: bigint,
    limit: bigint
  ): Promise<
    Array<{
      dataKey: string;
      user: string;
      timestamp: bigint;
      dataLink: string;
      domain: string;
      dataType: string;
      dataFormat: string;
      fileSizeInKB: bigint;
      primaryCategory: string;
      secondaryCategory: string;
      isRated: boolean;
    }>
  >;
  agentConfigs(agent: string): Promise<AgentCfg>;
  accumulatedCredits(user: string): Promise<bigint>;
};

function asSync(c: BaseContract): Sync {
  return c as unknown as Sync;
}

function asToken(c: BaseContract): Token {
  return c as unknown as Token;
}

describe("SyncContract (whole flow port)", () => {
  it("matches the Solana whole-flow behavior", async () => {
    const [payer, agent, user, newAuthority] = await ethers.getSigners();

    const Token = await ethers.getContractFactory("StrovaCreditToken");
    const token = (await Token.deploy("Strova Credit", "Strova", 9, payer.address)) as unknown as Token;
    await token.waitForDeployment();

    const Impl = await ethers.getContractFactory("SyncContract");
    const impl = await Impl.deploy();
    await impl.waitForDeployment();

    const initData = impl.interface.encodeFunctionData("initialize", [
      payer.address,
      await token.getAddress()
    ]);

    const Proxy = await ethers.getContractFactory("SyncContractProxy");
    const proxy = await Proxy.deploy(await impl.getAddress(), initData);
    await proxy.waitForDeployment();

    const sync = (await ethers.getContractAt("SyncContract", await proxy.getAddress())) as unknown as Sync;

    // Make the proxy (SyncContract) the mint authority before any minting happens.
    await asToken(token.connect(payer)).transferMintAuthority(await proxy.getAddress());
    expect(await token.mintAuthority()).to.equal(await proxy.getAddress());

    // Create agent config
    await expect(asSync(sync.connect(agent)).createAgent()).to.emit(sync, "AgentCreated").withArgs(agent.address);
    const agentCfg = await sync.agentConfigs(agent.address);
    expect(agentCfg.exists).to.equal(true);
    expect(agentCfg.isEnabled).to.equal(false);

    // Non-admin cannot allow agent
    await expect(asSync(sync.connect(agent)).allowAgent(agent.address)).to.be.revertedWithCustomError(
      sync,
      "ErrCallerNotAdmin"
    );

    // Submit data
    const dataLink = "Hello world";
    const primaryCategory = "Healthcare";
    const secondaryCategory = "PatientData";
    const domain = "ipfs";
    const dataType = "image";
    const dataFormat = "png";
    const fileSizeInKB = 123;

    const dataKey = await sync.getSubmissionKey(dataLink);
    await expect(
      asSync(sync.connect(user)).submitData(
        dataLink,
        primaryCategory,
        secondaryCategory,
        domain,
        dataType,
        dataFormat,
        fileSizeInKB
      )
    )
      .to.emit(sync, "DataSubmitted")
      .withArgs(dataKey, user.address, dataLink);

    const sub0 = await sync.getSubmission(dataKey);
    expect(sub0.exists).to.equal(true);
    expect(sub0.user).to.equal(user.address);
    expect(sub0.timestamp).to.not.equal(0);
    expect(sub0.domain).to.equal(domain);
    expect(sub0.dataType).to.equal(dataType);
    expect(sub0.dataFormat).to.equal(dataFormat);
    expect(sub0.fileSizeInKB).to.equal(BigInt(fileSizeInKB));
    expect(sub0.isRated).to.equal(false);
    expect(sub0.dataLink).to.equal(ethers.hexlify(ethers.toUtf8Bytes(dataLink)));
    expect(sub0.header.primaryCategory).to.equal(bytes32RightPadUtf8(primaryCategory));
    expect(sub0.header.secondaryCategory).to.equal(bytes32RightPadUtf8(secondaryCategory));

    // User index / pagination should include this submission
    expect(await sync.userSubmissionCount(user.address)).to.equal(1n);
    const keysPage0 = await sync.getUserSubmissionKeys(user.address, 0n, 10n);
    expect(keysPage0).to.deep.equal([dataKey]);
    const summaries0 = await sync.getUserSubmissionSummaries(user.address, 0n, 10n);
    expect(summaries0.length).to.equal(1);
    expect(summaries0[0].dataKey).to.equal(dataKey);
    expect(summaries0[0].user).to.equal(user.address);
    expect(summaries0[0].domain).to.equal(domain);
    expect(summaries0[0].dataType).to.equal(dataType);
    expect(summaries0[0].dataFormat).to.equal(dataFormat);
    expect(summaries0[0].fileSizeInKB).to.equal(BigInt(fileSizeInKB));

    // Disabled agent cannot rate
    await expect(
      asSync(sync.connect(agent)).rateData(dataLink, true, true, true, 100, false, "", false)
    ).to.be.revertedWithCustomError(sync, "ErrAgentIsNotEnabled");

    // Admin allows agent
    await expect(asSync(sync.connect(payer)).allowAgent(agent.address))
      .to.emit(sync, "AgentAllowed")
      .withArgs(agent.address);

    // Seed must be deleted before rating (if synthetic link present)
    await expect(
      asSync(sync.connect(agent)).rateData(
        dataLink,
        false,
        true,
        true,
        85,
        true,
        "Hello World processed",
        false
      )
    ).to.be.revertedWithCustomError(sync, "ErrSeedMustbeDeletedBeforeRating");

    // Valid rate (accumulate credits)
    await expect(
      asSync(sync.connect(agent)).rateData(
        dataLink,
        true,
        true,
        true,
        85,
        true,
        "Hello World processed",
        false
      )
    ).to.emit(sync, "DataRated");

    expect(await sync.accumulatedCredits(user.address)).to.equal(85);

    const sub1 = await sync.getSubmission(dataKey);
    expect(sub1.isRated).to.equal(true);
    expect(sub1.response.agent).to.equal(agent.address);
    expect(sub1.response.isValid).to.equal(true);
    expect(sub1.response.hasRating).to.equal(true);
    expect(sub1.response.rating).to.equal(85);
    expect(sub1.response.calculatedCredits).to.equal(85);
    expect(sub1.response.isSeedDeleted).to.equal(true);
    expect(sub1.response.hasSyntheticDataLink).to.equal(true);

    // Cannot rate again
    await expect(
      asSync(sync.connect(agent)).rateData(dataLink, true, true, true, 100, false, "", false)
    ).to.be.revertedWithCustomError(sync, "ErrDataAlreadyRated");

    // Second submission: invalid but rated => credits 0, accumulated remains 85
    const dataLink1 = "Hello world 1";
    await asSync(sync.connect(user)).submitData(
      dataLink1,
      primaryCategory,
      secondaryCategory,
      domain,
      dataType,
      dataFormat,
      fileSizeInKB
    );
    await asSync(sync.connect(agent)).rateData(dataLink1, true, false, true, 82, false, "", false);
    expect(await sync.accumulatedCredits(user.address)).to.equal(85);
    expect(await sync.userSubmissionCount(user.address)).to.equal(2n);

    // Valid but NO rating => credits 0, accumulated unchanged
    const dataLink1b = "Hello world 1b";
    const key1b = await sync.getSubmissionKey(dataLink1b);
    await asSync(sync.connect(user)).submitData(
      dataLink1b,
      primaryCategory,
      secondaryCategory,
      domain,
      dataType,
      dataFormat,
      fileSizeInKB
    );
    await asSync(sync.connect(agent)).rateData(
      dataLink1b,
      true,
      true, // isValid
      false, // hasRating (omitted)
      0, // rating ignored
      true, // hasSyntheticDataLink (optional)
      "Hello World processed",
      false
    );
    expect(await sync.accumulatedCredits(user.address)).to.equal(85);
    const sub1b = await sync.getSubmission(key1b);
    expect(sub1b.isRated).to.equal(true);
    expect(sub1b.response.isValid).to.equal(true);
    expect(sub1b.response.hasRating).to.equal(false);
    expect(sub1b.response.calculatedCredits).to.equal(0);
    expect(await sync.userSubmissionCount(user.address)).to.equal(3n);

    // Third submission: valid & rated => +20 credits
    const dataLink2 = "Hello world 2";
    await asSync(sync.connect(user)).submitData(
      dataLink2,
      primaryCategory,
      secondaryCategory,
      domain,
      dataType,
      dataFormat,
      fileSizeInKB
    );
    await asSync(sync.connect(agent)).rateData(
      dataLink2,
      true,
      true,
      true,
      20,
      true,
      "Hello World processed",
      false
    );
    expect(await sync.accumulatedCredits(user.address)).to.equal(105);
    expect(await sync.userSubmissionCount(user.address)).to.equal(4n);

    // Fourth submission: send tokens immediately => balance increases, accumulated unchanged
    const dataLink3 = "Hello world 3";
    await asSync(sync.connect(user)).submitData(
      dataLink3,
      primaryCategory,
      secondaryCategory,
      domain,
      dataType,
      dataFormat,
      fileSizeInKB
    );

    const beforeBal = await token.balanceOf(user.address);
    await asSync(sync.connect(agent)).rateData(
      dataLink3,
      true,
      true,
      true,
      57,
      true,
      "Hello World processed",
      true
    );
    const afterBal = await token.balanceOf(user.address);
    expect(afterBal - beforeBal).to.equal(57n * 10n ** 9n);
    expect(await sync.accumulatedCredits(user.address)).to.equal(105);
    expect(await sync.userSubmissionCount(user.address)).to.equal(5n);

    // Pagination: fetch last 2 keys
    const last2 = await sync.getUserSubmissionKeys(user.address, 3n, 10n);
    expect(last2.length).to.equal(2);

    // Claim credits => user gets +105 tokens and accumulated resets
    const beforeClaim = await token.balanceOf(user.address);
    await asSync(sync.connect(user)).claimCredits();
    const afterClaim = await token.balanceOf(user.address);
    expect(afterClaim - beforeClaim).to.equal(105n * 10n ** 9n);
    expect(await sync.accumulatedCredits(user.address)).to.equal(0);
    expect(afterClaim).to.equal((57n + 105n) * 10n ** 9n);

    // Only admin can transfer mint authority
    await expect(
      asSync(sync.connect(user)).transferMintAuthority(newAuthority.address)
    ).to.be.revertedWithCustomError(sync, "ErrCallerNotAdmin");

    // Admin transfers mint authority away from sync contract (proxy)
    await asSync(sync.connect(payer)).transferMintAuthority(newAuthority.address);
    expect(await token.mintAuthority()).to.equal(newAuthority.address);

    // Now claim credits must fail because SyncContract is no longer mint authority
    await expect(asSync(sync.connect(user)).claimCredits()).to.be.revertedWithCustomError(
      token,
      "NotMintAuthority"
    );
  });
});


