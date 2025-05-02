# CHATS Smart Contracts

This repository contains the smart contracts that power the CHATS platform's blockchain functionality. The contracts are implemented in both Rust (for Stellar Soroban) and Solidity (for Ethereum), providing cross-chain compatibility.

## Core Contracts

### 1. Project Contract (Soroban)
[`projectContract/src/lib.rs`](contracts/stellar-contracts/soroban-contract/contracts/projectContract/src/lib.rs)

A Stellar Soroban smart contract that manages cash and item allowances for NGOs, vendors, and beneficiaries. It implements a role-based access control system and handles allowance allocation, claiming, and transfers.

Key Features:
- Role-based access control (NGO, Vendor, Beneficiary)
- Cash and item allowance management
- Batch operations for efficiency
- Expiry-based allowances
- Event emission for tracking
- Pause/unpause functionality

### 2. Project Contract (ERC20)
[`ProjectContract.sol`](contracts/contracts/ProjectContract.sol)

An Ethereum-based ERC20 implementation of the project contract. It manages token-based allowances and vendor interactions.

Key Features:
- ERC20 token integration
- Role-based access control
- Allowance allocation and claiming
- Vendor registration and management
- Spending limit controls
- Reentrancy protection

### 3. USDT Test Token
[`USDTContract.sol`](contracts/contracts/USDTContract.sol)

A simplified USDT implementation for testing purposes on the Sepolia testnet.

Key Features:
- Basic ERC20 functionality
- Minting capability
- Transfer and approval mechanisms
- Event emission

## Contract Architecture

### Stellar Soroban Implementation
The Soroban implementation focuses on:
- Efficient state management
- Role-based permissions
- Allowance tracking
- Event emission
- Batch operations

### Ethereum Implementation
The Ethereum implementation provides:
- ERC20 token integration
- Access control
- Allowance management
- Vendor interactions
- Security features
