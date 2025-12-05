
import {
    Keypair,
    StrKey,
  } from "stellar-sdk";

  import bip39 from 'bip39';
  import ed25519 from 'ed25519-hd-key';

// Replace with your mnemonic (seed phrase)
const mnemonic = 
"girl uphold idle spoon sugar awkward illness flip enact rival until note detect warrior mimic final hold coil rich define first success cherry plug"

// Convert the mnemonic into a seed (synchronously)
const seed = bip39.mnemonicToSeedSync(mnemonic);

// Define the derivation path for Stellar
const derivationPath = "m/44'/148'/0'";

// Derive the key using the ed25519-hd-key package
const derived = ed25519.derivePath(derivationPath, seed.toString('hex'));

// The derived key (32 bytes) is the raw secret key
const secretKeyHex = derived.key.toString('hex');
console.log("Raw Secret Key (hex):", secretKeyHex);

// Optional: Convert the raw secret key into Stellar's secret key format (StrKey)
// Stellar's secret keys are encoded in base32 with a checksum. Use the StrKey.encodeEd25519SecretSeed function.
const stellarSecretKey = StrKey.encodeEd25519SecretSeed(Buffer.from(derived.key));
console.log("Stellar Secret Key (StrKey):", stellarSecretKey);

// You can also derive the corresponding public key if needed:
const publicKey = Keypair.fromSecret(stellarSecretKey).publicKey();
console.log("Stellar Public Key:", publicKey);
