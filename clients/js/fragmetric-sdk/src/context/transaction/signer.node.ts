import {
  Address,
  createKeyPairSignerFromBytes,
  getBase58Decoder,
  getBase58Encoder,
  getBase64Encoder,
  SignatureBytes,
  TransactionPartialSigner,
} from '@solana/kit';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { TransactionSignerResolver } from '../address';
import {
  HardwareWalletSignerResolver,
  HardwareWalletSignerResolverOptions,
  markAsHardwareWalletSignerResolver,
} from './signer';

export function createLedgerSignerResolver(
  options?: HardwareWalletSignerResolverOptions
): HardwareWalletSignerResolver {
  let resolving: Promise<TransactionPartialSigner> | undefined;

  const resolver = (): Promise<TransactionPartialSigner> => {
    if (resolving) {
      return resolving;
    }

    return (resolving = new Promise<TransactionPartialSigner>(
      async (resolve, reject) => {
        try {
          let connection = await createLedgerConnection();

          const signer: TransactionPartialSigner = {
            get address() {
              return connection.address;
            },
            async signTransactions(transactions, config) {
              return Promise.all(
                transactions.map((tx) => {
                  return connection.solanaApp
                    .signTransaction(
                      connection.derivationPath,
                      Buffer.from(tx.messageBytes)
                    )
                    .then((result) => {
                      const res: Record<Address, SignatureBytes> = {
                        [connection.address]: Uint8Array.from(
                          result.signature
                        ) as SignatureBytes,
                      };
                      return res;
                    });
                })
              ).catch(async (err) => {
                if (
                  err.toString().includes('DisconnectedDeviceDuringOperation')
                ) {
                  connection = await createLedgerConnection(options); // create a new connection
                  return signer.signTransactions(transactions, config);
                } else {
                  throw err;
                }
              });
            },
          };

          resolve(signer);
        } catch (err) {
          resolving = undefined;
          reject(err);
        }
      }
    ));
  };

  return markAsHardwareWalletSignerResolver(resolver);
}

// hacky-way to ship prebuilt native-modules and support ESM and CJS default export mixes
// ref: https://github.com/LedgerHQ/ledger-live/tree/develop/libs/ledgerjs/packages/hw-transport-node-hid-singleton
// ref: https://github.com/LedgerHQ/ledger-live/tree/develop/libs/ledgerjs/packages/hw-app-solana
import type Solana from '@ledgerhq/hw-app-solana';
import type { Subscription } from '@ledgerhq/hw-transport';
import type TransportNodeHidSingleton from '@ledgerhq/hw-transport-node-hid-singleton';
import { Module } from 'node:module';

let __ledger: ReturnType<typeof ledger> | null = null;

function ledger(): {
  TransportNodeHidSingleton: typeof TransportNodeHidSingleton;
  Solana: typeof Solana;
} {
  if (__ledger) return __ledger;

  // hijack `@ledgerhq/hw-transport-node-hid-singleton` -> `node-hid@2` to -> `node-hid@3` which ships prebuilt native modules.
  const require$ = Module.createRequire(import.meta.url);
  const originalLoad = (Module as any)._load;
  (Module as any)._load = function (request: any) {
    if (request === 'node-hid') {
      const resolved = require$.resolve('node-hid');
      return require$(resolved);
    }
    return originalLoad.apply(this, arguments);
  };

  // now load modules
  const TransportNodeHidSingletonModule = require$(
    '@ledgerhq/hw-transport-node-hid-singleton'
  );
  const SolanaModule = require$('@ledgerhq/hw-app-solana');
  return (__ledger = {
    TransportNodeHidSingleton:
      (TransportNodeHidSingletonModule as any).default ??
      TransportNodeHidSingletonModule,
    Solana: (SolanaModule as any).default ?? SolanaModule,
  });
}

async function createLedgerConnection(
  options?: HardwareWalletSignerResolverOptions
) {
  const { TransportNodeHidSingleton, Solana } = ledger();

  const derivationPath = options?.derivationPath ?? `44'/501'/0'`;
  const connectionTimeoutSeconds = Math.max(
    isNaN(options?.connectionTimeoutSeconds as number)
      ? 5
      : options!.connectionTimeoutSeconds!,
    1
  );

  const transport = await new Promise<TransportNodeHidSingleton>(
    (resolve, reject) => {
      let found = false;
      let subscription: Subscription | null = null;
      let timeoutId = setTimeout(() => {
        subscription?.unsubscribe();
        reject(new Error('ledger connection timed out'));
      }, connectionTimeoutSeconds * 1000);

      subscription = TransportNodeHidSingleton.listen({
        next: (event) => {
          found = true;
          subscription?.unsubscribe();
          clearTimeout(timeoutId);
          TransportNodeHidSingleton.open(event.descriptor).then(
            resolve,
            reject
          );
        },
        error: (err) => {
          clearTimeout(timeoutId);
          reject(err);
        },
        complete: () => {
          clearTimeout(timeoutId);
          if (!found) {
            reject(new Error('ledger not found'));
          }
        },
      });
    }
  );

  const solanaApp = new Solana(transport);
  const address = await solanaApp
    .getAddress(derivationPath)
    .then((res) => getBase58Decoder().decode(res.address) as Address)
    .catch((err) => {
      throw new Error(
        `failed to get address from the ledger: ${err?.toString()}`
      );
    });

  const solanaAppConfig = await solanaApp.getAppConfiguration().catch((err) => {
    throw new Error(
      `failed to get solana app configuration: device locked or app not open ... ${err?.toString()}`
    );
  });

  if (!solanaAppConfig.blindSigningEnabled) {
    throw new Error(`blind signing is disabled in ledger settings`);
  }

  return { address, derivationPath, solanaApp };
}

export function createTransactionSignerResolvers(
  ...args: Parameters<typeof createTransactionSignerResolversMap>
): TransactionSignerResolver[] {
  return Object.values(createTransactionSignerResolversMap(...args));
}

/**
 * Create TransactionSignerResolver map from: JSON keypair path, directory of keypairs, base58/JSON keypair literal, or 'ledger' to use Ledger hardware wallet.
 * May returns $ledger, $literal0, $literal1, file_path (without .json ext and replacement of '/', '\' to '__'.
 **/
export function createTransactionSignerResolversMap(config: {
  rootDir: string;
  keypairs: string[];
}): Record<string, TransactionSignerResolver> {
  const resolvers: Record<string, TransactionSignerResolver> = {};
  let literalCount = 0;

  for (const input of config.keypairs) {
    const resolvedPath = path.isAbsolute(input)
      ? input
      : path.resolve(config.rootDir, input);

    if (input === 'ledger') {
      resolvers[`$ledger`] = createLedgerSignerResolver();
    } else if (fs.existsSync(resolvedPath)) {
      const stat = fs.statSync(resolvedPath);

      if (stat.isDirectory()) {
        const files = fs.readdirSync(resolvedPath);
        for (const file of files) {
          const filePath = path.join(resolvedPath, file);
          if (filePath.endsWith('.json')) {
            resolvers[filePath.substring(0, filePath.length - 5)] =
              loadSignerFromFile(filePath);
          }
        }
      } else if (resolvedPath.endsWith('.json')) {
        resolvers[resolvedPath.substring(0, resolvedPath.length - 5)] =
          loadSignerFromFile(resolvedPath);
      }
    } else {
      resolvers[`$literal${literalCount++}`] = loadSignerFromLiteral(input);
    }
  }

  return stripSharedPrefixFromObject(resolvers);
}

function loadSignerFromFile(filePath: string): TransactionSignerResolver {
  try {
    const content = fs.readFileSync(filePath, 'utf-8');
    const bytes = new Uint8Array(JSON.parse(content));
    return () => createKeyPairSignerFromBytes(bytes);
  } catch (err) {
    throw new Error(
      `failed to create signer from file path: ${filePath}\n${err}`
    );
  }
}

function loadSignerFromLiteral(literal: string): TransactionSignerResolver {
  if (literal.trim().startsWith('[')) {
    const bytes = new Uint8Array(JSON.parse(literal));
    return () => createKeyPairSignerFromBytes(bytes);
  } else {
    try {
      const bytes = getBase58Encoder().encode(literal);
      if (bytes.length != 64) {
        throw new Error('keypair bytes must be have 64 length');
      }
      return () => createKeyPairSignerFromBytes(bytes);
    } catch (err) {
      try {
        const bytes = getBase64Encoder().encode(literal);
        if (bytes.length != 64) {
          throw new Error('keypair bytes must be have 64 length');
        }
        return () => createKeyPairSignerFromBytes(bytes);
      } catch (err2) {
        throw new Error(
          `failed to create signer from literal: ${literal}\n${err}\n${err2}`
        );
      }
    }
  }
}

function stripSharedPrefixFromObject(
  obj: Record<string, any>
): Record<string, any> {
  const entries = Object.entries(obj);
  const pathEntries = entries.filter(([k]) => !k.startsWith('$'));

  if (pathEntries.length === 0) return obj;

  const keys = pathEntries.map(([k]) => k);
  const prefix = findCommonPrefix(keys);

  const strippedObj: Record<string, any> = {};
  for (const [k, v] of entries) {
    let newKey = k.startsWith(prefix)
      ? k
          .slice(prefix.length)
          .replace(/^\/+/, '')
          .replace(/[\/\\]/, '__')
      : k;
    while (strippedObj[newKey]) {
      newKey += '_';
    }
    strippedObj[newKey] = v;
  }
  return strippedObj;
}

function findCommonPrefix(strings: string[]): string {
  if (strings.length === 0) return '';
  let prefix = strings[0];
  for (const s of strings) {
    while (!s.startsWith(prefix)) {
      prefix = prefix.slice(0, -1);
      if (!prefix) return '';
    }
  }
  return prefix;
}
