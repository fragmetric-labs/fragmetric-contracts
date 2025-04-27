import {
  BaseAccountContext,
  IterativeAccountContext,
  TokenAccountContext,
} from '../../context';
import * as restaking from '../../generated/restaking';
import { RestakingRewardAccountContext } from './reward';

export class RestakingRewardReserveAccountContext extends BaseAccountContext<RestakingRewardAccountContext> {
  constructor(readonly parent: RestakingRewardAccountContext) {
    super(parent, async (parent) => {
      const ix = await restaking.getFundManagerSettleRewardInstructionAsync(
        {
          receiptTokenMint: (await parent.parent.resolveAddress())!,
          rewardTokenMintArg: this.program.address,
          amount: 0n,
        } as any,
        {
          programAddress: this.program.address,
        }
      );
      return ix.accounts[3].address;
    });
  }

  readonly rewardTokens = new IterativeAccountContext(
    this,
    async (parent) => {
      const [self, reward] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.resolve(true),
      ]);
      if (!(self && reward)) return null;
      return Promise.all(
        reward.rewards.map((item) => {
          return TokenAccountContext.findAssociatedTokenAccountAddress({
            owner: self,
            mint: item.mint,
            tokenProgram: item.program,
          });
        })
      );
    },
    async (parent, address) => {
      return new TokenAccountContext(parent, address);
    }
  );
}
