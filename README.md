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
- reward account

You just can generate any keypair with below cli command.
```
$ solana-keygen new -o ./tests/restaking/fragsolMint.json
$ solana-keygen new -o ./tests/restaking/rewardAccount.json
```
2. Delete rust test files.

To build the anchor idl file correctly, you need to delete the rust test files first. Delete or move the `./programs/restaking/tests/` directory.

3. Run e2e test

You can configure test sequence by manipulating `./tests/restaking/restaking.ts`. The command below runs e2e test against that sequence.
```
$ anchor test -p restaking
```
