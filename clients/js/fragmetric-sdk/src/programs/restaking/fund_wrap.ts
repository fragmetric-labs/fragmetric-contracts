import { BaseAccountContext, TokenAccountContext } from '../../context';
import { RestakingFundAccountContext } from './fund';
import { RestakingFundWrapAccountRewardAccountContext } from './fund_wrap_reward';

export class RestakingFundWrapAccountContext extends BaseAccountContext<RestakingFundAccountContext> {
  readonly reward = new RestakingFundWrapAccountRewardAccountContext(this);

  readonly receiptToken = TokenAccountContext.fromAssociatedTokenSeeds2022(
    this,
    async (parent) => {
      const [owner, receiptTokenMint] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.parent.resolveAddress(),
      ]);
      if (owner && receiptTokenMint) {
        return {
          owner: owner,
          mint: receiptTokenMint,
        };
      }
      return null;
    }
  );
}
