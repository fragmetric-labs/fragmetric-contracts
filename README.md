# Fragmetric Solana Programs
## 1. Introduction
TODO...

# 2. Contribution Guide
## 2.1. Developer Configuration

- Install `solana-cli`, `anchor-cli 0.30.1` with this [reference](https://solana.com/developers/guides/getstarted/setup-local-development).
- Install testing tool dependencies.
```
$ npm install
```

## 2.2. Local Development
```
# Update code and build the updated binary
$ anchor build -p dummy
...


# Run Solana network locally
$ solana-test-validator
...


# Prepare your Solana Wallet account for program deployment/upgrade transactions 
# In case of the Fragmetric inhouse members, run below script to fetch a shared wallet keypair from the cloud.
$ aws sso login --profile encrypt_dev
...
$ anchor run set-dev-wallet
encrypt_dev/wallet data copied to ./id.json


# Deploy or Upgrade the program
# Be noted that the "./id.json" keypair will have the upgrade authority of your local program,
# And already have the upgrade authoirty of the devnet program.
$ anchor deploy --provider.wallet ./id.json --provider.cluster=localnet --program-name dummy --program-keypair ./programs/dummy/id.json


# If there is no enough buffer in the program data account, refer the below command to extend the buffer size with the given number of bytes.
# Be noted that the maximum accounts size is 10MB.
$ solana program extend [PROGRAM_ADDRESS] 1000 --keypair ./id.json
Extended Program Id A58NQYmJCyDPsc1EfaQZ99piFopPtCYArP242rLTbYbV by 1000 bytes
```

## 2.3. Devnet Deployment
```
# We've used the same program keypair for both local, devnet environment for the convenience.
$ anchor deploy --provider.wallet ./id.json --provider.cluster=localnet --program-name dummy --program-keypair ./programs/dummy/id.json
```

## 2.4. Testing
1. Run the localnet at the seperate terminal.
If it halts, use `--reset` flag.
```
$ solana-test-validator
```

2. Run test codes.
Be noted that devnet usually fails to get airdrop to create a new account for clean test.
So, you can use pre-funded accounts' keypairs in `./tests/user1.json, ...` to deal with devnet test-cases.
```
$ anchor run test-dummy --provider.cluster=localnet
...

$ anchor run test-dummy --provider.cluster=devnet
...
```
