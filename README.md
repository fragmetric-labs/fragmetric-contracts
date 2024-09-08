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

# for authorized engineers:
$ anchor run download-keypairs
```


## 2. Run E2E Test

```
$ anchor test -p restaking
```


## 3. Build Artifacts
```
$ anchor build -p restaking -- --features devnet|mainnet
```


## 4. REPL for operation and testing

### Basic usage
```
$ tsx tools/restaking/repl_entrypoint.ts
[?] select target environment (local/devnet/mainnet): local
[7:04:17 PM] [keychain] loaded local wallet

...

[!] Type 'restaking.' and press TAB to start...
http://0.0.0.0:8899 >
```

### Easy local testing
```
# test-validator will be still running after initialization test done.
$ JUST_INIT=1 anchor test --detach -p restaking
...

# now connect to test-validator
$ tsx tools/restaking/repl_entrypoint.ts -- local
...
```
