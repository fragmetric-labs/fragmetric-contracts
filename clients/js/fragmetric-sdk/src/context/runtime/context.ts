import {
  AddressLookupTable,
  getAddressLookupTableDecoder,
} from '@solana-program/address-lookup-table';
import { decodeNonce } from '@solana-program/system';
import {
  Address,
  AddressesByLookupTableAddress,
  Blockhash,
  Commitment,
  EncodedAccount,
  Nonce,
  sendAndConfirmDurableNonceTransactionFactory,
  sendAndConfirmTransactionFactory,
  Signature,
} from '@solana/kit';
import { Buffer } from 'buffer';
import DataLoader from 'dataloader';
import { LRUCache } from 'lru-cache';
import { Context } from '../context';
import { ProgramDerivedContext } from '../program';
import { TransactionTemplateOverrides } from '../transaction';
import {
  createRuntime,
  Runtime,
  RuntimeCluster,
  RuntimeConfig,
  RuntimeRPC,
  RuntimeRPCSubscriptions,
  RuntimeType,
} from './runtime';

export type RuntimeContextOptions<
  P extends ProgramDerivedContext<any> = ProgramDerivedContext<any>,
> = {
  rpc: {
    // default is 2s
    accountDeduplicationIntervalSeconds: number;
    // default is 10s
    accountCacheTTLSeconds: number;
    // default is 50ms
    accountBatchIntervalMilliseconds: number;
    // default is 100req
    accountBatchMaxSize: number;

    // default is 250ms
    blockhashCacheTTLMilliseconds: number;
    // default is 50ms
    blockhashBatchIntervalMilliseconds: number;
    // default is 100req
    blockhashBatchMaxSize: number;
  };
  transaction: {
    // set global fee payer address for all tx
    feePayer: TransactionTemplateOverrides<P, any>['feePayer'];

    // set global signers
    signers: TransactionTemplateOverrides<P, any>['signers'];

    // set global hooks
    executionHooks: TransactionTemplateOverrides<P, any>['executionHooks'];

    // set global compute unit budget
    computeBudget: TransactionTemplateOverrides<P, any>['computeBudget'];

    // default is confirmed level
    confirmationCommitment: Commitment;
  };
  debug: boolean;
};

type TwoLevelPartial<T> = {
  [K in keyof T]?: T[K] extends object
    ? T[K] extends any[]
      ? T[K]
      : { [P in keyof T[K]]?: T[K][P] }
    : T[K];
};

export type RuntimeContextPartialOptions<
  P extends ProgramDerivedContext<any> = ProgramDerivedContext<any>,
> = TwoLevelPartial<RuntimeContextOptions<P>>;

type LatestBlockhashResponse = Readonly<{
  blockhash: Blockhash;
  lastValidBlockHeight: bigint;
}>;

export class RuntimeContext extends Context<null> implements Runtime {
  resolve() {
    return Promise.resolve(undefined);
  }

  readonly parent = null;
  readonly type: RuntimeType;
  readonly cluster: RuntimeCluster;
  readonly rpc: RuntimeRPC;
  readonly rpcSubscriptions: RuntimeRPCSubscriptions | null;
  readonly options: RuntimeContextOptions;
  readonly sendAndConfirmTransaction: ReturnType<
    typeof sendAndConfirmTransactionFactory
  > | null;
  readonly sendAndConfirmDurableNonceTransaction: ReturnType<
    typeof sendAndConfirmDurableNonceTransactionFactory
  > | null;

  private readonly __accountCache: LRUCache<
    string,
    Promise<EncodedAccount | null>
  >;
  private readonly __accountLoader: DataLoader<string, EncodedAccount | null>;
  private readonly __latestBlockhashCache: LRUCache<
    string,
    Promise<LatestBlockhashResponse>,
    unknown
  >;
  private readonly __latestBlockhashLoader: DataLoader<
    string,
    LatestBlockhashResponse,
    string
  >;

  constructor(
    readonly config: RuntimeConfig,
    options?: RuntimeContextPartialOptions
  ) {
    super();
    const runtime = createRuntime(config);
    this.type = runtime.type;
    this.cluster = runtime.cluster;
    this.rpc = runtime.rpc;
    this.rpcSubscriptions = runtime.rpcSubscriptions ?? null;
    this.sendAndConfirmTransaction = this.rpcSubscriptions
      ? sendAndConfirmTransactionFactory({
          rpc: this.rpc,
          rpcSubscriptions: this.rpcSubscriptions,
        })
      : null;
    this.sendAndConfirmDurableNonceTransaction = this.rpcSubscriptions
      ? sendAndConfirmDurableNonceTransactionFactory({
          rpc: this.rpc,
          rpcSubscriptions: this.rpcSubscriptions,
        })
      : null;

    this.options = {
      rpc: {
        accountDeduplicationIntervalSeconds: NaN,
        accountCacheTTLSeconds: NaN,
        accountBatchIntervalMilliseconds: NaN,
        accountBatchMaxSize: NaN,
        blockhashCacheTTLMilliseconds: NaN,
        blockhashBatchIntervalMilliseconds: NaN,
        blockhashBatchMaxSize: NaN,
        ...options?.rpc,
      },
      transaction: {
        signers: undefined,
        executionHooks: undefined,
        confirmationCommitment: 'confirmed',
        ...options?.transaction,
        feePayer:
          options?.transaction?.feePayer ??
          options?.transaction?.signers?.[0] ??
          undefined,
        computeBudget: options?.transaction?.computeBudget ?? undefined,
      },
      debug: options?.debug == true,
    };

    if (isNaN(this.options.rpc.accountDeduplicationIntervalSeconds)) {
      this.options.rpc.accountDeduplicationIntervalSeconds = 2;
    }
    if (isNaN(this.options.rpc.accountCacheTTLSeconds)) {
      this.options.rpc.accountCacheTTLSeconds = 10;
    }
    if (isNaN(this.options.rpc.accountBatchMaxSize)) {
      this.options.rpc.accountBatchMaxSize = 100;
    }
    if (isNaN(this.options.rpc.accountBatchIntervalMilliseconds)) {
      this.options.rpc.accountBatchIntervalMilliseconds = 50;
    }
    if (isNaN(this.options.rpc.blockhashCacheTTLMilliseconds)) {
      this.options.rpc.blockhashCacheTTLMilliseconds = 250;
    }
    if (isNaN(this.options.rpc.blockhashBatchIntervalMilliseconds)) {
      this.options.rpc.blockhashBatchIntervalMilliseconds = 50;
    }
    if (isNaN(this.options.rpc.blockhashBatchMaxSize)) {
      this.options.rpc.blockhashBatchMaxSize = 50;
    }

    this.__accountCache = new LRUCache<string, Promise<EncodedAccount | null>>({
      max: 100,
      ttl: this.options.rpc.accountCacheTTLSeconds * 1000,
      allowStale: false,
    });
    this.__accountLoader = new DataLoader<string, EncodedAccount | null>(
      this.__fetchBatchedAccounts.bind(this),
      {
        cache: this.options.rpc.accountCacheTTLSeconds > 0,
        batch:
          this.options.rpc.accountBatchMaxSize > 1 &&
          this.options.rpc.accountBatchIntervalMilliseconds > 0,
        maxBatchSize: this.options.rpc.accountBatchMaxSize,
        batchScheduleFn: (callback) =>
          setTimeout(
            callback,
            this.options.rpc.accountBatchIntervalMilliseconds
          ),
        cacheKeyFn: (key) => key,
        cacheMap: this.__accountCache,
      }
    );

    this.__latestBlockhashCache = new LRUCache<
      string,
      Promise<LatestBlockhashResponse>
    >({
      max: 1,
      ttl: this.options.rpc.blockhashCacheTTLMilliseconds,
      allowStale: false,
    });
    this.__latestBlockhashLoader = new DataLoader<
      string,
      LatestBlockhashResponse
    >(this.__fetchBatchedLatestBlockhash.bind(this), {
      cache:
        this.type == 'solana' &&
        this.options.rpc.blockhashCacheTTLMilliseconds > 0,
      batch:
        this.options.rpc.blockhashBatchMaxSize > 1 &&
        this.options.rpc.blockhashBatchIntervalMilliseconds > 0,
      maxBatchSize: this.options.rpc.blockhashBatchMaxSize,
      batchScheduleFn: (callback) =>
        setTimeout(
          callback,
          this.options.rpc.blockhashBatchIntervalMilliseconds
        ),
      cacheKeyFn: (key) => key,
      cacheMap: this.__latestBlockhashCache,
    });
  }

  public get abortController() {
    return this.__abortController;
  }
  private __abortController: AbortController = this.__createAbortController();
  private __createAbortController() {
    const controller = new AbortController();
    controller.signal.addEventListener('abort', () => {
      this.__abortController = this.__createAbortController();
    });
    return controller;
  }

  toContextDescription() {
    const desc = super.toContextDescription();
    return {
      ...desc,
      properties: {
        ...desc.properties,
        type: this.type,
        cluster: this.cluster,
      },
    };
  }

  async fetchLatestBlockhash() {
    return this.__latestBlockhashLoader.load('');
  }

  private async __fetchBatchedLatestBlockhash(
    keys: readonly string[]
  ): Promise<(LatestBlockhashResponse | Error)[]> {
    try {
      if (this.options.debug) {
        if (keys.length == 1) {
          console.debug('fetching recent blockhash in a batch (single)');
        } else {
          console.debug(
            `fetching recent blockhash in a batch (${keys.length})`
          );
        }
      }

      const res = await this.rpc
        .getLatestBlockhash()
        .send()
        .then((res) => {
          return res.value;
        });

      return keys.map((key, i) => {
        return res;
      });
    } catch (err) {
      return keys.map(() => err as Error);
    }
  }

  async fetchNonceConfig(address: string) {
    const encodedNonceAccount = await this.fetchAccount(address, true);
    if (encodedNonceAccount) {
      const nonceAccount = decodeNonce(encodedNonceAccount);
      return {
        nonce: nonceAccount.data.blockhash as string as Nonce,
        nonceAccountAddress: nonceAccount.address,
        nonceAuthorityAddress: nonceAccount.data.authority,
      };
    }
    return null;
  }

  async fetchAddressLookupTable(
    key: string,
    noCache: boolean = false
  ): Promise<AddressLookupTable | null> {
    const account = await this.fetchAccount(key, noCache);
    if (!account) {
      return null;
    }
    return getAddressLookupTableDecoder().decode(account.data);
  }

  async fetchTransaction(
    signature: string,
    commitment: Commitment = 'confirmed'
  ) {
    return this.rpc
      .getTransaction(signature as Signature, {
        encoding: 'base64',
        commitment,
        maxSupportedTransactionVersion: 0,
      })
      .send();
  }

  async fetchSlot(commitment: Commitment = 'confirmed') {
    return this.rpc.getSlot({ commitment }).send();
  }

  async fetchEpochInfo(commitment: Commitment = 'confirmed') {
    return this.rpc.getEpochInfo({ commitment }).send();
  }

  async fetchMultipleAddressLookupTables(
    keys: string[],
    noCache: boolean = false
  ): Promise<AddressesByLookupTableAddress> {
    return Promise.all(
      keys.map((key) => this.fetchAddressLookupTable(key, noCache))
    ).then((tables) => {
      return tables.reduce((addresses, table, index) => {
        if (table) {
          const tableAddress = keys[index] as Address;
          addresses[tableAddress] = table.addresses;
        }
        return addresses;
      }, {} as AddressesByLookupTableAddress);
    });
  }

  async fetchAccount(
    key: string,
    noCache: boolean = false
  ): Promise<EncodedAccount | null> {
    if (noCache) {
      if (this.options.debug) {
        console.debug('fetching an account (no cache):', key);
      }
      const res = await this.rpc
        .getAccountInfo(key as Address, { encoding: 'base64' })
        .send();
      const account = res.value;
      if (!account) return null;
      const result = {
        address: key as Address,
        executable: account.executable,
        lamports: account.lamports,
        data: Uint8Array.from(Buffer.from(account.data[0], account.data[1])),
        programAddress: account.owner,
        space: account.space,
      };
      this.__accountLoader.clear(key).prime(key, result);
      return result;
    }
    return this.__accountLoader.load(key);
  }

  async fetchMultipleAccounts(
    keys: string[]
  ): Promise<(EncodedAccount | null | Error)[]> {
    return this.__accountLoader.loadMany(keys);
  }

  invalidateAccount(key: string) {
    if (this.__accountCache.has(key)) {
      this.__accountLoader.clear(key);
      if (this.options.debug) {
        console.debug('invalidated an account:', key);
      }
      return true;
    }
    return false;
  }

  private async __fetchBatchedAccounts(
    keys: readonly string[]
  ): Promise<(EncodedAccount | null | Error)[]> {
    try {
      if (this.options.debug) {
        if (keys.length == 1) {
          console.debug('fetching accounts in a batch (single):', keys);
        } else {
          console.debug(`fetching accounts in a batch (${keys.length}):`, keys);
        }
      }

      const res =
        keys.length == 1
          ? await this.rpc
              .getAccountInfo(keys[0] as Address, { encoding: 'base64' })
              .send()
              .then((res) => [res.value])
          : await this.rpc
              .getMultipleAccounts(keys as Address[], { encoding: 'base64' })
              .send()
              .then((res) => res.value);

      return keys.map((key, i) => {
        const account = res[i];
        if (!account) return null;
        return {
          address: key as Address,
          executable: account.executable,
          lamports: account.lamports,
          data: Uint8Array.from(Buffer.from(account.data[0], account.data[1])),
          programAddress: account.owner,
          space: account.space,
        };
      });
    } catch (err) {
      return keys.map(() => err as Error);
    }
  }
}
