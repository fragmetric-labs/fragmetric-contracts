import DataLoader from 'dataloader';
import { LRUCache } from 'lru-cache';
import { AccountAddressResolverVariant } from '../address';
import { Context } from '../context';
import { RuntimeContext } from '../runtime';
import { AccountContext } from './context';

export type FragmetricMetadata<D = Record<string, any>> = {
  address: string;
  type: string;
  symbol: string;
  displayName: string;
  data: {
    apy: number;
    oneTokenAsSOL: number;
    oneTokenAsUSD: number;
    tvlAsSOL: number;
    tvlAsUSD: number;
  } & D;
  updatedAt: string;
  updateIntervalSeconds: number;
};

export class FragmetricMetadataContext<
  P extends Context<any>,
  D extends Record<string, any> = Record<string, any>,
> extends AccountContext<P, FragmetricMetadata<D>> {
  async resolve(noCache?: boolean) {
    return this.resolveAccount(noCache);
  }

  static from<
    P extends AccountContext<any, any>,
    D extends Record<string, any> = Record<string, any>,
  >(parent: P) {
    if (parent instanceof FragmetricMetadataContext) {
      throw new Error(
        `cannot create a circular fragmetric market feed context`
      );
    }
    return new FragmetricMetadataContext<P, D>(parent, (parent) =>
      parent.resolveAddress()
    );
  }

  protected static __feedLoaderAndCacheMap: Map<
    RuntimeContext,
    {
      loader: DataLoader<string, FragmetricMetadata | null>;
      cache: LRUCache<string, Promise<FragmetricMetadata | null>>;
    }
  > = new Map();

  protected readonly __feedLoader: DataLoader<
    string,
    FragmetricMetadata | null
  >;
  protected readonly __feedCache: LRUCache<
    string,
    Promise<FragmetricMetadata | null>
  >;

  constructor(
    readonly parent: P,
    addressResolver: AccountAddressResolverVariant<P>
  ) {
    super(parent, addressResolver);

    const existing = FragmetricMetadataContext.__feedLoaderAndCacheMap.get(
      this.runtime
    );
    if (existing) {
      this.__feedLoader = existing.loader;
      this.__feedCache = existing.cache;
      return;
    }

    const url = `https://api${this.runtime.cluster == 'devnet' ? '.dev' : ''}.fragmetric.xyz/v1/public/feeds?addresses=`;
    const options = this.runtime.options.rpc;

    this.__feedCache = new LRUCache<string, Promise<FragmetricMetadata | null>>(
      {
        max: 100,
        ttl: Math.max(options.accountCacheTTLSeconds, 60) * 1000,
        allowStale: false,
      }
    );

    this.__feedLoader = new DataLoader<string, FragmetricMetadata | null>(
      async (
        keys: readonly string[]
      ): Promise<(FragmetricMetadata | Error)[]> => {
        try {
          if (this.__debug) {
            console.debug('fetching fragmetric market feed in a batch:', keys);
          }
          const res = await fetch(
            url + (keys.length < 10 ? keys.join(',') : '')
          );
          if (!res.ok) {
            const err = new Error(`${res.statusText}: ${await res.text()}`);
            return keys.map(() => err);
          }
          const data = (await res.json()) as {
            [key: string]: FragmetricMetadata;
          };
          return keys.map((key) => {
            return data[key] ?? null;
          });
        } catch (err) {
          return keys.map(() => err as Error);
        }
      },
      {
        cache: true,
        batch: true,
        maxBatchSize: 100,
        batchScheduleFn: (callback) =>
          setTimeout(callback, options.accountBatchIntervalMilliseconds),
        cacheKeyFn: (key) => key,
        cacheMap: this.__feedCache,
      }
    );

    FragmetricMetadataContext.__feedLoaderAndCacheMap.set(this.runtime, {
      loader: this.__feedLoader,
      cache: this.__feedCache,
    });
    if (this.__debug) {
      console.debug(`created fragmetric market feed dataloader:`, url);
    }
  }

  protected async __fetchAccount(noCache: boolean): Promise<any> {
    const address = await this.resolveAddress(noCache);
    if (address) {
      if (noCache) {
        if (this.__feedCache.delete(address) && this.__debug) {
          console.debug('invalidated fragmetric market feed:', address);
        }
      }
      return this.__feedLoader.load(address);
    }
    return null;
  }

  protected __decodeAccount(data: any) {
    return data;
  }
}
