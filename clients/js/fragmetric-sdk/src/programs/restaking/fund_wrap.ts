import {
  BaseAccountContext,
  IterativeAccountContext,
  TokenAccountContext,
} from '../../context';
import { RestakingFundAccountContext } from './fund';
import {
  RestakingFundWrappedTokenHolderRewardAccountContext,
  RestakingFundWrapRewardAccountContext,
} from './user_reward';

export class RestakingFundWrapAccountContext extends BaseAccountContext<RestakingFundAccountContext> {
  async resolve(noCache = false) {
    return this.__deduplicated(
      {
        method: 'resolve',
        params: [noCache],
        alternativeParams: noCache ? null : [true],
        intervalSeconds: noCache
          ? 0
          : this.__maybeRuntimeOptions?.rpc.accountDeduplicationIntervalSeconds,
      },
      async () => {
        const fund = await this.parent.resolveAccount(noCache);
        if (!fund) return null;

        const wrappedToken = fund.data.wrappedToken;
        if (wrappedToken.enabled != 1) return null;

        return {
          wrappedToken: {
            mint: wrappedToken.mint,
            program: wrappedToken.program,
            decimals: wrappedToken.decimals,
          },
          wrappedAmount: wrappedToken.supply,
          retainedAmount: wrappedToken.retainedAmount,
          holders: wrappedToken.holders.slice(0, wrappedToken.numHolders),
        };
      }
    );
  }

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
      const addresses = holders.map((holder) => holder.tokenAccount);
      return addresses;
    },
    async (parent, address) => {
      return new RestakingFundWrappedTokenHolderContext(parent, address);
    }
  );
}

export class RestakingFundWrappedTokenHolderContext extends TokenAccountContext<RestakingFundWrapAccountContext> {
  readonly reward = new RestakingFundWrappedTokenHolderRewardAccountContext(
    this
  );
}
