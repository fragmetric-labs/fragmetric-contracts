# Fragmetric Solana Programs

This repository contains the full business logic of Fragmetric on-chain programs and it's test codes.

# Guide
## 1. Developer Configuration

- Install `solana-cli`, `anchor-cli 0.30.1` with this [reference](https://solana.com/developers/guides/getstarted/setup-local-development).
- For testing:
```
# install testing tool dependencies:
$ yarn

# install program keypair to ./target/deploy/ dir:
$ tsx tools/restaking/keychain_init_local.ts

# if cannot find 'tsx' binary, add below path to your shell profile like below:
export PATH=$PATH:/usr/local/lib/node_modules/node/bin:./node_modules/.bin
```

## 2. Run E2E test
```
$ anchor test -p restaking
```

## 3. Build deploy artifacts
```
$ anchor build -p restaking -- --features devnet|mainnet
```