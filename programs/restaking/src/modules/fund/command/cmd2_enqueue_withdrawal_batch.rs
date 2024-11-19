use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};
use crate::modules::fund;
use crate::modules::fund::FundService;
use crate::utils::PDASeeds;
use anchor_lang::prelude::*;
use anchor_spl::token_2022;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct EnqueueWithdrawalBatchCommand {
    state: EnqueueWithdrawalBatchCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum EnqueueWithdrawalBatchCommandState {
    Init,
    Enqueue,
}

impl SelfExecutable for EnqueueWithdrawalBatchCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        match self.state {
            EnqueueWithdrawalBatchCommandState::Init => {
                let mut command = self.clone();
                command.state = EnqueueWithdrawalBatchCommandState::Enqueue;

                return Ok(Some(
                    OperationCommand::EnqueueWithdrawalBatch(command).with_required_accounts(vec![
                        ctx.fund_account.key(),
                        ctx.fund_account.find_receipt_token_program_address(),
                        ctx.fund_account
                            .find_receipt_token_lock_account_address()?
                            .0,
                    ]),
                ));
            }
            EnqueueWithdrawalBatchCommandState::Enqueue => {
                let [receipt_token_program, receipt_token_lock_account, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };

                let mut fund_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;
                fund_service.enqueue_withdrawal_batch(
                    receipt_token_program,
                    receipt_token_lock_account,
                    remaining_accounts,
                )?;
            }
        }

        Ok(None)
    }
}
