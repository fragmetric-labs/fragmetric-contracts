import { BaseAccountContext, IterativeAccountContext, TokenAccountContext } from '../../context';
import { RestakingFundAccountContext } from './fund';
import { RestakingFundWrappedTokenHolderRewardAccountContext, RestakingFundWrapRewardAccountContext } from './user_reward';

export class RestakingFundWrapAccountContext extends BaseAccountContext<RestakingFundAccountContext> {
  readonly reward = new RestakingFundWrapRewardAccountContext(this);

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

  readonly holders = new IterativeAccountContext<
    RestakingFundWrapAccountContext,
    RestakingFundWrappedTokenHolderContext
  >(
    this,
    async (parent) => {
      const fund = await parent.parent.resolveAccount(true);
      if (!fund) return null;
      const wrappedToken = fund.data.wrappedToken;
      const holders = wrappedToken.holders.slice(0, wrappedToken.numHolders);
      const addresses = holders.map(holder => holder.tokenAccount);
      return addresses;
    },
    async (parent, address) => {
      return new RestakingFundWrappedTokenHolderContext(parent, address);
    },
  )
}

export class RestakingFundWrappedTokenHolderContext extends TokenAccountContext<RestakingFundWrapAccountContext> {
  readonly reward = new RestakingFundWrappedTokenHolderRewardAccountContext(this);
}
