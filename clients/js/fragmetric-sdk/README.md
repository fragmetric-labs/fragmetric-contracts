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
- [../examples/react](./examples/react): For web browser application with bundlers like rollup, webpack and more (ESM, CJS).
- [../examples/html](./examples/html): For web browser application without bundlers (UMD)
- [../examples/react](./examples/react): For Node.js environment (CJS, ESM)
