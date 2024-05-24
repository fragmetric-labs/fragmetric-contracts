# Structure

Main logic of deposit-program is at `programs/deposit-program/src/lib.rs`.

# Setting

You have to install `solana-cli`, `anchor` at your local.
Take a look at this [reference](https://solana.com/developers/guides/getstarted/setup-local-development).

And you have to set the solana config rpc url to local.
```
$ solana config set --url localhost
```

# Build the Program

```
$ anchor build
```

# Run Test Code

1. Run the localnet at the seperate terminal.
```
$ solana-test-validator
```
It seems to get halts sometimes. If it halts, use `--reset` flag.

2. Run test code.
```
$ anchor test --skip-local-validator
```

# Deploy the Program to Devnet

1. Set the solana config rpc url to devnet.
```
$ solana config set --url devnet
```

2. Change provider cluster at `Anchor.toml` to devnet.
```
[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"
```

3. Build again
4. Deploy
```
$ anchor deploy
```