import {
  fromLegacyKeypair,
  fromLegacyPublicKey,
  fromLegacyTransactionInstruction,
  fromVersionedTransaction,
} from '@solana/compat';
import {
  AccountInfoBase,
  AccountInfoWithBase64EncodedData,
  Base64EncodedWireTransaction,
  Lamports,
  lamports,
} from '@solana/kit';

export const web3Compat = {
  fromLegacyKeypair,
  fromLegacyPublicKey,
  fromLegacyTransactionInstruction,
  fromVersionedTransaction,
  fromLegacyLamports(value: number): Lamports {
    return lamports(BigInt(value));
  },
  toNumber(value: bigint): number | undefined {
    if (
      value >= BigInt(Number.MIN_SAFE_INTEGER) &&
      value <= BigInt(Number.MAX_SAFE_INTEGER)
    ) {
      return Number(value);
    }
  },
  toPublicKey(address: string) {
    return new (web3().PublicKey)(address.toString());
  },
  toLegacyAccountInfoBytes(
    account: AccountInfoBase & AccountInfoWithBase64EncodedData
  ) {
    return {
      executable: account.executable,
      lamports: Number(account.lamports.valueOf()),
      owner: web3Compat.toPublicKey(account.owner),
      data: Uint8Array.from(Buffer.from(account.data[0], account.data[1])),
      rentEpoch: web3Compat.toNumber(account.rentEpoch),
    };
  },
  toLegacyVersionedTransaction(
    base64EncodedWireTx: Base64EncodedWireTransaction
  ) {
    return web3().VersionedTransaction.deserialize(
      Buffer.from(base64EncodedWireTx, 'base64')
    );
  },
};

import type { Keypair, PublicKey, VersionedTransaction } from '@solana/web3.js';
import { Module } from 'node:module';

let __web3: ReturnType<typeof web3> | null = null;

function web3(): {
  PublicKey: typeof PublicKey;
  Keypair: typeof Keypair;
  VersionedTransaction: typeof VersionedTransaction;
} {
  if (__web3) return __web3;

  // hijack `@solana/web3.js` -> `bigint-buffer` to -> `@trufflesuite/bigint-buffer` which ships prebuilt native modules.
  const require$ = Module.createRequire(import.meta.url);
  // const originalLoad = (Module as any)._load;

  // (Module as any)._load = function (request: any) {
  //   if (request === 'bigint-buffer') {
  //     const resolved = require$.resolve('@trufflesuite/bigint-buffer');
  //     return require$(resolved);
  //   }
  //   return originalLoad.apply(this, arguments);
  // };

  // now load modules
  return (__web3 = require$('@solana/web3.js'));
}
