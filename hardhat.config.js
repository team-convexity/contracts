require("@nomicfoundation/hardhat-toolbox");

const { ethers } = require("ethers");

require('dotenv').config();

const ALCHEMY_API_KEY = process.env.ALCHEMY_API_KEY;

const SEPOLIA_PRIVATE_KEY = process.env.SEPOLIA_PRIVATE_KEY;

const ETHERSCAN_API_KEY = process.env.ETHERSCAN_API_KEY;

module.exports = {
    // ...rest of your config...
    solidity: "0.8.28",
    etherscan: {
        apiKey: ETHERSCAN_API_KEY,
    },
    networks: {
        sepolia: {
            url: `https://eth-sepolia.g.alchemy.com/v2/${ALCHEMY_API_KEY}`,
            accounts: [SEPOLIA_PRIVATE_KEY],
            // gasPrice: ethers.parseUnits('20', 'gwei'),
        },
        hardhat: {
            accounts: {
                count: 10, // Increase the number of accounts
            },
        },
    },
};