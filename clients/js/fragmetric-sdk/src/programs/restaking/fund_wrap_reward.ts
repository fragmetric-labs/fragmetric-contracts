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
import { RestakingFundWrapAccountContext } from './fund_wrap';
import { RestakingProgram } from './program';
import { RestakingUserRewardAccountContext } from './user_reward';

export class RestakingFundWrapAccountRewardAccountContext extends AccountContext<
  RestakingFundWrapAccountContext,
  Account<restaking.UserRewardAccount>
> {
  protected __decodeAccount(account: EncodedAccount) {
    return restaking.decodeUserRewardAccount(account);
  }

  readonly resolve =
    RestakingUserRewardAccountContext.prototype.resolve.bind(this);

  constructor(readonly parent: RestakingFundWrapAccountContext) {
    super(parent, async (parent) => {
      const [owner, receiptTokenMint] = await Promise.all([
        parent.resolveAddress(true),
        parent.parent.parent.resolveAddress(),
      ]);
      if (owner && receiptTokenMint) {
        const ix =
          await restaking.getUserCreateRewardAccountIdempotentInstructionAsync(
            { user: { address: owner }, receiptTokenMint } as any,
            { programAddress: parent.program.address }
          );
        return ix.accounts[6].address;
      }
      return null;
    });
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
          const [receiptTokenMint, owner] = await Promise.all([
            parent.parent.parent.parent.resolveAddress(),
            parent.parent.resolveAddress(true),
          ]);
          if (!(receiptTokenMint && owner)) throw new Error('invalid context');

          return Promise.all([
            restaking.getUserUpdateRewardPoolsInstructionAsync(
              {
                user: createNoopSigner(owner),
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

  // TODO [sdk]: RestakingFundWrapAccountRewardAccountContext should support arbitrary reward accounts of PDAs; including fund wrap and DeFi pools' token accounts.
  readonly initializeOrUpdateAccount = new TransactionTemplateContext(
    this,
    v.pipe(v.nullish(v.null(), null), v.description('no args required')),
    {
      description: `initialize or update a PDA's reward account`,
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'userCreatedOrUpdatedRewardAccount'
      ),
      instructions: [
        // TODO [sdk]: update to support defi pools
        async (parent, args, overrides) => {
          const [receiptTokenMint, current, payer] = await Promise.all([
            parent.parent.parent.parent.resolveAddress(),
            parent.resolveAccount(true),
            transformAddressResolverVariant(
              overrides.feePayer ??
                this.runtime.options.transaction.feePayer ??
                (() => Promise.resolve(null))
            )(parent),
          ]);
          let owner =
            current?.address ??
            ((await parent.parent.resolveAddress(true)) as Address);
          if (!(receiptTokenMint && owner)) throw new Error('invalid context');
          const admin = (this.program as RestakingProgram).knownAddresses.admin;

          return Promise.all([
            token.getCreateAssociatedTokenIdempotentInstructionAsync({
              payer: createNoopSigner(payer! as Address),
              mint: receiptTokenMint,
              owner: owner,
              tokenProgram: token2022.TOKEN_2022_PROGRAM_ADDRESS,
            }),
            current?.data
              ? restaking.getAdminUpdateFundWrapAccountRewardAccountIfNeededInstructionAsync(
                  {
                    payer: createNoopSigner(payer! as Address),
                    admin: createNoopSigner(admin),
                    fundWrapAccount: owner,
                    program: this.program.address,
                    receiptTokenMint: receiptTokenMint,
                    desiredAccountSize: null,
                  },
                  {
                    programAddress: this.program.address,
                  }
                )
              : restaking.getAdminInitializeFundWrapAccountRewardAccountInstructionAsync(
                  {
                    payer: createNoopSigner(payer! as Address),
                    admin: createNoopSigner(admin),
                    fundWrapAccount: owner,
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
