# Fragmetric JavaScript Client SDK

The Client SDK for Fragmetric Solana programs serves both end users and operational administrators.
It is designed for compatibility with web browsers and Node.js environments, offering support for web browser bundles (in ESM, CJS, and UMD formats) as well as Node.js bundles (in ESM and CJS formats).

## 1. How to Use?

### Installation
This library requires `@solana/web3.js` as peer dependency for both web browser and Node.js environment.

```sh
$ yarn add @fragmetric-labs/sdk @solana/web3.js
```

And to use SDK with Ledger hardware wallet in Node.js environment, it needs to install devDependencies as well.

### Examples
- [./examples/react](./examples/react): For web browser application with bundlers like rollup, webpack and more (ESM, CJS).
- [./examples/html](./examples/html): For web browser application without bundlers (UMD)
- [./examples/react](./examples/react): For Node.js environment (CJS, ESM)

## 2. How to Contribute?

```sh
# Do dev/build/test for SDK
$ yarn workspace @fragmetric-labs/sdk dev
$ yarn workspace @fragmetric-labs/sdk dev:node # build only node:cjs, node:esm
$ yarn workspace @fragmetric-labs/sdk dev:node:cjs # fastest for nodejs playground testing
$ yarn workspace @fragmetric-labs/sdk dev:browser # build only web:cjs, web:esm, web:umd
$ yarn workspace @fragmetric-labs/sdk dev:browser:esm # fastest for browser sdk testing
$ yarn workspace @fragmetric-labs/sdk build
$ yarn workspace @fragmetric-labs/sdk test

# Do dev/build/test for examples
$ yarn workspace @fragmetric-labs/sdk-example-react dev
$ yarn workspace @fragmetric-labs/sdk-example-html dev
$ yarn workspace @fragmetric-labs/sdk-example-node dev

# Build all including examples
$ yarn workspaces run build

# Test all including examples
$ yarn workspaces run test

# Publish SDK
$ yarn workspace @fragmetric-labs/sdk publish
```
...
