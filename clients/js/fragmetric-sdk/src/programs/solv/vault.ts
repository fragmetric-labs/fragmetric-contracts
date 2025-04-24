import { Account, EncodedAccount } from '@solana/kit';
import { AccountContext } from '../../context';
import * as solv from '../../generated/solv';
import { SolvVaultReceiptTokenMintAccountContext } from './receipt_token_mint';

export class SolvVaultAccountContext extends AccountContext<
  SolvVaultReceiptTokenMintAccountContext,
  Account<solv.VaultAccount>
> {
  async resolve(noCache = false) {
    // TODO: impl main resolve fn for solv Vault
    return this.resolveAccount(noCache);
  }

  constructor(readonly parent: SolvVaultReceiptTokenMintAccountContext) {
    super(parent, async (parent) => {
      const receiptTokenMint = await parent.resolveAddress();
      if (receiptTokenMint) {
        const ix = await solv.getInitializeVaultAccountInstructionAsync(
          { receiptTokenMint } as any,
          { programAddress: parent.program.address }
        );
        return ix.accounts![6].address;
      }
      return null;
    });
  }

  protected __decodeAccount(account: EncodedAccount) {
    return solv.decodeVaultAccount(account);
  }

  // TODO: readonly initializeOrUpdateAccount = new TransactionTemplateContext(
}
