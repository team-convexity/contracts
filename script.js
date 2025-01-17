const { ethers } = require("ethers");
const crypto = require("crypto");
const fs = require("fs");
const path = require("path");

// Encryption Configuration
const ENCRYPTION_SECRET = process.env.ENCRYPTION_SECRET || "super-secure-password"; 
const ALGORITHM = "aes-256-cbc";

// Function to encrypt the private key
function encryptPrivateKey(privateKey) {
    const iv = crypto.randomBytes(16);
    const cipher = crypto.createCipheriv(
        ALGORITHM,
        crypto.scryptSync(ENCRYPTION_SECRET, "salt", 32),
        iv
    );
    const encrypted = Buffer.concat([cipher.update(privateKey), cipher.final()]);
    return `${iv.toString("hex")}:${encrypted.toString("hex")}`;
}

// Function to decrypt the private key
function decryptPrivateKey(encryptedPrivateKey) {
    const [ivHex, encryptedDataHex] = encryptedPrivateKey.split(":");
    const decipher = crypto.createDecipheriv(
        ALGORITHM,
        crypto.scryptSync(ENCRYPTION_SECRET, "salt", 32),
        Buffer.from(ivHex, "hex")
    );
    const decrypted = Buffer.concat([
        decipher.update(Buffer.from(encryptedDataHex, "hex")),
        decipher.final(),
    ]);
    return decrypted.toString();
}

// Generate Ethereum Address and Securely Store the Private Key
async function generateSecureEthereumAddress() {
    // Generate a new wallet
    const wallet = ethers.Wallet.createRandom();

    // Encrypt the private key
    const encryptedPrivateKey = encryptPrivateKey(wallet.privateKey);

    const filePath = path.join(__dirname, "secure-keys.json");
    const data = { address: wallet.address, encryptedPrivateKey };
    fs.writeFileSync(filePath, JSON.stringify(data, null, 2));

    return wallet.address;
}

// Decrypt and Use the Private Key
async function usePrivateKeyForSigning(address) {
    // Read the encrypted private key from storage
    const filePath = path.join(__dirname, "secure-keys.json");
    const { encryptedPrivateKey } = JSON.parse(fs.readFileSync(filePath, "utf-8"));

    // Decrypt the private key
    const privateKey = decryptPrivateKey(encryptedPrivateKey);

    // Create a wallet instance
    const wallet = new ethers.Wallet(privateKey);
    
   return wallet;
   
}

// Main Function
(async () => {
    const address = await generateSecureEthereumAddress();
    console.log(`Generated Address: ${address}`);

    const wallet = await usePrivateKeyForSigning(address);

     // Sign a transaction
     const signedMessage = await wallet.signMessage("Secure signing");
     console.log("Signed Message:", signedMessage);
})();
