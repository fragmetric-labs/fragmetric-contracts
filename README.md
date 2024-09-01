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
1. Generate placeholder keypairs with below commands.
```
# payer wallet account
$ solana-keygen new -o ./id.json

# program global accounts
$ solana-keygen new -o ./tests/restaking/keypairs/mint_fragsol_FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo.json && \
    solana-keygen new -o ./tests/restaking/keypairs/devnet_admin_fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP.json && \
    solana-keygen new -o ./tests/restaking/keypairs/devnet_fund_manager_fragHx7xwt9tXZEHv2bNo3hGTtcHP9geWkqc2Ka6FeX.json && \
    solana-keygen new -o ./tests/restaking/keypairs/devnet_program_frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ.json && \
    mkdir -p ./target/deploy && ln -s ../../tests/restaking/keypairs/devnet_program_frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ.json ./target/deploy/restaking-keypair.json
```

And update corresponding public keys in `programs/restaking/src/constants.rs` file.
```
$ echo "FRAGSOL_MINT_ADDRESS = $(solana -k ./tests/restaking/keypairs/mint_fragsol_FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo.json address)" && \
    echo "ADMIN_PUBKEY = $(solana -k ./tests/restaking/keypairs/devnet_admin_fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP.json address)" && \
    echo "FUND_MANAGER_PUBKEY = $(solana -k ./tests/restaking/keypairs/devnet_fund_manager_fragHx7xwt9tXZEHv2bNo3hGTtcHP9geWkqc2Ka6FeX.json address)" && \
    echo "PROGRAM_ID = $(solana -k ./tests/restaking/keypairs/devnet_program_frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ.json address)"
```


2. Run E2E test

You can configure test sequence by manipulating `./tests/restaking/restaking.ts`. The command below runs e2e test against that sequence.
```
$ anchor test -p restaking
```
