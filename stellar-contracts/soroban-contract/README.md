# Soroban Project

## Project Structure

This repository uses the recommended structure for a Soroban project:
```text
.
├── contracts
│   └── hello_world
│       ├── src
│       │   ├── lib.rs
│       │   └── test.rs
│       └── Cargo.toml
├── Cargo.toml
└── README.md
```

- New Soroban contracts can be put in `contracts`, each in their own directory. There is already a `hello_world` contract in there to get you started.
- If you initialized this project with any other example contracts via `--with-example`, those contracts will be in the `contracts` directory as well.
- Contracts should have their own `Cargo.toml` files that rely on the top-level `Cargo.toml` workspace for their dependencies.
- Frontend libraries can be added to the top-level directory as well. If you initialized this project with a frontend template via `--frontend-template` you will have those files already included.

## Compiling Contracts

To compile a Soroban contract, navigate to the contract directory and run:

```sh
cargo build --release
```

This will generate the WASM file in the `target/wasm32-unknown-unknown/release/` directory.

## Running Tests

To run tests for a contract, use:

```sh
cargo test
```

This will execute all unit and integration tests defined in the contract's source files.

## Deploying the Contract

To deploy the compiled contract to the Soroban network:

1. Ensure you have the Soroban CLI installed. See [Soroban CLI Installation](https://soroban.stellar.org/docs/getting-started/installation).
2. Use the following command to deploy:

```sh
soroban contract deploy --wasm target/wasm32-unknown-unknown/release/<contract_name>.wasm --network <network> --source <account>
```

Replace `<contract_name>`, `<network>`, and `<account>` with your contract's name, the target network, and your account address.

For more details, refer to the [Soroban documentation](https://soroban.stellar.org/docs/).

