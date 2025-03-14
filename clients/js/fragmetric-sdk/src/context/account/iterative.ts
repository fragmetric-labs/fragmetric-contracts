import { EncodedAccount } from '@solana/kit';
import { Context } from '../context';
import { AccountContext } from './context';

export class IterativeAccountContext<
    P extends Context<any>,
    C extends AccountContext<P, A>,
    A = C extends AccountContext<P, infer A> ? A : EncodedAccount,
  >
  extends AccountContext<P, (A | null)[]>
  implements Iterable<AccountContext<P, A>>
{
  async resolve(noCache?: boolean) {
    await this.resolveAccount(noCache);
    return Promise.all(this.children.map((c) => c.resolve(noCache)));
  }

  constructor(
    readonly parent: P,
    protected readonly addressesResolver: (
      parent: P
    ) => Promise<string[] | null>,
    protected readonly accountResolver: (
      parent: P,
      address: string
    ) => Promise<AccountContext<P, A> | null>
  ) {
    super(parent, async (parent) => {
      return addressesResolver(parent).then((addresses) =>
        addresses ? addresses.join(', ') : null
      );
    });
  }

  protected readonly __children: AccountContext<P, A>[] = [];

  get children() {
    return this.__children;
  }

  get length() {
    return this.__children.length;
  }

  [Symbol.iterator](): Iterator<AccountContext<P, A>> {
    return this.children.values();
  }

  toContextDescription() {
    const desc = super.toContextDescription();
    const res = {
      ...desc,
      properties: {
        ...desc.properties,
        length: this.__account?.length,
        types:
          Array.from(
            new Set(
              this.children.map((c) => c.toContextDescription().label)
            ).keys()
          ).join(',') || undefined,
      },
      unresolved: typeof this.__account == 'undefined',
      unused: this.__children?.every((item) => item.account == null) ?? false,
    };
    delete res.properties?.address;
    return res;
  }

  async resolveAccount(noCache = false) {
    return this.resolveAccountTree(noCache);
  }

  protected readonly __useLazyAccountTreeResolve = true;

  protected async __resolveAccount(noCache: boolean) {
    // resolve child addresses
    const addresses = await this.addressesResolver(this.parent);
    if (!addresses) {
      this.__account = null;
      return null;
    }

    // create and resolve child accounts
    const children = (
      await Promise.all(
        addresses.map((address) => {
          const existingChild = this.__children?.find(
            (child) => child.address == address
          );
          if (existingChild) {
            return existingChild;
          }
          return this.accountResolver(this.parent, address);
        })
      )
    ).filter((child) => !!child);

    for (let i = 0; i < this.__children.length; i++) {
      delete (this as any)[i];
      delete (this as any)['_' + i];
    }
    this.__children.splice(0, this.__children.length);
    this.__children.push(...children);

    for (let i = 0; i < this.__children.length; i++) {
      Object.defineProperty(this, i, {
        get: () => this.__children[i],
        enumerable: true,
        configurable: true,
      });
      Object.defineProperty(this, '_' + i, {
        get: () => this.__children[i],
        enumerable: true,
        configurable: true,
      });
    }

    return (this.__account = await Promise.all(
      children.map((c) => c.resolveAccount(noCache))
    ));
  }

  protected __decodeAccount(data: EncodedAccount): (A | null)[] {
    throw new Error('unused method');
  }
}
