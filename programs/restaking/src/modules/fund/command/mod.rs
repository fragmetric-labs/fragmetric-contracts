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
mod cmd7_unrestake_vst;
mod cmd8_unstake_lst;
mod cmd9_stake_lst;

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
pub use cmd7_unrestake_vst::*;
pub use cmd8_unstake_lst::*;
pub use cmd9_stake_lst::*;

use crate::modules::fund;
use anchor_lang::prelude::*;

// propagate common accounts and values to all commands
pub(super) struct OperationCommandContext<'info, 'a>
where
    'info: 'a,
{
    pub(super) fund_account: &'a mut Account<'info, fund::FundAccount>,
    pub(super) receipt_token_mint: Pubkey,
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
    UnrestakeVST(UnrestakeVSTCommand),
    UnstakeLST(UnstakeLSTCommand),
    StakeLST(StakeLSTCommand),
    NormalizeLST(NormalizeLSTCommand),
    RestakeVST(RestakeVSTCommand),
    DelegateVST(DelegateVSTCommand),
}

impl OperationCommand {
    fn get_inner(&self) -> &dyn SelfExecutable {
        match self {
            OperationCommand::Initialize(command) => command,
            OperationCommand::ClaimUnstakedSOL(command) => command,
            OperationCommand::EnqueueWithdrawalBatch(command) => command,
            OperationCommand::ProcessWithdrawalBatch(command) => command,
            OperationCommand::ClaimUnrestakedVST(command) => command,
            OperationCommand::DenormalizeNT(command) => command,
            OperationCommand::UndelegateVST(command) => command,
            OperationCommand::UnrestakeVST(command) => command,
            OperationCommand::UnstakeLST(command) => command,
            OperationCommand::StakeLST(command) => command,
            OperationCommand::NormalizeLST(command) => command,
            OperationCommand::RestakeVST(command) => command,
            OperationCommand::DelegateVST(command) => command,
        }
    }
}

impl SelfExecutable for OperationCommand {
    fn execute(
        &self,
        context: &OperationCommandContext,
        accounts: Vec<&AccountInfo>,
    ) -> Result<Vec<OperationCommandEntry>> {
        self.get_inner().execute(context, accounts)
    }

    fn compute_required_accounts(&self, context: &OperationCommandContext) -> Result<Vec<Pubkey>> {
        self.get_inner().compute_required_accounts(context)
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
    pub fn build(&self, context: &OperationCommandContext) -> Result<OperationCommandEntry> {
        Ok(OperationCommandEntry {
            command: self.clone(),
            required_accounts: self.compute_required_accounts(context)?,
        })
    }
}

pub(super) trait SelfExecutable {
    fn execute(
        &self,
        context: &OperationCommandContext,
        accounts: Vec<&AccountInfo>,
    ) -> Result<Vec<OperationCommandEntry>>;

    fn compute_required_accounts(&self, context: &OperationCommandContext) -> Result<Vec<Pubkey>>;
}
