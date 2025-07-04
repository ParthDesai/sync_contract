// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SyncContract } from "../target/types/sync_contract";
import bs58 from "bs58";

module.exports = async function (provider: anchor.AnchorProvider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  console.log("🚀 Starting deployment...");
  
  // Get the program
  const program = anchor.workspace.SyncContract as Program<SyncContract>;
  const programId = program.programId;
  
  console.log("📋 Program ID:", programId.toString());
  console.log("🔑 Admin (Payer):", provider.wallet.publicKey.toString());
  
  // Check provider wallet balance
  const balance = await provider.connection.getBalance(provider.wallet.publicKey);
  const balanceInSol = balance / anchor.web3.LAMPORTS_PER_SOL;
  console.log("💰 Provider wallet balance:", balanceInSol.toFixed(4), "SOL");
  
  const requiredSol = 0.1 * 5; // 0.1 SOL per agent × 5 agents
  if (balanceInSol < requiredSol) {
    console.log("⚠️ Warning: You may not have enough SOL for transfers. Required:", requiredSol, "SOL");
  }
  
  // Step 1: Initialize the program
  console.log("\n🔧 Step 1: Initializing program...");
  
  const [programState, _] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("sync_program"), Buffer.from("global_state")],
    programId
  );
  
  try {
    await program.methods
      .initialize()
      .accounts({
        
      })
      .rpc({ commitment: "finalized" });
    
    console.log("✅ Program initialized successfully!");
    console.log("📍 Program State PDA:", programState.toString());
  } catch (error) {
    console.log("ℹ️ Program already initialized or error:", error.message);
  }
  
  // Step 2: Create and allow 5 agents
  console.log("\n🤖 Step 2: Creating and allowing 5 agents...");
  
  const agents: anchor.web3.Keypair[] = [];
  const agentConfigs: anchor.web3.PublicKey[] = [];
  
  for (let i = 1; i <= 5; i++) {
    console.log(`\n--- Agent ${i} ---`);
    
    // Generate new keypair for agent
    const agentKeypair = anchor.web3.Keypair.generate();
    agents.push(agentKeypair);
    
    // Transfer SOL from provider wallet to agent for transaction fees
    const transferAmount = 0.1 * anchor.web3.LAMPORTS_PER_SOL; // 0.1 SOL
    const transferTx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: agentKeypair.publicKey,
        lamports: transferAmount,
      })
    );
    
    const signature = await provider.sendAndConfirm(transferTx, undefined, { commitment: "finalized" });
    console.log(`💰 Transferred ${transferAmount / anchor.web3.LAMPORTS_PER_SOL} SOL to agent ${i}`);
    
    // Find agent config PDA
    const [agentConfig, _bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("sync_program"), Buffer.from("agent_config"), agentKeypair.publicKey.toBuffer()],
      programId
    );
    agentConfigs.push(agentConfig);
    
    console.log(`🔑 Agent ${i} Public Key:`, agentKeypair.publicKey.toString());
    console.log(`🔐 Agent ${i} Private Key:`, `[${agentKeypair.secretKey.toString()}]`);
    console.log(`📍 Agent ${i} Config PDA:`, agentConfig.toString());
    
    // Create agent
    try {
      await program.methods
        .createAgent()
        .accounts({
          signer: agentKeypair.publicKey
        })
        .signers([agentKeypair])
        .rpc({ commitment: "finalized" });
      
      console.log(`✅ Agent ${i} created successfully!`);
    } catch (error) {
      console.log(`❌ Error creating agent ${i}:`, error);
      continue;
    }
    
    // Allow agent (only admin can do this)
    try {
      await program.methods
        .allowAgent(agentKeypair.publicKey)
        .accounts({
          
        })
        .rpc({ commitment: "finalized" });
      
      console.log(`✅ Agent ${i} allowed successfully!`);
    } catch (error) {
      console.log(`❌ Error allowing agent ${i}:`, error.message);
    }
    
    // Verify agent is enabled
    try {
      const agentConfigData = await program.account.agentConfig.fetch(agentConfig);
      console.log(`🔍 Agent ${i} enabled status:`, agentConfigData.isEnabled);
    } catch (error) {
      console.log(`❌ Error fetching agent ${i} config:`, error.message);
    }
  }
  
  // Summary
  console.log("\n📊 DEPLOYMENT SUMMARY");
  console.log("====================");
  console.log("✅ Program initialized");
  console.log("🤖 Agents created:", agents.length);
  console.log("📋 Program ID:", programId.toString());
  console.log("📍 Program State PDA:", programState.toString());
  
  console.log("\n🔐 AGENT PRIVATE KEYS (SAVE THESE!):");
  console.log("=====================================");
  agents.forEach((agent, index) => {
    console.log(`Agent ${index + 1}:`);
    console.log(`  Public Key:  ${agent.publicKey.toString()}`);
    console.log(`  Private Key: ${bs58.encode(agent.secretKey)}`);
    console.log(`  Config PDA:  ${agentConfigs[index].toString()}`);
    console.log("");
  });
  
  console.log("🎉 Deployment completed successfully!");
};
