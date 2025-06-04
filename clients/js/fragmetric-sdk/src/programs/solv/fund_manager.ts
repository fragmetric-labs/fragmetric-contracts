import { EncodedAccount } from '@solana/kit';
import { BaseAccountContext, TokenAccountContext } from '../../context';
import { SolvVaultAccountContext } from './vault';

export class FundManagerAccountContext extends BaseAccountContext<SolvVaultAccountContext> {
  constructor(parent: SolvVaultAccountContext) {
    super(parent, async (parent) => {
      const vault = await parent.resolveAccount(true);
      if (
        !vault ||
        vault.data.fundManager == vault.data.vaultManager
      )
        return null;
      return vault.data.fundManager;
    });
  }

  async resolve(noCache = false): Promise<any> {
    return this.resolveAccount(noCache);
  }

  protected __decodeAccount(account: EncodedAccount): EncodedAccount {
    return account;
  }

  readonly receiptToken = TokenAccountContext.fromAssociatedTokenSeeds(
    this,
    async (parent) => {
      const [mint, owner] = await Promise.all([
        parent.parent.receiptTokenMint.resolveAddress(true),
        parent.resolveAddress(true),
      ]);
      if (!(mint && owner)) return null;
      return {
        mint,
        owner,
      };
    }
  );

  readonly supportedToken = TokenAccountContext.fromAssociatedTokenSeeds(
    this,
    async (parent) => {
      const [mint, owner] = await Promise.all([
        parent.parent.supportedTokenMint.resolveAddress(true),
        parent.resolveAddress(true),
      ]);
      if (!(mint && owner)) return null;
      return {
        mint,
        owner,
      };
    }
  );
}
