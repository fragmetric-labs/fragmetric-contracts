use anchor_lang::prelude::*;

use super::{
    ClaimUnstakedSOLCommand, FundService, OperationCommandContext, OperationCommandEntry,
    OperationCommandResult, ProcessWithdrawalBatchCommand, SelfExecutable,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct EnqueueWithdrawalBatchCommand {
    forced: bool,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct EnqueueWithdrawalBatchCommandResult {
    pub enqueued_receipt_token_amount: u64,
    pub total_queued_receipt_token_amount: u64,
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
            FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                .enqueue_withdrawal_batches(self.forced)?;
        let total_queued_receipt_token_amount = ctx
            .fund_account
            .load()?
            .get_total_receipt_token_withdrawal_obligated_amount();

        Ok((
            if enqueued_receipt_token_amount > 0 {
                Some(
                    EnqueueWithdrawalBatchCommandResult {
                        enqueued_receipt_token_amount,
                        total_queued_receipt_token_amount,
                    }
                    .into(),
                )
            } else {
                None
            },
            // TODO/v0.4: transition to Some(ClaimUnrestakedVSTCommand::default().without_required_accounts()),
            Some(ClaimUnstakedSOLCommand::default().without_required_accounts()),
        ))
    }
}
