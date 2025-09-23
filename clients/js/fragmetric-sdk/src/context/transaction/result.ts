import {
  TransactionMessage,
  TransactionMessageWithFeePayer,
  GetTransactionApi,
  ReadonlyUint8Array,
  Signature,
} from '@solana/kit';
import { ProgramDerivedContext } from '../program';
import { TransactionTemplateContext } from './template';

const narrowedGetTransactionAPI = (api: GetTransactionApi) =>
  api.getTransaction('' as Signature, {
    maxSupportedTransactionVersion: 0,
    encoding: 'base64',
  });

export type ExecutedTransactionResult = Exclude<
  ReturnType<typeof narrowedGetTransactionAPI>,
  'transaction'
> & { transaction: TransactionMessage & TransactionMessageWithFeePayer };

export type ExecutedTransactionEvents<EVENTS extends Record<string, any>> =
  Partial<EVENTS> & { unknown: ReadonlyUint8Array[] };

export class TransactionResultContext<
  P extends TransactionTemplateContext<any, any, any>,
  ARGS,
  EVENTS extends Record<string, any>,
> extends ProgramDerivedContext<P> {
  constructor(
    readonly parent: P,
    readonly signature: string,
    readonly args: ARGS | null,
    readonly events: ExecutedTransactionEvents<EVENTS> | null = null,
    readonly result: ExecutedTransactionResult | null = null
  ) {
    super();
    this.succeeded = !!(this.result?.meta && !this.result.meta.err);
  }

  public readonly succeeded: boolean;
  private __chainedTransactionExecutor?: () => Promise<this>;

  toContextDescription() {
    const desc = super.toContextDescription();
    return {
      ...desc,
      properties: {
        ...desc.properties,
        signature: this.signature,
        succeeded: this.succeeded,
        slot: this.result?.slot,
      },
    };
  }

  toJSON() {
    return {
      signature: this.signature,
      succeeded: this.succeeded,
      slot: this.result?.slot,
      args: this.args,
      events: this.events,
    };
  }

  __setNextTransactionExecutor(executor: () => Promise<this>) {
    if (this.__chainedTransactionExecutor) {
      throw new Error(`chained transaction template executor is already set`);
    }
    this.__chainedTransactionExecutor = executor;
  }

  get executeChainedTransaction() {
    return this.__chainedTransactionExecutor;
  }

  get slot() {
    return this.result?.slot;
  }

  async resolve() {
    return {
      ...this.toJSON(),
      logs: this.result?.meta?.logMessages,
    };
  }
}
