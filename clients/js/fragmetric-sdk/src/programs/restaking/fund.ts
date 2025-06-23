import { getSetComputeUnitLimitInstruction } from '@solana-program/compute-budget';
import * as system from '@solana-program/system';
import * as token from '@solana-program/token';
import * as token2022 from '@solana-program/token-2022';
import {
  Account,
  AccountRole,
  Address,
  createNoopSigner,
  EncodedAccount,
  getAddressEncoder,
  getBytesEncoder,
  getProgramDerivedAddress,
  IAccountMeta,
  ReadonlyUint8Array,
} from '@solana/kit';
import * as v from 'valibot';
import {
  AccountContext,
  IterativeAccountContext,
  TokenAccountContext,
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../context';
import * as jitoRestaking from '../../generated/jito_restaking';
import * as jitoVault from '../../generated/jito_vault';
import * as restaking from '../../generated/restaking';
import * as solv from '../../generated/solv';
import { SolvBTCVaultProgram } from '../solv';
import { SolvVaultAccountContext } from '../solv/vault';
import { getRestakingAnchorEventDecoders } from './events';
import { RestakingFundAddressLookupTableAccountContext } from './fund_address_lookup_table';
import { RestakingFundReserveAccountContext } from './fund_reserve';
import { RestakingFundTreasuryAccountContext } from './fund_treasury';
import { RestakingFundWithdrawalBatchAccountContext } from './fund_withdrawal_batch';
import { RestakingFundWrapAccountContext } from './fund_wrap';
import { RestakingProgram } from './program';
import { RestakingReceiptTokenMintAccountContext } from './receipt_token_mint';
import { JitoVaultAccountContext } from './restaking_vault_jito';
import { VirtualVaultAccountContext } from './restaking_vault_virtual';

export class RestakingFundAccountContext extends AccountContext<
  RestakingReceiptTokenMintAccountContext,
  Account<restaking.FundAccount>
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
          generalStrategy,
          assetStrategies,
          tokenSwapStrategies,
          restakingVaultStrategies,
        ] = await Promise.all([
          this.resolveGeneralStrategy(noCache),
          this.resolveAssetStrategies(noCache),
          this.resolveTokenSwapStrategies(noCache),
          this.resolveRestakingVaultStrategies(noCache),
        ]);
        return {
          generalStrategy,
          assetStrategies,
          tokenSwapStrategies,
          restakingVaultStrategies,
        };
      }
    );
  }

  constructor(readonly parent: RestakingReceiptTokenMintAccountContext) {
    super(parent, async (parent) => {
      const receiptTokenMint = await parent.resolveAddress();
      if (receiptTokenMint) {
        const ix =
          await restaking.getAdminInitializeFundAccountInstructionAsync(
            { receiptTokenMint } as any,
            { programAddress: parent.program.address }
          );
        return ix.accounts![5].address;
      }
      return null;
    });
  }

  protected __decodeAccount(account: EncodedAccount) {
    return restaking.decodeFundAccount(account);
  }

  readonly reserve = new RestakingFundReserveAccountContext(this);

  readonly lockedReceiptToken =
    TokenAccountContext.fromAssociatedTokenSeeds2022(this, async (parent) => {
      const [receiptTokenMint, fund] = await Promise.all([
        parent.parent.resolveAddress(),
        parent.resolveAddress(),
      ]);
      if (!fund || !receiptTokenMint) return null;

      return {
        owner: fund,
        mint: receiptTokenMint,
      };
    });

  withdrawalBatch(tokenMint: string | null, batchId: bigint) {
    return new RestakingFundWithdrawalBatchAccountContext(
      this,
      async (parent) => {
        const [fund, receiptTokenMint] = await Promise.all([
          parent.resolveAddress(),
          parent.parent.resolveAddress(),
        ]);
        if (!fund || !receiptTokenMint) return null;
        const ix = await restaking.getUserWithdrawSolInstructionAsync(
          {
            user: { address: receiptTokenMint },
            receiptTokenMint,
            batchId,
            requestId: 0n,
          } as any,
          { programAddress: parent.program.address }
        );
        return ix!.accounts[7].address ?? null;
      }
    );
  }

  private async __getWithdrawalBatchAccountAddress(
    asset: restaking.AssetState,
    receiptTokenMint: string
  ) {
    if (asset.tokenMint == system.SYSTEM_PROGRAM_ADDRESS) {
      const ix = await restaking.getUserWithdrawSolInstructionAsync(
        {
          user: { address: receiptTokenMint },
          receiptTokenMint,
          batchId: asset.withdrawalLastProcessedBatchId,
          requestId: 0n,
        } as any,
        { programAddress: this.program.address }
      );
      return ix!.accounts[7].address ?? null;
    } else {
      const ix = await restaking.getUserWithdrawSupportedTokenInstructionAsync(
        {
          user: { address: receiptTokenMint },
          receiptTokenMint,
          supportedTokenMint: asset.tokenMint,
          supportedTokenProgram: asset.tokenProgram,
          batchId: asset.withdrawalLastProcessedBatchId,
          requestId: 0n,
        } as any,
        { programAddress: this.program.address }
      );
      return ix!.accounts[10].address ?? null;
    }
  }

  readonly latestWithdrawalBatches = new IterativeAccountContext<
    RestakingFundAccountContext,
    RestakingFundWithdrawalBatchAccountContext
  >(
    this,
    async (parent) => {
      const [receiptTokenMint, fund] = await Promise.all([
        parent.parent.resolveAddress(),
        parent.parent.fund.resolveAccount(true),
      ]);
      if (!receiptTokenMint || !fund) return null;
      const addresses = await Promise.all(
        [fund.data.sol]
          .concat(
            fund.data.supportedTokens
              .slice(0, fund.data.numSupportedTokens)
              .map((item) => item.token)
          )
          .filter((item) => item.withdrawable)
          .map(async (item) =>
            this.__getWithdrawalBatchAccountAddress(item, receiptTokenMint)
          )
      );
      return addresses.filter((address) => !!address);
    },
    async (parent, address) => {
      return new RestakingFundWithdrawalBatchAccountContext(parent, address);
    }
  );

  readonly restakingVaults = new IterativeAccountContext<
    any,
    AccountContext<
      any,
      Account<jitoVault.Vault | solv.VaultAccount | ReadonlyUint8Array>
    >
  >(
    this,
    async (parent: RestakingFundAccountContext) => {
      const fund = await parent.resolveAccount(true);
      if (!fund) return null;

      return fund.data.restakingVaults
        .slice(0, fund.data.numRestakingVaults)
        .map((v) => {
          return `${v.vault}/${v.program}`;
        });
    },
    async (parent, address) => {
      const [vault, program] = address.split('/');
      return this.restakingVault(vault, program) as AccountContext<
        any,
        Account<jitoVault.Vault | solv.VaultAccount>
      >;
    }
  );

  get jitoRestakingVaults(): JitoVaultAccountContext[] | undefined {
    return this.restakingVaults.children.filter(
      (v) => v instanceof JitoVaultAccountContext
    );
  }

  get solvBTCVaults(): SolvVaultAccountContext[] | undefined {
    return this.restakingVaults.children.filter(
      (v) => v instanceof SolvVaultAccountContext
    );
  }

  get virtualVaults(): VirtualVaultAccountContext[] | undefined {
    return this.restakingVaults.children.filter(
      (v) => v instanceof VirtualVaultAccountContext
    );
  }

  restakingVault(vault: string | null, program: string) {
    switch (program) {
      case jitoVault.JITO_VAULT_PROGRAM_ADDRESS:
        return new JitoVaultAccountContext(this, vault!);
      case this.__solvBTCVaultProgram.address:
        return this.__solvBTCVaultProgram.vault(vault!);
      case system.SYSTEM_PROGRAM_ADDRESS:
        return new VirtualVaultAccountContext(this);
    }
  }

  readonly wrap = new RestakingFundWrapAccountContext(this, async (parent) => {
    const fund = await parent.resolveAccount(true);
    if (fund?.data.wrappedToken?.enabled) {
      return fund.data.wrapAccount;
    }
    return null;
  });

  readonly treasury = new RestakingFundTreasuryAccountContext(this);

  readonly addressLookupTable =
    new RestakingFundAddressLookupTableAccountContext(this);

  private __resolveAddressLookupTable = (parent: this) =>
    parent.parent
      .resolve(true)
      .then((data) => data?.__lookupTableAddress ?? null);

  /** operator transactions **/
  readonly updatePrices = new TransactionTemplateContext(this, null, {
    description:
      'manually triggers price updates for the receipt token and underlying assets',
    anchorEventDecoders: getRestakingAnchorEventDecoders(
      'operatorUpdatedFundPrices'
    ),
    addressLookupTables: [this.__resolveAddressLookupTable],
    instructions: [
      getSetComputeUnitLimitInstruction({ units: 1_400_000 }),
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

        const ix = await restaking.getOperatorUpdateFundPricesInstructionAsync(
          {
            operator: createNoopSigner(operator as Address),
            program: this.program.address,
            receiptTokenMint: data.receiptTokenMint!,
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

  readonly donate = new TransactionTemplateContext(
    this,
    v.object({
      assetMint: v.pipe(
        v.nullish(v.string(), null),
        v.description('supported token mint to donate, null to donate SOL')
      ),
      assetAmount: v.pipe(v.bigint(), v.description('amount to donate')),
      offsetReceivable: v.pipe(
        v.nullish(v.boolean(), true),
        v.description(
          'to prioritize offsetting receivables with donations instead of increasing receipt token value'
        )
      ),
      applyPresetComputeUnitLimit: v.pipe(
        v.nullish(v.boolean(), true),
        v.description('apply preset CU limit')
      ),
    }),
    {
      description:
        'donate supported assets to the fund for operational purposes',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'operatorDonatedToFund'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
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

          return Promise.all([
            args.applyPresetComputeUnitLimit
              ? getSetComputeUnitLimitInstruction({ units: 1_400_000 })
              : null,

            (async () => {
              const ix = await (args.assetMint
                ? restaking.getOperatorDonateSupportedTokenToFundInstructionAsync(
                    {
                      operator: createNoopSigner(operator as Address),
                      receiptTokenMint: data.receiptTokenMint,
                      supportedTokenProgram: token.TOKEN_PROGRAM_ADDRESS,
                      program: this.program.address,
                      operatorSupportedTokenAccount:
                        await TokenAccountContext.findAssociatedTokenAccountAddress(
                          {
                            owner: operator,
                            mint: args.assetMint,
                            tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
                          }
                        ),
                      supportedTokenMint: args.assetMint as Address,
                      amount: args.assetAmount,
                      offsetReceivable: args.offsetReceivable,
                    },
                    {
                      programAddress: this.program.address,
                    }
                  )
                : restaking.getOperatorDonateSolToFundInstructionAsync(
                    {
                      operator: createNoopSigner(operator as Address),
                      receiptTokenMint: data.receiptTokenMint,
                      program: this.program.address,
                      amount: args.assetAmount,
                      offsetReceivable: args.offsetReceivable,
                    },
                    {
                      programAddress: this.program.address,
                    }
                  ));

              for (const accountMeta of data.__pricingSources) {
                ix.accounts.push(accountMeta);
              }

              return ix;
            })(),
          ]);
        },
      ],
    }
  );

  readonly runCommand = new TransactionTemplateContext(
    this,
    v.nullish(
      v.object({
        operator: v.pipe(
          v.nullish(v.string(), null),
          v.description('set operator account (default is feePayer)')
        ),
        forceResetCommand: v.pipe(
          v.nullish(
            v.union([
              v.any() as v.GenericSchema<restaking.OperationCommandEntryArgs>,
              v.string() as unknown as v.UnionSchema<
                v.GenericSchema<
                  restaking.OperationCommandEntryArgs['command']['__kind']
                >[],
                undefined
              >,
            ]),
            null
          ),
          v.description(
            'to forcibly run a specific command with authorized signer (entire command object or name)'
          )
        ),
      }),
      {}
    ),
    {
      description: 'execute the next fund command to circulate assets',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'operatorRanFundCommand'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [data, fund, operator] = await Promise.all([
            parent.parent.resolve(true),
            parent.resolveAccount(true),
            args.operator
              ? args.operator
              : args.forceResetCommand
                ? (this.program as RestakingProgram).knownAddresses.admin
                : transformAddressResolverVariant(
                    overrides.feePayer ??
                      this.runtime.options.transaction.feePayer ??
                      (() => Promise.resolve(null))
                  )(parent),
          ]);
          if (!(data && fund && operator)) throw new Error('invalid context');

          let forceResetCommand: restaking.OperationCommandEntryArgs | null =
            null;
          if (args.forceResetCommand) {
            if (typeof args.forceResetCommand == 'string') {
              const command = {
                __kind: args.forceResetCommand,
                fields: [{ state: { __kind: 'New' } }],
              } as const;
              if (
                args.forceResetCommand == 'EnqueueWithdrawalBatch' ||
                args.forceResetCommand == 'ProcessWithdrawalBatch'
              ) {
                (command.fields[0] as any).forced = true;
              }
              forceResetCommand = {
                command:
                  command as unknown as restaking.OperationCommandEntryArgs['command'],
                requiredAccounts: [],
              };
            } else {
              forceResetCommand = args.forceResetCommand;
            }
          }

          return Promise.all([
            getSetComputeUnitLimitInstruction({ units: 1_400_000 }),

            (async () => {
              const ix =
                await restaking.getOperatorRunFundCommandInstructionAsync(
                  {
                    operator: createNoopSigner(operator as Address),
                    receiptTokenMint: data.receiptTokenMint,
                    program: this.program.address,
                    forceResetCommand: forceResetCommand,
                  },
                  {
                    programAddress: this.program.address,
                  }
                );

              // prepare accounts according to the current state of operation.
              // - can add 58 accounts out of 64 with reserved 6 accounts.
              // - order doesn't matter, no need to put duplicate.
              const requiredAccounts: Map<Address, IAccountMeta> = new Map();

              // add pricing sources
              for (const accountMeta of data.__pricingSources) {
                requiredAccounts.set(accountMeta.address, accountMeta);
              }

              // add required accounts for next command
              if (forceResetCommand) {
                for (const accountMeta of forceResetCommand.requiredAccounts) {
                  requiredAccounts.set(accountMeta.pubkey, {
                    address: accountMeta.pubkey,
                    role: accountMeta.isWritable
                      ? AccountRole.WRITABLE
                      : AccountRole.READONLY,
                  });
                }
              } else {
                const nextCommand = fund.data.operation.nextCommand;
                for (let i = 0; i < nextCommand.numRequiredAccounts; i++) {
                  const accountMeta = nextCommand.requiredAccounts[i];
                  requiredAccounts.set(accountMeta.pubkey, {
                    address: accountMeta.pubkey,
                    role:
                      accountMeta.isWritable != 0
                        ? AccountRole.WRITABLE
                        : AccountRole.READONLY,
                  });
                }
              }

              for (const accountMeta of requiredAccounts.values()) {
                (ix.accounts as IAccountMeta[]).push(accountMeta);
              }

              return ix;
            })(),
          ]);
        },
      ],
    },
    async (parent, args, events) => {
      if (!events?.operatorRanFundCommand) {
        throw new Error(
          `invalid context: failed to parse event during chaining`
        );
      }
      if (
        typeof events?.operatorRanFundCommand?.nextSequence != 'undefined' &&
        events.operatorRanFundCommand.nextSequence != 0
      ) {
        return {
          args: {
            ...args,
            forceResetCommand: null,
          },
        };
      }
      return null;
    }
  );

  /** authorized transactions **/
  readonly initializeOrUpdateAccount = new TransactionTemplateContext(
    this,
    v.object({
      targetVersion: v.number(),
    }),
    {
      description: 'initialize or update fund account',
      instructions: [
        getSetComputeUnitLimitInstruction({ units: 1_400_000 }),
        async (parent, args, overrides) => {
          const [receiptTokenMint, reserve, treasury, currentVersion, payer] =
            await Promise.all([
              parent.parent.resolveAddress(),
              parent.reserve.resolveAddress(),
              parent.treasury.resolveAddress(),
              parent
                .resolveAccount(true)
                .then((fund) => fund?.data.dataVersion ?? 0)
                .catch((err) => 0),
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
              ? await (async () => {
                  const ix =
                    await restaking.getAdminInitializeFundAccountInstructionAsync(
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
                  const fundAccount = ix.accounts[5].address;
                  return Promise.all([
                    token2022.getCreateAssociatedTokenIdempotentInstructionAsync(
                      {
                        payer: createNoopSigner(payer! as Address),
                        mint: receiptTokenMint,
                        owner: fundAccount,
                        tokenProgram: token2022.TOKEN_2022_PROGRAM_ADDRESS,
                      }
                    ),
                    ix,
                  ]);
                })()
              : []),
            ...Array(Math.min(args.targetVersion - currentVersion, 35))
              .fill(null)
              .map(() => {
                return restaking.getAdminUpdateFundAccountIfNeededInstructionAsync(
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
        .then((fund) => fund?.data.dataVersion ?? 0)
        .catch((err) => 0);

      if (currentVersion < args.targetVersion) {
        return {
          args,
        } as any;
      }
      return null;
    }
  );

  async resolveGeneralStrategy(
    noCache = false
  ): Promise<restaking.FundManagerUpdateFundStrategyInstructionDataArgs | null> {
    const fund = await this.resolveAccount(noCache);
    if (!fund) return null;

    return {
      depositEnabled: fund.data.depositEnabled == 1,
      donationEnabled: fund.data.donationEnabled == 1,
      transferEnabled: fund.data.transferEnabled == 1,
      operationEnabled: fund.data.operationEnabled == 1,
      withdrawalBatchThresholdSeconds:
        fund.data.withdrawalBatchThresholdIntervalSeconds,
      withdrawalEnabled: fund.data.withdrawalEnabled == 1,
      withdrawalFeeRateBps: fund.data.withdrawalFeeRateBps,
    };
  }

  readonly updateGeneralStrategy = new TransactionTemplateContext(
    this,
    v.partial(
      v.object({
        depositEnabled: v.boolean(),
        donationEnabled: v.boolean(),
        transferEnabled: v.boolean(),
        withdrawalEnabled: v.boolean(),
        operationEnabled: v.boolean(),
        withdrawalBatchThresholdSeconds: v.number(),
        withdrawalFeeRateBps: v.pipe(
          v.number(),
          v.description('1 fee rate = 1bps = 0.01%')
        ),
      }) as v.GenericSchema<restaking.FundManagerUpdateFundStrategyInstructionDataArgs> as unknown as v.StrictObjectSchema<
        any,
        any
      >
    ) as v.GenericSchema<
      Partial<restaking.FundManagerUpdateFundStrategyInstructionDataArgs>
    >,
    {
      description: 'update general strategy of the fund',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedFund'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [receiptTokenMint, current, payer] = await Promise.all([
            parent.parent.resolveAddress(),
            parent.resolveGeneralStrategy(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!(receiptTokenMint && current))
            throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;
          const newArgs = {
            ...current,
            ...args,
          };

          return Promise.all([
            restaking.getFundManagerUpdateFundStrategyInstructionAsync(
              {
                ...newArgs,
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
      description: 'add a new supported token',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedFund'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [data, payer] = await Promise.all([
            parent.parent.resolve(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!data) throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          const ix =
            await restaking.getFundManagerAddSupportedTokenInstructionAsync(
              {
                pricingSource:
                  args.pricingSource as restaking.TokenPricingSourceArgs,
                supportedTokenMint: args.mint as Address,
                supportedTokenProgram: args.program as Address,
                fundManager: createNoopSigner(fundManager),
                receiptTokenMint: data.receiptTokenMint,
                program: this.program.address,
              },
              {
                programAddress: this.program.address,
              }
            );
          for (const accountMeta of data.__pricingSources) {
            ix.accounts.push(accountMeta);
          }
          ix.accounts.push({
            address: args.pricingSource.address as Address,
            role: AccountRole.READONLY,
          });

          const fundReserve = ix.accounts[3].address;
          const fundTreasury = ix.accounts[4].address;

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(payer as Address),
              mint: args.mint as Address,
              owner: fundReserve,
              tokenProgram: args.program as Address,
            }),
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(payer as Address),
              mint: args.mint as Address,
              owner: fundTreasury,
              tokenProgram: args.program as Address,
            }),
            ix,
          ]);
        },
      ],
    }
  );

  readonly removeSupportedToken = new TransactionTemplateContext(
    this,
    v.object({
      mint: v.string(),
    }),
    {
      description: 'remove unused supported token',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedFund'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        getSetComputeUnitLimitInstruction({ units: 1_400_000 }),
        async (parent, args, overrides) => {
          const [normalizedTokenMint, normalizedTokenPoolAccount, data] =
            await Promise.all([
              parent.parent.normalizedTokenMint.resolveAddress(true),
              parent.parent.normalizedTokenPool.resolveAddress(true),
              parent.parent.resolve(true),
            ]);
          if (!data) throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          return restaking
            .getFundManagerRemoveSupportedTokenInstructionAsync(
              {
                fundManager: createNoopSigner(fundManager),
                receiptTokenMint: data.receiptTokenMint,
                supportedTokenMint: args.mint as Address,
                normalizedTokenMint: normalizedTokenMint ?? undefined,
                normalizedTokenPoolAccount:
                  normalizedTokenPoolAccount ?? undefined,
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
              return [ix];
            });
        },
      ],
    }
  );

  async resolveAssetStrategies(
    noCache = false
  ): Promise<
    | (
        | restaking.FundManagerUpdateSolStrategyInstructionDataArgs
        | restaking.FundManagerUpdateSupportedTokenStrategyInstructionDataArgs
      )[]
    | null
  > {
    const fund = await this.resolveAccount(noCache);
    if (!fund) return null;

    const sol: restaking.FundManagerUpdateSolStrategyInstructionDataArgs = {
      solDepositable: fund.data.sol.depositable == 1,
      solAccumulatedDepositAmount: fund.data.sol.accumulatedDepositAmount,
      solAccumulatedDepositCapacityAmount:
        fund.data.sol.accumulatedDepositCapacityAmount,
      solWithdrawable: fund.data.sol.withdrawable == 1,
      solWithdrawalNormalReserveRateBps: fund.data.sol.normalReserveRateBps,
      solWithdrawalNormalReserveMaxAmount: fund.data.sol.normalReserveMaxAmount,
    };
    const tokens = fund.data.supportedTokens
      .slice(0, fund.data.numSupportedTokens)
      .map((item) => {
        const token: restaking.FundManagerUpdateSupportedTokenStrategyInstructionDataArgs =
          {
            tokenMint: item.token.tokenMint,
            tokenDepositable: item.token.depositable == 1,
            tokenAccumulatedDepositAmount: item.token.accumulatedDepositAmount,
            tokenAccumulatedDepositCapacityAmount:
              item.token.accumulatedDepositCapacityAmount,
            tokenWithdrawable: item.token.withdrawable == 1,
            tokenWithdrawalNormalReserveRateBps:
              item.token.normalReserveRateBps,
            tokenWithdrawalNormalReserveMaxAmount:
              item.token.normalReserveMaxAmount,
            solAllocationWeight: item.solAllocationWeight,
            solAllocationCapacityAmount: item.solAllocationCapacityAmount,
          };
        return token;
      });
    return [sol, ...tokens];
  }

  readonly updateAssetStrategy = new TransactionTemplateContext(
    this,
    v.union([
      v.intersect([
        v.object({
          tokenMint: v.string(),
        }),
        v.partial(
          v.object({
            tokenDepositable: v.boolean(),
            tokenAccumulatedDepositAmount: v.nullish(v.bigint(), null),
            tokenAccumulatedDepositCapacityAmount: v.bigint(),
            tokenWithdrawable: v.boolean(),
            tokenWithdrawalNormalReserveRateBps: v.pipe(
              v.number(),
              v.description('1 reserve rate = 1bps = 0.01%')
            ),
            tokenWithdrawalNormalReserveMaxAmount: v.bigint(),
            tokenRebalancingAmount: v.pipe(
              v.bigint(),
              v.description('unused now')
            ),
            solAllocationWeight: v.bigint(),
            solAllocationCapacityAmount: v.bigint(),
          }) as v.GenericSchema<
            Omit<
              restaking.FundManagerUpdateSupportedTokenStrategyInstructionDataArgs,
              'tokenMint'
            >
          > as unknown as v.StrictObjectSchema<any, any>
        ) as v.GenericSchema<
          Partial<
            Omit<
              restaking.FundManagerUpdateSupportedTokenStrategyInstructionDataArgs,
              'tokenMint'
            >
          >
        >,
      ]),
      v.intersect([
        v.object({
          tokenMint: v.null(),
        }),
        v.partial(
          v.object({
            solDepositable: v.boolean(),
            solAccumulatedDepositAmount: v.nullish(v.bigint(), null),
            solAccumulatedDepositCapacityAmount: v.bigint(),
            solWithdrawable: v.boolean(),
            solWithdrawalNormalReserveRateBps: v.pipe(
              v.number(),
              v.description('1 reserve rate = 1bps = 0.01%')
            ),
            solWithdrawalNormalReserveMaxAmount: v.bigint(),
          }) as v.GenericSchema<restaking.FundManagerUpdateSolStrategyInstructionDataArgs> as unknown as v.StrictObjectSchema<
            any,
            any
          >
        ) as v.GenericSchema<
          Partial<restaking.FundManagerUpdateSolStrategyInstructionDataArgs>
        >,
      ]),
    ]),
    {
      description: 'update asset strategy of the fund',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedFund'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [data, current, payer] = await Promise.all([
            parent.parent.resolve(true),
            parent.resolveAssetStrategies(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!(data && current)) throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          const newArgs = args.tokenMint
            ? {
                ...(current.find((item) => {
                  return (
                    'tokenMint' in item && item.tokenMint == args.tokenMint
                  );
                })! as restaking.FundManagerUpdateSupportedTokenStrategyInstructionDataArgs),
                tokenAccumulatedDepositAmount: null,
                ...(args as Partial<restaking.FundManagerUpdateSupportedTokenStrategyInstructionDataArgs>),
              }
            : {
                ...(current.find((item) => {
                  return !('tokenMint' in item);
                })! as restaking.FundManagerUpdateSolStrategyInstructionDataArgs),
                solAccumulatedDepositAmount: null,
                ...(args as Partial<restaking.FundManagerUpdateSolStrategyInstructionDataArgs>),
              };

          return Promise.all([
            'tokenMint' in newArgs && newArgs.tokenMint
              ? restaking.getFundManagerUpdateSupportedTokenStrategyInstructionAsync(
                  {
                    ...(newArgs as restaking.FundManagerUpdateSupportedTokenStrategyInstructionDataArgs),
                    fundManager: createNoopSigner(fundManager),
                    receiptTokenMint: data.receiptTokenMint,
                    program: this.program.address,
                  },
                  {
                    programAddress: this.program.address,
                  }
                )
              : restaking.getFundManagerUpdateSolStrategyInstructionAsync(
                  {
                    ...(newArgs as restaking.FundManagerUpdateSolStrategyInstructionDataArgs),
                    fundManager: createNoopSigner(fundManager),
                    receiptTokenMint: data.receiptTokenMint,
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

  public readonly __tokenSwapStrategiesDiscriminantMap = this.__memoized(
    'tokenSwapStrategiesDiscriminantMap',
    () => {
      // hacky-way to calculate discriminant map to convert POD to Borsh compatible args.
      // note that: it assumes the order of discriminants are same in POD and Borsh types.
      const map = new Map<number, restaking.TokenSwapSource['__kind']>();
      const decoder = restaking.getTokenSwapSourceDecoder();
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

  async resolveTokenSwapStrategies(
    noCache = false
  ): Promise<
    restaking.FundManagerAddTokenSwapStrategyInstructionDataArgs[] | null
  > {
    const fund = await this.resolveAccount(noCache);
    if (!fund) return null;

    return fund.data.tokenSwapStrategies
      .slice(0, fund.data.numTokenSwapStrategies)
      .map((item) => {
        const strategy: restaking.FundManagerAddTokenSwapStrategyInstructionDataArgs =
          {
            swapSource: {
              __kind: this.__tokenSwapStrategiesDiscriminantMap.get(
                item.swapSource.discriminant
              )!,
              address: item.swapSource.address,
            },
          };
        return strategy;
      });
  }

  readonly addTokenSwapStrategy = new TransactionTemplateContext(
    this,
    v.object({
      fromTokenMint: v.string(),
      toTokenMint: v.string(),
      swapSource: v.pipe(
        v.object({
          __kind: v.picklist(['OrcaDEXLiquidityPool']),
          address: v.string(),
        }) as v.GenericSchema<
          Omit<restaking.TokenSwapSourceArgs, 'address'> & {
            address: string;
          }
        >
      ),
    }),
    {
      description: 'add a new token swap strategy',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedFund'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [data, payer] = await Promise.all([
            parent.parent.resolve(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!data) throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          return Promise.all([
            restaking.getFundManagerAddTokenSwapStrategyInstructionAsync(
              {
                fromTokenMint: args.fromTokenMint as Address,
                toTokenMint: args.toTokenMint as Address,
                swapSourceAccount: args.swapSource.address as Address,
                swapSource: args.swapSource as restaking.TokenSwapSourceArgs,
                fundManager: createNoopSigner(fundManager),
                receiptTokenMint: data.receiptTokenMint,
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

  readonly removeTokenSwapStrategy = new TransactionTemplateContext(
    this,
    v.object({
      fromTokenMint: v.string(),
      toTokenMint: v.string(),
      swapSource: v.pipe(
        v.object({
          __kind: v.picklist(['OrcaDEXLiquidityPool']),
          address: v.string(),
        }) as v.GenericSchema<
          Omit<restaking.TokenSwapSourceArgs, 'address'> & {
            address: string;
          }
        >
      ),
    }),
    {
      description: 'remove a token swap strategy',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedFund'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [data, payer] = await Promise.all([
            parent.parent.resolve(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!data) throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          return Promise.all([
            restaking.getFundManagerRemoveTokenSwapStrategyInstructionAsync(
              {
                fromTokenMint: args.fromTokenMint as Address,
                toTokenMint: args.toTokenMint as Address,
                swapSourceAccount: args.swapSource.address as Address,
                swapSource: args.swapSource as restaking.TokenSwapSourceArgs,
                fundManager: createNoopSigner(fundManager),
                receiptTokenMint: data.receiptTokenMint,
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

  async resolveRestakingVaultStrategies(noCache = false): Promise<
    | (restaking.FundManagerUpdateRestakingVaultStrategyInstructionDataArgs & {
        pricingSource: restaking.TokenPricingSourceArgs;
        compoundingRewardTokens: Omit<restaking.RewardTokenArgs, 'reserved'>[];
        distributingRewardTokens: Omit<restaking.RewardTokenArgs, 'reserved'>[];
        delegations: Omit<
          restaking.FundManagerUpdateRestakingVaultDelegationStrategyInstructionDataArgs,
          'vault'
        >[];
      })[]
    | null
  > {
    const fund = await this.resolveAccount(noCache);
    if (!fund) return null;

    return fund.data.restakingVaults
      .slice(0, fund.data.numRestakingVaults)
      .map((item) => {
        const strategy: restaking.FundManagerUpdateRestakingVaultStrategyInstructionDataArgs & {
          pricingSource: restaking.TokenPricingSourceArgs;
          compoundingRewardTokens: Omit<
            restaking.RewardTokenArgs,
            'reserved'
          >[];
          distributingRewardTokens: Omit<
            restaking.RewardTokenArgs,
            'reserved'
          >[];
          delegations: Omit<
            restaking.FundManagerUpdateRestakingVaultDelegationStrategyInstructionDataArgs,
            'vault'
          >[];
        } = {
          vault: item.vault,
          pricingSource: {
            __kind: this.parent.__tokenPricingSourceDiscriminantMap.get(
              item.receiptTokenPricingSource.discriminant
            )!,
            address: item.receiptTokenPricingSource.address,
          },
          solAllocationWeight: item.solAllocationWeight,
          solAllocationCapacityAmount: item.solAllocationCapacityAmount,
          delegations: item.delegations
            .slice(0, item.numDelegations)
            .map((delegationItem) => {
              const delegationStrategy: Omit<
                restaking.FundManagerUpdateRestakingVaultDelegationStrategyInstructionDataArgs,
                'vault'
              > = {
                operator: delegationItem.operator,
                tokenAllocationWeight:
                  delegationItem.supportedTokenAllocationWeight,
                tokenAllocationCapacityAmount:
                  delegationItem.supportedTokenAllocationCapacityAmount,
                tokenRedelegatingAmount:
                  delegationItem.supportedTokenRedelegatingAmount,
              };
              return delegationStrategy;
            }),
          compoundingRewardTokens: item.compoundingRewardTokens
            .slice(0, item.numCompoundingRewardTokens)
            .map((compoundingRewardTokenItem) => {
              const { reserved, ...compoundingRewardToken } =
                compoundingRewardTokenItem;
              return compoundingRewardToken;
            }),
          distributingRewardTokens: item.distributingRewardTokens
            .slice(0, item.numDistributingRewardTokens)
            .map((distributingRewardTokenItem) => {
              const { reserved, ...distributingRewardToken } =
                distributingRewardTokenItem;
              return distributingRewardToken;
            }),
        };
        return strategy;
      });
  }

  readonly __solvBTCVaultProgram = this.__memoized('solv', () => {
    return SolvBTCVaultProgram.connect(this.runtime.config);
  });

  readonly addRestakingVault = new TransactionTemplateContext(
    this,
    v.object({
      vault: v.string(),
      pricingSource: v.pipe(
        v.object({
          __kind: v.picklist([
            'JitoRestakingVault',
            'SolvBTCVault',
            'VirtualVault',
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
      description: 'add a new restaking vault',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedFund'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [data, fund, payer] = await Promise.all([
            parent.parent.resolve(true),
            parent.resolveAddress(),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!(data && fund)) throw new Error('invalid context');
          const admin = (this.program as RestakingProgram).knownAddresses.admin;
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          if (args.pricingSource.__kind == 'JitoRestakingVault') {
            const vaultContext = parent.restakingVault(
              args.vault,
              jitoVault.JITO_VAULT_PROGRAM_ADDRESS
            );
            if (
              !(vaultContext && vaultContext instanceof JitoVaultAccountContext)
            ) {
              throw new Error('invalid context: jito vault not found');
            }
            const vaultAccount = await vaultContext?.resolveAccount(true);
            if (!vaultAccount) {
              throw new Error('invalid context: jito vault account not found');
            }

            const ix =
              await restaking.getFundManagerInitializeFundRestakingVaultInstructionAsync(
                {
                  vaultAccount: vaultAccount.address,
                  vaultReceiptTokenMint: vaultAccount.data.vrtMint,
                  vaultSupportedTokenMint: vaultAccount.data.supportedMint,
                  fundManager: createNoopSigner(fundManager),
                  receiptTokenMint: data.receiptTokenMint,
                  pricingSource:
                    args.pricingSource as restaking.TokenPricingSourceArgs,
                  program: this.program.address,
                },
                {
                  programAddress: this.program.address,
                }
              );
            for (const accountMeta of data.__pricingSources) {
              ix.accounts.push(accountMeta);
            }
            ix.accounts.push({
              address: args.pricingSource.address as Address,
              role: AccountRole.READONLY,
            });

            const fundReserve = ix.accounts[3].address;
            const vaultFeeWallet = vaultAccount.data.feeWallet;
            const vaultProgramFeeWallet =
              vaultContext.knownAddresses.programFeeWallet;

            return Promise.all([
              token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: createNoopSigner(payer as Address),
                mint: vaultAccount.data.vrtMint,
                owner: fundReserve,
                tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
              }),
              token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: createNoopSigner(payer as Address),
                mint: vaultAccount.data.vrtMint,
                owner: vaultFeeWallet,
                tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
              }),
              token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: createNoopSigner(payer as Address),
                mint: vaultAccount.data.vrtMint,
                owner: vaultProgramFeeWallet,
                tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
              }),
              token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: createNoopSigner(payer as Address),
                mint: vaultAccount.data.supportedMint,
                owner: vaultAccount.address,
                tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
              }),
              jitoVault.getSetSecondaryAdminInstruction({
                config: vaultContext.knownAddresses.config,
                vault: vaultAccount.address,
                admin: createNoopSigner(fundManager),
                newAdmin: fund,
                vaultAdminRole: jitoVault.VaultAdminRole.DelegationAdmin,
              }),
              ix,
            ]);
          } else if (args.pricingSource.__kind == 'SolvBTCVault') {
            const vaultContext = parent.restakingVault(
              args.vault,
              this.__solvBTCVaultProgram.address
            );
            if (
              !(vaultContext && vaultContext instanceof SolvVaultAccountContext)
            ) {
              throw new Error('invalid context: solv vault not found');
            }
            const vaultAccount = await vaultContext?.resolveAccount(true);
            if (!vaultAccount) {
              throw new Error('invalid context: solv vault account not found');
            }

            const ix =
              await restaking.getFundManagerInitializeFundRestakingVaultInstructionAsync(
                {
                  vaultAccount: vaultAccount.address,
                  vaultReceiptTokenMint:
                    vaultAccount.data.vaultReceiptTokenMint,
                  vaultSupportedTokenMint:
                    vaultAccount.data.vaultSupportedTokenMint,
                  fundManager: createNoopSigner(fundManager),
                  receiptTokenMint: data.receiptTokenMint,
                  pricingSource:
                    args.pricingSource as restaking.TokenPricingSourceArgs,
                  program: this.program.address,
                },
                {
                  programAddress: this.program.address,
                }
              );
            for (const accountMeta of data.__pricingSources) {
              ix.accounts.push(accountMeta);
            }
            ix.accounts.push({
              address: args.pricingSource.address as Address,
              role: AccountRole.READONLY,
            });

            const fundReserve = ix.accounts[3].address;
            return Promise.all([
              token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: createNoopSigner(payer as Address),
                mint: vaultAccount.data.vaultReceiptTokenMint,
                owner: fundReserve,
                tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
              }),
              token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: createNoopSigner(payer as Address),
                mint: vaultAccount.data.vaultSupportedTokenMint,
                owner: args.vault as Address,
                tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
              }),
              solv.getUpdateVaultAdminRoleInstructionAsync(
                {
                  oldVaultAdmin: createNoopSigner(admin),
                  newVaultAdmin: fund,
                  vaultReceiptTokenMint:
                    vaultAccount.data.vaultReceiptTokenMint,
                  program: vaultAccount.programAddress,
                  role: solv.VaultAdminRole.FundManager,
                },
                {
                  programAddress: vaultAccount.programAddress,
                }
              ),
              ix,
            ]);
          } else if (args.pricingSource.__kind == 'VirtualVault') {
            const vaultContext = parent.restakingVault(
              null,
              system.SYSTEM_PROGRAM_ADDRESS
            );
            const vault = (await vaultContext?.resolveAddress())!;
            const vrtMint =
              (await vaultContext?.receiptTokenMint.resolveAddress())!;
            if (
              !(
                vault &&
                vrtMint &&
                vaultContext &&
                vaultContext instanceof VirtualVaultAccountContext
              )
            ) {
              throw new Error('invalid context: virtual vault not found');
            }
            if (args.vault != vault || args.pricingSource.address != vault) {
              throw new Error(
                'invalid context: virtual vault address is deterministic: use ' +
                  vault
              );
            }

            const ix =
              await restaking.getFundManagerInitializeFundRestakingVaultInstructionAsync(
                {
                  vaultAccount: vault,
                  vaultReceiptTokenMint: vrtMint,
                  vaultSupportedTokenMint: vrtMint, // use same VRRT - notes no cash-in flow
                  fundManager: createNoopSigner(fundManager),
                  receiptTokenMint: data.receiptTokenMint,
                  pricingSource:
                    args.pricingSource as restaking.TokenPricingSourceArgs,
                  program: this.program.address,
                },
                {
                  programAddress: this.program.address,
                }
              );
            for (const accountMeta of data.__pricingSources) {
              ix.accounts.push(accountMeta);
            }
            ix.accounts.push({
              address: vault,
              role: AccountRole.READONLY,
            });

            const fundReserve = ix.accounts[3].address;

            const rent = await this.runtime.rpc
              .getMinimumBalanceForRentExemption(BigInt(0n))
              .send();
            return Promise.all([
              // to make the vault account look alive
              system.getTransferSolInstruction({
                source: createNoopSigner(payer as Address),
                destination: vault,
                amount: rent,
              }),
              token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: createNoopSigner(payer as Address),
                mint: vrtMint,
                owner: fundReserve,
                tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
              }),
              token.getCreateAssociatedTokenIdempotentInstructionAsync({
                payer: createNoopSigner(payer as Address),
                mint: vrtMint,
                owner: vault,
                tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
              }),
              ix,
            ]);
          }

          throw new Error('unsupported restaking vault pricing source');
        },
      ],
    }
  );

  readonly addRestakingVaultDelegation = new TransactionTemplateContext(
    this,
    v.object({
      vault: v.string(),
      operator: v.string(),
    }),
    {
      description: 'add a new operator delegation to a restaking vault',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedFund'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [data, vaultStrategies, payer] = await Promise.all([
            parent.parent.resolve(true),
            parent.resolveRestakingVaultStrategies(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          const vaultStrategy = vaultStrategies?.find(
            (item) => item.vault == args.vault
          );
          if (!(data && vaultStrategy)) throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          if (vaultStrategy.pricingSource.__kind == 'JitoRestakingVault') {
            const [[operatorVaultTicket], [vaultOperatorDelegation]] =
              await Promise.all([
                getProgramDerivedAddress({
                  programAddress: jitoRestaking.JITO_RESTAKING_PROGRAM_ADDRESS,
                  seeds: [
                    getBytesEncoder().encode(
                      Buffer.from('operator_vault_ticket')
                    ),
                    getAddressEncoder().encode(args.operator as Address),
                    getAddressEncoder().encode(args.vault as Address),
                  ],
                }),
                getProgramDerivedAddress({
                  programAddress: jitoVault.JITO_VAULT_PROGRAM_ADDRESS,
                  seeds: [
                    getBytesEncoder().encode(
                      Buffer.from('vault_operator_delegation')
                    ),
                    getAddressEncoder().encode(args.vault as Address),
                    getAddressEncoder().encode(args.operator as Address),
                  ],
                }),
              ]);

            // console.log({ operatorVaultTicket });

            return Promise.all([
              jitoVault.getInitializeVaultOperatorDelegationInstruction({
                admin: createNoopSigner(fundManager),
                config: JitoVaultAccountContext.knownAddresses(
                  this.runtime.cluster
                ).config,
                vault: args.vault as Address,
                operator: args.operator as Address,
                operatorVaultTicket,
                payer: createNoopSigner(payer as Address),
                vaultOperatorDelegation,
              }),
              restaking.getFundManagerInitializeFundRestakingVaultDelegationInstructionAsync(
                {
                  vaultOperatorDelegation,
                  vaultAccount: args.vault as Address,
                  operatorAccount: args.operator as Address,
                  fundManager: createNoopSigner(fundManager),
                  program: this.program.address,
                  receiptTokenMint: data.receiptTokenMint,
                },
                {
                  programAddress: this.program.address,
                }
              ),
            ]);
          }

          throw new Error('unsupported restaking vault pricing source');
        },
      ],
    }
  );

  readonly updateRestakingVaultStrategy = new TransactionTemplateContext(
    this,
    v.intersect([
      v.object({
        vault: v.string(),
      }),
      v.partial(
        v.object({
          solAllocationWeight: v.bigint(),
          solAllocationCapacityAmount: v.bigint(),
          delegations: v.array(
            v.intersect([
              v.object({
                operator: v.string(),
              }),
              v.partial(
                v.object({
                  tokenAllocationWeight: v.bigint(),
                  tokenAllocationCapacityAmount: v.bigint(),
                  tokenRedelegatingAmount: v.nullish(v.bigint(), null),
                })
              ),
            ])
          ),
        }) as v.GenericSchema<
          Omit<
            restaking.FundManagerUpdateRestakingVaultStrategyInstructionDataArgs,
            'vault'
          > & {
            delegations: ({ operator: string } & Partial<
              Omit<
                restaking.FundManagerUpdateRestakingVaultDelegationStrategyInstructionDataArgs,
                'vault' | 'operator'
              >
            >)[];
          }
        > as unknown as v.StrictObjectSchema<any, any>
      ) as v.GenericSchema<
        Partial<
          Omit<
            restaking.FundManagerUpdateRestakingVaultStrategyInstructionDataArgs,
            'vault'
          > & {
            delegations: ({ operator: string } & Partial<
              Omit<
                restaking.FundManagerUpdateRestakingVaultDelegationStrategyInstructionDataArgs,
                'vault' | 'operator'
              >
            >)[];
          }
        >
      >,
    ]),
    {
      description: 'update restaking vault strategy of the fund',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedFund'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [data, current, payer] = await Promise.all([
            parent.parent.resolve(true),
            parent.resolveRestakingVaultStrategies(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!(data && current)) throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          const { delegations: partialDelegations, ...partialArgs } = args;
          const {
            pricingSource,
            compoundingRewardTokens,
            distributingRewardTokens,
            delegations: currentDelegationStrategies,
            ...currentVaultStrategy
          } = current.find((item) => item.vault == args.vault)!;
          const newVaultStrategy =
            Object.keys(partialArgs).length > 1
              ? {
                  ...currentVaultStrategy,
                  ...partialArgs,
                  vault: partialArgs.vault as Address,
                }
              : null;
          const newDelegationStrategies =
            partialDelegations?.slice(0, 6).map((newDelegation) => {
              const currentDelegation = currentDelegationStrategies.find(
                (item) => item.operator == newDelegation.operator
              )!;
              return {
                ...currentDelegation,
                tokenRedelegatingAmount: null,
                ...newDelegation,
                operator: newDelegation.operator as Address,
                vault: partialArgs.vault as Address,
              };
            }) ?? [];

          return Promise.all([
            getSetComputeUnitLimitInstruction({ units: 1_400_000 }),
            newVaultStrategy
              ? restaking.getFundManagerUpdateRestakingVaultStrategyInstructionAsync(
                  {
                    ...newVaultStrategy,
                    fundManager: createNoopSigner(fundManager),
                    program: this.program.address,
                    receiptTokenMint: data.receiptTokenMint,
                  },
                  {
                    programAddress: this.program.address,
                  }
                )
              : null,
            ...newDelegationStrategies.map((newDelegationStrategy) => {
              return restaking.getFundManagerUpdateRestakingVaultDelegationStrategyInstructionAsync(
                {
                  ...newDelegationStrategy,
                  fundManager: createNoopSigner(fundManager),
                  program: this.program.address,
                  receiptTokenMint: data.receiptTokenMint,
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
      const { delegations: partialDelegations, ...partialArgs } = args;
      if (partialDelegations?.length && partialDelegations?.length > 6) {
        return {
          args: {
            vault: partialArgs.vault,
            delegations: partialDelegations.slice(6),
          },
        };
      }
      return null;
    }
  );

  readonly addRestakingVaultCompoundingReward = new TransactionTemplateContext(
    this,
    v.object({
      vault: v.string(),
      rewardTokenMint: v.string(),
    }),
    {
      description: 'add a new compounding reward to a restaking vault',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedFund'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [data, payer] = await Promise.all([
            parent.parent.resolve(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!data) throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          return Promise.all([
            restaking.getFundManagerAddRestakingVaultCompoundingRewardTokenInstructionAsync(
              {
                vault: args.vault as Address,
                compoundingRewardTokenMint: args.rewardTokenMint as Address,
                fundManager: createNoopSigner(fundManager),
                program: this.program.address,
                receiptTokenMint: data.receiptTokenMint,
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

  readonly removeRestakingVaultCompoundingReward =
    new TransactionTemplateContext(
      this,
      v.object({
        vault: v.string(),
        rewardTokenMint: v.string(),
      }),
      {
        description: 'remove a compounding reward from a restaking vault',
        anchorEventDecoders: getRestakingAnchorEventDecoders(
          'fundManagerUpdatedFund'
        ),
        addressLookupTables: [this.__resolveAddressLookupTable],
        instructions: [
          async (parent, args, overrides) => {
            const [data, payer] = await Promise.all([
              parent.parent.resolve(true),
              transformAddressResolverVariant(
                overrides.feePayer ??
                  this.runtime.options.transaction.feePayer ??
                  (() => Promise.resolve(null))
              )(parent),
            ]);
            if (!data) throw new Error('invalid context');
            const fundManager = (this.program as RestakingProgram)
              .knownAddresses.fundManager;

            return Promise.all([
              restaking.getFundManagerRemoveRestakingVaultCompoundingRewardTokenInstructionAsync(
                {
                  vault: args.vault as Address,
                  compoundingRewardTokenMint: args.rewardTokenMint as Address,
                  fundManager: createNoopSigner(fundManager),
                  program: this.program.address,
                  receiptTokenMint: data.receiptTokenMint,
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

  readonly addRestakingVaultDistributingReward = new TransactionTemplateContext(
    this,
    v.object({
      vault: v.string(),
      rewardTokenMint: v.string(),
    }),
    {
      description: 'add a new distributing reward to a restaking vault',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedFund'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [data, payer] = await Promise.all([
            parent.parent.resolve(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          if (!data) throw new Error('invalid context');
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          return Promise.all([
            restaking.getFundManagerAddRestakingVaultDistributingRewardTokenInstructionAsync(
              {
                vault: args.vault as Address,
                distributingRewardTokenMint: args.rewardTokenMint as Address,
                fundManager: createNoopSigner(fundManager),
                program: this.program.address,
                receiptTokenMint: data.receiptTokenMint,
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

  readonly updateRestakingVaultRewardHarvestThreshold =
    new TransactionTemplateContext(
      this,
      v.object({
        vault: v.string(),
        rewardTokenMint: v.string(),
        harvestThresholdMinAmount: v.bigint(),
        harvestThresholdMaxAmount: v.bigint(),
        harvestThresholdIntervalSeconds: v.bigint(),
      }),
      {
        description: 'update reward token threshold',
        anchorEventDecoders: getRestakingAnchorEventDecoders(
          'fundManagerUpdatedFund'
        ),
        addressLookupTables: [this.__resolveAddressLookupTable],
        instructions: [
          async (parent, args, overrides) => {
            const [data, payer] = await Promise.all([
              parent.parent.resolve(true),
              transformAddressResolverVariant(
                overrides.feePayer ??
                  this.runtime.options.transaction.feePayer ??
                  (() => Promise.resolve(null))
              )(parent),
            ]);
            if (!data) throw new Error('invalid context');
            const fundManager = (this.program as RestakingProgram)
              .knownAddresses.fundManager;

            return Promise.all([
              restaking.getFundManagerUpdateRestakingVaultRewardTokenHarvestThresholdInstructionAsync(
                {
                  vault: args.vault as Address,
                  rewardTokenMint: args.rewardTokenMint as Address,
                  harvestThresholdMinAmount: args.harvestThresholdMinAmount,
                  harvestThresholdMaxAmount: args.harvestThresholdMaxAmount,
                  harvestThresholdIntervalSeconds:
                    args.harvestThresholdIntervalSeconds,
                  fundManager: createNoopSigner(fundManager),
                  program: this.program.address,
                  receiptTokenMint: data.receiptTokenMint,
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

  readonly removeRestakingVaultDistributingReward =
    new TransactionTemplateContext(
      this,
      v.object({
        vault: v.string(),
        rewardTokenMint: v.string(),
      }),
      {
        description: 'remove a distributing reward from a restaking vault',
        anchorEventDecoders: getRestakingAnchorEventDecoders(
          'fundManagerUpdatedFund'
        ),
        addressLookupTables: [this.__resolveAddressLookupTable],
        instructions: [
          async (parent, args, overrides) => {
            const [data, payer] = await Promise.all([
              parent.parent.resolve(true),
              transformAddressResolverVariant(
                overrides.feePayer ??
                  this.runtime.options.transaction.feePayer ??
                  (() => Promise.resolve(null))
              )(parent),
            ]);
            if (!data) throw new Error('invalid context');
            const fundManager = (this.program as RestakingProgram)
              .knownAddresses.fundManager;

            return Promise.all([
              restaking.getFundManagerRemoveRestakingVaultDistributingRewardTokenInstructionAsync(
                {
                  vault: args.vault as Address,
                  distributingRewardTokenMint: args.rewardTokenMint as Address,
                  fundManager: createNoopSigner(fundManager),
                  program: this.program.address,
                  receiptTokenMint: data.receiptTokenMint,
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

  readonly initializeNormalizedToken = new TransactionTemplateContext(
    this,
    v.object({
      mint: v.string(),
    }),
    {
      description: 'initialize normalized token pool account and enable',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedFund'
      ),
      addressLookupTables: [this.__resolveAddressLookupTable],
      instructions: [
        async (parent, args, overrides) => {
          const [existingNormalizedTokenMint, data, fundReserve, payer] =
            await Promise.all([
              parent.parent.normalizedTokenMint.resolveAddress(true),
              parent.parent.resolve(true),
              parent.parent.fund.reserve.resolveAddress(),
              transformAddressResolverVariant(
                overrides.feePayer ??
                  this.runtime.options.transaction.feePayer ??
                  (() => Promise.resolve(null))
              )(parent),
            ]);
          if (!(!existingNormalizedTokenMint && data && fundReserve))
            throw new Error('invalid context');
          const admin = (this.program as RestakingProgram).knownAddresses.admin;
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          return Promise.all([
            restaking.getAdminInitializeNormalizedTokenPoolAccountInstructionAsync(
              {
                payer: createNoopSigner(payer as Address),
                admin: createNoopSigner(admin),
                program: this.program.address,
                normalizedTokenMint: args.mint as Address,
              },
              {
                programAddress: this.program.address,
              }
            ),
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(payer as Address),
              mint: args.mint as Address,
              owner: fundReserve,
              tokenProgram: token.TOKEN_PROGRAM_ADDRESS,
            }),
            restaking
              .getFundManagerInitializeFundNormalizedTokenInstructionAsync(
                {
                  fundManager: createNoopSigner(fundManager),
                  receiptTokenMint: data.receiptTokenMint,
                  program: this.program.address,
                  normalizedTokenMint: args.mint as Address,
                },
                {
                  programAddress: this.program.address,
                }
              )
              .then((ix) => {
                // add pricing sources
                for (const accountMeta of data.__pricingSources) {
                  ix.accounts.push(accountMeta);
                }
                ix.accounts.push(ix.accounts[7]); // ntp
                return ix;
              }),
          ]);
        },
      ],
    }
  );

  readonly initializeWrappedToken = new TransactionTemplateContext(
    this,
    v.object({
      mint: v.string(),
    }),
    {
      description: 'enable wrapped token',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'fundManagerUpdatedFund'
      ),
      instructions: [
        async (parent, args, overrides) => {
          const [existingWrappedTokenMint, receiptTokenMint, payer] =
            await Promise.all([
              parent.parent.wrappedTokenMint.resolveAddress(true),
              parent.parent.resolveAddress(),
              transformAddressResolverVariant(
                overrides.feePayer ??
                  this.runtime.options.transaction.feePayer ??
                  (() => Promise.resolve(null))
              )(parent),
            ]);
          if (!(!existingWrappedTokenMint && receiptTokenMint))
            throw new Error('invalid context');
          const admin = (this.program as RestakingProgram).knownAddresses.admin;
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          const ix =
            await restaking.getFundManagerInitializeFundWrappedTokenInstructionAsync(
              {
                admin: createNoopSigner(admin),
                fundManager: createNoopSigner(fundManager),
                receiptTokenMint: receiptTokenMint,
                program: this.program.address,
                wrappedTokenMint: args.mint as Address,
              },
              {
                programAddress: this.program.address,
              }
            );
          const fundWrapAccount = ix.accounts[2].address;

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(payer! as Address),
              mint: receiptTokenMint,
              owner: fundWrapAccount,
              tokenProgram: token2022.TOKEN_2022_PROGRAM_ADDRESS,
            }),
            restaking.getAdminCreateUserRewardAccountIdempotentInstructionAsync(
              {
                admin: createNoopSigner(admin),
                payer: createNoopSigner(payer! as Address),
                user: fundWrapAccount,
                program: this.program.address,
                receiptTokenMint: receiptTokenMint,
                desiredAccountSize: null,
              },
              {
                programAddress: this.program.address,
              }
            ),
            ix,
          ]);
        },
      ],
    }
  );

  readonly initializeWrappedTokenHolder = new TransactionTemplateContext(
    this,
    v.object({
      wrappedTokenAccount: v.pipe(
        v.string(),
        v.description(
          'any wrapped token account to isolate contribution accruals independently'
        )
      ),
    }),
    {
      description: 'add new wrapped token holder',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'userCreatedOrUpdatedRewardAccount',
        'fundManagerUpdatedFund'
      ),
      instructions: [
        async (parent, args, overrides) => {
          const [wrappedTokenMint, receiptTokenMint, payer] = await Promise.all(
            [
              parent.parent.wrappedTokenMint.resolveAddress(true),
              parent.parent.resolveAddress(),
              transformAddressResolverVariant(
                overrides.feePayer ??
                  this.runtime.options.transaction.feePayer ??
                  (() => Promise.resolve(null))
              )(parent),
            ]
          );
          if (!(wrappedTokenMint && receiptTokenMint))
            throw new Error('invalid context');
          const admin = (this.program as RestakingProgram).knownAddresses.admin;
          const fundManager = (this.program as RestakingProgram).knownAddresses
            .fundManager;

          return Promise.all([
            token2022.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(payer! as Address),
              mint: receiptTokenMint,
              owner: args.wrappedTokenAccount as Address,
              tokenProgram: token2022.TOKEN_2022_PROGRAM_ADDRESS,
            }),
            restaking.getAdminCreateUserRewardAccountIdempotentInstructionAsync(
              {
                admin: createNoopSigner(admin),
                payer: createNoopSigner(payer! as Address),
                user: args.wrappedTokenAccount as Address,
                receiptTokenMint: receiptTokenMint,
                desiredAccountSize: null,
                program: this.program.address,
              },
              {
                programAddress: this.program.address,
              }
            ),
            restaking.getFundManagerAddWrappedTokenHolderInstructionAsync(
              {
                fundManager: createNoopSigner(fundManager),
                receiptTokenMint: receiptTokenMint,
                wrappedTokenMint: wrappedTokenMint,
                wrappedTokenHolder: args.wrappedTokenAccount as Address,
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
}
