import {
  Address,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
} from '@solana/kit';
import { undefined } from 'valibot';
import {
  RuntimeCluster,
  RuntimeConfig,
  RuntimeContext,
  RuntimeContextPartialOptions,
} from '../runtime';
import { ProgramDerivedContext } from './derived';

export class ProgramContext extends ProgramDerivedContext<RuntimeContext> {
  static readonly addresses: { [k in RuntimeCluster]?: string | null } = {
    mainnet: null,
    devnet: null,
    testnet: null,
    local: null,
  };
  readonly address: Address;

  constructor(
    readonly parent: RuntimeContext,
    programAddress?: string
  ) {
    super();
    const address =
      programAddress ??
      (this.constructor as typeof ProgramContext).addresses[parent.cluster];
    if (!address && this.runtime.options.debug) {
      console.debug(`program is not supported in cluster: ${parent.cluster}`);
    }
    this.address = (address ?? 'unknown') as Address;
  }

  static connect<T extends ProgramContext>(
    this: new (...args: any) => T,
    config: RuntimeConfig,
    options?: RuntimeContextPartialOptions,
    programAddress?: string
  ) {
    return new this(new RuntimeContext(config, options), programAddress);
  }

  static mainnet<T extends ProgramContext>(
    this: new (...args: any) => T,
    rpcURL?: string | null, // default https://api.mainnet-beta.solana.com
    options?: RuntimeContextPartialOptions,
    programAddress?: string
  ) {
    rpcURL = rpcURL ?? 'https://api.mainnet-beta.solana.com';
    const rpcSubscriptionsURL = rpcURL
      .replace('https://', 'wss://')
      .replace('http://', 'ws://');
    return new this(
      new RuntimeContext(
        {
          rpc: createSolanaRpc(rpcURL),
          rpcSubscriptions: createSolanaRpcSubscriptions(rpcSubscriptionsURL),
          cluster: 'mainnet',
        },
        options
      ),
      programAddress
    );
  }

  static devnet<T extends ProgramContext>(
    this: new (...args: any) => T,
    rpcURL?: string | null, // omit or give null to use https://api.devnet.solana.com
    options?: RuntimeContextPartialOptions,
    programAddress?: string
  ) {
    rpcURL = rpcURL ?? 'https://api.devnet.solana.com';
    const rpcSubscriptionsURL = rpcURL
      .replace('https://', 'wss://')
      .replace('http://', 'ws://');
    return new this(
      new RuntimeContext(
        {
          rpc: createSolanaRpc(rpcURL),
          rpcSubscriptions: createSolanaRpcSubscriptions(rpcSubscriptionsURL),
          cluster: 'devnet',
        },
        options
      ),
      programAddress
    );
  }

  static testnet<T extends ProgramContext>(
    this: new (...args: any) => T,
    rpcURL?: string | null, // omit or give null to use https://api.testnet.solana.com
    options?: RuntimeContextPartialOptions,
    programAddress?: string
  ) {
    rpcURL = rpcURL ?? 'https://api.testnet.solana.com';
    const rpcSubscriptionsURL = rpcURL
      .replace('https://', 'wss://')
      .replace('http://', 'ws://');
    return new this(
      new RuntimeContext(
        {
          rpc: createSolanaRpc(rpcURL),
          rpcSubscriptions: createSolanaRpcSubscriptions(rpcSubscriptionsURL),
          cluster: 'testnet',
        },
        options
      ),
      programAddress
    );
  }

  static local<T extends ProgramContext>(
    this: new (...args: any) => T,
    rpcURL?: string | null, // omit or give null to use http://localhost:8899
    rpcSubscriptionURL?: string | null, // omit or give null to use derived one from given rpcURL
    options?: RuntimeContextPartialOptions,
    programAddress?: string
  ) {
    rpcURL = rpcURL ?? 'http://localhost:8899';
    let resolvedRPCSubscriptionsURL =
      rpcSubscriptionURL ??
      rpcURL.replace('https://', 'wss://').replace('http://', 'ws://');
    if (!rpcSubscriptionURL) {
      const rpcURLPort = parseInt(new URL(resolvedRPCSubscriptionsURL).port);
      if (!isNaN(rpcURLPort)) {
        resolvedRPCSubscriptionsURL = resolvedRPCSubscriptionsURL.replace(
          `:${rpcURLPort}`,
          `:${rpcURLPort + 1}`
        );
      }
    }
    return new this(
      new RuntimeContext(
        {
          rpc: createSolanaRpc(rpcURL),
          rpcSubscriptions: createSolanaRpcSubscriptions(
            resolvedRPCSubscriptionsURL
          ),
          cluster: 'local',
        },
        options
      ),
      programAddress
    );
  }

  toContextDescription() {
    const desc = super.toContextDescription();
    return {
      ...desc,
      properties: {
        ...desc.properties,
        address: this.address,
      },
    };
  }

  get runtime() {
    return this.parent;
  }

  protected get __debug() {
    return this.parent.options.debug;
  }

  get program() {
    return this;
  }

  protected get __maybeRuntimeOptions() {
    return this.parent.options;
  }

  resolve(noCache = false): Promise<any> {
    return Promise.resolve(undefined);
  }
}
