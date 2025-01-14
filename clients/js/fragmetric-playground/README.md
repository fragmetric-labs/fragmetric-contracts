# Fragmetric Playground

This library depends on @fragmetric-labs/sdk as its core. It provides operational helpers for REPL usage, Anchor integration tests, and additional features built on top of the SDK — primarily for local TDD and other operational tasks.

## 1. How to Contribute?

The SDK (which targets both browser and Node.js users) uses stable IDL files. So you’ll often develop both the SDK and the Playground together, to override the SDK’s IDL with your locally built version, run:

```sh
$ yarn workspace @fragmetric-labs/sdk dev:local
```

...
