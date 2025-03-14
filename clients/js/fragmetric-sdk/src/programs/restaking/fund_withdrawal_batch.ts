import { Account, Address, EncodedAccount, Some } from '@solana/kit';
import { AccountContext } from '../../context';
import * as restaking from '../../generated/restaking';
import { RestakingFundAccountContext } from './fund';

export class RestakingFundWithdrawalBatchAccountContext extends AccountContext<
  RestakingFundAccountContext,
  Account<restaking.FundWithdrawalBatchAccount>
> {
  async resolve(noCache = false) {
    const account = await this.resolveAccount(noCache);
    if (!account) {
      return null;
    }
    const {
      discriminator,
      dataVersion,
      bump,
      processedAt,
      reserved,
      ...props
    } = account.data;

    return {
      ...props,
      processedAt: new Date(Number(processedAt) * 1000),
    };
  }

  protected __decodeAccount(account: EncodedAccount) {
    return restaking.decodeFundWithdrawalBatchAccount(account);
  }

  toContextDescription() {
    const desc = super.toContextDescription();
    return {
      ...desc,
      properties: {
        ...desc.properties,
        mint: this.__account
          ? ((this.__account?.data.supportedTokenMint as Some<Address>)
              ?.value ?? null)
          : undefined,
      },
    };
  }
}
