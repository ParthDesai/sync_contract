import { arrayToBase58, base58ToArray, arrayToKeypair, convertExampleKeys } from './convert-keys';

console.log("🔑 Key Conversion Demo");
console.log("======================");
console.log("");

// Run the example conversion function
convertExampleKeys();

console.log("\n🧪 Manual Test:");
console.log("================");

// Create a specific example with a generated keypair
import * as anchor from "@coral-xyz/anchor";
const testKeypair = anchor.web3.Keypair.generate();
const testArray = Array.from(testKeypair.secretKey);

console.log("Generated keypair:");
console.log("📋 Public Key:", testKeypair.publicKey.toString());
console.log("🔐 Array format:", JSON.stringify(testArray));

// Convert to base58
const base58Key = arrayToBase58(testArray);
console.log("🔐 Base58 format:", base58Key);

// Convert back to verify
const backToArray = base58ToArray(base58Key);
const recreatedKeypair = arrayToKeypair(backToArray);

console.log("\n✅ Verification:");
console.log("Arrays match:", JSON.stringify(testArray) === JSON.stringify(backToArray));
console.log("Public keys match:", testKeypair.publicKey.equals(recreatedKeypair.publicKey));

console.log("\n📖 To convert deployment script output:");
console.log("Copy the array from deployment output and run:");
console.log(`yarn convert-keys array-to-base58 '${JSON.stringify(testArray)}'`); 