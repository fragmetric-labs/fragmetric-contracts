use anchor_lang::prelude::*;

use crate::modules::fund;

use super::{OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct EnqueueWithdrawalBatchCommand {
    state: EnqueueWithdrawalBatchCommandState,
    forced: bool,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum EnqueueWithdrawalBatchCommandState {
    #[default]
    Init,
    Enqueue,
}

impl SelfExecutable for EnqueueWithdrawalBatchCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        match self.state {
            EnqueueWithdrawalBatchCommandState::Init => {
                let mut command = self.clone();
                command.state = EnqueueWithdrawalBatchCommandState::Enqueue;

                return Ok(Some(
                    OperationCommand::EnqueueWithdrawalBatch(command).with_required_accounts(vec![
                        ctx.fund_account.find_receipt_token_program_address(),
                        ctx.fund_account.find_receipt_token_lock_account_address()?,
                    ]),
                ));
            }
            EnqueueWithdrawalBatchCommandState::Enqueue => {
                let [receipt_token_program, receipt_token_lock_account, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(ErrorCode::AccountNotEnoughKeys)?
                };

                let mut fund_service =
                    fund::FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;
                fund_service.enqueue_withdrawal_batch(
                    receipt_token_program,
                    receipt_token_lock_account,
                    remaining_accounts.iter().cloned(),
                    self.forced,
                )?;
            }
        }

        Ok(None)
    }
}
