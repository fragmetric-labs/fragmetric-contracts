# Tools

## 1. JavaScript Client SDK

### How to contribute?
```
# dev/build/test SDK
$ yarn workspace @fragmetric-labs/sdk dev
$ yarn workspace @fragmetric-labs/sdk dev:node # build only node:cjs, node:esm
$ yarn workspace @fragmetric-labs/sdk dev:node:cjs # fastest for nodejs playground testing
$ yarn workspace @fragmetric-labs/sdk dev:browser # build only web:cjs, web:esm, web:umd
$ yarn workspace @fragmetric-labs/sdk dev:browser:esm # fastest for browser sdk testing
$ yarn workspace @fragmetric-labs/sdk build
$ yarn workspace @fragmetric-labs/sdk test

# dev/build/test examples
$ yarn workspace @fragmetric-labs/sdk-example-react dev
$ yarn workspace @fragmetric-labs/sdk-example-html dev
$ yarn workspace @fragmetric-labs/sdk-example-nodejs dev

# build all including examples
$ yarn workspaces run build

# test all including examples
$ yarn workspaces run test

# publish sdk
$ yarn workspace @fragmetric-labs/sdk publish
```

## 2. Node.js Playground

### ...

## 3. Rust Crates for CPI

### ...


# TODO
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
