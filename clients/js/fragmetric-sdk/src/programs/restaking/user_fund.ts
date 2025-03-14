import * as token from '@solana-program/token';
import * as token2022 from '@solana-program/token-2022';
import { Account, createNoopSigner, EncodedAccount } from '@solana/kit';
import * as v from 'valibot';
import { AccountContext, TransactionTemplateContext } from '../../context';
import * as restaking from '../../generated/restaking';
import { getRestakingAnchorEventDecoders } from './events';
import { RestakingUserAccountContext } from './user';

export class RestakingUserFundAccountContext extends AccountContext<
  RestakingUserAccountContext,
  Account<restaking.UserFundAccount>
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
      user,
      receiptTokenMint,
      receiptTokenAmount,
      reserved,
      withdrawalRequests,
      ...props
    } = account.data;
    return {
      user,
      // dataVersion,
      receiptTokenMint,
      receiptTokenAmountRecorded: receiptTokenAmount,
      withdrawalRequests: withdrawalRequests.map((req) => {
        const { createdAt, ...props } = req;
        return {
          ...props,
          createdAt: new Date(Number(createdAt) * 1000),
        };
      }),
      ...props,
    };
  }

  constructor(readonly parent: RestakingUserAccountContext) {
    super(parent, async (parent) => {
      const [user, receiptTokenMint] = await Promise.all([
        parent.resolveAddress(true),
        parent.parent.resolveAddress(),
      ]);
      if (user && receiptTokenMint) {
        const ix =
          await restaking.getUserCreateFundAccountIdempotentInstructionAsync(
            { user: { address: user }, receiptTokenMint } as any,
            { programAddress: parent.program.address }
          );
        return ix.accounts[5].address;
      }
      return null;
    });
  }

  protected __decodeAccount(account: EncodedAccount) {
    return restaking.decodeUserFundAccount(account);
  }

  readonly initializeOrUpdateAccount = new TransactionTemplateContext(
    this,
    v.pipe(v.nullish(v.null(), null), v.description('no args required')),
    {
      description: 'initialize or update user fund account',
      anchorEventDecoders: getRestakingAnchorEventDecoders(
        'userCreatedOrUpdatedFundAccount'
      ),
      instructions: [
        async (parent, args) => {
          const [receiptTokenMint, user] = await Promise.all([
            parent.parent.parent.resolveAddress(),
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
}
