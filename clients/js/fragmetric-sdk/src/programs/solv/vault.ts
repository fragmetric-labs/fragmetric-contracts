import * as system from '@solana-program/system';
import * as token from '@solana-program/token';
import {
  Account,
  Address,
  createNoopSigner,
  EncodedAccount,
} from '@solana/kit';
import * as v from 'valibot';
import {
  AccountContext,
  IterativeAccountContext,
  TokenAccountContext,
  TokenMintAccountContext,
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../context';
import * as solv from '../../generated/solv';
import { SolvBTCVaultProgram } from './program';

export class SolvVaultAccountContext extends AccountContext<
  SolvBTCVaultProgram,
  Account<solv.VaultAccount>
> {
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
        const [
          vault,
          receiptTokenMint,
          supportedTokenMint,
          supportedToken,
          solvReceiptToken,
          rewardTokens,
        ] = await Promise.all([
          this.resolveAccount(noCache),
          this.receiptTokenMint.resolveAccount(noCache),
          this.supportedTokenMint.resolveAccount(noCache),
          this.supportedToken.resolveAccount(noCache),
          this.solvReceiptToken.resolveAccount(noCache),
          this.rewardTokens.resolveAccount(noCache),
        ]);
        if (
          !(
            vault &&
            receiptTokenMint &&
            supportedTokenMint &&
            supportedToken &&
            solvReceiptToken
          )
        ) {
          return null;
        }

        const withdrawalRequests = vault.data.withdrawalRequests
          .slice(0, vault.data.numWithdrawalRequests)
          .map(r => {
            return {
              id: r.requestId,
              receiptTokenEnqueuedAmount: r.vrtWithdrawalRequestedAmount,
              supportedTokenTotalEstimatedAmount: r.vstWithdrawalTotalEstimatedAmount,
              supportedTokenLockedAmount: r.vstWithdrawalLockedAmount,
              solvReceiptTokenLockedAmount: r.srtWithdrawalLockedAmount,
              state: r.state,
            };
          });

        return {
          admin: {
            vaultManager: vault.data.vaultManager,
            rewardManager: vault.data.rewardManager,
            fundManager: vault.data.fundManager,
            solvManager: vault.data.solvManager,
          },

          receiptTokenMint: receiptTokenMint.address,
          receiptTokenSupply: receiptTokenMint.data.supply,
          receiptTokenProgram: receiptTokenMint.programAddress,
          receiptTokenDecimals: receiptTokenMint.data.decimals,
          oneReceiptTokenAsSupportedTokenAmount: vault.data.oneVrtAsMicroVst / 1_000_000n,

          supportedTokenMint: supportedTokenMint.address,
          supportedTokenProgram: supportedTokenMint.programAddress,
          supportedTokenDecimals: supportedTokenMint.data.decimals,
          supportedTokenAmount: supportedToken.data.amount,
          supportedTokenOperationReservedAmount: vault.data.vstOperationReservedAmount,

          solvProtocolWallet: vault.data.solvProtocolWallet,
          solvProtocolWithdrawalFeeRate: vault.data.solvProtocolWithdrawalFeeRateBps / 10000,
          solvReceiptTokenMint: vault.data.solvReceiptTokenMint,
          solvReceiptTokenDecimals: vault.data.solvReceiptTokenDecimals,
          solvReceiptTokenAmount: solvReceiptToken.data.amount,
          solvReceiptTokenOperationReservedAmount: vault.data.srtOperationReservedAmount,
          solvReceiptTokenOperationReceivableAmount: vault.data.srtOperationReceivableAmount,
          oneSolvReceiptTokenAsSupportedTokenAmount: vault.data.oneSrtAsMicroVst / 1_000_000n,

          withdrawal: {
            enqueued: {
              receiptTokenEnqueuedAmount: vault.data.vrtWithdrawalEnqueuedAmount,
              supportedTokenLockedAmount: vault.data.vstWithdrawalLockedAmount,
              solvReceiptTokenLockedAmount: vault.data.srtWithdrawalLockedAmount,
              requests: withdrawalRequests.filter(req => req.state == 0).map(({ state , ...req}) => req),
            },
            processing: {
              receiptTokenProcessingAmount: vault.data.vrtWithdrawalProcessingAmount,
              supportedTokenReceivableAmount: vault.data.vstReceivableAmountToClaim,
              requests: withdrawalRequests.filter(req => req.state == 1).map(({ state , ...req}) => req),
            },
            completed: {
              receiptTokenProcessedAmount: vault.data.vrtWithdrawalCompletedAmount,
              supportedTokenTotalClaimableAmount: vault.data.vstReservedAmountToClaim + vault.data.vstExtraAmountToClaim,
              supportedTokenExtraClaimableAmount: vault.data.vstExtraAmountToClaim,
              supportedTokenDeductedFeeAmount: vault.data.vstDeductedFeeAmount,
              requests: withdrawalRequests.filter(req => req.state == 2).map(({ state , ...req}) => req),
            },
          },

          delegatedRewardTokens: (rewardTokens ?? [])
            .filter((token) => !!token)
            .map((token) => {
              return {
                mint: token.data.mint,
                amount: token.data.amount,
                delegate: token.data.delegate,
              };
            }),
        };
      }
    );
  }

  protected __decodeAccount(account: EncodedAccount) {
    return solv.decodeVaultAccount(account);
  }

  static fromSeeds(
    parent: SolvBTCVaultProgram,
    seeds: {
      receiptTokenMint: string;
      supportedTokenMint: string;
    }
  ) {
    return new SolvVaultAccountContext(parent, async (parent) => {
      const ix = await solv.getVaultManagerInitializeVaultAccountInstructionAsync(
        {
          vaultReceiptTokenMint: seeds.receiptTokenMint as Address,
          vaultSupportedTokenMint: seeds.supportedTokenMint as Address,
          solvReceiptTokenMint: system.SYSTEM_PROGRAM_ADDRESS,
        } as any,
        { programAddress: parent.program.address }
      );
      return ix.accounts[2].address;
    });
  }

  readonly receiptTokenMint = new TokenMintAccountContext(
    this,
    async (parent) => {
      const vault = await parent.resolveAccount(true);
      if (!vault) return null;
      return vault.data.vaultReceiptTokenMint;
    }
  );

  readonly supportedTokenMint = new TokenMintAccountContext(
    this,
    async (parent) => {
      const vault = await parent.resolveAccount(true);
      if (!vault) return null;
      return vault.data.vaultSupportedTokenMint;
    }
  );

  readonly supportedToken = TokenAccountContext.fromAssociatedTokenSeeds(
    this,
    async (parent) => {
      const vault = await parent.resolveAccount(true);
      if (!vault) return null;
      return {
        owner: vault.address,
        mint: vault.data.vaultSupportedTokenMint,
      };
    }
  );

  readonly solvReceiptTokenMint = new TokenMintAccountContext(
    this,
    async (parent) => {
      const vault = await parent.resolveAccount(true);
      if (!vault) return null;
      return vault.data.solvReceiptTokenMint;
    }
  );

  readonly solvReceiptToken = TokenAccountContext.fromAssociatedTokenSeeds(
    this,
    async (parent) => {
      const vault = await parent.resolveAccount(true);
      if (!vault) return null;
      return {
        owner: vault.address,
        mint: vault.data.solvReceiptTokenMint,
      };
    }
  );

  readonly rewardTokens = new IterativeAccountContext(
    this,
    async (parent) => {
      const vault = await parent.resolveAccount(true);
      if (!vault) return null;
      return (await Promise.all(
        vault.data.delegatedRewardTokenMints
          .slice(0, vault.data.numDelegatedRewardTokenMints)
          .map((item) => {
            return TokenAccountContext.findAssociatedTokenAccountAddress({
              owner: vault.address,
              mint: item,
            });
          })
      )) as Address[];
    },
    async (parent, address) => {
      return new TokenAccountContext(parent, address);
    }
  );

  /** transactions **/
  readonly initialize = new TransactionTemplateContext(
    this,
    v.object({
      admin: v.string(),
      receiptTokenMint: v.string(),
      supportedTokenMint: v.string(),
    }),
    {
      description: 'initialize receipt token mint and vault',
      instructions: [
        async (parent, args, overrides) => {
          const [payer] = await Promise.all([
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!(args.receiptTokenMint && args.supportedTokenMint))
            throw new Error('invalid context');

          const ix = await solv.getVaultManagerInitializeVaultAccountInstructionAsync(
            {
              payer: createNoopSigner(payer! as Address),
              admin: createNoopSigner(args.admin as Address),
              delegateRewardTokenAdmin: createNoopSigner(args.admin as Address),
              receiptTokenMint: args.receiptTokenMint as Address,
              supportedTokenMint: args.supportedTokenMint as Address,
              program: this.program.address,
            },
            {
              programAddress: this.program.address,
            }
          );
          const vault = ix.accounts[7].address;

          const vrtSpace = token.getMintSize();
          const vrtRent = await this.runtime.rpc
            .getMinimumBalanceForRentExemption(BigInt(vrtSpace))
            .send();

          return Promise.all([
            system.getCreateAccountInstruction({
              payer: createNoopSigner(payer! as Address),
              newAccount: createNoopSigner(args.receiptTokenMint as Address),
              lamports: vrtRent,
              space: vrtSpace,
              programAddress: token.TOKEN_PROGRAM_ADDRESS,
            }),
            token.getInitializeMint2Instruction({
              mint: args.receiptTokenMint as Address,
              decimals: 8,
              freezeAuthority: null,
              mintAuthority: args.admin as Address,
            }),
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(payer as Address),
              mint: args.receiptTokenMint as Address,
              owner: vault,
            }),
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(payer as Address),
              mint: args.supportedTokenMint as Address,
              owner: vault,
            }),
            ix,
          ]);
        },
      ],
    }
  );

  // readonly delegateRewardTokenAccount = new TransactionTemplateContext(
  //   this,
  //   v.object({
  //     mint: v.string(),
  //     delegate: v.string(),
  //   }),
  //   {
  //     description: 'delegate reward token mint',
  //     instructions: [
  //       async (parent, args, overrides) => {
  //         const [vault, payer] = await Promise.all([
  //           parent.resolveAccount(true),
  //           transformAddressResolverVariant(
  //             overrides.feePayer ??
  //               this.runtime.options.transaction.feePayer ??
  //               (() => Promise.resolve(null))
  //           )(parent),
  //         ]);
  //         if (!vault) throw new Error('invalid context');
  //
  //         return Promise.all([
  //           token.getCreateAssociatedTokenIdempotentInstructionAsync({
  //             payer: createNoopSigner(payer as Address),
  //             mint: args.mint as Address,
  //             owner: vault.address,
  //           }),
  //           solv.getDelegateVaultRewardTokenAccountInstructionAsync(
  //             {
  //               admin: createNoopSigner(
  //                 vault.data.delegateRewardTokenAdmin as Address
  //               ),
  //               delegate: args.delegate as Address,
  //               receiptTokenMint: vault.data.receiptTokenMint,
  //               rewardTokenMint: args.mint as Address,
  //               program: this.program.address,
  //             },
  //             {
  //               programAddress: this.program.address,
  //             }
  //           ),
  //         ]);
  //       },
  //     ],
  //   }
  // );
}
