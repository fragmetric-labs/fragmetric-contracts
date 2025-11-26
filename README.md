# Fragmetric Solana Programs

This repository contains the full business logic of Fragmetric on-chain programs, test codes and client SDK.

# Contribution Guide

## 1. Install Requirements
Refer to the [Dockerfile](./.github/anchor.Dockerfile) of verifiable builder image.

```shell
# install Solana CLI (v3.0.6 target)
$ sh -c "$(curl -sSfL https://release.anza.xyz/v3.0.6/install)"

# install Rust toolchain (v1.91.1 target)
$ sh -c "$(curl -sSfL https://sh.rustup.rs)"
$ rustup default stable
$ rustup update

# if toolchain 'solana' is uninstalled
$ cargo-build-sbf --force-tools-install

# install AVM (anchor version manager)
$ cargo install --git https://github.com/coral-xyz/anchor avm --force
$ avm use 0.31.1

# Setup nodejs env (v24.11.1 target)
# install pnpm (v20.19.0 target)
$ sh -c "$(curl -fsSL https://get.pnpm.io/install.sh)"

# install js packages
$ pnpm i
```

## 2. Configure Program KeyPairs
```shell
# to sync shared local keypairs
$ pnpm keypairs:local

# to sync authorized keypairs for in-house engineers
$ pnpm keypairs
```

## 3. Build Program
```shell
# to build all artifacts
$ anchor build

# to build a single program
$ anchor build -p restaking

# to build release candidates, verifiable release artifacts will be built from CI workflow
$ anchor build -p restaking -- --features mainnet
$ anchor build -p solv -- --features devnet 
```

## 4. Run Unit Tests
```shell
$ cargo test-sbf

# or
$ cargo test-sbf -p restaking
```

## 5. Run Integration Tests
```shell
# ensure fresh local builds
$ anchor build

# using LiteSVM - which is quite faster than solana-test-validator
$ pnpm test ./programs/restaking/tests/*.test.ts

# using solana-test-validator
$ RUNTIME=svm pnpm test ./programs/restaking/tests/*.test.ts

# with logs for debugging
$ DEBUG=1 pnpm test ./programs/restaking/tests/*.test.ts

# to test all programs
$ pnpm test ./programs/**/*test.ts

# to print the list of test cases
$ pnpm test list ./programs/**/*.test.ts

# to utilize watch & web-ui support
$ pnpm test:watch ./programs/**/*test.ts
```

## 6. Operation
```shell
# connect to a local build via LiteSVM
$ pnpm dev

# connect to Solana RPC
$ pnpm connect --help

Usage: fragmetric connect [options]

Create a REPL to interact with programs.

Options:
  -e, --eval <EXPRESSION>       Evaluate an expression and quit.
  -h, --help                    display help for command

Global Options:
  -V, --version                 output the version number
  -u, --url <URL_OR_MONIKER>    RPC URL or shorthand: [mainnet, devnet, testnet, local] (default: "mainnet")
  --ws <URL>                    Custom WebSocket RPC URL (overrides derived one)
  -c, --cluster <CLUSTER>       Program environment when using custom RPC URL (overrides derived one): [mainnet, devnet, testnet, local]
  -k, --keypairs <KEYPAIRS...>  One or more keypairs to automatically use as signers for transactions. First keypair will be used as feePayer. Accepts: JSON file
                                path, directory of keypairs, base58/JSON literal, or literal for hardware wallets: [ledger].
  --format <FORMAT>             Set output format for evaluation: [pretty, json] (default: "pretty")
  --inspection <BOOL>           Set verbose logs in default transaction hooks: [true, false] (default: cluster != "mainnet")

# for private RPC, you can configure env vars to your shell profile
export SOLANA_RPC_MAINNET=https://...
export SOLANA_RPC_DEVNET=https://...

# then below command will utilize preset RPC url
$ pnpm connect -u m
```

## 7. SDK Development
Above `pnpm connect or pnpm dev` actually invokes SDK REPL from the source code: `./clients/js/fragmetric-sdk/src/...`.
While end-user package is delivered as [@fragmetric-labs/sdk](https://www.npmjs.com/package/@fragmetric-labs/sdk).
See [README.md](./clients/js/fragmetric-sdk/README.md) for details of the SDK.

```
# ensure fresh local builds
$ anchor build

# run codegen
$ pnpm codegen

# modify SDK source codes to ship new features
# ...

# then build SDK bundle
$ pnpm build

# can test distirubiton bundle with existing test suites
# when CI env is set, test suites utilize dist bundle instead of source code of SDK.
$ CI=1 pnpm test ./programs/**/*.test.ts
```

## 8. Integration Test Development
Implement new features into SDK and extend existing test suites in `./programs/*/tests/...`.

A few backgrounds until details are ready:
- Programs and mock accounts in `./programs/*/tests/mocks/..` are automatically loaded for all test suites.
- From mock accounts, testing runtime will automatically find token mints to create token airdrop faucets.
- Implements all the business logic into SDK - `./clients/js/fragmetric-sdk/src/...`, even for local only purposes.
- ...