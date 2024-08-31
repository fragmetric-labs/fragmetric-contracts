# Fragmetric Solana Programs

This repository contains the full business logic of Fragmetric on-chain programs and it's test codes.

# Guide
## 1. Developer Configuration

- Install `solana-cli`, `anchor-cli 0.30.1` with this [reference](https://solana.com/developers/guides/getstarted/setup-local-development).
- Install testing tool dependencies.
```
$ npm install
```

## 2. Test Guide
1. You have to prepare 2 accounts before running test.
- receipt token mint account

You just can generate any keypair with below cli command.
```
# admin account
$ solana-keygen new -o ./id.json

# global accounts
$ solana-keygen new -o ./tests/restaking/fragsolMint.json
```

And update `ADMIN_PUBKEY, FRAGSOL_MINT_ADDRESS` public keys in `programs/restaking/src/constants.rs` file.


2. Run E2E test

You can configure test sequence by manipulating `./tests/restaking/restaking.ts`. The command below runs e2e test against that sequence.
```
$ anchor test -p restaking
```
