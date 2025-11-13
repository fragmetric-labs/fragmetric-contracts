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

  readonly vaultReceiptTokenMint = new TokenMintAccountContext(
    this,
    'A4npkVMUk88rX4iMQ32QALivUiWZr1GoNQGHbfygitZt' // TODO: This address need to be replaced
  );

  readonly receiptTokenMint = new TokenMintAccountContext(
    this,
    'A4npkVMUk88rX4iMQ32QALivUiWZr1GoNQGHbfygitZt' // TODO: This address need to be replaced
  );
}
