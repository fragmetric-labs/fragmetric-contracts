# Fragmetric Solana Programs

This repository contains the full business logic of Fragmetric on-chain programs and it's test codes.

# Guide
## 1. Developer Configuration

- Install `solana-cli` with this [reference](https://solana.com/developers/guides/getstarted/setup-local-development).
- Install `anchor-cli (v0.30.1)` 
```
# instead of using AVM, install from latest git rev to pick a fix for `anchor idl build -- CARGO_ARGS`.
$ cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked --rev 1c0b2132ec4713343f9c672479721f432ccbf904 --force

# check proper CLI has been installed
$ anchor idl build --help
Generates the IDL for the program using the compilation method

Usage: anchor idl build [OPTIONS] [-- <CARGO_ARGS>...]

Arguments:
  [CARGO_ARGS]...  Arguments to pass to the underlying `cargo test` command
...
```

- Initialize testing tools:
```
# install node packages
$ yarn

# add below PATH to your shell profile:
export PATH=$PATH:/usr/local/lib/node_modules/node/bin:./node_modules/.bin

# to sync program keypair to ./target/deploy/ dir:
$ anchor run sync-keypairs -- local
```

## 2. Run E2E Test

### For all test suites
```
$ anchor test -p restaking
```

### For specific test suite
```
$ anchor test --detach -p restaking --run ./tests/restaking/1_initialize.ts
# ... and keep the local test validator from 1_initialize test suite

$ anchor test --skip-local-validator --skip-deploy --run ./tests/restaking/2_deposit_sol.ts

# ... and more as you want 
```

## 3. Build Artifacts
```
$ anchor run sync-keypairs -- local|devnet|mainnet
$ anchor build -p restaking -- --features devnet|mainnet
```