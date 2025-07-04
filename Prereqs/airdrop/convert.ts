import bs58 from "bs58";
import promptSync from 'prompt-sync';
const prompt = promptSync(); // Initialize prompt-sync

// Convert Base58 private key to a Uint8Array (Solana Wallet format)
function base58ToWallet(): Uint8Array {
    const base58Key = prompt("Enter your Base58 private key: ").trim();
    try {
        const decodedKey = bs58.decode(base58Key);
        console.log("Decoded Private Key (Uint8Array):", decodedKey);
        return decodedKey;
    } catch (error) {
        console.error("Error decoding Base58:", error);
        return new Uint8Array();
    }
}

// Convert Uint8Array (Solana Wallet format) to Base58 (Phantom format)
function walletToBase58(): string {
    const input = prompt("Enter your private key as a comma-separated byte array: ").trim();
    try {
        const byteArray = new Uint8Array(input.split(",").map(Number));
        const encodedKey = bs58.encode(byteArray);
        console.log("Base58 Encoded Private Key:", encodedKey);
        return encodedKey;
    } catch (error) {
        console.error("Error encoding Base58:", error);
        return "";
    }
}

// Main function
function main() {
    console.log("Choose an option:");
    console.log("1. Convert Base58 to Wallet Byte Array");
    console.log("2. Convert Wallet Byte Array to Base58");
    const choice = prompt("Enter choice (1 or 2): ").trim();

    if (choice === "1") {
        base58ToWallet();
    } else if (choice === "2") {
        walletToBase58();
    } else {
        console.log("Invalid choice. Exiting.");
    }
}

main();