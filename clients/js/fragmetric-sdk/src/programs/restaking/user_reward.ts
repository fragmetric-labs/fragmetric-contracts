import * as computeBudget from '@solana-program/compute-budget';
import * as system from '@solana-program/system';
import * as token from '@solana-program/token';
import * as token2022 from '@solana-program/token-2022';
import {
  Account,
  Address,
  createNoopSigner,
  EncodedAccount,
} from '@solana/kit';
import * as v from 'valibot';
import {
  AccountContext,
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../context';
import * as restaking from '../../generated/restaking';
import { getRestakingAnchorEventDecoders } from './events';
import {
  RestakingFundWrapAccountContext,
  RestakingFundWrappedTokenHolderContext,
} from './fund_wrap';
import { RestakingProgram } from './program';
import { RestakingRewardAccountContext } from './reward';
import { RestakingUserAccountContext } from './user';

abstract class RestakingAbstractUserRewardAccountContext<
  P extends AccountContext<any, Account<any>>,
> extends AccountContext<P, Account<restaking.UserRewardAccount>> {
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
      padding,
      baseUserRewardPool,
      bonusUserRewardPool,
      reserved,
      reserved2,
      reserved3,
      delegate,
      ...props
    } = account.data;
    const pools = [baseUserRewardPool, bonusUserRewardPool].map((pool) => {
      const {
        tokenAllocatedAmount,
        contribution,
        updatedSlot,
        rewardPoolId,
        numRewardSettlements,
        reserved,
        rewardSettlements1,
        ...props
      } = pool;
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
      delegate: delegate == system.SYSTEM_PROGRAM_ADDRESS ? null : delegate,
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

  readonly updatePools = new TransactionTemplateContext(this, null, {
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
          computeBudget.getSetComputeUnitLimitInstruction({ units: 1_400_000 }),
          restaking.getUserUpdateRewardPoolsInstructionAsync(
            {
              user: user,
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
  });

  readonly initializeOrUpdateAccount = new TransactionTemplateContext(
    this,
    null,
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
            restaking.getUserCreateRewardAccountIdempotentInstructionAsync(
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

  readonly claim = new TransactionTemplateContext(
    this,
    v.object({
      delegate: v.pipe(
        v.nullish(v.string(), null),
        v.description('set delegate signer if eligible')
      ),
      isBonus: v.pipe(
        v.nullish(v.boolean(), false),
        v.description(
          'whether the reward is from bonus reward pool, default is false'
        )
      ),
      mint: v.string(),
      amount: v.union(
        [v.bigint(), v.null()],
        'set null to claim all the possible amount'
      ),
      recipient: v.union(
        [v.string(), v.null()],
        'set recipient address (owner of ATA), set null to send to reward account owner'
      ),
      applyPresetComputeUnitLimit: v.pipe(
        v.nullish(v.boolean(), true),
        v.description('apply preset CU limit')
      ),
    }),
    {
      description: 'claim rewards',
      anchorEventDecoders: getRestakingAnchorEventDecoders('userClaimedReward'),
      instructions: [
        async (parent, args, overrides) => {
          let [receiptTokenMint, reward, userAddress, user, userReward, payer] =
            await Promise.all([
              this.__globalRewardAccount.parent.resolveAddress(),
              this.__globalRewardAccount.resolve(),
              parent.parent.resolveAddress(true),
              parent.parent.resolveAccount(true),
              parent.resolveAccount(true),
              transformAddressResolverVariant(
                overrides.feePayer ??
                  this.runtime.options.transaction.feePayer ??
                  (() => Promise.resolve(null))
              )(parent),
            ]);
          userAddress =
            userAddress ?? (user?.address as Address | null) ?? null;
          if (!(receiptTokenMint && reward && userAddress && userReward))
            throw new Error('invalid context');

          const targetReward = reward.rewards.find(
            (item) => item.mint == args.mint
          );
          if (!targetReward)
            throw new Error('invalid context: reward not found');
          if (!targetReward.claimable)
            throw new Error('invalid context: non-claimable reward');

          // Determine the correct signer—either the user or the current delegate if not given explicitly.
          // If the user is a PDA (e.g., a token account for a fund wrapper or its holders), treat the current delegate as the required authority.
          // If no delegate is set, fall back to the user as the authority.
          const claimAuthority = (args.delegate ??
            (user && user.programAddress == system.SYSTEM_PROGRAM_ADDRESS
              ? userAddress
              : userReward.data.delegate == system.SYSTEM_PROGRAM_ADDRESS
                ? userAddress
                : userReward.data.delegate)) as Address;

          // prevent possible human fault when the recipient is not given while the user account is PDA.
          if (
            !args.recipient &&
            user?.programAddress != system.SYSTEM_PROGRAM_ADDRESS
          )
            throw new Error(
              'invalid context: set recipient address explicitly to send reward to ATA of PDA'
            );

          const recipient = (args.recipient ?? userAddress) as Address;
          const ix =
            await token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(payer! as Address),
              mint: targetReward.mint,
              owner: recipient,
              tokenProgram: targetReward.program,
            });
          const recipientRewardTokenAccount = ix.accounts[1].address;

          return Promise.all([
            args.applyPresetComputeUnitLimit
              ? computeBudget.getSetComputeUnitLimitInstruction({
                  units: 1_400_000,
                })
              : null,
            ix,
            restaking.getUserClaimRewardInstructionAsync(
              {
                claimAuthority: createNoopSigner(claimAuthority),
                user: userAddress as Address,
                receiptTokenMint: receiptTokenMint,
                rewardTokenMint: targetReward.mint,
                rewardTokenProgram: targetReward.program,
                destinationRewardTokenAccount: recipientRewardTokenAccount,
                isBonusPool: args.isBonus,
                amount: args.amount,
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

  readonly delegate = new TransactionTemplateContext(
    this,
    v.object({
      delegate: v.nullish(v.string(), null),
      newDelegate: v.string(),
    }),
    {
      description: 'delegate user reward account',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'userDelegatedRewardAccount'
      ),
      instructions: [
        async (parent, args) => {
          let [receiptTokenMint, userAddress, user, userReward] =
            await Promise.all([
              this.__globalRewardAccount.parent.resolveAddress(),
              parent.parent.resolveAddress(true),
              parent.parent.resolveAccount(true),
              parent.resolveAccount(true),
            ]);
          userAddress =
            userAddress ?? (user?.address as Address | null) ?? null;
          if (!(receiptTokenMint && userAddress && userReward))
            throw new Error('invalid context');

          // Determine the correct signer—either the user or the current delegate if not given explicitly.
          // If the user is a PDA (e.g., a token account for a fund wrapper or its holders), treat the current delegate as the required authority.
          // If no delegate is set, fall back to the user as the authority.
          const delegateAuthority = (args.delegate ??
            (user && user.programAddress == system.SYSTEM_PROGRAM_ADDRESS
              ? userAddress
              : userReward.data.delegate == system.SYSTEM_PROGRAM_ADDRESS
                ? userAddress
                : userReward.data.delegate)) as Address;
          const newDelegate =
            args.newDelegate == system.SYSTEM_PROGRAM_ADDRESS
              ? null
              : (args.newDelegate as Address);
          if (
            user?.programAddress != system.SYSTEM_PROGRAM_ADDRESS &&
            !newDelegate
          ) {
            throw new Error(
              'invalid context: irreversible delegation is not allowed for PDA user'
            );
          }

          return Promise.all([
            restaking.getUserDelegateRewardAccountInstructionAsync(
              {
                delegateAuthority: createNoopSigner(delegateAuthority),
                user: userAddress,
                receiptTokenMint: receiptTokenMint,
                delegate: newDelegate,
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

  readonly resetDelegate = new TransactionTemplateContext(this, null, {
    description:
      'reset delegate of reward account (in case of either fund wrap or wrapped token holder, delegate will be reset to fund manager)',
    anchorEventDecoders: getRestakingAnchorEventDecoders(
      'userDelegatedRewardAccount'
    ),
    instructions: [
      async (parent, args) => {
        let [receiptTokenMint, wrappedTokenMint, fundWrap, userAddress, user] =
          await Promise.all([
            this.__globalRewardAccount.parent.resolveAddress(),
            this.__globalRewardAccount.parent.wrappedTokenMint.resolveAddress(),
            this.__globalRewardAccount.parent.fund.wrap.resolveAddress(),
            this.parent.resolveAddress(true),
            this.parent.resolveAccount(true),
          ]);
        userAddress = userAddress ?? (user?.address as Address | null) ?? null;
        if (!(receiptTokenMint && wrappedTokenMint && fundWrap && userAddress))
          throw new Error('invalid context');
        return Promise.all([
          userAddress == fundWrap
            ? restaking.getFundManagerResetFundWrapAccountRewardAccountDelegateInstructionAsync(
                {
                  fundManager: createNoopSigner(
                    (this.program as RestakingProgram).knownAddresses
                      .fundManager
                  ),
                  receiptTokenMint: receiptTokenMint,
                },
                {
                  programAddress: this.program.address,
                }
              )
            : user && user.programAddress == system.SYSTEM_PROGRAM_ADDRESS
              ? restaking.getUserDelegateRewardAccountInstructionAsync(
                  {
                    delegateAuthority: createNoopSigner(userAddress),
                    user: userAddress,
                    receiptTokenMint: receiptTokenMint,
                    delegate: null,
                    program: this.program.address,
                  },
                  {
                    programAddress: this.program.address,
                  }
                )
              : restaking.getFundManagerResetWrappedTokenHolderRewardAccountDelegateInstructionAsync(
                  {
                    fundManager: createNoopSigner(
                      (this.program as RestakingProgram).knownAddresses
                        .fundManager
                    ),
                    receiptTokenMint: receiptTokenMint,
                    wrappedTokenMint: wrappedTokenMint,
                    wrappedTokenHolder: userAddress,
                    program: this.program.address,
                  },
                  {
                    programAddress: this.program.address,
                  }
                ),
        ]);
      },
    ],
  });
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

export class RestakingFundWrappedTokenHolderRewardAccountContext extends RestakingAbstractUserRewardAccountContext<RestakingFundWrappedTokenHolderContext> {
  protected get __globalRewardAccount() {
    return this.parent.parent.parent.parent.reward;
  }
}
