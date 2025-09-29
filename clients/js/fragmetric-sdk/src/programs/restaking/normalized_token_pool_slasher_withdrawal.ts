import { EncodedAccount } from '@solana/kit';
import { AccountContext } from '../../context';
import * as restaking from '../../generated/restaking';
import { RestakingSlasherAccountContext } from './normalized_token_pool_slasher';

export class RestakingSlasherWithdrawalAccountContext extends AccountContext<
  RestakingSlasherAccountContext,
  restaking.NormalizedTokenWithdrawalAccount
> {
  public resolve(noCache = false) {
    return this.resolveAccount(noCache);
  }

  protected __decodeAccount(account: EncodedAccount) {
    return restaking.decodeNormalizedTokenWithdrawalAccount(account).data;
  }
}
