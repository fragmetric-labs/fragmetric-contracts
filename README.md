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

# if cannot find 'tsx' binary, add below PATH to your shell profile:
export PATH=$PATH:/usr/local/lib/node_modules/node/bin:./node_modules/.bin
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
$ anchor idl build -p restaking

$ anchor build -p restaking -- --features devnet|mainnet
```