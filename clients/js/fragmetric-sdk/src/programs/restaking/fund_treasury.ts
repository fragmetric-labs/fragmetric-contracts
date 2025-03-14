import {
  BaseAccountContext,
  IterativeAccountContext,
  TokenAccountContext,
} from '../../context';
import * as restaking from '../../generated/restaking';
import { RestakingFundAccountContext } from './fund';

export class RestakingFundTreasuryAccountContext extends BaseAccountContext<RestakingFundAccountContext> {
  constructor(readonly parent: RestakingFundAccountContext) {
    super(parent, async (parent) => {
      const ix =
        await restaking.getFundManagerAddSupportedTokenInstructionAsync(
          {
            receiptTokenMint: (await parent.parent.resolveAddress())!,
            program: this.program.address,
            supportedTokenMint: this.program.address,
            supportedTokenProgram: this.program.address,
            pricingSource: {
              __kind: 'SPLStakePool',
              address: this.program.address,
            },
          },
          {
            programAddress: this.program.address,
          }
        );
      return ix.accounts[4].address;
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
}
