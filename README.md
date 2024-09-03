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
./ $ cd keypairs

# payer wallet account
./keypairs $ solana-keygen new -o ./wallet.json

# program global accounts
./keypairs $ solana-keygen new -o mint_fragsol_FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo.json && \
    solana-keygen new -o devnet_admin_fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP.json && \
    solana-keygen new -o devnet_fund_manager_fragHx7xwt9tXZEHv2bNo3hGTtcHP9geWkqc2Ka6FeX.json && \
    solana-keygen new -o devnet_program_frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ.json && \
    mkdir -p ../target/deploy && ln -s ../../keypairs/restaking/devnet_program_frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ.json ../target/deploy/restaking-keypair.json
```

Then update corresponding public keys in `./programs/restaking/src/constants.rs` file.
And program id declaration in `./programs/restaking/src/lib.rs` file.
```
./keypairs $ echo "FRAGSOL_MINT_ADDRESS = $(solana -k mint_fragsol_FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo.json address)" && \
    echo "ADMIN_PUBKEY = $(solana -k devnet_admin_fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP.json address)" && \
    echo "FUND_MANAGER_PUBKEY = $(solana -k devnet_fund_manager_fragHx7xwt9tXZEHv2bNo3hGTtcHP9geWkqc2Ka6FeX.json address)" && \
    echo "PROGRAM_ID = $(solana -k devnet_program_frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ.json address)"
```

## 3. Run E2E test

You can configure test sequence by manipulating `./tests/restaking/restaking.ts`. The command below runs e2e test against that sequence.
```
$ anchor test -p restaking
```
