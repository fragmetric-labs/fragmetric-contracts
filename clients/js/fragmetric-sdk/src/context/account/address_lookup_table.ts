import * as alt from '@solana-program/address-lookup-table';
import { Account, EncodedAccount } from '@solana/kit';
import { Context } from '../context';
import { AccountContext } from './context';

export class AddressLookupTableAccountContext<
  P extends Context<any>,
> extends AccountContext<P, Account<alt.AddressLookupTable>> {
  async resolve(noCache = false) {
    return this.resolveAccount(noCache).then(
      (account) => account?.data.addresses ?? null
    );
  }

  protected __decodeAccount(account: EncodedAccount) {
    return alt.decodeAddressLookupTable(account);
  }

  toContextDescription() {
    const desc = super.toContextDescription();
    return {
      ...desc,
      properties: {
        ...desc.properties,
        length: this.__account?.data.addresses.length,
        lastExtendedSlot: this.__account?.data.lastExtendedSlot,
      },
    };
  }
}
