# CHATS Smart Contracts

Multi-chain smart contracts powering the CHATS humanitarian aid platform. Supports both **EVM** (Ethereum, Polygon, etc.) and **Stellar Soroban** blockchains.

```
chats-contracts/
├── evm/                    # Solidity contracts (Hardhat)
├── soroban/                # Rust contracts (Stellar)
└── shared/                 # ABIs, types, documentation
```

## Quick Start

### EVM Contracts

```bash
cd evm
npm install
npx hardhat compile
npx hardhat test
```

### Soroban Contracts

```bash
cd soroban
cargo build --release
cargo test
```

## Contracts Overview

| Chain | Contract | Description |
|-------|----------|-------------|
| Soroban | [`project-contract`](/contracts/soroban/project-contract/src/lib.rs) | Cash & item allowance management with RBAC |
| EVM | [`ProjectContract.sol`](evm/contracts/ProjectContract.sol) | ERC20-based allowance management |
| EVM | [`ProjectFactory.sol`](evm/contracts/ProjectFactory.sol) | Factory for deploying project instances |
| EVM | [`ERC20Token.sol`](evm/contracts/ERC20Token.sol) | Base ERC20 token |

## Architecture

Both implementations share the same core functionality:

- **Role-Based Access Control**: Super Admin, Admin, NGO, Vendor, Beneficiary
- **Allowance Management**: Cash and item allowances with expiry
- **Batch Operations**: Efficient bulk allocation and claiming
- **Security**: Pause functionality, reentrancy protection, input validation

```
┌─────────────────────────────────────────────────────────┐
│                    CHATS Platform                        │
├─────────────────────────┬───────────────────────────────┤
│      EVM Networks       │      Stellar Network          │
│  (Ethereum, Polygon)    │        (Soroban)              │
├─────────────────────────┼───────────────────────────────┤
│  ProjectContract.sol    │    project-contract (Rust)    │
│  - ERC20 Integration    │    - Native token support     │
│  - Hardhat tooling      │    - WASM compilation         │
└─────────────────────────┴───────────────────────────────┘
```

## Documentation

- [EVM Contracts](evm/README.md) - Setup, deployment, and usage
- [Soroban Contracts](soroban/README.md) - Setup, deployment, and usage
- [Architecture](shared/docs/architecture.md) - Detailed design docs

## Development

### Testing
- Write comprehensive unit tests
- Test both individual and integrated functionality
- Include edge cases and error conditions

### Security
- Follow best practices for smart contract security
- Implement proper access controls
- Include reentrancy protection
- Validate all inputs
- Document events
- Provide usage examples

### Deployment
- Test thoroughly on testnets
- Verify contract interactions
- Monitor gas usage
- Document deployment steps
- Maintain deployment records

## Getting Started

### Prerequisites
- Rust (for Soroban contracts)
- Solidity (for Ethereum contracts)
- Node.js and npm
- Hardhat or Truffle
- Soroban CLI

### Installation
```bash
# build Soroban contracts
cd soroban
cargo build --target wasm32-unknown-unknown --release

# Install dependencies
npm install

```

### Testing
```bash
# Test Ethereum contracts
npm test

# Test Soroban contracts
cargo test
```

## Contributing
1. Fork the repository
2. Create your feature branch
3. Write tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 