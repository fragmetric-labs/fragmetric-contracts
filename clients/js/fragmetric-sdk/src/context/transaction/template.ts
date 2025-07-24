import * as computeBudget from '@solana-program/compute-budget';
import {
  Address,
  addSignersToTransactionMessage,
  appendTransactionMessageInstructions,
  Blockhash,
  Commitment,
  compressTransactionMessageUsingAddressLookupTables,
  createTransactionMessage,
  type Decoder,
  decompileTransactionMessage,
  getBase58Decoder,
  getBase58Encoder,
  getBase64EncodedWireTransaction,
  getBytesEncoder,
  getCompiledTransactionMessageDecoder,
  getProgramDerivedAddress,
  getSignatureFromTransaction,
  getTransactionDecoder,
  IInstruction,
  isDurableNonceTransaction,
  isSolanaError,
  isTransactionSendingSigner,
  isTransactionSigner,
  Nonce,
  partiallySignTransactionMessageWithSigners,
  pipe,
  ReadonlyUint8Array,
  RpcSendOptions,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  setTransactionMessageLifetimeUsingDurableNonce,
  Signature,
  signTransactionMessageWithSigners,
  Slot,
  SOLANA_ERROR__INVALID_NONCE,
  SolanaError,
  TransactionSendingSigner,
  TransactionSigner,
} from '@solana/kit';
import * as v from 'valibot';
import { ObjectSchema } from 'valibot';
import { AccountContext } from '../account';
import {
  AccountAddressResolver,
  AccountAddressResolverVariant,
  TransactionSignerResolver,
} from '../address';
import { ProgramDerivedContext } from '../program';
import { ExecutedTransactionEvents, TransactionResultContext } from './result';
import {
  HardwareWalletSignerResolver,
  isHardwareWalletSignerResolver,
} from './signer';

export type InstructionsResolver<P extends ProgramDerivedContext<any>, ARGS> = (
  parent: P,
  args: ARGS,
  overrides: TransactionTemplateOverrides<P, ARGS>
) => Promise<(IInstruction | null)[]>;

export type TransactionTemplateExecutionHook<
  P extends ProgramDerivedContext<any>,
  ARGS,
  EVENTS extends Record<string, any>,
> = {
  /**
   * Called after the transaction is assembled and signed,
   * but before it is sent to the network.
   * Useful for logging the signature or preparing UI state.
   */
  onSignature?: (
    parent: TransactionTemplateContext<P, any, any>,
    signature: Signature,
    args: ARGS
  ) => void;

  /**
   * Called when an error occurs during the transaction pipeline,
   * excluding transaction-level failures (i.e., this is not for failed on-chain execution).
   * For example, this could catch serialization, signing, simulation failure or RPC errors.
   */
  onError?: (
    parent: TransactionTemplateContext<P, any, any>,
    err: any,
    args: ARGS | null
  ) => void;

  /**
   * Called after the transaction completes (either success or failure).
   * The result contains transaction status, logs, and any parsed events.
   * This hook will be triggered even if the transaction failed on-chain.
   */
  onResult?: (
    parent: TransactionTemplateContext<P, any, any>,
    result: TransactionResultContext<
      TransactionTemplateContext<P, any, any>,
      ARGS,
      EVENTS
    >,
    args: ARGS
  ) => void;
};

export type TransactionTemplateConfig<
  P extends ProgramDerivedContext<any>,
  ARGS,
  EVENTS extends Record<string, any>,
> = {
  description?: string;

  feePayer?: AccountAddressResolverVariant<P>;

  durableNonce?: {
    nonceAccountAddress: string | AccountAddressResolver<P>;
  };

  addressLookupTables?: (string | AccountAddressResolver<P>)[];

  instructions?: (null | IInstruction | InstructionsResolver<P, ARGS>)[];

  signers?: (TransactionSigner | TransactionSignerResolver)[];

  anchorEventDecoders?: {
    [K in keyof EVENTS]: {
      discriminator: ReadonlyUint8Array;
      decoder: Decoder<EVENTS[K]>;
    };
  };

  executionHooks?: TransactionTemplateExecutionHook<P, ARGS, EVENTS>;

  // Prepends compute budget instructions unless already defined in the template's instructions.
  // Each option will be ignored if a corresponding instruction is manually provided.
  computeBudget?: {
    limit?: number; // default is unset, up to 1_400_000 compute units.
    priceInMicroLamports?: number; // default is unset, up to 1_000_000 (1 lamport per CU); higher values will be ignored.
  };
};

export type TransactionTemplateOverrides<
  P extends ProgramDerivedContext<any>,
  ARGS,
> = {
  feePayer?: TransactionTemplateConfig<P, ARGS, any>['feePayer'];

  durableNonce?:
    | TransactionTemplateConfig<P, ARGS, any>['durableNonce']
    | {
        nonce: string;
        nonceAuthorityAddress: string;
        nonceAccountAddress: string;
      };

  recentBlockhash?: {
    blockhash: string;
    lastValidBlockHeight: bigint;
  } | null; // set null to not to fetch recent blockhash

  addressLookupTables?: TransactionTemplateConfig<
    P,
    ARGS,
    any
  >['addressLookupTables'];

  prependedInstructions?: TransactionTemplateConfig<
    P,
    ARGS,
    any
  >['instructions'];
  appendedInstructions?: TransactionTemplateConfig<
    P,
    ARGS,
    any
  >['instructions'];

  executionHooks?: TransactionTemplateConfig<P, ARGS, any>['executionHooks'];

  signers?: TransactionTemplateConfig<P, ARGS, any>['signers'];

  computeBudget?: TransactionTemplateConfig<P, ARGS, any>['computeBudget'];
};

export class TransactionTemplateContext<
  P extends ProgramDerivedContext<any>,
  EVENTS extends Record<string, any>,
  ARGS_SCHEME extends v.BaseSchema<any, any, any>,
  ARGS_INPUT = v.InferInput<ARGS_SCHEME>,
  ARGS = v.InferOutput<ARGS_SCHEME>,
> extends ProgramDerivedContext<P> {
  public readonly config: TransactionTemplateConfig<P, ARGS, EVENTS>;

  constructor(
    readonly parent: P,
    readonly args: ARGS_SCHEME | null,
    config: TransactionTemplateConfig<P, ARGS, EVENTS>,
    private readonly chainedArgsBuilder?: (
      parent: P,
      args: ARGS,
      events: ExecutedTransactionEvents<EVENTS>
    ) => Promise<{ args: v.InferInput<ARGS_SCHEME> } | null>
  ) {
    super();
    this.config = config;
  }

  toContextDescription() {
    const desc = super.toContextDescription();
    const newDesc = {
      ...desc,
      properties: {
        ...desc.properties,
        args:
          this.args && v.isOfType('object', this.args)
            ? v
                .keyof(this.args as unknown as ObjectSchema<any, any>)
                .options.join(',')
            : null,
        events:
          Object.keys(this.config.anchorEventDecoders ?? {}).join(',') ||
          undefined,
        description: this.config.description,
      },
      mutable: true,
    };
    if (this.chainedArgsBuilder) {
      newDesc.label = `${newDesc.label} (chained)`;
    }
    return newDesc;
  }

  /**
   * Executes the full pipeline for the initial transaction, then automatically executes any chained transactions in sequence.
   * Execution stops if any transaction in the chain fails. This method is designed for operational workflows that require multiple successive transactions.
   * This method does not support `SendingSigner`.
   *
   * @returns The result of the final executed transaction (either the last successful one or the one that failed).
   */
  async executeChained(
    args: ARGS_INPUT,
    overrides?: TransactionTemplateOverrides<P, ARGS>,
    config?: {
      commitment?: Commitment;
      encoding?: 'base64';
      maxRetries?: bigint;
      minContextSlot?: Slot;
      preflightCommitment?: Commitment;
      skipPreflight?: boolean;
      chainingIntervalSeconds?: number;
    },
    options?: RpcSendOptions
  ) {
    const abortSignal =
      options?.abortSignal ?? this.runtime.abortController.signal;
    let aborted = false;
    abortSignal.addEventListener('abort', () => {
      aborted = true;
    });
    const { chainingIntervalSeconds = 0, ...partialConfig } = config ?? {};
    let res = await this.execute(args, overrides, partialConfig, options);
    if (res.succeeded) {
      while (res.executeChainedTransaction && !aborted) {
        if (chainingIntervalSeconds) {
          await new Promise((resolve) =>
            setTimeout(resolve, chainingIntervalSeconds * 1000)
          );
        }
        res = await res.executeChainedTransaction();
        if (!res.succeeded) {
          break;
        }
      }
    }
    return res;
  }

  /**
   * Executes the full pipeline: assemble transaction, send and confirm, then fetch and parse the result.
   * `SendingSigner` is not supported in this method.
   */
  async execute(
    args: ARGS_INPUT,
    overrides?: TransactionTemplateOverrides<P, ARGS>,
    config?: {
      commitment?: Commitment;
      encoding?: 'base64';
      maxRetries?: bigint;
      minContextSlot?: Slot;
      preflightCommitment?: Commitment;
      skipPreflight?: boolean;
    },
    options?: RpcSendOptions
  ) {
    // invalidation -> template hook -> overrides hook -> global hook
    const hooks = (
      this.config.executionHooks ? [this.config.executionHooks] : []
    )
      .concat(overrides?.executionHooks ? [overrides?.executionHooks] : [])
      .concat(
        this.runtime.options.transaction.executionHooks
          ? [
              this.runtime.options.transaction
                .executionHooks as TransactionTemplateExecutionHook<
                any,
                any,
                any
              >,
            ]
          : []
      );

    let argsResolved: ARGS | null = null;
    try {
      // validate args first
      argsResolved = (this.args ? v.parse(this.args, args) : null) as ARGS;

      let signature: Signature;
      let retriesOnBlockErrors = 0;
      while (true) {
        try {
          signature = await this.sendAndConfirm(
            args,
            overrides,
            config,
            options
          );
          break;
        } catch (err) {
          if (
            retriesOnBlockErrors <
            this.runtime.options.transaction.maxRetriesOnBlockErrors
          ) {
            if (err instanceof SolanaError) {
              const blockError =
                /network has progressed|blockhash not found|already been processed/i;
              const causeMsg = err.cause?.toString() || '';
              const msg = `${err.message}${causeMsg ? ` - ${causeMsg}` : ''}`;
              if (blockError.test(msg)) {
                console.error(
                  `Dangerously retrying the same transaction (${retriesOnBlockErrors}): ${msg}`
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

      hooks.forEach((hook) =>
        hook?.onSignature?.(this, signature, argsResolved!)
      );

      const result = await this.parse(signature, argsResolved);
      hooks.forEach((hook) => hook?.onResult?.(this, result, argsResolved!));

      // set chained transaction template
      if (result.succeeded && this.chainedArgsBuilder) {
        const repeat = await this.chainedArgsBuilder(
          this.parent,
          argsResolved,
          result.events!
        );
        if (repeat?.args) {
          result.__setNextTransactionExecutor(() =>
            this.execute(repeat.args, overrides, config, options)
          );
        }
      }

      return result;
    } catch (err) {
      hooks.forEach((hook) => hook?.onError?.(this, err, argsResolved));
      throw err;
    }
  }

  /**
   * Sends the transaction and waits for confirmation.
   * Does not parse logs or results. This method will return a signature instead of throwing an error when a skip-preflight request fails.
   * `SendingSigner` is not supported in this method.
   */
  async sendAndConfirm(
    args: ARGS_INPUT,
    overrides?: TransactionTemplateOverrides<P, ARGS>,
    config?: {
      commitment?: Commitment;
      encoding?: 'base64';
      maxRetries?: bigint;
      minContextSlot?: Slot;
      preflightCommitment?: Commitment;
      skipPreflight?: boolean;
    },
    options?: RpcSendOptions
  ): Promise<Signature> {
    let signature: Signature | null = null;
    const { transaction } = await this.__assemble(args, overrides);

    try {
      if (isDurableNonceTransaction(transaction)) {
        const serializedTransaction =
          await signTransactionMessageWithSigners(transaction);
        signature = getSignatureFromTransaction(serializedTransaction);

        if (this.runtime.sendAndConfirmDurableNonceTransaction) {
          try {
            await this.runtime.sendAndConfirmDurableNonceTransaction(
              serializedTransaction,
              {
                commitment:
                  this.runtime.options.transaction.confirmationCommitment,
                // encoding: 'base64',
                ...config,
                abortSignal: this.runtime.abortController.signal,
                ...options,
              }
            );
          } catch (err) {
            let isFastLandedTransaction = false;
            if (isSolanaError(err, SOLANA_ERROR__INVALID_NONCE)) {
              const res = await this.runtime.rpc
                .getSignatureStatuses([signature])
                .send();
              if (res.value?.[0]?.confirmationStatus) {
                isFastLandedTransaction = true;
              }
            }
            if (!isFastLandedTransaction) {
              throw err;
            }
          }
        } else {
          // litesvm
          await this.runtime.rpc
            .sendTransaction(
              getBase64EncodedWireTransaction(serializedTransaction),
              {
                encoding: 'base64',
                ...config,
              }
            )
            .send({
              abortSignal: this.runtime.abortController.signal,
              ...options,
            });
        }
      } else {
        const serializedTransaction =
          await signTransactionMessageWithSigners(transaction);
        signature = getSignatureFromTransaction(serializedTransaction);

        if (this.runtime.sendAndConfirmTransaction) {
          await this.runtime.sendAndConfirmTransaction(serializedTransaction, {
            commitment: this.runtime.options.transaction.confirmationCommitment,
            // encoding: 'base64',
            ...config,
            abortSignal: this.runtime.abortController.signal,
            ...options,
          });
        } else {
          // litesvm
          await this.runtime.rpc
            .sendTransaction(
              getBase64EncodedWireTransaction(serializedTransaction),
              {
                encoding: 'base64',
                ...config,
              }
            )
            .send({
              abortSignal: this.runtime.abortController.signal,
              ...options,
            });
        }
      }
      return signature;
    } catch (err) {
      if (config?.skipPreflight && signature) {
        if (this.__debug) {
          console.debug(signature, err);
        }
        return signature;
      }
      throw err;
    }
  }

  /**
   * Sends the serialized transaction to the network. Does not wait for confirmation or fetch any results.
   * If any `SendingSigner` is configured, this method is only way to invoke `SendingSigner`.
   */
  async send(
    args: ARGS_INPUT,
    overrides?: TransactionTemplateOverrides<P, ARGS>,
    config?: {
      encoding?: 'base64';
      maxRetries?: bigint;
      minContextSlot?: Slot;
      preflightCommitment?: Commitment;
      skipPreflight?: boolean;
    },
    options?: RpcSendOptions
  ) {
    const { transaction, sendingSigner } = await this.__assemble(
      args,
      overrides
    );

    if (sendingSigner) {
      const serializedTransaction =
        await partiallySignTransactionMessageWithSigners(transaction);
      const signatures = await sendingSigner.signAndSendTransactions(
        [serializedTransaction],
        {
          ...config,
          abortSignal: this.runtime.abortController.signal,
          ...options,
        }
      );
      return getBase58Decoder().decode(signatures[0]);
    }

    const serializedTransaction =
      await signTransactionMessageWithSigners(transaction);
    return this.runtime.rpc
      .sendTransaction(getBase64EncodedWireTransaction(serializedTransaction), {
        ...config,
        encoding: 'base64',
      })
      .send({
        abortSignal: this.runtime.abortController.signal,
        ...options,
      });
  }

  /**
   * Simulates the transaction.
   * Returns the simulation result including logs, compute units, errors, etc.
   * `SendingSigner` is not supported in this method.
   */
  async simulate(
    args: ARGS_INPUT,
    overrides?: TransactionTemplateOverrides<P, ARGS>,
    config?: {
      encoding?: 'base64';
      maxRetries?: bigint;
      minContextSlot?: Slot;
      commitment?: Commitment;
    } & (
      | {
          sigVerify?: true;
          replaceRecentBlockhash?: false;
        }
      | {
          sigVerify?: false;
          replaceRecentBlockhash?: true;
        }
    ),
    options?: RpcSendOptions
  ) {
    const serializedTransaction = await this.serializeToBase64(
      args,
      overrides,
      false
    );
    return this.runtime.rpc
      .simulateTransaction(serializedTransaction, {
        ...config,
        encoding: 'base64',
      })
      .send({
        abortSignal: this.runtime.abortController.signal,
        ...options,
      });
  }

  /**
   * Serializes the entire transaction blueprint into a base64-encoded string.
   * Applies signing before serialization. `SendingSigner` is not supported in this method.
   * Useful for RPC send or simulation endpoints. For backward `@solana/web3.js` compatibility, try `web3.VersionedTransaction.deserialize(Buffer.from(serializedTransaction, 'base64'))`
   */
  async serializeToBase64(
    args: ARGS_INPUT,
    overrides?: TransactionTemplateOverrides<P, ARGS>,
    allowPartiallySigned = false
  ) {
    const serializedTransaction = await this.serialize(
      args,
      overrides,
      allowPartiallySigned
    );
    return getBase64EncodedWireTransaction(serializedTransaction);
  }

  /**
   * Serializes the message of transaction blueprint into raw bytes (`Uint8Array`).
   * Applies configured signers. `SendingSigner` is not supported in this method.
   */
  async serialize(
    args: ARGS_INPUT,
    overrides?: TransactionTemplateOverrides<P, ARGS>,
    allowPartiallySigned = false
  ) {
    const { transaction } = await this.__assemble(args, overrides);
    return allowPartiallySigned
      ? partiallySignTransactionMessageWithSigners(transaction)
      : signTransactionMessageWithSigners(transaction);
  }

  /**
   * Assembles the full transaction message using instructions, fee payer, and lifetime strategy.
   * This method does not perform signing, but returns a fully prepared transaction blueprint.
   *
   * If multiple signers are provided for the same address, only one is retained based on the following priority order (higher overrides lower):
   * 1. Override-provided signers
   * 2. Global runtime-configured signers
   * 3. Template-configured signers
   * 4. Fee payer signer
   * 5. Instruction-attached signers
   *
   * This ensures consistent and deterministic signer resolution before signing or serialization.
   */
  async assemble(
    args: ARGS_INPUT,
    overrides?: TransactionTemplateOverrides<P, ARGS>
  ) {
    return this.__assemble(args, overrides).then((res) => res.transaction);
  }

  private async __assemble(
    args: ARGS_INPUT,
    overrides?: TransactionTemplateOverrides<P, ARGS>
  ) {
    // validate args first
    const argsResolved = (this.args ? v.parse(this.args, args) : null) as ARGS;

    let sendingSigner = null as TransactionSendingSigner | null;
    const lowestPrioritySigners: TransactionSigner[] = [];
    const transaction = await pipe(
      createTransactionMessage({ version: 0 }),

      // set instructions
      async (tx) => {
        const instructions = (
          await Promise.all(
            [
              ...(overrides?.prependedInstructions ?? []),
              ...(this.config.instructions ?? []),
              ...(overrides?.appendedInstructions ?? []),
            ].map((ix) =>
              typeof ix == 'function'
                ? ix(this.parent, argsResolved, overrides ?? {})
                : Promise.resolve([ix])
            )
          )
        )
          .flat()
          .filter((ix) => !!ix);

        // set compute budget ixs, config priority: runtime (global) < template < overrides < explicit ix
        const computeBudgetConfig = {
          ...this.runtime.options?.transaction?.computeBudget,
          ...this.config?.computeBudget,
          ...overrides?.computeBudget,
        };
        if (
          computeBudgetConfig.priceInMicroLamports &&
          computeBudgetConfig.priceInMicroLamports <= 1_000_000 &&
          !instructions.some(
            (ix) =>
              ix.programAddress ==
                computeBudget.COMPUTE_BUDGET_PROGRAM_ADDRESS &&
              ix.data?.[0] ==
                computeBudget.ComputeBudgetInstruction.SetComputeUnitPrice
          )
        ) {
          instructions.unshift(
            computeBudget.getSetComputeUnitPriceInstruction({
              microLamports: computeBudgetConfig.priceInMicroLamports,
            })
          );
        }
        if (
          computeBudgetConfig.limit &&
          !instructions.some(
            (ix) =>
              ix.programAddress ==
                computeBudget.COMPUTE_BUDGET_PROGRAM_ADDRESS &&
              ix.data?.[0] ==
                computeBudget.ComputeBudgetInstruction.SetComputeUnitLimit
          )
        ) {
          instructions.unshift(
            computeBudget.getSetComputeUnitLimitInstruction({
              units: computeBudgetConfig.limit,
            })
          );
        }

        const sanitizedInstructions = instructions.map((instruction) => {
          return {
            ...instruction,
            accounts:
              instruction.accounts?.map((account) => {
                // to rearrange the order of applying signers.
                if ('signer' in account) {
                  const { signer, ...sanitizedAccount } = account;
                  lowestPrioritySigners.push(signer as any);
                  return sanitizedAccount;
                }
                return account;
              }) ?? [],
          };
        });
        tx = appendTransactionMessageInstructions(sanitizedInstructions, tx);

        // set address lookup table
        const altAddresses = (
          await Promise.all(
            [
              ...(this.config?.addressLookupTables ?? []),
              ...(overrides?.addressLookupTables ?? []),
            ].map((address) =>
              typeof address == 'function'
                ? address(this.parent)
                : Promise.resolve(address)
            )
          )
        )
          .flat()
          .filter((address) => !!address) as string[];

        if (altAddresses.length > 0) {
          const addressesMap =
            await this.runtime.fetchMultipleAddressLookupTables(altAddresses);
          tx = compressTransactionMessageUsingAddressLookupTables(
            tx,
            addressesMap
          );
        }

        return tx;
      },

      // set fee payer
      async (tx) => {
        const txResolved = await tx;

        // overrides <- global config <- template config <- parent account
        const feePayer =
          overrides?.feePayer ??
          this.runtime.options?.transaction?.feePayer ??
          this.config?.feePayer ??
          (this.parent instanceof AccountContext
            ? () =>
                (this.parent as unknown as AccountContext<any>).resolveAddress()
            : undefined);
        if (feePayer) {
          const feePayerResolved =
            typeof feePayer == 'function'
              ? await feePayer(this.parent)
              : feePayer;
          if (feePayerResolved) {
            if (typeof feePayerResolved == 'object') {
              if (isTransactionSigner(feePayerResolved)) {
                // to rearrange the order of applying signers.
                lowestPrioritySigners.push(feePayerResolved);
                return setTransactionMessageFeePayer(
                  feePayerResolved.address,
                  txResolved
                );
              }
            } else {
              return setTransactionMessageFeePayer(
                feePayerResolved as Address,
                txResolved
              );
            }
          }
        }

        throw new Error('failed to resolve fee payer');
      },

      // set recent block hash or nonce
      async (tx) => {
        const txResolved = await tx;

        const durableNonce: TransactionTemplateOverrides<
          P,
          ARGS
        >['durableNonce'] = overrides?.durableNonce ?? this.config.durableNonce;
        if (durableNonce) {
          if (
            'nonce' in durableNonce &&
            'nonceAuthorityAddress' in durableNonce
          ) {
            return setTransactionMessageLifetimeUsingDurableNonce(
              {
                nonce: durableNonce.nonce as Nonce,
                nonceAccountAddress:
                  durableNonce.nonceAccountAddress as Address,
                nonceAuthorityAddress:
                  durableNonce.nonceAuthorityAddress as Address,
              },
              txResolved
            );
          }

          const nonceAccountAddress =
            await (typeof durableNonce.nonceAccountAddress == 'function'
              ? durableNonce.nonceAccountAddress(this.parent)
              : Promise.resolve(durableNonce.nonceAccountAddress));
          if (nonceAccountAddress) {
            const nonceConfig =
              await this.runtime.fetchNonceConfig(nonceAccountAddress);
            if (nonceConfig) {
              return setTransactionMessageLifetimeUsingDurableNonce(
                nonceConfig,
                txResolved
              );
            }
          }

          throw new Error('failed to resolve nonce account');
        }

        if (overrides?.recentBlockhash === null) {
          return setTransactionMessageLifetimeUsingBlockhash(
            {
              blockhash: '' as Blockhash,
              lastValidBlockHeight: 0n,
            },
            txResolved
          );
        } else {
          const recentBlockhash =
            overrides?.recentBlockhash ??
            (await this.runtime.fetchLatestBlockhash());
          if (recentBlockhash) {
            return setTransactionMessageLifetimeUsingBlockhash(
              {
                blockhash: recentBlockhash.blockhash as Blockhash,
                lastValidBlockHeight: recentBlockhash.lastValidBlockHeight,
              },
              txResolved
            );
          }

          throw new Error('failed to resolve recent blockhash');
        }
      },

      async (tx) => {
        const txResolved = await tx;
        const hardwareWalletSignerResolvers: HardwareWalletSignerResolver[] =
          [];

        // instruction signers -> fee payer signer -> template config signers -> global config signers -> overriding signers
        const signers = await Promise.all(
          [
            ...lowestPrioritySigners,
            ...(this.config.signers ?? []),
            ...(this.runtime.options.transaction.signers ?? []),
            ...(overrides?.signers ?? []),
          ]
            .filter((signer) => {
              // filter hardware wallet signers here to avoid unnecessary unlock request
              if (
                typeof signer == 'function' &&
                isHardwareWalletSignerResolver(signer)
              ) {
                hardwareWalletSignerResolvers.push(signer);
                return false;
              }
              return true;
            })
            .map((signer) => {
              return typeof signer == 'function'
                ? signer()
                : Promise.resolve(signer);
            })
        );
        const signersMap = new Map<Address, TransactionSigner>();
        for (const signer of signers) {
          if (signersMap.has(signer.address)) {
            if (sendingSigner?.address == signer.address) {
              sendingSigner = null;
            }
            if (this.__debug) {
              console.debug(
                `overriding already attached signer:`,
                signer.address
              );
            }
          }
          if (isTransactionSendingSigner(signer)) {
            sendingSigner = signer;
          }
          signersMap.set(signer.address, signer);
        }

        let txSignerAttached = addSignersToTransactionMessage(
          Array.from(signersMap.values()),
          txResolved
        );

        if (hardwareWalletSignerResolvers.length > 0) {
          // unlock hardware wallet signers only if there are missing signatures
          try {
            await signTransactionMessageWithSigners(txSignerAttached);
          } catch (_) {
            const hardwareWalletSigners = await Promise.all(
              hardwareWalletSignerResolvers.map((resolver) => resolver())
            );
            for (const signer of hardwareWalletSigners) {
              if (signersMap.has(signer.address)) {
                if (sendingSigner?.address == signer.address) {
                  sendingSigner = null;
                }
                if (this.__debug) {
                  console.debug(
                    `overriding already attached signer (hardware wallet):`,
                    signer.address
                  );
                }
              }
              signersMap.set(signer.address, signer);
            }
            txSignerAttached = addSignersToTransactionMessage(
              Array.from(signersMap.values()),
              txResolved
            );
          }
        }

        return txSignerAttached;
      }
    );

    if (this.__debug) {
      console.debug(`new transaction assembled`, transaction);
    }

    return {
      transaction,
      sendingSigner,
    };
  }

  /**
   * Parses transaction result and events.
   * Use this after confirming a transaction manually, if needed.
   */
  async parse(
    signature: string,
    args: ARGS | null = null
  ): Promise<TransactionResultContext<this, ARGS, EVENTS>> {
    const txFetchOptions = {
      encoding: 'base64',
      maxSupportedTransactionVersion: 0,
      commitment: this.runtime.options.transaction.confirmationCommitment,
    } as const;

    let rawResult = await this.runtime.rpc
      .getTransaction(signature as Signature, txFetchOptions)
      .send();

    let remainingAttempts =
      this.runtime.options.transaction.maxRetriesOnNotFoundError;
    const retryDelay =
      this.runtime.options.transaction.retryIntervalMillisecondsOnNotFoundError;
    // here a response with empty logMessages as not found one, assuming a RPC issue
    while (!rawResult?.meta?.logMessages && remainingAttempts-- > 0) {
      if (this.__debug) {
        console.error(`retry getTransaction in ${retryDelay}ms`);
      }
      await new Promise((resolve) => setTimeout(resolve, retryDelay));
      rawResult = await this.runtime.rpc
        .getTransaction(signature as Signature, txFetchOptions)
        .send();
    }

    if (!rawResult) {
      return new TransactionResultContext(this, signature, args);
    }

    const compiledTransaction = getTransactionDecoder().decode(
      Buffer.from(rawResult.transaction[0], rawResult.transaction[1])
    );
    const compiledTransactionMessage =
      getCompiledTransactionMessageDecoder().decode(
        compiledTransaction.messageBytes
      );
    const addressTableLookups =
      ('addressTableLookups' in compiledTransactionMessage
        ? compiledTransactionMessage.addressTableLookups
        : undefined) ?? [];
    const addressLookupTableAddresses = addressTableLookups.map(
      (lookup) => lookup.lookupTableAddress
    );
    const addressesByLookupTableAddress = addressLookupTableAddresses.length
      ? await this.runtime.fetchMultipleAddressLookupTables(
          addressLookupTableAddresses
        )
      : {};
    const addresses = compiledTransactionMessage.staticAccounts.concat(
      addressTableLookups.flatMap((lookup) => {
        const addresses =
          addressesByLookupTableAddress[lookup.lookupTableAddress];
        return lookup.writableIndexes
          .map((index) => addresses[index])
          .concat(lookup.readonlyIndexes.map((index) => addresses[index]));
      })
    ) as string[];
    const transaction = decompileTransactionMessage(
      compiledTransactionMessage,
      { addressesByLookupTableAddress }
    );

    const programAddressIndex = addresses.indexOf(this.program.address);

    let events = {
      unknown: [],
    } as unknown as ExecutedTransactionEvents<EVENTS>;
    if (this.config.anchorEventDecoders && programAddressIndex != -1) {
      const rawEvents: ReadonlyUint8Array[] = [];
      const anchorEventAuthorityIndex = addresses.indexOf(
        await this.__getAnchorEventAuthorityAddress()
      );
      if (anchorEventAuthorityIndex != -1) {
        const innerInstructions = rawResult.meta?.innerInstructions ?? [];
        for (const group of innerInstructions) {
          for (const instruction of group.instructions) {
            if (instruction.programIdIndex == programAddressIndex) {
              if (
                instruction.accounts.some(
                  (accountIndex) => accountIndex == anchorEventAuthorityIndex
                )
              ) {
                const data = getBase58Encoder().encode(instruction.data);
                if (
                  data
                    .slice(0, 8)
                    .every(
                      (byte, index) =>
                        byte ==
                        TransactionTemplateContext
                          .ANCHOR_EVENT_INSTRUCTION_DISCRIMINATOR[index]
                    )
                ) {
                  rawEvents.push(data.slice(8));
                }
              }
            }
          }
        }
      }
      if (rawEvents.length) {
        events = this.__decodeAnchorEvents(rawEvents);
      }
    }

    const { transaction: _, ...rawResultWithoutTransaction } = rawResult;
    const result = {
      ...rawResultWithoutTransaction,
      transaction,
    };

    return new TransactionResultContext(
      this,
      signature,
      args,
      events,
      result as any
    ); // TODO [sdk]: result typing not work
  }

  private async __getAnchorEventAuthorityAddress() {
    let anchorEventAuthorityAddress =
      TransactionTemplateContext.ANCHOR_EVENT_AUTHORITY_ADDRESS_BY_PROGRAM.get(
        this.program.address
      );
    if (!anchorEventAuthorityAddress) {
      anchorEventAuthorityAddress = (
        await getProgramDerivedAddress({
          programAddress: this.program.address as Address,
          seeds: [
            getBytesEncoder().encode(
              new Uint8Array([
                95, 95, 101, 118, 101, 110, 116, 95, 97, 117, 116, 104, 111,
                114, 105, 116, 121,
              ])
            ),
          ],
        })
      )[0];
      TransactionTemplateContext.ANCHOR_EVENT_AUTHORITY_ADDRESS_BY_PROGRAM.set(
        this.program.address,
        anchorEventAuthorityAddress
      );
    }
    return anchorEventAuthorityAddress;
  }

  private static readonly ANCHOR_EVENT_INSTRUCTION_DISCRIMINATOR =
    Uint8Array.from([228, 69, 165, 46, 81, 203, 154, 29]);

  private static readonly ANCHOR_EVENT_AUTHORITY_ADDRESS_BY_PROGRAM = new Map<
    string,
    string
  >();

  private __decodeAnchorEvents(
    rawEvents: ReadonlyUint8Array[]
  ): ExecutedTransactionEvents<EVENTS> {
    const events = {
      unknown: [],
    } as unknown as ExecutedTransactionEvents<EVENTS>;

    for (const rawEvent of rawEvents) {
      let parsed = false;
      if (this.config.anchorEventDecoders) {
        for (const [eventName, entry] of Object.entries(
          this.config.anchorEventDecoders
        ) as [
          keyof typeof this.config.anchorEventDecoders,
          (typeof this.config.anchorEventDecoders)[keyof typeof this.config.anchorEventDecoders],
        ][]) {
          try {
            if (entry.discriminator.every((v, i) => v == rawEvent[i])) {
              let key = eventName as keyof typeof events;
              const value = entry.decoder.decode(rawEvent);
              while (events[key]) {
                (key as string) += '_';
              }
              delete (value as any).discriminator;
              events[key] = value;
              parsed = true;
              break;
            }
          } catch (err) {
            if (this.__debug) {
              console.debug(err);
            }
          }
        }
      }
      if (!parsed) {
        events.unknown.push(rawEvent);
      }
    }

    return events;
  }

  async resolve() {
    return {
      description: this.config.description,
      args: this.args,
      events: Object.keys(this.config.anchorEventDecoders ?? {}),
    };
  }
}
