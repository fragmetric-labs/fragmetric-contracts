import * as alt from '@solana-program/address-lookup-table';
import * as computeBudget from '@solana-program/compute-budget';
import * as system from '@solana-program/system';
import * as token from '@solana-program/token';
import * as token2022 from '@solana-program/token-2022';
import { Address, createNoopSigner } from '@solana/kit';
import * as v from 'valibot';
import {
  AddressLookupTableAccountContext,
  TransactionTemplateContext,
  transformAddressResolverVariant,
} from '../../context';
import * as restaking from '../../generated/restaking';
import { RestakingFundAccountContext } from './fund';
import { RestakingProgram } from './program';

export class RestakingFundAddressLookupTableAccountContext extends AddressLookupTableAccountContext<RestakingFundAccountContext> {
  constructor(readonly parent: RestakingFundAccountContext) {
    super(parent, async (parent) => {
      const fund = await parent.resolveAccount(true);
      if (!fund?.data.addressLookupTableEnabled) {
        return null;
      }
      return fund.data.addressLookupTableAccount;
    });
  }

  /** authorized transactions **/
  readonly initializeOrUpdateAccount = new TransactionTemplateContext(
    this,
    v.object({
      addresses: v.pipe(
        v.array(v.string()),
        v.minLength(1),
        v.description(`can extend up to 27 addresses for a single update`)
      ),
    }),
    {
      description: 'initialize or update address lookup table',
      instructions: [
        computeBudget.getSetComputeUnitLimitInstruction({ units: 1_400_000 }),
        async (parent, args, overrides) => {
          const [receiptTokenMint, existingAddressLookupTableAccount, payer] =
            await Promise.all([
              parent.parent.parent.resolveAddress(),
              parent.resolveAccount(true),
              transformAddressResolverVariant(
                overrides.feePayer ??
                  this.runtime.options.transaction.feePayer ??
                  (() => Promise.resolve(null))
              )(parent),
            ]);
          if (!receiptTokenMint) throw new Error('invalid context');
          const admin = (this.program as RestakingProgram).knownAddresses.admin;

          const newAddresses = new Set(args.addresses);
          if (existingAddressLookupTableAccount) {
            for (const address of existingAddressLookupTableAccount?.data
              .addresses ?? []) {
              newAddresses.delete(address);
            }
          }

          let altAddress = existingAddressLookupTableAccount?.address;
          return Promise.all([
            ...(altAddress
              ? [
                  newAddresses.size == 0
                    ? null
                    : alt.getExtendLookupTableInstruction({
                        address: altAddress,
                        authority: createNoopSigner(admin),
                        payer: createNoopSigner(payer! as Address),

                        // 20 (addresses) + 5 (authority, payer, alt_program, alt, system_program)
                        addresses: Array.from(newAddresses).slice(
                          0,
                          20
                        ) as Address[],
                      }),
                ]
              : await (async () => {
                  {
                    const recentSlot = await this.runtime.rpc
                      .getSlot({ commitment: 'finalized' })
                      .send();
                    const ix = await alt.getCreateLookupTableInstructionAsync({
                      payer: createNoopSigner(payer! as Address),
                      authority: createNoopSigner(admin),
                      recentSlot,
                    });
                    altAddress = ix.accounts[0].address;
                    return [
                      ix,
                      await restaking.getAdminSetAddressLookupTableAccountInstructionAsync(
                        {
                          admin: createNoopSigner(admin),
                          receiptTokenMint,
                          program: this.program.address,
                          addressLookupTableAccount: altAddress,
                        },
                        {
                          programAddress: this.program.address,
                        }
                      ),
                    ];
                  }
                })()),
          ]);
        },
      ],
    },
    async (parent, args, events) => {
      const remainingAddresses = new Set(args.addresses);
      const addressLookupTableAccount = await parent.resolveAccount(true);
      for (const address of addressLookupTableAccount?.data.addresses ?? []) {
        remainingAddresses.delete(address);
      }
      if (remainingAddresses.size) {
        return {
          args: { addresses: Array.from(remainingAddresses) },
        } as any;
      }
      return null;
    }
  );

  async resolveFrequentlyUsedAddresses() {
    const ctx = this.parent.parent;
    const addressesList = await Promise.all([
      // get accounts for major transactions
      Promise.all([
        ctx.payer.deposit.assemble({ assetMint: null, assetAmount: 0n }),
        ctx.payer.wrap.assemble({ receiptTokenAmount: 0n }).catch(() => null),
        ctx.fund.updatePrices.assemble(null),
        ctx.reward.updatePools.assemble(null),
        ctx.normalizedTokenPool.updatePrices.assemble(null).catch(() => null),
      ]).then((txs) =>
        txs
          .filter((tx) => !!tx)
          .flatMap((tx) =>
            tx.instructions.flatMap(
              (ix) => ix.accounts?.map((a) => a.address) ?? []
            )
          )
          .filter((address) => address != ctx.payer.address)
      ),

      // put some programs
      computeBudget.COMPUTE_BUDGET_PROGRAM_ADDRESS,
      token.TOKEN_PROGRAM_ADDRESS,
      token.ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
      token2022.TOKEN_2022_PROGRAM_ADDRESS,
      system.SYSTEM_PROGRAM_ADDRESS,
      'Ed25519SigVerify111111111111111111111111111',
      this.program.address,

      // get frequently used accounts
      Promise.all([
        ctx
          .resolve(true)
          .then((data) => data?.__pricingSources.map((a) => a.address)),
        ctx.fund.lockedReceiptToken.resolveAddress(true),
        ctx.fund.reserve.resolveAddress(true),
        ctx.fund.reserve.supportedTokens
          .resolveAccount(true)
          .then((accounts) => accounts?.map((a) => a?.address)),
        ctx.fund.reserve.normalizedToken.resolveAddress(true),
        ctx.fund.reserve.restakingVaultReceiptTokens
          .resolveAccount(true)
          .then((accounts) => accounts?.map((a) => a?.address)),
        ctx.fund.restakingVaults
          .resolveAccount(true)
          .then((accounts) =>
            accounts?.flatMap((a) => [a?.address, a?.programAddress])
          ),
      ]).then((results) =>
        results
          .filter((res) => !!res)
          .flatMap((res) => {
            if (Array.isArray(res)) {
              return res.filter((address) => !!address) as string[];
            }
            return res as string;
          })
      ),
    ]);
    return Array.from(new Set(addressesList.flat()).values()) as Address[];
  }
}
