import { getSetComputeUnitLimitInstruction } from '@solana-program/compute-budget';
import * as token from '@solana-program/token';
import {
  Account,
  AccountRole,
  Address,
  createNoopSigner,
  EncodedAccount,
} from '@solana/kit';
import * as v from 'valibot';
import {
  AccountContext,
  IterativeAccountContext,
  TokenAccountContext,
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../context';
import * as restaking from '../../generated/restaking';
import { getRestakingAnchorEventDecoders } from './events';
import { RestakingProgram } from './program';
import { RestakingReceiptTokenMintAccountContext } from './receipt_token_mint';

export class RestakingNormalizedTokenPoolAccountContext extends AccountContext<
  RestakingReceiptTokenMintAccountContext,
  Account<restaking.NormalizedTokenPoolAccount>
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
      supportedTokens,
      reserved,
      ...props
    } = account.data;

    return {
      ...props,
      supportedTokens: supportedTokens.map((item) => {
        const { reserved, ...props } = item;

        return {
          ...props,
        };
      }),
    };
  }
  constructor(parent: RestakingReceiptTokenMintAccountContext) {
    super(parent, async (parent) => {
      const fund = await parent.fund.resolveAccount(true);
      if (!fund?.data.normalizedToken?.enabled) {
        return null;
      }

      const ix =
        await restaking.getAdminInitializeNormalizedTokenPoolAccountInstructionAsync(
          { normalizedTokenMint: fund.data.normalizedToken.mint } as any,
          { programAddress: parent.program.address }
        );
      return ix.accounts![5].address;
    });
  }

  protected __decodeAccount(account: EncodedAccount) {
    return restaking.decodeNormalizedTokenPoolAccount(account);
  }

  readonly supportedTokens = new IterativeAccountContext(
    this,
    async (parent) => {
      const [self, fund] = await Promise.all([
        parent.resolveAccount(true),
        parent.parent.fund.resolveAccount(true),
      ]);
      if (!(self && fund)) return null;
      const addresses = await Promise.all(
        self.data.supportedTokens.map((item) => {
          return TokenAccountContext.findAssociatedTokenAccountAddress({
            owner: self.address,
            mint: item.mint,
            tokenProgram: item.program,
          });
        })
      );
      return addresses.filter((address) => !!address);
    },
    async (parent, address) => {
      return new TokenAccountContext(parent, address);
    }
  );

  private __resolveAddressLookupTable = (parent: this) =>
    parent.parent.resolve().then((data) => data?.__lookupTableAddress ?? null);

  /** operator transactions **/
  readonly updatePrices = new TransactionTemplateContext(this, null, {
    description:
      'manually triggers price updates for the normalized token and underlying assets',
    anchorEventDecoders: getRestakingAnchorEventDecoders(
      'operatorUpdatedNormalizedTokenPoolPrices'
    ),
    addressLookupTables: [this.__resolveAddressLookupTable],
    instructions: [
      getSetComputeUnitLimitInstruction({ units: 1_400_000 }),
      async (parent, args, overrides) => {
        const [data, ntp, operator] = await Promise.all([
          parent.parent.resolve(true),
          parent.resolveAccount(),
          transformAddressResolverVariant(
            overrides.feePayer ??
              this.runtime.options.transaction.feePayer ??
              (() => Promise.resolve(null))
          )(parent),
        ]);
        if (!(data && ntp && operator)) throw new Error('invalid context');

        const ix =
          await restaking.getOperatorUpdateNormalizedTokenPoolPricesInstructionAsync(
            {
              operator: createNoopSigner(operator as Address),
              program: this.program.address,
              normalizedTokenMint: ntp.data.normalizedTokenMint,
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
  });

  /** authorized transactions **/
  readonly addSupportedToken = new TransactionTemplateContext(
    this,
    v.object({
      mint: v.string(),
      program: v.nullish(v.string(), token.TOKEN_PROGRAM_ADDRESS),
      pricingSource: v.pipe(
        v.object({
          __kind: v.picklist([
            'SPLStakePool',
            'MarinadeStakePool',
            'OrcaDEXLiquidityPool',
            'SanctumSingleValidatorSPLStakePool',
            'SanctumMultiValidatorSPLStakePool',
            'PeggedToken',
          ]),
          address: v.string(),
        }) as v.GenericSchema<
          Omit<restaking.TokenPricingSourceArgs, 'address'> & {
            address: string;
          }
        >
      ),
    }),
    {
      description: 'add a new supported token to the normalized token pool',
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [data, ntp, payer] = await Promise.all([
            parent.parent.resolve(true),
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!(data && ntp)) throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(payer as Address),
              owner: ntp.address,
              mint: args.mint as Address,
              tokenProgram: args.program as Address,
            }),
            restaking
              .getFundManagerAddNormalizedTokenPoolSupportedTokenInstructionAsync(
                {
                  fundManager: createNoopSigner(fundManager),
                  normalizedTokenMint: ntp.data.normalizedTokenMint,
                  supportedTokenMint: args.mint as Address,
                  supportedTokenProgram: args.program as Address,
                  pricingSource:
                    args.pricingSource as restaking.TokenPricingSourceArgs,
                  program: this.program.address,
                },
                {
                  programAddress: this.program.address,
                }
              )
              .then((ix) => {
                for (const accountMeta of data.__pricingSources) {
                  ix.accounts.push(accountMeta);
                }
                ix.accounts.push({
                  address: args.pricingSource.address as Address,
                  role: AccountRole.READONLY,
                });
                return ix;
              }),
          ]);
        },
      ],
    }
  );

  readonly removeSupportedToken = new TransactionTemplateContext(
    this,
    v.object({
      mint: v.string(),
      program: v.nullish(v.string(), token.TOKEN_PROGRAM_ADDRESS),
    }),
    {
      description: 'remove an unused and unfunded supported token',
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [data, ntp] = await Promise.all([
            parent.parent.resolve(true),
            parent.resolveAccount(true),
          ]);
          if (!(data && ntp)) throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          return Promise.all([
            restaking
              .getFundManagerRemoveNormalizedTokenPoolSupportedTokenInstructionAsync(
                {
                  fundManager: createNoopSigner(fundManager),
                  normalizedTokenMint: ntp.data.normalizedTokenMint,
                  supportedTokenMint: args.mint as Address,
                  supportedTokenProgram: args.program as Address,
                  program: this.program.address,
                },
                {
                  programAddress: this.program.address,
                }
              )
              .then((ix) => {
                for (const accountMeta of data.__pricingSources) {
                  ix.accounts.push(accountMeta);
                }
                return ix;
              }),
          ]);
        },
      ],
    }
  );
}
