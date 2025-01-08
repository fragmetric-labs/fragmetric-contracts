# Tools

## 1. JavaScript Client SDK

### How to contribute?
```
# dev/build/test SDK
$ yarn workspace @fragmetric-labs/sdk dev
$ yarn workspace @fragmetric-labs/sdk dev:node
$ yarn workspace @fragmetric-labs/sdk build
$ yarn workspace @fragmetric-labs/sdk test

# dev/build/test examples
$ yarn workspace @fragmetric-labs/sdk-example-react run dev
$ yarn workspace @fragmetric-labs/sdk-example-html run dev
$ yarn workspace @fragmetric-labs/sdk-example-nodejs run dev

# build all including examples
$ yarn workspaces run build

# test all including examples
$ yarn workspaces run test
```

## 2. Node.js Playground

### ...

## 3. Rust Crates for CPI

### ...


# TODO
- ledger adapter for node
- state cache for browser,node
- deposit methods
- pricing methods
- update examples
- publish & version up test and readme
- other all operation methods
- playground(? .. = REPL + wrapper functions (?) for testing and operation? or just into sdk?)
- fragSOL, fragJTO playgrounds...
- snapshot feature for some stuffs.. for fixture based testin
- testing/playground migration...
- update keychain stuff.. and aws secret.. readme..
- migrate `restaking`, `lib`, `../keypairs`, `../idls`, `../cpis`
- rewrite `tests`
