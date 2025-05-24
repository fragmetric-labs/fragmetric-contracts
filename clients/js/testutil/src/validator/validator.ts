import * as token from '@solana-program/token';
import {
  AccountInfoBase,
  AccountInfoWithBase64EncodedData,
  AccountInfoWithPubkey,
  Address,
  appendTransactionMessageInstructions,
  Base64EncodedBytes,
  Blockhash,
  createKeyPairSignerFromBytes,
  createTransactionMessage,
  getBase64Decoder,
  getBase64EncodedWireTransaction,
  KeyPairSigner,
  Lamports,
  pipe,
  Rpc,
  RpcSubscriptions,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
  SolanaError,
  SolanaRpcApi,
  SolanaRpcSubscriptionsApi,
} from '@solana/kit';
import * as web3 from '@solana/web3.js';
import fs from 'fs';
import type { LiteSVM } from 'litesvm';
import { createHash } from 'node:crypto';
import path from 'path';

export type TestValidatorType = 'svm' | 'litesvm';
export type Commitment = 'processed' | 'confirmed' | 'finalized';
export interface GetSlotOptions {
  commitment?: Commitment; // default: confirmed
}

export type TestValidatorOptions<T extends TestValidatorType> = {
  type: T;
  slotsPerEpoch: bigint; // default: 432000
  ticksPerSlot: number; // default: 64 ~= 400ms
  limitLedgerSize: number; // default: 10000
  mock?: TestValidatorMockOptions;
  debug?: boolean;

  // for parallel testing
  tag?: string;
  instanceNo?: number;
};
export type TestValidatorMockOptions = {
  rootDir: string;
  programs: Array<
    (
      | {
          keypairFilePath: string;
        }
      | {
          pubkey: string;
        }
    ) & {
      soFilePath: string;
    }
  >;
  accounts: Array<
    | {
        pubkey?: string;
        jsonFilePath: string;
      }
    | {
        jsonFileDirPath: string;
      }
    | AccountInfoWithPubkey<AccountInfoBase & AccountInfoWithBase64EncodedData>
  >;
};

export type TestValidatorRuntime<T extends TestValidatorType> = {
  type: T;
  instanceNo: number;
} & (T extends 'litesvm'
  ? {
      type: 'litesvm';
      svm: LiteSVM;
    }
  : {
      type: 'svm';
      cluster: 'local';
      rpc: Rpc<SolanaRpcApi>;
      rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;
      rpcURL: string;
      rpcSubscriptionsURL: string;
    });

type Optional<T, K extends keyof T> = Pick<Partial<T>, K> & Omit<T, K>;

export abstract class TestValidator<T extends TestValidatorType> {
  static async create<T extends TestValidatorType>(
    options?: Optional<
      TestValidatorOptions<T>,
      'slotsPerEpoch' | 'ticksPerSlot' | 'limitLedgerSize'
    >
  ): Promise<TestValidator<T>> {
    const resolvedOptions: TestValidatorOptions<T> = {
      slotsPerEpoch: 432000n,
      ticksPerSlot: 64,
      limitLedgerSize: 10000,
      type: options?.type ?? ('svm' as T),
      ...options,
    };

    // add mock ATAs of token faucet
    await TestValidator.transformMockOptions(resolvedOptions.mock);

    // run validator
    const validator = await (resolvedOptions.type == 'svm'
      ? import('./svm').then((module) =>
          module.SVMValidator.initialize(
            resolvedOptions as TestValidatorOptions<'svm'>
          )
        )
      : import('./litesvm').then((module) =>
          module.LiteSVMValidator.initialize(
            resolvedOptions as TestValidatorOptions<'litesvm'>
          )
        ));

    // airdrop a SOL to token faucet
    await validator.airdrop(
      TestValidator.tokenFaucetKeyPair.publicKey.toBase58(),
      1_000_000_000n
    );

    return validator as TestValidator<T>;
  }

  abstract get runtime(): TestValidatorRuntime<T>;
  abstract get options(): TestValidatorOptions<T>;
  abstract quit(): Promise<void>;

  abstract getSlot(opts?: GetSlotOptions): Promise<bigint>;

  abstract warpToSlot(slot: bigint): Promise<void>;
  async skipSlots(slots: bigint): Promise<void> {
    const currentSlot = await this.getSlot({ commitment: 'processed' });
    await this.warpToSlot(currentSlot + slots);
  }

  async getEpoch(opts?: GetSlotOptions): Promise<bigint> {
    return (await this.getSlot(opts)) / this.options.slotsPerEpoch;
  }

  async skipEpoch(): Promise<void> {
    const currentProcessedSlot = await this.getSlot({
      commitment: 'processed',
    });
    const remainingSlots =
      this.options.slotsPerEpoch -
      (currentProcessedSlot % this.options.slotsPerEpoch);
    return this.warpToSlot(currentProcessedSlot + remainingSlots);
  }

  abstract airdrop(pubkey: string, lamports: bigint): Promise<void>;

  async newSigner(
    seed: string,
    lamports: bigint = 100_000_000_000n
  ): Promise<KeyPairSigner> {
    const signer = await this.getSigner(seed);
    if (lamports < 1_000_000n) {
      // approx. min rent for base account
      lamports = 1_000_000n;
    }
    await this.airdrop(signer.address, lamports);
    return signer;
  }

  async getSigner(seed: string): Promise<KeyPairSigner> {
    const seedBuffer = createHash('sha256').update(seed).digest(); // 32byte
    const keypair = web3.Keypair.fromSeed(new Uint8Array(seedBuffer));
    return createKeyPairSignerFromBytes(keypair.secretKey);
  }

  abstract getAccount(
    pubkey: string
  ): Promise<(AccountInfoBase & AccountInfoWithBase64EncodedData) | null>;

  private static readonly tokenFaucetKeyPair = web3.Keypair.fromSeed(
    new Uint8Array(createHash('sha256').update('tokenFaucet').digest())
  );
  private static readonly tokenFaucetSigner = createKeyPairSignerFromBytes(
    TestValidator.tokenFaucetKeyPair.secretKey
  );
  private static readonly tokenFaucetAddress =
    TestValidator.tokenFaucetKeyPair.publicKey.toBase58() as Address;

  private static async transformMockOptions(
    mock: TestValidatorOptions<any>['mock']
  ) {
    if (!mock) {
      return;
    }
    mock.accounts = mock.accounts.slice();

    function resolvePath(p: string) {
      return path.isAbsolute(p) ? p : path.join(mock!.rootDir, p);
    }

    const tokenFaucetAddress = TestValidator.tokenFaucetAddress;
    const accountFiles: AccountInfoWithPubkey<
      AccountInfoBase & AccountInfoWithBase64EncodedData
    >[] = [];
    for (const account of mock.accounts) {
      if ('jsonFileDirPath' in account) {
        const resolvedDirPath = resolvePath(account.jsonFileDirPath);
        for (const filePath of fs.readdirSync(resolvedDirPath)) {
          if (filePath.endsWith('.json')) {
            accountFiles.push(
              JSON.parse(
                fs.readFileSync(path.join(resolvedDirPath, filePath)).toString()
              ) as AccountInfoWithPubkey<
                AccountInfoBase & AccountInfoWithBase64EncodedData
              >
            );
          }
        }
      } else if ('jsonFilePath' in account) {
        accountFiles.push(
          JSON.parse(
            fs.readFileSync(resolvePath(account.jsonFilePath)).toString()
          ) as AccountInfoWithPubkey<
            AccountInfoBase & AccountInfoWithBase64EncodedData
          >
        );
      } else {
        accountFiles.push(account);
      }
    }

    for (const accountFile of accountFiles) {
      if (
        accountFile.account.owner == token.TOKEN_PROGRAM_ADDRESS &&
        !accountFile.account.executable
      ) {
        let isMintAccount = false;
        try {
          token.decodeMint({
            ...accountFile.account,
            address: accountFile.pubkey as Address,
            data: Uint8Array.from(
              Buffer.from(
                accountFile.account.data[0],
                accountFile.account.data[1]
              )
            ),
            programAddress: accountFile.account.owner,
          });
          isMintAccount = true;
        } catch (e) {}
        if (isMintAccount) {
          const mint = accountFile.pubkey as Address;
          const [address] = await token.findAssociatedTokenPda({
            owner: tokenFaucetAddress,
            mint: mint,
            tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
          });
          const tokenData = token.getTokenEncoder().encode({
            mint: mint,
            owner: tokenFaucetAddress,
            amount: 1_000_000_000_000_000n,
            delegate: null,
            state: token.AccountState.Initialized,
            isNative: null,
            delegatedAmount: 0,
            closeAuthority: null,
          });
          const tokenAccount: AccountInfoWithPubkey<
            AccountInfoBase & AccountInfoWithBase64EncodedData
          > = {
            pubkey: address,
            account: {
              data: [
                getBase64Decoder().decode(tokenData) as Base64EncodedBytes,
                'base64',
              ],
              rentEpoch: 18_446_744_073_709_551_615n,
              space: BigInt(tokenData.length),
              executable: false,
              owner: token.TOKEN_PROGRAM_ADDRESS,
              lamports: 1_000_000_000n as Lamports,
            },
          };
          mock.accounts.push(tokenAccount);
        }
      }
    }
  }

  async airdropToken(pubkey: string, mockMint: string, amount: bigint) {
    // transfer token from tokenFaucet to pubkey
    const tokenFaucetAddress = TestValidator.tokenFaucetAddress;
    const tokenFaucetSigner = await TestValidator.tokenFaucetSigner;
    const [[src], [dst]] = await Promise.all([
      token.findAssociatedTokenPda({
        owner: tokenFaucetAddress,
        mint: mockMint as Address,
        tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
      }),
      token.findAssociatedTokenPda({
        owner: pubkey as Address,
        mint: mockMint as Address,
        tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
      }),
    ]);

    const createTransaction = () => {
      return pipe(
        createTransactionMessage({ version: 0 }),
        async (tx) =>
          appendTransactionMessageInstructions(
            [
              await token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: tokenFaucetSigner,
                mint: mockMint as Address,
                owner: pubkey as Address,
                tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
              }),
              token.getTransferInstruction({
                source: src,
                destination: dst,
                authority: tokenFaucetSigner,
                amount: amount,
              }),
            ],
            tx
          ),
        async (tx) =>
          setTransactionMessageFeePayer(tokenFaucetAddress, await tx),
        async (tx) => {
          if (this.runtime.type == 'litesvm') {
            const svm = (this.runtime as TestValidatorRuntime<'litesvm'>).svm;
            return setTransactionMessageLifetimeUsingBlockhash(
              {
                blockhash: svm.latestBlockhash() as Blockhash,
                lastValidBlockHeight: svm.getClock().slot,
              },
              await tx
            );
          } else {
            const rpc = (this.runtime as TestValidatorRuntime<'svm'>).rpc;
            return setTransactionMessageLifetimeUsingBlockhash(
              (await rpc.getLatestBlockhash().send()).value,
              await tx
            );
          }
        },
        async (tx) => signTransactionMessageWithSigners(await tx)
      );
    };

    if (this.runtime.type == 'litesvm') {
      const svm = (this.runtime as TestValidatorRuntime<'litesvm'>).svm;
      svm.withBlockhashCheck(false); // just turn off block hash check for short interval airdrop
      const res = svm.sendTransaction(
        web3.VersionedTransaction.deserialize(
          Buffer.from(
            getBase64EncodedWireTransaction(await createTransaction()),
            'base64'
          )
        )
      );
      svm.withBlockhashCheck(true);
      if ('err' in res) {
        throw new Error(`failed to mint token from faucet: ${res.toString()}`);
      }
    } else {
      const solana = this.runtime as TestValidatorRuntime<'svm'>;
      const sendAndConfirm = sendAndConfirmTransactionFactory({
        rpc: solana.rpc,
        rpcSubscriptions: solana.rpcSubscriptions,
      });

      let retriesOnBlockErrors = 0;
      while (true) {
        try {
          await sendAndConfirm(await createTransaction(), {
            commitment: 'confirmed',
            skipPreflight: true,
          });
          await new Promise((resolve) => setTimeout(resolve, 1000));
          break;
        } catch (err) {
          if (retriesOnBlockErrors < 100) {
            if (err instanceof SolanaError) {
              const blockError =
                /network has progressed|blockhash not found|already been processed/i;
              const causeMsg = err.cause?.toString() || '';
              const msg = `${err.message}${causeMsg ? ` - ${causeMsg}` : ''}`;
              if (blockError.test(msg)) {
                console.error(
                  `Retrying the same airdrop transaction (${retriesOnBlockErrors}): ${msg}`
                );
                retriesOnBlockErrors++;
                await new Promise((resolve) =>
                  setTimeout(resolve, Math.floor(Math.random() * 5000) + 1000)
                );
                continue;
              }
            }
          }

          throw err;
        }
      }
    }
  }
}
