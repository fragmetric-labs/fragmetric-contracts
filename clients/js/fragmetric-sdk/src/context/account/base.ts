import { EncodedAccount } from '@solana/kit';
import { Context } from '../context';
import { AccountContext } from './context';

export class BaseAccountContext<
  P extends Context<any>,
> extends AccountContext<P> {
  async resolve(noCache = false): Promise<any> {
    return this.resolveAccount(noCache);
  }

  toContextDescription() {
    const desc = super.toContextDescription();
    return {
      ...desc,
      properties: {
        ...desc.properties,
        lamports: this.__account?.lamports,
      },
    };
  }

  protected __decodeAccount(account: EncodedAccount) {
    return account;
  }
}
