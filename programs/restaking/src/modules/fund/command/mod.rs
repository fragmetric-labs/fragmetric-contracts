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

use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::modules::fund;

// propagate common accounts and values to all commands
pub struct OperationCommandContext<'info: 'a, 'a> {
    pub(super) operator: &'a Signer<'info>,
    pub(super) receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    pub(super) fund_account: &'a mut Account<'info, fund::FundAccount>,
    pub(super) system_program: &'a Program<'info, System>,
}

// enum to hold all command variants
#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum OperationCommand {
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
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
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
pub struct OperationCommandEntry {
    pub(super) command: OperationCommand,
    #[max_len(OPERATION_COMMAND_MAX_ACCOUNT_SIZE)]
    required_accounts: Vec<OperationCommandAccountMeta>,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct OperationCommandAccountMeta {
    pub(super) pubkey: Pubkey,
    pub(super) is_writable: bool,
}

impl OperationCommandEntry {
    pub(super) fn append_readonly_accounts(&mut self, accounts: impl IntoIterator<Item = Pubkey>) {
        self.required_accounts
            .extend(accounts.into_iter().map(|account| OperationCommandAccountMeta {
                pubkey: account,
                is_writable: false,
            }));
    }
}

impl<'a> From<&'a OperationCommandEntry>
    for (&'a OperationCommand, &'a [OperationCommandAccountMeta])
{
    fn from(value: &'a OperationCommandEntry) -> Self {
        (&value.command, &value.required_accounts)
    }
}

pub(super) trait SelfExecutable: Into<OperationCommand> {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>>;

    fn with_required_accounts(
        self,
        required_accounts: impl IntoIterator<Item = (Pubkey, bool)>,
    ) -> OperationCommandEntry {
        OperationCommandEntry {
            command: self.into(),
            required_accounts: required_accounts
                .into_iter()
                .map(|(pubkey, is_writable)| OperationCommandAccountMeta {
                    pubkey,
                    is_writable,
                })
                .collect(),
        }
    }

    fn without_required_accounts(self) -> OperationCommandEntry {
        OperationCommandEntry {
            command: self.into(),
            required_accounts: Vec::with_capacity(0),
        }
    }
}
