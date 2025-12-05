# CHATS EVM Smart Contracts

Ethereum/EVM-compatible smart contracts for the CHATS platform.

## Contracts

| Contract | Description |
|----------|-------------|
| `ProjectContract.sol` | Main contract for managing token-based allowances and vendor interactions |
| `ProjectFactory.sol` | Factory contract for deploying new project instances |
| `ERC20Token.sol` | Base ERC20 token implementation |
| `USDTContract.sol` | Test USDT token for Sepolia testnet |

## Features
- ERC20 token integration
- Role-based access control (NGO, Vendor, Beneficiary)
- Allowance allocation and claiming
- Vendor registration and management
- Spending limit controls
- Reentrancy protection

## Setup
```bash

# Install dependencies
npm install

# Compile contracts
npx hardhat compile

# Run tests
npx hardhat test

# Deploy to Sepolia
npx hardhat run scripts/deploy.js --network sepolia
```

## Environment Variables

Create a `.env` file:

```env
ALCHEMY_API_KEY=your_alchemy_api_key
SEPOLIA_PRIVATE_KEY=your_private_key
ETHERSCAN_API_KEY=your_etherscan_api_key
```

## Deployments

See `deployments/` for deployed contract addresses per network.

## License

See [LICENSE](../LICENSE)
