import * as anchor from "@coral-xyz/anchor";
import bs58 from "bs58";

/**
 * Converts a private key array to base58 encoded string
 * @param privateKeyArray - Array of numbers representing the private key
 * @returns Base58 encoded string
 */
function arrayToBase58(privateKeyArray: number[]): string {
  const uint8Array = new Uint8Array(privateKeyArray);
  return bs58.encode(uint8Array);
}

/**
 * Converts a base58 encoded string to private key array
 * @param base58String - Base58 encoded private key string
 * @returns Array of numbers representing the private key
 */
function base58ToArray(base58String: string): number[] {
  const uint8Array = bs58.decode(base58String);
  return Array.from(uint8Array);
}

/**
 * Creates a keypair from a private key array
 * @param privateKeyArray - Array of numbers representing the private key
 * @returns Solana Keypair object
 */
function arrayToKeypair(privateKeyArray: number[]): anchor.web3.Keypair {
  const uint8Array = new Uint8Array(privateKeyArray);
  return anchor.web3.Keypair.fromSecretKey(uint8Array);
}

/**
 * Main function to handle command line arguments
 */
function main() {
  const args = process.argv.slice(2);
  
  if (args.length === 0) {
    console.log("🔑 Private Key Converter");
    console.log("========================");
    console.log("");
    console.log("Usage:");
    console.log("  npm run convert-keys <command> [options]");
    console.log("");
    console.log("Commands:");
    console.log("  array-to-base58 <array>     Convert array to base58 string");
    console.log("  base58-to-array <string>    Convert base58 string to array");
    console.log("  validate <array>            Validate array and show all formats");
    console.log("");
    console.log("Examples:");
    console.log("  npm run convert-keys array-to-base58 '[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64]'");
    console.log("  npm run convert-keys base58-to-array 'YourBase58KeyHere'");
    console.log("  npm run convert-keys validate '[1,2,3,...]'");
    console.log("");
    return;
  }
  
  const command = args[0];
  const input = args[1];
  
  try {
    switch (command) {
      case "array-to-base58":
        if (!input) {
          console.error("❌ Error: Please provide a private key array");
          return;
        }
        
        const privateKeyArray = JSON.parse(input);
        if (!Array.isArray(privateKeyArray) || privateKeyArray.length !== 64) {
          console.error("❌ Error: Private key must be an array of 64 numbers");
          return;
        }
        
        const base58Key = arrayToBase58(privateKeyArray);
        const keypair = arrayToKeypair(privateKeyArray);
        
        console.log("✅ Conversion successful!");
        console.log("📋 Public Key:", keypair.publicKey.toString());
        console.log("🔐 Private Key (Base58):", base58Key);
        break;
        
      case "base58-to-array":
        if (!input) {
          console.error("❌ Error: Please provide a base58 private key string");
          return;
        }
        
        const arrayKey = base58ToArray(input);
        const keypairFromBase58 = anchor.web3.Keypair.fromSecretKey(bs58.decode(input));
        
        console.log("✅ Conversion successful!");
        console.log("📋 Public Key:", keypairFromBase58.publicKey.toString());
        console.log("🔐 Private Key (Array):", JSON.stringify(arrayKey));
        break;
        
      case "validate":
        if (!input) {
          console.error("❌ Error: Please provide a private key array");
          return;
        }
        
        const validateArray = JSON.parse(input);
        if (!Array.isArray(validateArray) || validateArray.length !== 64) {
          console.error("❌ Error: Private key must be an array of 64 numbers");
          return;
        }
        
        const validatedBase58 = arrayToBase58(validateArray);
        const validatedKeypair = arrayToKeypair(validateArray);
        
        console.log("🔍 Key Validation & Conversion");
        console.log("==============================");
        console.log("📋 Public Key:", validatedKeypair.publicKey.toString());
        console.log("🔐 Private Key (Array):", JSON.stringify(validateArray));
        console.log("🔐 Private Key (Base58):", validatedBase58);
        console.log("✅ Key is valid!");
        break;
        
      default:
        console.error("❌ Error: Unknown command. Use 'array-to-base58', 'base58-to-array', or 'validate'");
    }
  } catch (error) {
    console.error("❌ Error:", error.message);
  }
}

// Example usage function for testing
export function convertExampleKeys() {
  console.log("🔑 Example Key Conversions");
  console.log("=========================");
  
  // Generate a sample keypair
  const sampleKeypair = anchor.web3.Keypair.generate();
  const sampleArray = Array.from(sampleKeypair.secretKey);
  const sampleBase58 = bs58.encode(sampleKeypair.secretKey);
  
  console.log("📋 Sample Public Key:", sampleKeypair.publicKey.toString());
  console.log("🔐 Sample Private Key (Array):", JSON.stringify(sampleArray));
  console.log("🔐 Sample Private Key (Base58):", sampleBase58);
  console.log("");
  
  // Convert back to verify
  const convertedKeypair = arrayToKeypair(sampleArray);
  console.log("✅ Verification - Public keys match:", 
    sampleKeypair.publicKey.equals(convertedKeypair.publicKey));
}

// Run main function if this script is executed directly
if (require.main === module) {
  main();
}

export { arrayToBase58, base58ToArray, arrayToKeypair }; 