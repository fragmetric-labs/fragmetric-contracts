use anchor_lang::prelude::*;

use super::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct EnqueueWithdrawalBatchCommand {
    state: EnqueueWithdrawalBatchCommandState,
    forced: bool,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub enum EnqueueWithdrawalBatchCommandState {
    #[default]
    New,
    /// Unused state, just to ensure that any kind of XXXCommandState is generated as a ComplexEnum in client-side code generation.
    Unused { unused: bool },
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct EnqueueWithdrawalBatchCommandResult {
    pub enqueued_receipt_token_amount: u64,
    pub total_queued_receipt_token_amount: u64,
}

impl SelfExecutable for EnqueueWithdrawalBatchCommand {
    fn execute<'a, 'info>(
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
            Some(ClaimUnrestakedVSTCommand::default().without_required_accounts()),
        ))
    }
}
