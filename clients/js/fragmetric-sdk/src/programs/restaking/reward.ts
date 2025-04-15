import { getSetComputeUnitLimitInstruction } from '@solana-program/compute-budget';
import * as token from '@solana-program/token';
import {
  Account,
  Address,
  createNoopSigner,
  EncodedAccount,
  getUtf8Decoder,
} from '@solana/kit';
import * as v from 'valibot';
import {
  AccountContext,
  TokenAccountContext,
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../context';
import * as restaking from '../../generated/restaking';
import { getRestakingAnchorEventDecoders } from './events';
import { RestakingProgram } from './program';
import { RestakingReceiptTokenMintAccountContext } from './receipt_token_mint';

export class RestakingRewardAccountContext extends AccountContext<
  RestakingReceiptTokenMintAccountContext,
  Account<restaking.RewardAccount>
> {
  async resolve(noCache = false) {
    const account = await this.resolveAccount(noCache);
    if (!account) {
      return null;
    }
    const {
      discriminator,
      dataVersion,
      bump,
      reserveAccountBump,
      maxRewards,
      padding1,
      numRewards,
      padding2,
      reserveAccount,
      reserved,
      rewards1,
      baseRewardPool,
      bonusRewardPool,
      padding3,
      reserved2,
      ...props
    } = account.data;
    const rewards = rewards1.slice(0, numRewards).map((reward) => {
      const {
        id, name, description, reserved, claimable, ...props
      } = reward;
      return {
        id,
        name: getUtf8Decoder().decode(name),
        description: getUtf8Decoder().decode(description),
        claimable: claimable == 1,
        ...props,
      };
    });

    const pools = [baseRewardPool, bonusRewardPool].map((rewardPool) => {
      const {
        id,
        customContributionAccrualRateEnabled,
        tokenAllocatedAmount,
        padding,
        numRewardSettlements,
        reserved,
        rewardSettlements1,
        padding2,
        ...props
      } = rewardPool;

      return {
        // id,
        customContributionAccrualRateEnabled:
          customContributionAccrualRateEnabled == 1,
        ...props,
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
              rewardPoolId,
              numSettlementBlocks,
              settlementBlocksHead,
              settlementBlocksTail,
              settlementBlocks,
              padding2,
              ...props
            } = settlement;
            // [ head ... tail ] or [ tail ... head ]
            const blocks =
              settlementBlocksHead <= settlementBlocksTail
                ? settlementBlocks.slice(
                    settlementBlocksHead,
                    settlementBlocksTail
                  )
                : settlementBlocks
                    .slice(settlementBlocksTail, settlementBlocks.length)
                    .concat(settlementBlocks.slice(0, settlementBlocksHead));

            return {
              ...props,
              reward: rewards.find((reward) => reward.id === rewardId)!,
              blocks: blocks.map((block) => {
                const {
                  amount,
                  startingSlot,
                  startingRewardPoolContribution,
                  endingRewardPoolContribution,
                  endingSlot,
                  userSettledAmount,
                  userSettledContribution,
                  ...props
                } = block;

                return {
                  amount,
                  settledSlots: endingSlot - startingSlot,
                  startingSlot,
                  endingSlot,
                  settledContribution:
                    endingRewardPoolContribution -
                    startingRewardPoolContribution,
                  startingContribution: startingRewardPoolContribution,
                  endingContribution: endingRewardPoolContribution,
                  userSettledAmount,
                  userSettledContribution,
                  ...props,
                };
              }),
            };
          }),
      };
    });

    return {
      ...props,
      rewards,
      basePool: pools[0]!,
      bonusPool: pools[1]!,
    };
  }

  constructor(readonly parent: RestakingReceiptTokenMintAccountContext) {
    super(parent, async (parent) => {
      const receiptTokenMint = await parent.resolveAddress();
      if (receiptTokenMint) {
        const ix =
          await restaking.getAdminInitializeRewardAccountInstructionAsync(
            {
              receiptTokenMint,
            } as any,
            { programAddress: this.program.address }
          );
        return ix.accounts![4].address;
      }
      return null;
    });
  }
  protected __decodeAccount(account: EncodedAccount) {
    return restaking.decodeRewardAccount(account);
  }

  /** operator transactions **/
  readonly updatePools = new TransactionTemplateContext(
    this,
    v.pipe(v.nullish(v.null(), null), v.description('no args required')),
    {
      description:
        'manually triggers contribution synchronization for the global reward pools',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'operatorUpdatedRewardPools'
      ),
      instructions: [
        async (parent, args, overrides) => {
          const [data, operator] = await Promise.all([
            parent.parent.resolve(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!(data && operator)) throw new Error('invalid context');

          const ix =
            await restaking.getOperatorUpdateRewardPoolsInstructionAsync(
              {
                operator: createNoopSigner(operator as Address),
                program: this.program.address,
                receiptTokenMint: data.receiptTokenMint,
              },
              {
                programAddress: this.program.address,
              }
            );

          for (const accountMeta of data.__pricingSources) {
            ix.accounts.push(accountMeta);
          }

          return [ix];
        },
      ],
    }
  );

  /** authorized transactions **/
  readonly initializeOrUpdateAccount = new TransactionTemplateContext(
    this,
    v.object({
      targetVersion: v.number(),
    }),
    {
      description: 'initialize or update reward account',
      instructions: [
        getSetComputeUnitLimitInstruction({ units: 1_400_000 }),
        async (parent, args, overrides) => {
          const [receiptTokenMint, currentVersion, payer] = await Promise.all([
            parent.parent.resolveAddress(),
            parent
              .resolveAccount(true)
              .then((reward) => reward?.data.dataVersion ?? 0)
              .catch((err) => 35),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!receiptTokenMint) throw new Error('invalid context');
          const admin = (this.program as RestakingProgram).knownAddresses.admin;

          return Promise.all([
            ...(currentVersion == 0
              ? [
                  restaking.getAdminInitializeRewardAccountInstructionAsync(
                    {
                      payer: createNoopSigner(payer! as Address),
                      admin: createNoopSigner(admin),
                      receiptTokenMint,
                      program: this.program.address,
                    },
                    {
                      programAddress: this.program.address,
                    }
                  ),
                ]
              : []),
            ...Array(Math.min(args.targetVersion - currentVersion, 35))
              .fill(null)
              .map(() => {
                return restaking.getAdminUpdateRewardAccountIfNeededInstructionAsync(
                  {
                    payer: createNoopSigner(payer! as Address),
                    admin: createNoopSigner(admin),
                    receiptTokenMint,
                    program: this.program.address,
                    desiredAccountSize: null,
                  },
                  {
                    programAddress: this.program.address,
                  }
                );
              }),
          ]);
        },
      ],
    },
    async (parent, args, events) => {
      const currentVersion = await parent
        .resolveAccount(true)
        .then((reward) => reward?.data.dataVersion ?? 0)
        .catch((err) => 35);

      if (currentVersion < args.targetVersion) {
        return {
          args,
        } as any;
      }
      return null;
    }
  );

  readonly addReward = new TransactionTemplateContext(
    this,
    v.object({
      mint: v.string(),
      program: v.nullish(v.string(), token.TOKEN_PROGRAM_ADDRESS),
      decimals: v.number(),
      name: v.string(),
      description: v.string(),
    }),
    {
      description: 'register a new reward (non-claimable yet)',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedRewardPool'
      ),
      instructions: [
        async (parent, args, overrides) => {
          const [receiptTokenMint, payer] = await Promise.all([
            parent.parent.resolveAddress(),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!receiptTokenMint) throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          return Promise.all([
            restaking.getFundManagerAddRewardInstructionAsync(
              {
                fundManager: createNoopSigner(fundManager),
                name: args.name,
                description: args.description,
                mint: args.mint as Address,
                programArg: args.program as Address,
                decimals: args.decimals,
                claimable: false,
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

  readonly updateReward = new TransactionTemplateContext(
    this,
    v.object({
      mint: v.string(),
      newMint: v.nullish(v.string(), null),
      newProgram: v.nullish(v.string(), null),
      newDecimals: v.nullish(v.number(), null),
      claimable: v.boolean(),
    }),
    {
      description: 'update a non-claimable reward',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedRewardPool'
      ),
      instructions: [
        async (parent, args, overrides) => {
          const [receiptTokenMint, rewardAccount, payer] = await Promise.all([
            parent.parent.resolveAddress(),
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          const reward = rewardAccount?.data.rewards1
            .slice(0, rewardAccount.data.numRewards)
            .find((r) => r.mint == args.mint);
          if (!(receiptTokenMint && rewardAccount && reward))
            throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;
          const rewardMint = (args.newMint ?? reward.mint) as Address;
          const rewardProgram = (args.newProgram ?? reward.program) as Address;

          return Promise.all([
            args.claimable
              ? token.getCreateAssociatedTokenIdempotentInstructionAsync({
                  payer: createNoopSigner(payer! as Address),
                  mint: rewardMint,
                  owner: rewardAccount.data.reserveAccount,
                  tokenProgram: rewardProgram,
                })
              : null,
            restaking.getFundManagerUpdateRewardInstructionAsync(
              {
                rewardTokenMint: args.claimable ? rewardMint : undefined,
                rewardTokenProgram: args.claimable ? rewardProgram : undefined,
                rewardTokenReserveAccount: args.claimable
                  ? await TokenAccountContext.findAssociatedTokenAccountAddress(
                      {
                        owner: rewardAccount.data.reserveAccount,
                        mint: rewardMint,
                        tokenProgram: rewardProgram,
                      }
                    )
                  : undefined,
                mint: args.mint as Address,
                newMint: args.newMint as Address,
                newProgram: args.newProgram as Address,
                newDecimals: args.newDecimals,
                claimable: args.claimable,
                fundManager: createNoopSigner(fundManager),
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

  readonly settleReward = new TransactionTemplateContext(
    this,
    v.object({
      isBonus: v.pipe(
        v.nullish(v.boolean(), false),
        v.description('bonus is airdrop rewards from Fragmetric')
      ),
      mint: v.string(),
      amount: v.bigint(),
    }),
    {
      description: 'settle a reward',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedRewardPool'
      ),
      instructions: [
        async (parent, args, overrides) => {
          const [receiptTokenMint, rewardAccount, payer] = await Promise.all([
            parent.parent.resolveAddress(),
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          const reward = rewardAccount?.data.rewards1
            .slice(0, rewardAccount.data.numRewards)
            .find((r) => r.mint == args.mint);
          if (!(receiptTokenMint && rewardAccount && reward))
            throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          return Promise.all([
            restaking.getFundManagerSettleRewardInstructionAsync(
              {
                isBonusPool: args.isBonus,
                rewardTokenMintArg: args.mint as Address,
                amount: args.amount,
                rewardTokenMint: reward.claimable ? reward.mint : undefined,
                rewardTokenProgram: reward.claimable
                  ? reward.program
                  : undefined,
                rewardTokenReserveAccount: reward.claimable
                  ? await TokenAccountContext.findAssociatedTokenAccountAddress(
                      {
                        owner: rewardAccount.data.reserveAccount,
                        mint: reward.mint,
                        tokenProgram: reward.program,
                      }
                    )
                  : undefined,
                fundManager: createNoopSigner(fundManager),
                program: this.program.address,
                receiptTokenMint: receiptTokenMint,
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
