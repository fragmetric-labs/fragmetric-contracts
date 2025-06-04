import * as system from '@solana-program/system';
import * as token2022 from '@solana-program/token-2022';
import {
  AccountRole,
  Address,
  createNoopSigner,
  ReadonlyAccount,
} from '@solana/kit';
import * as v from 'valibot';
import {
  AccountAddressResolverVariant,
  FragmetricMetadataContext,
  TokenMintAccountContext,
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../context';
import { LAMPORTS_PER_SOL } from '../../context/constants';
import * as restaking from '../../generated/restaking';
import { RestakingFundAccountContext } from './fund';
import { RestakingNormalizedTokenMintAccountContext } from './normalized_token_mint';
import { RestakingNormalizedTokenPoolAccountContext } from './normalized_token_pool';
import { RestakingProgram } from './program';
import { RestakingRewardAccountContext } from './reward';
import { RestakingUserAccountContext } from './user';
import { RestakingWrappedTokenMintAccountContext } from './wrapped_token_mint';

export class RestakingReceiptTokenMintAccountContext extends TokenMintAccountContext<RestakingProgram> {
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
        const [receiptTokenMint, fund, normalizedTokenPool, metadata] =
          await Promise.all([
            this.resolveAccount(noCache),
            this.fund.resolveAccount(noCache),
            this.normalizedTokenPool.resolveAddress(noCache),
            this.metadata.resolveAccount(noCache),
          ]);
        if (!(receiptTokenMint && fund)) return null;

        const data = fund.data;

        const lookupTableAddress = data.addressLookupTableEnabled
          ? data.addressLookupTableAccount
          : null;

        const supportedTokens = data.supportedTokens.slice(
          0,
          data.numSupportedTokens
        );

        const supportedAssets = [
          {
            mint: null as Address | null,
            program: null as Address | null,
            decimals: 9,
            oneTokenAsSol: LAMPORTS_PER_SOL,
            oneTokenAsReceiptToken: data.oneReceiptTokenAsSol
              ? (LAMPORTS_PER_SOL * LAMPORTS_PER_SOL) /
                data.oneReceiptTokenAsSol
              : 0n,
            depositable: !!data.sol.depositable,
            withdrawable: !!data.sol.withdrawable,
            withdrawableValueAsReceiptTokenAmount:
              data.sol.withdrawableValueAsReceiptTokenAmount,
            withdrawalResidualMicroAssetAmount:
              data.sol.withdrawalResidualMicroAssetAmount,
            withdrawalUserReservedAmount: data.sol.withdrawalUserReservedAmount,
            withdrawalLastBatchProcessedAt: new Date(
              Number(data.sol.withdrawalLastBatchProcessedAt) * 1000
            ),
            operationTotalAmount:
              data.sol.operationReservedAmount +
              data.sol.operationReceivableAmount,
            operationReservedAmount: data.sol.operationReservedAmount,
            operationReceivableAmount: data.sol.operationReceivableAmount,
            unstakingAmountAsSOL: 0n,
          },
        ]
          .concat(
            supportedTokens.map((v) => {
              return {
                mint: v.token.tokenMint,
                program: v.token.tokenProgram,
                decimals: v.decimals,
                oneTokenAsSol: v.oneTokenAsSol,
                oneTokenAsReceiptToken: v.oneTokenAsReceiptToken,
                depositable: !!v.token.depositable,
                withdrawable: !!v.token.withdrawable,
                withdrawableValueAsReceiptTokenAmount:
                  v.token.withdrawableValueAsReceiptTokenAmount,
                withdrawalResidualMicroAssetAmount:
                  v.token.withdrawalResidualMicroAssetAmount,
                withdrawalUserReservedAmount:
                  v.token.withdrawalUserReservedAmount,
                withdrawalLastBatchProcessedAt: new Date(
                  Number(v.token.withdrawalLastBatchProcessedAt) * 1000
                ),
                operationTotalAmount:
                  v.token.operationReservedAmount +
                  v.token.operationReceivableAmount,
                operationReservedAmount: v.token.operationReservedAmount,
                operationReceivableAmount: v.token.operationReceivableAmount,
                unstakingAmountAsSOL: v.pendingUnstakingAmountAsSol,
              };
            })
          )
          .filter((a) => a.depositable || a.withdrawable);

        const restakingVaults = data.restakingVaults.slice(
          0,
          data.numRestakingVaults
        );

        const pricingSources: ReadonlyAccount[] = (
          data.numPricingSourceAddresses > 0
            ? data.pricingSourceAddresses.slice(
                0,
                data.numPricingSourceAddresses
              )
            : supportedTokens
                .filter(
                  (v) =>
                    this.__tokenPricingSourceDiscriminantMap.get(
                      v.pricingSource.discriminant
                    ) != 'PeggedToken'
                ) // skip pegged token
                .map((v) => v.pricingSource.address)
                .concat(normalizedTokenPool ? [normalizedTokenPool] : [])
                .concat(
                  restakingVaults.map(
                    (v) => v.receiptTokenPricingSource.address
                  )
                )
        ).map((address) => {
          return {
            address,
            role: AccountRole.READONLY,
          };
        });

        return {
          metadata,
          receiptTokenMint: receiptTokenMint.address,
          receiptTokenSupply: receiptTokenMint.data.supply,
          receiptTokenDecimals: receiptTokenMint.data.decimals,
          oneReceiptTokenAsSOL: fund.data.oneReceiptTokenAsSol,
          depositResidualMicroReceiptTokenAmount:
            fund.data.depositResidualMicroReceiptTokenAmount,
          wrappedTokenMint: fund.data.wrappedToken.enabled
            ? fund.data.wrappedToken.mint
            : null,
          supportedAssets,
          normalizedToken: normalizedTokenPool
            ? {
                mint: fund.data.normalizedToken.mint,
                program: fund.data.normalizedToken.program,
                oneTokenAsSol: fund.data.normalizedToken.oneTokenAsSol,
                operationReservedAmount:
                  fund.data.normalizedToken.operationReservedAmount,
              }
            : null,
          restakingVaultReceiptTokens: restakingVaults.map((v) => {
            return {
              vault: v.vault,
              mint: v.receiptTokenMint,
              program: v.receiptTokenProgram,
              oneReceiptTokenAsSol: v.oneReceiptTokenAsSol,
              operationTotalAmount:
                v.receiptTokenOperationReceivableAmount +
                v.receiptTokenOperationReservedAmount,
              operationReservedAmount: v.receiptTokenOperationReservedAmount,
              operationReceivableAmount:
                v.receiptTokenOperationReceivableAmount,
            };
          }),
          __lookupTableAddress: lookupTableAddress,
          __pricingSources: pricingSources,
        };
      }
    );
  }

  public readonly __tokenPricingSourceDiscriminantMap = this.__memoized(
    'tokenPricingSourceDiscriminantMap',
    () => {
      // hacky-way to calculate discriminant map to convert POD to Borsh compatible args.
      // note that: it assumes the order of discriminants are same in POD and Borsh types.
      const map = new Map<number, restaking.TokenPricingSource['__kind']>();
      const decoder = restaking.getTokenPricingSourceDecoder();
      for (let i = 0; i < 100; i++) {
        const buffer = new Uint8Array(33);
        buffer[0] = i;

        try {
          const [decoded] = decoder.read(buffer, 0);
          map.set(i + 1, decoded.__kind);
        } catch (e) {
          break;
        }
      }
      return map;
    }
  );

  readonly metadata = FragmetricMetadataContext.from(this);

  readonly fund = new RestakingFundAccountContext(this);

  readonly reward = new RestakingRewardAccountContext(this);

  readonly normalizedTokenMint = new RestakingNormalizedTokenMintAccountContext(
    this
  );

  readonly normalizedTokenPool = new RestakingNormalizedTokenPoolAccountContext(
    this
  );

  readonly wrappedTokenMint = new RestakingWrappedTokenMintAccountContext(this);

  user(
    addressResolver: AccountAddressResolverVariant<RestakingReceiptTokenMintAccountContext>
  ) {
    return new RestakingUserAccountContext(this, addressResolver);
  }

  readonly payer = this.user(
    this.runtime.options.transaction.feePayer ?? (() => Promise.resolve(null))
  );

  /** authorized transactions **/
  readonly initializeMint = new TransactionTemplateContext(
    this,
    v.object({
      name: v.string(),
      symbol: v.string(),
      uri: v.string(),
      description: v.string(),
      decimals: v.number(),
    }),
    {
      description: 'initialize receipt token mint',
      instructions: [
        async (parent, args, overrides) => {
          const [receiptTokenMint, payer] = await Promise.all([
            parent.resolveAddress(),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!receiptTokenMint) throw new Error('invalid context');
          const admin = (this.program as RestakingProgram).knownAddresses.admin;

          const extensions: token2022.ExtensionArgs[] = [
            {
              __kind: 'TransferHook',
              authority: admin,
              programId: this.program.address,
            },
            {
              __kind: 'MetadataPointer',
              authority: admin,
              metadataAddress: receiptTokenMint,
            },
            {
              __kind: 'TokenMetadata',
              updateAuthority: admin,
              mint: receiptTokenMint,
              name: args.name,
              symbol: args.symbol,
              uri: args.uri,
              additionalMetadata: new Map(
                Object.entries({
                  description: args.description,
                })
              ),
            },
          ];

          const space = token2022.getMintSize(extensions);
          const spaceWithoutPostInitializeExtensions = token2022.getMintSize(
            extensions.filter(
              (e) =>
                !['TokenMetadata', 'TokenGroup', 'TokenGroupMember'].includes(
                  e.__kind
                )
            )
          );
          const rent = await this.runtime.rpc
            .getMinimumBalanceForRentExemption(BigInt(space))
            .send();
          return [
            system.getCreateAccountInstruction({
              payer: createNoopSigner(payer! as Address),
              newAccount: createNoopSigner(receiptTokenMint),
              lamports: rent,
              space: spaceWithoutPostInitializeExtensions,
              programAddress: token2022.TOKEN_2022_PROGRAM_ADDRESS,
            }),
            ...token2022.getPreInitializeInstructionsForMintExtensions(
              receiptTokenMint,
              extensions
            ),
            token2022.getInitializeMint2Instruction({
              mint: receiptTokenMint,
              decimals: args.decimals,
              freezeAuthority: null,
              mintAuthority: admin,
            }),
            ...token2022.getPostInitializeInstructionsForMintExtensions(
              receiptTokenMint,
              createNoopSigner(admin),
              extensions
            ),
          ];
        },
      ],
    }
  );

  readonly initializeOrUpdateExtraAccountMetaList =
    new TransactionTemplateContext(this, v.nullish(v.null(), null), {
      description: 'initialize or update extra account meta list',
      instructions: [
        async (parent, args, overrides) => {
          const [receiptTokenMint, payer] = await Promise.all([
            parent.resolveAddress(),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!receiptTokenMint) throw new Error('invalid context');
          const admin = (this.program as RestakingProgram).knownAddresses.admin;

          const ix =
            await restaking.getAdminInitializeExtraAccountMetaListInstructionAsync(
              {
                payer: createNoopSigner(payer! as Address),
                admin: createNoopSigner(admin),
                receiptTokenMint,
                program: this.program.address,
              },
              {
                programAddress: this.program.address,
              }
            );
          const extraAccountMetaList = ix.accounts[4].address;
          const extraAccountMetaListAccount = await this.runtime.fetchAccount(
            extraAccountMetaList,
            true
          );

          return [
            extraAccountMetaListAccount
              ? await restaking.getAdminUpdateExtraAccountMetaListIfNeededInstructionAsync(
                  {
                    admin: createNoopSigner(admin),
                    receiptTokenMint,
                    program: this.program.address,
                  },
                  {
                    programAddress: this.program.address,
                  }
                )
              : ix,
          ];
        },
      ],
    });
}
