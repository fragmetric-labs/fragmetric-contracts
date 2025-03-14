import {
  AccountInfoBase,
  Address,
  Base58EncodedBytes,
  Base64EncodedBytes,
  Base64EncodedDataResponse,
  Base64EncodedWireTransaction,
  blockhash,
  createJsonRpcApi,
  createRpc,
  getBase58Decoder,
  getBase58Encoder,
  lamports,
  Lamports,
  RpcTransport,
  Signature,
  SolanaRpcResponse,
  TransactionError,
  TransactionVersion,
  UnixTimestamp,
} from '@solana/kit';
import type { LiteSVM } from 'litesvm';
import { web3Compat } from './litesvm.web3js.node';
import {
  Runtime,
  RuntimeFactory,
  RuntimeRPC,
  RuntimeRPCMethods,
  RuntimeRPCOptionalMethods,
} from './runtime';

export type LiteSVMRuntimeConfig = {
  type: 'litesvm';
  svm: LiteSVM;
};

export const createLiteSVMRuntime: RuntimeFactory<LiteSVMRuntimeConfig> = ({
  svm,
}): Runtime => {
  const rpc = createLiteSVMRPC(svm);
  return {
    type: 'litesvm',
    cluster: 'local',
    rpc,
    rpcSubscriptions: null,
  };
};

function assertBase64Encoding(
  config?: { encoding?: string } & Record<string, any>
) {
  const encoding =
    (config && 'encoding' in config ? config['encoding'] : undefined) ??
    'base64';
  if (encoding != 'base64') {
    throw new Error(
      `not supported data encoding for litesvm runtime: ${encoding}`
    );
  }
}

function createLiteSVMRPC(svm: LiteSVM): RuntimeRPC {
  const transactionsMap = new Map<Signature, Base64EncodedDataResponse>();

  const rpcMethods: RuntimeRPCMethods & RuntimeRPCOptionalMethods = {
    getAccountInfo(address, config) {
      assertBase64Encoding(config);

      const account = svm.getAccount(web3Compat.toPublicKey(address));
      const res: SolanaRpcResponse<AccountInfoBase & any> = {
        context: {
          slot: svm.getClock().slot,
        },
        value: account
          ? {
              data: [
                Buffer.from(account.data).toString(
                  'base64'
                ) as Base64EncodedBytes,
                'base64',
              ],
              executable: account.executable,
              lamports: web3Compat.fromLegacyLamports(account.lamports),
              owner: web3Compat.fromLegacyPublicKey(account.owner),
              rentEpoch: account.rentEpoch
                ? BigInt(account.rentEpoch)
                : BigInt('18446744073709551615'),
              space: BigInt(account.data.length),
            }
          : null,
      };
      return res;
    },
    getMultipleAccounts(addresses, config) {
      assertBase64Encoding(config);

      const items = addresses.map((address) =>
        rpcMethods.getAccountInfo(address)
      );
      const res: SolanaRpcResponse<(AccountInfoBase & any)[]> = {
        context: {
          slot: svm.getClock().slot,
        },
        value: items.map((item) => item.value),
      };
      return res;
    },
    getTransaction(signature, config) {
      assertBase64Encoding(config);
      const result = svm.getTransaction(
        Uint8Array.from(getBase58Encoder().encode(signature))
      );
      if (!result) return null;

      const meta = 'err' in result ? result.meta() : result;
      const err =
        result && 'err' in result
          ? (result.toString().match(/err:\s*([A-Za-z0-9_]+)/)?.[1] ?? null)
          : null;
      return {
        blockTime: BigInt(Date.now()) as UnixTimestamp,
        slot: svm.getClock().slot,
        version: 0 as TransactionVersion,
        transaction: transactionsMap.get(signature) ?? (null as any),
        meta: {
          computeUnitsConsumed: meta?.computeUnitsConsumed() ?? 0n,
          err: err as unknown as TransactionError | null,
          fee: 0n as Lamports,
          innerInstructions: meta
            ?.innerInstructions()
            .map((ixs) => {
              return ixs.map((ix) => {
                const compiled = ix.instruction();
                return {
                  index: compiled.programIdIndex(),
                  instructions: [
                    {
                      accounts: Array.from(ix.instruction().accounts()),
                      data: getBase58Decoder().decode(
                        ix.instruction().data()
                      ) as Base58EncodedBytes,
                      stackHeight: ix.stackHeight(),
                      programIdIndex: ix.instruction().programIdIndex(),
                    } as any,
                  ],
                };
              });
            })
            .flat(),
          loadedAddresses: {
            readonly: [],
            writable: [],
          },
          logMessages: meta?.logs(),
          postBalances: [],
          postTokenBalances: [],
          preBalances: [],
          preTokenBalances: [],
          rewards: [],
          status: {
            Ok: null,
          },
        },
      };
    },
    simulateTransaction(base64EncodedWireTransaction, config) {
      assertBase64Encoding(config);

      const tx = web3Compat.toLegacyVersionedTransaction(
        base64EncodedWireTransaction as Base64EncodedWireTransaction
      );
      // cannot support sigVerify=false
      const result = svm.simulateTransaction(tx);
      const errResult = 'err' in result ? result : null;
      let err = errResult
        ? (errResult.toString().match(/err:\s*([A-Za-z0-9_]+)/)?.[1] ?? null)
        : null;
      if (err) {
        // always run the simulation to deal with `BlockhashNotFound` error for nonce tx
        // ref: https://github.com/LiteSVM/litesvm/issues/111
        if (err == 'BlockhashNotFound') {
          svm.expireBlockhash();
          const result2 = svm.simulateTransaction(tx);
          if ('err' in result2) {
            // ...
          } else {
            err = null;
          }
        }
        if (err) {
          throw new Error(
            `Transaction simulation failed (litesvm): ${err
              .split(/\.?(?=[A-Z])/)
              .join(' ')
              .toLowerCase()}\n${result.meta().logs().join('\n')}`
          );
        }
      }

      return {
        context: {
          slot: svm.getClock().slot,
        },
        value: {
          accounts: null as any,
          err: err as unknown as TransactionError | null,
          innerInstructions: result
            .meta()
            .innerInstructions()
            .map((ixs) => {
              return ixs.map((ix) => {
                const compiled = ix.instruction();
                return {
                  index: compiled.programIdIndex(),
                  instructions: [
                    {
                      accounts: Array.from(ix.instruction().accounts()),
                      data: getBase58Decoder().decode(
                        ix.instruction().data()
                      ) as Base58EncodedBytes,
                      stackHeight: ix.stackHeight(),
                      programIdIndex: ix.instruction().programIdIndex(),
                    },
                  ],
                };
              });
            })
            .flat(),
          logs: result.meta().logs(),
          replacementBlockhash: undefined,
          returnData: {
            programId: getBase58Decoder().decode(
              result.meta().returnData().programId()
            ) as Address,
            data: [
              Buffer.from(result.meta().returnData().data()).toString('base64'),
              'base64',
            ] as Base64EncodedDataResponse,
          },
          unitsConsumed: result.meta().computeUnitsConsumed(),
        },
      };
    },
    sendTransaction(base64EncodedWireTransaction, config) {
      assertBase64Encoding(config);

      try {
        rpcMethods.simulateTransaction(base64EncodedWireTransaction, {
          ...config,
          encoding: 'base64',
        });
      } catch (err) {
        if (!config?.skipPreflight) {
          throw err;
        }
      }

      const tx = web3Compat.toLegacyVersionedTransaction(
        base64EncodedWireTransaction as Base64EncodedWireTransaction
      );
      const res = svm.sendTransaction(tx);

      const signature = getBase58Decoder().decode(
        'signature' in res ? res.signature() : tx.signatures[0]
      ) as Signature;

      transactionsMap.set(signature, [
        base64EncodedWireTransaction as unknown as Base64EncodedBytes,
        'base64',
      ]);
      return signature;
    },
    getMinimumBalanceForRentExemption(size, config) {
      return lamports(svm.getRent().minimumBalance(size));
    },
    getLatestBlockhash(config) {
      svm.expireBlockhash();
      const slot = svm.getClock().slot;
      return {
        context: {
          slot,
        },
        value: {
          lastValidBlockHeight: slot,
          blockhash: blockhash(svm.latestBlockhash()),
        },
      };
    },
    requestAirdrop(recipientAccount, lamports, config) {
      const address = web3Compat.toPublicKey(recipientAccount);
      svm.expireBlockhash();

      const res = svm.airdrop(address, lamports);
      if (res && 'signature' in res) {
        return getBase58Decoder().decode(res.signature()) as Signature;
      }
      throw new Error(`Airdrop failed (litesvm): ${res?.toString()}`);
    },
    getEpochInfo() {
      const slot = svm.getClock().slot;
      const slotsPerEpoch = svm.getEpochSchedule().slotsPerEpoch;
      return {
        absoluteSlot: slot,
        blockHeight: slot,
        epoch: slot / slotsPerEpoch,
        slotIndex: slot,
        slotsInEpoch: slot & slotsPerEpoch,
        transactionCount: null,
      };
    },
    getSlot() {
      return svm.getClock().slot;
    },
    getSignatureStatuses(signatures) {
      return {
        context: {
          slot: svm.getClock().slot,
        },
        value: signatures.map((signature) => {
          const tx = rpcMethods.getTransaction(signature);
          return !tx
            ? null
            : {
                confirmationStatus: 'finalized',
                confirmations: 1n,
                err: tx.meta?.err ?? null,
                slot: tx.slot,
                status: tx.meta?.err
                  ? {
                      Err: tx.meta?.err,
                    }
                  : {
                      Ok: null,
                    },
              };
        }),
      };
    },
  };

  return createRpc<RuntimeRPCMethods, RpcTransport>({
    api: createJsonRpcApi(),
    transport: (args) => {
      // @ts-ignore
      return rpcMethods[args.payload.method].apply(null, args.payload.params);
    },
  });
}
