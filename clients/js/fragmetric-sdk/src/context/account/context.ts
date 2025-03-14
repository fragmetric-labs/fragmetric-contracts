import { Address, EncodedAccount } from '@solana/kit';
import {
  AccountAddressResolver,
  AccountAddressResolverVariant,
  transformAddressResolverVariant,
} from '../address';
import { Context } from '../context';
import { ProgramDerivedContext } from '../program';

export abstract class AccountContext<
  P extends Context<any>,
  A = EncodedAccount,
> extends ProgramDerivedContext<P> {
  private readonly __addressResolver: AccountAddressResolver<P>;
  protected __address: Address | undefined;
  protected __account: A | null | undefined;

  toContextDescription() {
    const desc = super.toContextDescription();
    return {
      ...desc,
      properties: {
        ...desc.properties,
        address: this.__address,
      },
      unresolved: typeof this.__account == 'undefined',
      unused: typeof this.__account != 'undefined' && this.__account == null,
    };
  }

  constructor(
    readonly parent: P,
    // addressResolver: AccountAddressResolver<P>
    addressResolver: AccountAddressResolverVariant<P>
  ) {
    super();
    this.__addressResolver = transformAddressResolverVariant(addressResolver);
  }

  get address() {
    if (typeof this.__address == 'undefined' && this.__debug) {
      console.warn(`address is not computed yet: ${this}`);
    }
    return this.__address;
  }

  get account() {
    if (typeof this.__address == 'undefined' && this.__debug) {
      console.warn(`account is not resolved yet: ${this}`);
    }
    return this.__account;
  }

  async resolveAddress(noCache = false): Promise<Address | null> {
    return this.__deduplicated(
      {
        method: 'resolveAddress',
        params: [noCache],
        alternativeParams: noCache ? null : [true],
        intervalSeconds: noCache
          ? 0
          : this.__maybeRuntimeOptions?.rpc.accountDeduplicationIntervalSeconds,
      },
      () => this.__resolveAddress()
    );
  }

  protected async __resolveAddress(): Promise<Address | null> {
    const address = await this.__addressResolver(this.parent);
    if (address) {
      this.__address = address as Address;
      if (this.__debug) {
        console.debug(`computed account address: ${this}`);
      }
      return this.__address;
    }

    this.__address = undefined;
    if (this.__debug) {
      console.warn(`failed to compute account address: ${this}`);
    }
    return null;
  }

  async resolveAccount(noCache = false): Promise<A | null> {
    return this.__deduplicatedResolveAccount(noCache);
  }

  protected async __deduplicatedResolveAccount(
    noCache = false
  ): Promise<A | null> {
    return this.__deduplicated(
      {
        method: 'resolveAccount',
        params: [noCache],
        alternativeParams: noCache ? null : [true],
        intervalSeconds: noCache
          ? 0
          : this.__maybeRuntimeOptions?.rpc.accountDeduplicationIntervalSeconds,
      },
      () => this.__resolveAccount(noCache)
    );
  }

  protected async __resolveAccount(noCache: boolean): Promise<A | null> {
    const encodedAccount = await this.__fetchAccount(noCache);
    return (this.__account = encodedAccount
      ? this.__decodeAccount(encodedAccount)
      : null);
  }

  async resolveAccountTree(noCache = false, maxDepth = 10): Promise<A | null> {
    return this.__deduplicated(
      {
        method: 'resolveAccountTree',
        params: [noCache, maxDepth],
        alternativeParams: noCache ? null : [true, maxDepth],
        intervalSeconds: noCache
          ? 0
          : this.__maybeRuntimeOptions?.rpc.accountDeduplicationIntervalSeconds,
      },
      () => this.__resolveAccountTree(noCache, maxDepth)
    );
  }

  // to resolve self then children instead of parallel resolving to ensure dynamic population of child contexts from tree walk
  protected readonly __useLazyAccountTreeResolve: boolean = false;

  protected async __resolveAccountTree(
    noCache: boolean,
    maxDepth: number,
    recurring = false
  ) {
    const promises: Promise<any>[] = [];
    this.__visitContextGraph((node) => {
      const ctx = node.context;
      if (ctx instanceof AccountContext && !(recurring && ctx === this)) {
        if (ctx.__useLazyAccountTreeResolve) {
          promises.push(
            ctx
              .__deduplicatedResolveAccount(noCache)
              .then(() =>
                ctx.__resolveAccountTree(noCache, maxDepth - node.depth, true)
              )
          );
        } else {
          promises.push(ctx.__deduplicatedResolveAccount(noCache));
        }
      }
      return { in: 0, out: Math.min(maxDepth, maxDepth - node.depth) };
    });
    await Promise.all(promises);

    if (recurring) {
      return Promise.all(promises);
    }
    return promises[0];
  }

  protected async __fetchAccount(
    noCache: boolean
  ): Promise<EncodedAccount | null> {
    const address = await this.__resolveAddress();
    if (address) {
      if (noCache) {
        this.runtime.invalidateAccount(address);
      }
      return this.runtime.fetchAccount(address);
    }
    return null;
  }

  protected abstract __decodeAccount(account: EncodedAccount): A;

  abstract resolve(noCache?: boolean): Promise<any>;
}
