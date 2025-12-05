// const StellarSdk = require('stellar-sdk');

import {
  Keypair,
  Horizon,
  TransactionBuilder,
  Networks,
  Operation,
  Asset,
  BASE_FEE,
} from "stellar-sdk";

// Use the Stellar public network or testnet
// const server = new StellarSdk.Server('https://horizon.stellar.org'); // Public network
// const server = new StellarSdk.Server('https://horizon-testnet.stellar.org'); // Testnet

const horizonUrl = "https://horizon-testnet.stellar.org";// Testnet

const server = new Horizon.Server(horizonUrl);

// Your secret key and public key
const senderSecret = 'SCW6OUXFHRVAQ3STRZTC5VBGVIRN4SPNOEAQQNGQXAJCPEOIY33S5ORF'; 
const senderKeypair = Keypair.fromSecret(senderSecret);
const senderPublicKey = senderKeypair.publicKey();


const recipientKeypair = Keypair.random();
// Recipient's Stellar address
const recipientPublicKey = recipientKeypair.publicKey(); // Replace with recipient's Stellar address

console.log(
  `recipient account keys:\n${recipientKeypair.publicKey()}\n${recipientKeypair.secret()}\n`,
);

// Asset to send (USDC on Stellar)
const usdc = new Asset(
  'USDC',
  'GCYEIQEWOCTTSA72VPZ6LYIZIK4W4KNGJR72UADIXUXG45VDFRVCQTYE'
);

async function sendUSDC(amount) {
  try {
    // Load the sender's account
    const senderAccount = await server.loadAccount(senderPublicKey);

    // Create the transaction
    const transaction = new TransactionBuilder(senderAccount, {
      fee: BASE_FEE,
      networkPassphrase: Networks.TESTNET, // Use `TESTNET` for test network
    })
      .addOperation(
        Operation.payment({
          destination: recipientPublicKey,
          asset: usdc,
          amount: amount.toString(), // Amount as a string
        })
      )
      .setTimeout(30)
      .build();

    // Sign the transaction
    transaction.sign(senderKeypair);

    // Submit the transaction to the Stellar network
    const result = await server.submitTransaction(transaction);
    console.log('Transaction successful!', result);
  } catch (error) {
    console.error('Error sending USDC:', error.response?.data || error.message);
  }
}

// Amount of USDC to send
sendUSDC(5); // Replace with the desired amount
