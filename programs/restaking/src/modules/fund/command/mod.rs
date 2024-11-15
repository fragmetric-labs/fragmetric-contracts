mod cmd0_initialize;
mod cmd10_normalize_lst;
mod cmd11_restake_vst;
mod cmd12_delegate_vst;
mod cmd1_claim_unstaked_sol;
mod cmd2_enqueue_withdrawal_batch;
mod cmd3_process_withdrawal_batch;
mod cmd4_claim_unrestaked_vst;
mod cmd5_denormalize_nt;
mod cmd6_undelegate_vst;
mod cmd7_unrestake_vrt;
mod cmd8_unstake_lst;
mod cmd9_stake_sol;

pub use cmd0_initialize::*;
pub use cmd10_normalize_lst::*;
pub use cmd11_restake_vst::*;
pub use cmd12_delegate_vst::*;
pub use cmd1_claim_unstaked_sol::*;
pub use cmd2_enqueue_withdrawal_batch::*;
pub use cmd3_process_withdrawal_batch::*;
pub use cmd4_claim_unrestaked_vst::*;
pub use cmd5_denormalize_nt::*;
pub use cmd6_undelegate_vst::*;
pub use cmd7_unrestake_vrt::*;
pub use cmd8_unstake_lst::*;
pub use cmd9_stake_sol::*;

use crate::modules::fund;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

// propagate common accounts and values to all commands
pub(super) struct OperationCommandContext<'info, 'a>
where
    'info: 'a,
{
    pub(super) receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    pub(super) fund_account: &'a mut Account<'info, fund::FundAccount>,
    pub(super) system_program: &'a Program<'info, System>,
}

// enum to hold all command variants
#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) enum OperationCommand {
    Initialize(InitializeCommand),
    ClaimUnstakedSOL(ClaimUnstakedSOLCommand),
    EnqueueWithdrawalBatch(EnqueueWithdrawalBatchCommand),
    ProcessWithdrawalBatch(ProcessWithdrawalBatchCommand),
    ClaimUnrestakedVST(ClaimUnrestakedVSTCommand),
    DenormalizeNT(DenormalizeNTCommand),
    UndelegateVST(UndelegateVSTCommand),
    UnrestakeVRT(UnrestakeVRTCommand),
    UnstakeLST(UnstakeLSTCommand),
    StakeSOL(StakeSOLCommand),
    NormalizeLST(NormalizeLSTCommand),
    RestakeVST(RestakeVSTCommand),
    DelegateVST(DelegateVSTCommand),
}

impl SelfExecutable for OperationCommand {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>> {
        match self {
            OperationCommand::Initialize(command) => command.execute(ctx, accounts),
            OperationCommand::ClaimUnstakedSOL(command) => command.execute(ctx, accounts),
            OperationCommand::EnqueueWithdrawalBatch(command) => command.execute(ctx, accounts),
            OperationCommand::ProcessWithdrawalBatch(command) => command.execute(ctx, accounts),
            OperationCommand::ClaimUnrestakedVST(command) => command.execute(ctx, accounts),
            OperationCommand::DenormalizeNT(command) => command.execute(ctx, accounts),
            OperationCommand::UndelegateVST(command) => command.execute(ctx, accounts),
            OperationCommand::UnrestakeVRT(command) => command.execute(ctx, accounts),
            OperationCommand::UnstakeLST(command) => command.execute(ctx, accounts),
            OperationCommand::StakeSOL(command) => command.execute(ctx, accounts),
            OperationCommand::NormalizeLST(command) => command.execute(ctx, accounts),
            OperationCommand::RestakeVST(command) => command.execute(ctx, accounts),
            OperationCommand::DelegateVST(command) => command.execute(ctx, accounts),
        }
    }
}

const OPERATION_COMMAND_MAX_ACCOUNT_SIZE: usize = 24;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct OperationCommandEntry {
    pub command: OperationCommand,
    #[max_len(OPERATION_COMMAND_MAX_ACCOUNT_SIZE)]
    pub required_accounts: Vec<Pubkey>,
}

impl OperationCommand {
    pub fn with_required_accounts(self, required_accounts: Vec<Pubkey>) -> OperationCommandEntry {
        OperationCommandEntry {
            command: self,
            required_accounts,
        }
    }
}

pub(super) trait SelfExecutable {
    fn execute(
        &self,
        ctx: &mut OperationCommandContext,
        accounts: &[AccountInfo],
    ) -> Result<Vec<OperationCommandEntry>>;
}
