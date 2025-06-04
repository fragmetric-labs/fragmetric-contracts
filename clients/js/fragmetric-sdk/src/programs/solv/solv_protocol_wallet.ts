import { EncodedAccount } from '@solana/kit';
import { BaseAccountContext, TokenAccountContext } from '../../context';
import { SolvVaultAccountContext } from './vault';
import * as system from '@solana-program/system';

export class SolvProtocolWalletAccountContext extends BaseAccountContext<SolvVaultAccountContext> {
  constructor(parent: SolvVaultAccountContext) {
    super(parent, async (parent) => {
      const vault = await parent.resolveAccount(true);
      if (
        !vault ||
        vault.data.solvProtocolWallet == system.SYSTEM_PROGRAM_ADDRESS
      )
        return null;
      return vault.data.solvProtocolWallet;
    });
  }

  async resolve(noCache = false): Promise<any> {
    return this.resolveAccount(noCache);
  }

  protected __decodeAccount(account: EncodedAccount): EncodedAccount {
    return account;
  }

  readonly solvReceiptToken = TokenAccountContext.fromAssociatedTokenSeeds(
    this,
    async (parent) => {
      const [mint, owner] = await Promise.all([
        parent.parent.solvReceiptTokenMint.resolveAddress(true),
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
