# Structure

Main logic of dummy is at `programs/dummy/src/lib.rs`.

# Setting

You have to install `solana-cli`, `anchor` at your local.
Take a look at this [reference](https://solana.com/developers/guides/getstarted/setup-local-development).

And you have to set the solana config rpc url to local.
```
$ solana config set --url localhost
```

And install dependencies.
```
$ npm install
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

2. Run test codes.
```
$ anchor test --skip-local-validator
```
If you want to run the specific test file,
first, add the test command at `Anchor.toml` file's `[scripts]` section.
For example, there's `test-dummy` command.

If you want to run only the `tests/dummy.ts` test file, then run the below command.
```
$ anchor run test-dummy
```


# Deploy the Program to Devnet

1. Set the solana config rpc url to devnet.
```
$ solana config set --url devnet
```


2. prepare the developer Solana key from AWS.
```
$ aws sso login --profile encrypt_dev
...
$ anchor run set-dev-wallet
```

2. Change provider cluster at `Anchor.toml` to devnet.
```
[provider]
cluster = "devnet"
wallet = "./id.json"
```

3. Build again
4. Deploy
```
$ anchor deploy
```

## For RateLimit Error from RPC node

1. Set the solana config rpc url to the QuickNode url.
```
$ solana config set --url https://palpable-few-ensemble.solana-devnet.quiknode.pro/187c644705468fcb556c12b70dc5a41dfd355961/
```

2. Change provider cluster at `Anchor.toml` to the QuickNode url.
```
[provider]
cluster = "https://palpable-few-ensemble.solana-devnet.quiknode.pro/187c644705468fcb556c12b70dc5a41dfd355961/"
```

## To Deal with Multiple Programs at the Same Repository

1. If you want to make another anchor program at this repository, you can use this command.
```
$ anchor new <another program name>
```
