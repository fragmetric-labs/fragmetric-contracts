import { getSetComputeUnitLimitInstruction } from '@solana-program/compute-budget';
import * as token from '@solana-program/token';
import { Address, createNoopSigner } from '@solana/kit';
import {
  BaseAccountContext,
  IterativeAccountContext,
  TokenAccountContext,
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../context';
import * as restaking from '../../generated/restaking';
import { RestakingNormalizedTokenPoolAccountContext } from './normalized_token_pool';
import { RestakingSlasherWithdrawalAccountContext } from './normalized_token_pool_slasher_withdrawal';

export class RestakingSlasherAccountContext extends BaseAccountContext<RestakingNormalizedTokenPoolAccountContext> {
  public resolve(noCache = false) {
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
        const [
          normalizedTokenPool,
          slasher,
          slasherAddress,
          slasherNormalizedToken,
          slasherSupportedTokens,
          withdrawal,
        ] = await Promise.all([
          this.parent.resolveAccount(noCache),
          this.resolveAccount(noCache),
          this.resolveAddress(noCache),
          this.normalizedToken.resolveAccount(noCache),
          this.supportedTokens.resolveAccountTree(noCache),
          this.withdrawal.resolve(noCache),
        ]);
        if (!(normalizedTokenPool && slasherAddress)) return null;

        const supportedTokens = normalizedTokenPool.data.supportedTokens.map(
          (item) => {
            return {
              mint: item.mint,
              program: item.program,
              decimals: item.decimals,
              amount:
                slasherSupportedTokens?.find((v) => v?.data.mint == item.mint)
                  ?.data.amount ?? 0n,
              claimableAmount:
                withdrawal?.claimableTokens.find(
                  (v) => !v.claimed && v.mint == item.mint
                )?.claimableAmount ?? 0n,
            };
          }
        );

        return {
          slasher: slasherAddress,
          lamports: slasher?.lamports ?? 0n,
          normalizedTokenAmount: slasherNormalizedToken?.data?.amount ?? 0n,
          claimable:
            withdrawal?.claimableTokens.some((v) => !v.claimed) || false,
          supportedTokens,
        };
      }
    );
  }

  readonly normalizedToken = TokenAccountContext.fromAssociatedTokenSeeds(
    this,
    async (parent) => {
      const [slasher, normalizedTokenMint] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.parent.normalizedTokenMint.resolveAddress(),
      ]);
      if (slasher && normalizedTokenMint) {
        return {
          owner: slasher,
          mint: normalizedTokenMint,
        };
      }
      return null;
    }
  );

  readonly supportedTokens = new IterativeAccountContext(
    this,
    async (parent) => {
      const [slasher, normalizedTokenPool] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.resolve(true),
      ]);
      if (slasher && normalizedTokenPool) {
        return (
          await Promise.all(
            normalizedTokenPool.supportedTokens.map((item) => {
              return TokenAccountContext.findAssociatedTokenAccountAddress({
                owner: slasher,
                mint: item.mint,
                tokenProgram: item.program,
              });
            })
          )
        ).filter((address) => !!address) as Address[];
      }
      return null;
    },
    async (parent, address) => {
      return new TokenAccountContext(parent, address);
    }
  );

  readonly withdrawal = new RestakingSlasherWithdrawalAccountContext(
    this,
    async (parent) => {
      const [slasher, normalizedTokenMint] = await Promise.all([
        parent.resolveAddress(),
        parent.parent.parent.normalizedTokenMint.resolveAddress(),
      ]);

      if (slasher && normalizedTokenMint) {
        const tmp =
          await restaking.getSlasherWithdrawNormalizedTokenInstructionAsync(
            {
              slasher: createNoopSigner(slasher as Address),
              normalizedTokenMint: normalizedTokenMint,
              // here pads invalid supported token info just to calculate the address of withdrawal ticket account
              supportedTokenMint: token.TOKEN_PROGRAM_ADDRESS,
              supportedTokenProgram: token.TOKEN_PROGRAM_ADDRESS,
              destinationSupportedTokenAccount: token.TOKEN_PROGRAM_ADDRESS,
              destinationRentLamportsAccount: slasher as Address,
              program: this.program.address,
            },
            {
              programAddress: this.program.address,
            }
          );
        return tmp.accounts[4].address;
      }
      return null;
    }
  );

  private __resolveAddressLookupTable = (parent: this) =>
    parent.parent.parent
      .resolve()
      .then((data) => data?.__lookupTableAddress ?? null);

  readonly initializeWithdrawal = new TransactionTemplateContext(this, null, {
    description:
      'create a withdrawal request to convert normalized tokens back into supported assets',
    addressLookupTables: [this.__resolveAddressLookupTable],
    instructions: [
      async (parent, args, overrides) => {
        const [data, feePayer, slasher] = await Promise.all([
          parent.parent.parent.resolve(true),
          transformAddressResolverVariant(
            overrides.feePayer ??
              this.runtime.options.transaction.feePayer ??
              (() => Promise.resolve(null))
          )(parent),
          this.resolveAddress(true),
        ]);
        if (!(data?.normalizedToken && feePayer && slasher))
          throw new Error('invalid context');

        const ix =
          await token.getCreateAssociatedTokenIdempotentInstructionAsync({
            payer: createNoopSigner(feePayer as Address),
            mint: data.normalizedToken.mint,
            owner: slasher,
          });
        const slasherNormalizedTokenAccount = ix.accounts[1].address;

        const ix2 =
          await restaking.getSlasherInitializeNormalizedTokenWithdrawalAccountInstructionAsync(
            {
              payer: createNoopSigner(feePayer as Address),
              slasher: createNoopSigner(slasher as Address),
              normalizedTokenMint: data.normalizedToken.mint,
              slasherNormalizedTokenAccount,
              program: this.program.address,
            },
            {
              programAddress: this.program.address,
            }
          );

        for (const accountMeta of data.__pricingSources) {
          ix2.accounts.push(accountMeta);
        }

        return [
          getSetComputeUnitLimitInstruction({ units: 1_400_000 }),
          ix,
          ix2,
        ];
      },
    ],
  });

  readonly withdraw = new TransactionTemplateContext(
    this,
    null,
    {
      description:
        'withdraw supported assets repeatedly from a already created withdrawal ticket',
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [data, withdrawal, feePayer, slasher] = await Promise.all([
            parent.parent.parent.resolve(true),
            parent.withdrawal.resolve(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
            this.resolveAddress(true),
          ]);
          if (!(data?.normalizedToken && withdrawal && feePayer && slasher))
            throw new Error('invalid context');

          return [
            getSetComputeUnitLimitInstruction({ units: 1_400_000 }),
            ...(
              await Promise.all(
                withdrawal.claimableTokens
                  .filter((claimableToken) => !claimableToken.claimed)
                  .slice(0, 6)
                  .map(async (claimableToken) => {
                    const ix =
                      await token.getCreateAssociatedTokenIdempotentInstructionAsync(
                        {
                          payer: createNoopSigner(feePayer as Address),
                          mint: claimableToken.mint,
                          tokenProgram: claimableToken.program,
                          owner: slasher as Address,
                        }
                      );
                    return [
                      ix,
                      await restaking.getSlasherWithdrawNormalizedTokenInstructionAsync(
                        {
                          slasher: createNoopSigner(slasher),
                          normalizedTokenMint: data.normalizedToken!.mint,
                          supportedTokenMint: claimableToken.mint,
                          supportedTokenProgram: claimableToken.program,
                          destinationSupportedTokenAccount:
                            ix.accounts[1].address,
                          destinationRentLamportsAccount: slasher,
                          program: this.program.address,
                        },
                        {
                          programAddress: this.program.address,
                        }
                      ),
                    ];
                  })
              )
            ).flat(),
          ];
        },
      ],
    },
    async (parent, args, events) => {
      const withdrawal = await this.withdrawal.resolve(true);
      if (
        withdrawal?.claimableTokens.filter(
          (claimableToken) => !claimableToken.claimed
        ).length
      ) {
        return {
          args: {
            ...args,
          },
        };
      }
      return null;
    }
  );
}
