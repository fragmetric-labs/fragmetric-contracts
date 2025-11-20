import { Account, EncodedAccount } from '@solana/kit';
import { AccountContext, TokenMintAccountContext } from '../../context';
import * as driftVault from '../../generated/drift_vault';
import { RestakingFundAccountContext } from './fund';

export class DriftVaultAccountContext extends AccountContext<
  RestakingFundAccountContext,
  Account<driftVault.Vault>
> {
  async resolve(noCache = false): Promise<any> {
    return this.resolveAccount(noCache);
  }

  protected __decodeAccount(account: EncodedAccount) {
    return driftVault.decodeVault(account);
  }

  readonly receiptTokenMint = new TokenMintAccountContext(
    this,
    this.runtime.cluster != 'local'
      ? 'UVRT7fFA6hnDb4MPeW7z7gfedcGgkVyhTHaqZzpLYf9'
      : 'HqZT3PvbUtYN8teegyunhvZGK7YLwtmFvyr9927SgCK7'
  );
}