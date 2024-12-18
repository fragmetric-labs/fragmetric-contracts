use anchor_lang::prelude::*;

use crate::modules::fund;

use super::cmd3_process_withdrawal_batch::ProcessWithdrawalBatchCommand;
use super::{
    OperationCommand, OperationCommandContext, OperationCommandEntry, OperationCommandResult,
    SelfExecutable,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct EnqueueWithdrawalBatchCommand {
    forced: bool,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct EnqueueWithdrawalBatchCommandResult {
    pub enqueued_receipt_token_amount: u64,
}

impl SelfExecutable for EnqueueWithdrawalBatchCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        _accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let enqueued_receipt_token_amount =
            fund::FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                .enqueue_withdrawal_batches(self.forced)?;

        Ok((
            Some(
                EnqueueWithdrawalBatchCommandResult {
                    enqueued_receipt_token_amount,
                }
                .into(),
            ),
            Some(ProcessWithdrawalBatchCommand::default().without_required_accounts()),
        ))
    }
}
