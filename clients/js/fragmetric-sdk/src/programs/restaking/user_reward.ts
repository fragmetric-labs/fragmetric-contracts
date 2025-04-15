import * as token from '@solana-program/token';
import * as token2022 from '@solana-program/token-2022';
import { Account, createNoopSigner, EncodedAccount } from '@solana/kit';
import * as v from 'valibot';
import { AccountContext, TransactionTemplateContext } from '../../context';
import * as restaking from '../../generated/restaking';
import { getRestakingAnchorEventDecoders } from './events';
import { RestakingUserAccountContext } from './user';
import { RestakingRewardAccountContext } from './reward';
import { RestakingFundWrapAccountContext } from './fund_wrap';

abstract class RestakingAbstractUserRewardAccountContext<P extends AccountContext<any>> extends AccountContext<
  P,
  Account<restaking.UserRewardAccount>
> {
  protected abstract __globalRewardAccount: RestakingRewardAccountContext;
  
  async resolve(noCache = false) {
    const [account, global] = await Promise.all([
      this.resolveAccount(noCache),
      this.__globalRewardAccount.resolve(noCache),
    ]);
    if (!(account && global)) {
      return null;
    }
    const {
      discriminator,
      dataVersion,
      bump,
      receiptTokenMint,
      user,
      numUserRewardPools,
      maxUserRewardPools,
      padding,
      userRewardPools1,
      ...props
    } = account.data;
    const pools = userRewardPools1.slice(0, numUserRewardPools).map((req) => {
      const {
        tokenAllocatedAmount,
        contribution,
        updatedSlot,
        rewardPoolId,
        numRewardSettlements,
        padding,
        reserved,
        rewardSettlements1,
        ...props
      } = req;
      return {
        // rewardPoolId,
        contribution,
        updatedSlot,
        tokenAllocatedAmount: {
          totalAmount: tokenAllocatedAmount.totalAmount,
          records: tokenAllocatedAmount.records
            .slice(0, tokenAllocatedAmount.numRecords)
            .map((record) => {
              const { amount, contributionAccrualRate } = record;
              return {
                amount,
                contributionAccrualRate: contributionAccrualRate / 100,
              };
            }),
        },
        settlements: rewardSettlements1
          .slice(0, numRewardSettlements)
          .map((settlement) => {
            const {
              rewardId,
              padding,
              totalClaimedAmount,
              totalSettledAmount,
              totalSettledContribution,
              lastSettledSlot,
              ...props
            } = settlement;
            return {
              reward: global.rewards.find((reward) => reward.id === rewardId)!,
              settledSlot: lastSettledSlot,
              settledContribution: totalSettledContribution,
              settledAmount: totalSettledAmount,
              claimedAmount: totalClaimedAmount,
              ...props,
            };
          }),
        ...props,
      };
    });
    return {
      user,
      // dataVersion,
      receiptTokenMint,
      ...props,
      basePool: pools[0],
      bonusPool: pools[1],
    };
  }

  constructor(readonly parent: P) {
    super(parent, async (parent) => {
      const [user, receiptTokenMint] = await Promise.all([
        parent.resolveAddress(true),
        this.__globalRewardAccount.parent.resolveAddress(),
      ]);
      if (user && receiptTokenMint) {
        const ix =
          await restaking.getUserCreateRewardAccountIdempotentInstructionAsync(
            { user: { address: user }, receiptTokenMint } as any,
            { programAddress: parent.program.address }
          );
        return ix.accounts[6].address;
      }
      return null;
    });
  }

  protected __decodeAccount(account: EncodedAccount) {
    return restaking.decodeUserRewardAccount(account);
  }

  readonly updatePools = new TransactionTemplateContext(
    this,
    v.pipe(v.nullish(v.null(), null), v.description('no args required')),
    {
      description:
        'manually triggers contribution synchronization for the user reward pools',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'userUpdatedRewardPool'
      ),
      instructions: [
        async (parent, args) => {
          const [receiptTokenMint, user] = await Promise.all([
            this.__globalRewardAccount.parent.resolveAddress(),
            parent.parent.resolveAddress(true),
          ]);
          if (!(receiptTokenMint && user)) throw new Error('invalid context');

          return Promise.all([
            restaking.getUserUpdateRewardPoolsInstructionAsync(
              {
                user: createNoopSigner(user),
                receiptTokenMint: receiptTokenMint,
                program: this.program.address,
              },
              {
                programAddress: this.program.address,
              }
            ),
          ]);
        },
      ],
    }
  );

  readonly initializeOrUpdateAccount = new TransactionTemplateContext(
    this,
    v.pipe(v.nullish(v.null(), null), v.description('no args required')),
    {
      description: 'initialize or update user reward account',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'userCreatedOrUpdatedRewardAccount'
      ),
      instructions: [
        async (parent, args) => {
          const [receiptTokenMint, user] = await Promise.all([
            this.__globalRewardAccount.parent.resolveAddress(),
            parent.parent.resolveAddress(true),
          ]);
          if (!(receiptTokenMint && user)) throw new Error('invalid context');

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(user),
              mint: receiptTokenMint,
              owner: user,
              tokenProgram: token2022.TOKEN_2022_PROGRAM_ADDRESS,
            }),
            restaking.getUserCreateFundAccountIdempotentInstructionAsync(
              {
                user: createNoopSigner(user),
                program: this.program.address,
                receiptTokenMint: receiptTokenMint,
                desiredAccountSize: null,
              },
              {
                programAddress: this.program.address,
              }
            ),
          ]);
        },
      ],
    }
  );
}

export class RestakingUserRewardAccountContext extends RestakingAbstractUserRewardAccountContext<RestakingUserAccountContext> {
  protected get __globalRewardAccount() {
    return this.parent.parent.reward;
  }
}

export class RestakingFundWrapRewardAccountContext extends RestakingAbstractUserRewardAccountContext<RestakingFundWrapAccountContext> {
  protected get __globalRewardAccount() {
    return this.parent.parent.parent.reward;
  }
}