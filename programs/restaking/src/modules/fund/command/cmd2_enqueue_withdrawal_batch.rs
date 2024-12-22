use anchor_lang::prelude::*;

use crate::modules::fund;

use super::cmd9_process_withdrawal_batch::ProcessWithdrawalBatchCommand;
use super::{
    ClaimUnrestakedVSTCommand, OperationCommand, OperationCommandContext, OperationCommandEntry,
    OperationCommandResult, SelfExecutable,
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
        let mut fund_service = fund::FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;
        let enqueued_receipt_token_amount = fund_service.enqueue_withdrawal_batches(self.forced)?;
        let total_queued_receipt_token_amount =
            fund_service.get_total_receipt_token_withdrawal_obligated_amount()?;

        Ok((
            Some(
                EnqueueWithdrawalBatchCommandResult {
                    enqueued_receipt_token_amount,
                    total_queued_receipt_token_amount,
                }
                .into(),
            ),
            // TODO/v0.4: transition to Some(ClaimUnrestakedVSTCommand::default().without_required_accounts()),
            Some(ProcessWithdrawalBatchCommand::default().without_required_accounts()),
        ))
    }
}
