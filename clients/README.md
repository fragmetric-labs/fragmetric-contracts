# Client Libraries

### 1. Codegen
It generates JavaScript, Rust codebase utilizing [codama]() library.
See `./codegen.config.ts` file for comprehensive understanding of this tool.
It basically use IDLs on `./codegen/idls` directory following the above config file. which contains both third-party IDLs and fragmetric's IDLs.

```sh
$ yarn workspace @fragmetric-labs/codegen run build
```

Above command will generates packages/crates into `./js`, `./rust` dirs with change detection once created.

```sh
$ yarn workspace @fragmetric-labs/codegen run build:all
```

Above command will generates all the packages/crates without change detection.

```sh
$ yarn workspace @fragmetric-labs/codegen run build:local
```

Above command will generates the packages/crates. But with utilizing local IDL file from (`./target/idl`) for fragmetric codes.
And also repeatedly watch the IDL file updates to generate code dynamically.
This command is for seamless development of `@fragmetric-labs/playground` and `@fragmetric-labs/sdk` for in-house engineers.

### 2. JavaScript SDK (js/fragmetric-sdk)

```sh
# Initialization
$ yarn workspace @fragmetric-labs/sdk install
$ yarn workspace @fragmetric-labs/sdk build

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

# Publish SDK
$ yarn workspace @fragmetric-labs/sdk publish
```

## 3. Node.js Playground (js/fragmetric-playground)
...


# TODO
- playground
  - REPL endpoint
  - migrate keychain loading.., aws secrets
  - keychain sync
  - fragSOL, fragJTO instance helper?
  - other all operation methods
- rewrite `tests`
  - snapshot feature for some stuffs.. for fixture based testing
