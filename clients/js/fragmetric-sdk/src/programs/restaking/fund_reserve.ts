import {
  BaseAccountContext,
  IterativeAccountContext,
  TokenAccountContext,
} from '../../context';
import * as restaking from '../../generated/restaking';
import { RestakingFundAccountContext } from './fund';

export class RestakingFundReserveAccountContext extends BaseAccountContext<RestakingFundAccountContext> {
  constructor(readonly parent: RestakingFundAccountContext) {
    super(parent, async (parent) => {
      const ix = await restaking.getAdminInitializeFundAccountInstructionAsync(
        {
          receiptTokenMint: (await parent.parent.resolveAddress())!,
          program: this.program.address,
        } as any,
        {
          programAddress: this.program.address,
        }
      );
      return ix.accounts[7].address;
    });
  }

  readonly supportedTokens = new IterativeAccountContext(
    this,
    async (parent) => {
      const [self, fund] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.resolveAccount(true),
      ]);
      if (!(self && fund)) return null;
      const addresses = await Promise.all(
        fund.data.supportedTokens
          .slice(0, fund.data.numSupportedTokens)
          .map((item) => {
            return TokenAccountContext.findAssociatedTokenAccountAddress({
              owner: self,
              mint: item.mint,
              tokenProgram: item.program,
            });
          })
      );
      return addresses.filter((address) => !!address);
    },
    async (parent, address) => {
      return new TokenAccountContext(parent, address);
    }
  );

  readonly normalizedToken = TokenAccountContext.fromAssociatedTokenSeeds(
    this,
    async (parent) => {
      const [self, fund] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.resolveAccount(true),
      ]);
      if (fund?.data.normalizedToken?.enabled && self) {
        return {
          owner: self,
          mint: fund.data.normalizedToken.mint,
          tokenProgram: fund.data.normalizedToken.program,
        };
      }
      return null;
    }
  );

  readonly restakingVaultReceiptTokens = new IterativeAccountContext(
    this,
    async (parent) => {
      const [self, fund] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.resolveAccount(true),
      ]);
      if (!self || !fund) return null;
      const addresses = await Promise.all(
        fund.data.restakingVaults
          .slice(0, fund.data.numRestakingVaults)
          .map((item) => {
            return TokenAccountContext.findAssociatedTokenAccountAddress({
              owner: self,
              mint: item.receiptTokenMint,
              tokenProgram: item.receiptTokenProgram,
            });
          })
      );
      return addresses.filter((address) => !!address);
    },
    async (parent, address) => {
      return new TokenAccountContext(parent, address);
    }
  );
}
