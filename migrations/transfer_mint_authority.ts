// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SyncContract } from "../target/types/sync_contract";
import bs58 from "bs58";
import { 
  TOKEN_2022_PROGRAM_ID, 
  getMint
} from "@solana/spl-token";

// Configuration for mint authority transfer
// Set these values to enable mint authority transfer
const MINT_AUTHORITY_TRANSFER_CONFIG = {
  enabled: true, // Set to true to enable mint authority transfer
  mintAddress: "5oQf8sZdiMvzrBAdDk6M1m1BZEhYPD6FqHnDdSmHWUAe", // The mint address to transfer authority for
  newAuthorityAddress: "232udCxuTdF5Q9mWEGi3R77N4WzQpPDyepaNoYqZ3yn6", // The new authority address (provide public key as string)
};

// Helper function to get program state PDA
function getProgramStatePDA(programId: anchor.web3.PublicKey): [anchor.web3.PublicKey, number] {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("sync_program"), Buffer.from("global_state")],
    programId
  );
}

// Transfer mint authority using the program's instruction
async function transferMintAuthority(
  program: Program<SyncContract>,
  provider: anchor.AnchorProvider,
  config: typeof MINT_AUTHORITY_TRANSFER_CONFIG
): Promise<void> {
  try {
    console.log("🔄 Starting mint authority transfer...");
    
    // Validate configuration
    if (!config.mintAddress || !config.newAuthorityAddress) {
      throw new Error("Missing required configuration. Please provide mintAddress, newAuthorityAddress");
    }

    // Parse addresses
    const mintPubkey = new anchor.web3.PublicKey(config.mintAddress);
    const newAuthorityPubkey = new anchor.web3.PublicKey(config.newAuthorityAddress);
    
    // Get program state PDA
    const [programStatePDA] = getProgramStatePDA(program.programId);
    
    console.log("📋 Configuration:");
    console.log(`  Mint Address: ${mintPubkey.toString()}`);
    console.log(`  Admin: ${provider.publicKey.toString()}`);
    console.log(`  Current Authority (Program State PDA): ${programStatePDA.toString()}`);
    console.log(`  New Authority: ${newAuthorityPubkey.toString()}`);
    
    // Verify the mint exists and get its current authority
    try {
      const mintInfo = await getMint(
        provider.connection,
        mintPubkey,
        'confirmed',
        TOKEN_2022_PROGRAM_ID
      );
      
      console.log(`  Current Mint Authority: ${mintInfo.mintAuthority?.toString() || 'None'}`);
      
      // Verify that the current authority matches the program state PDA
      if (!mintInfo.mintAuthority?.equals(programStatePDA)) {
        console.warn(`⚠️  Warning: Current mint authority (${mintInfo.mintAuthority?.toString()}) does not match program state PDA (${programStatePDA.toString()})`);
      }
    } catch (error) {
      throw new Error(`Failed to get mint info: ${error.message}`);
    }
    
    // Call the program's transfer_mint_authority instruction
    console.log("🚀 Calling program's transfer_mint_authority instruction...");
    
    const tx = await program.methods
      .transferMintAuthority(newAuthorityPubkey)
      .accounts({
        signer: provider.publicKey,
        mint: mintPubkey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      }) // Type assertion needed due to Anchor TypeScript generation quirk
      .signers([provider.wallet.payer])
      .rpc({
        commitment: "finalized",
      });
    
    console.log("✅ Mint authority transfer completed successfully!");
    console.log(`  Transaction: ${tx}`);
    
    // Verify the transfer
    const updatedMintInfo = await getMint(
      provider.connection,
      mintPubkey,
      'confirmed',
      TOKEN_2022_PROGRAM_ID
    );
    
    console.log(`  New Mint Authority: ${updatedMintInfo.mintAuthority?.toString() || 'None'}`);
    
    if (updatedMintInfo.mintAuthority?.equals(newAuthorityPubkey)) {
      console.log("✅ Authority transfer verified successfully!");
    } else {
      console.error("❌ Authority transfer verification failed!");
    }
    
  } catch (error) {
    throw error;
  }
}

module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  // Add your deploy script here.
  const program = anchor.workspace.SyncContract as Program<SyncContract>;
  
  console.log("🚀 Starting deployment...");
  console.log(`Program ID: ${program.programId.toString()}`);
  
  // Transfer mint authority if enabled
  if (MINT_AUTHORITY_TRANSFER_CONFIG.enabled) {
    await transferMintAuthority(program, provider, MINT_AUTHORITY_TRANSFER_CONFIG);
  } else {
    console.log("ℹ️  Mint authority transfer is disabled. Set MINT_AUTHORITY_TRANSFER_CONFIG.enabled = true to enable it.");
  }
  
  console.log("✅ Deployment completed!");
};
