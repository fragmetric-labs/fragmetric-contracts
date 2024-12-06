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

const OPERATION_COMMAND_BUFFER_SIZE: usize = 320;

#[derive(Debug)]
#[zero_copy]
pub struct OperationCommandPod {
    discriminant: u8,
    buffer: [u8; OPERATION_COMMAND_BUFFER_SIZE],
}

impl From<OperationCommand> for OperationCommandPod {
    fn from(src: OperationCommand) -> Self {
        let mut pod = Self {
            discriminant: 0,
            buffer: [0; OPERATION_COMMAND_BUFFER_SIZE],
        };
        src.serialize(&mut pod.buffer[..]).unwrap();
        pod.discriminant = match src {
            OperationCommand::Initialize(_) => 1,
            OperationCommand::ClaimUnstakedSOL(_) => 2,
            OperationCommand::EnqueueWithdrawalBatch(_) => 3,
            OperationCommand::ProcessWithdrawalBatch(_) => 4,
            OperationCommand::ClaimUnrestakedVST(_) => 5,
            OperationCommand::DenormalizeNT(_) => 6,
            OperationCommand::UndelegateVST(_) => 7,
            OperationCommand::UnrestakeVRT(_) => 8,
            OperationCommand::UnstakeLST(_) => 9,
            OperationCommand::StakeSOL(_) => 10,
            OperationCommand::NormalizeLST(_) => 11,
            OperationCommand::RestakeVST(_) => 12,
            OperationCommand::DelegateVST(_) => 13,
        };

        pod
    }
}

impl From<OperationCommandPod> for OperationCommand {
    fn from(pod: OperationCommandPod) -> Self {
        match pod.discriminant {
            1 => InitializeCommand::try_from_slice(&pod.buffer[..])
                .unwrap()
                .into(),
            2 => ClaimUnstakedSOLCommand::try_from_slice(&pod.buffer[..])
                .unwrap()
                .into(),
            3 => EnqueueWithdrawalBatchCommand::try_from_slice(&pod.buffer[..])
                .unwrap()
                .into(),
            4 => ClaimUnstakedSOLCommand::try_from_slice(&pod.buffer[..])
                .unwrap()
                .into(),
            5 => ClaimUnrestakedVSTCommand::try_from_slice(&pod.buffer[..])
                .unwrap()
                .into(),
            6 => DenormalizeNTCommand::try_from_slice(&pod.buffer[..])
                .unwrap()
                .into(),
            7 => UndelegateVSTCommand::try_from_slice(&pod.buffer[..])
                .unwrap()
                .into(),
            8 => UnrestakeVRTCommand::try_from_slice(&pod.buffer[..])
                .unwrap()
                .into(),
            9 => UnstakeLSTCommand::try_from_slice(&pod.buffer[..])
                .unwrap()
                .into(),
            10 => StakeSOLCommand::try_from_slice(&pod.buffer[..])
                .unwrap()
                .into(),
            11 => NormalizeLSTCommand::try_from_slice(&pod.buffer[..])
                .unwrap()
                .into(),
            12 => RestakeVSTCommand::try_from_slice(&pod.buffer[..])
                .unwrap()
                .into(),
            13 => DelegateVSTCommand::try_from_slice(&pod.buffer[..])
                .unwrap()
                .into(),
            _ => panic!("invalid discriminant for OperationCommand"),
        }
    }
}

impl OperationCommand {
    #[inline]
    fn as_inner_command(&self) -> &dyn SelfExecutable {
        match self {
            OperationCommand::Initialize(command)
            | OperationCommand::ClaimUnstakedSOL(command)
            | OperationCommand::EnqueueWithdrawalBatch(command)
            | OperationCommand::ProcessWithdrawalBatch(command)
            | OperationCommand::ClaimUnrestakedVST(command)
            | OperationCommand::DenormalizeNT(command)
            | OperationCommand::UndelegateVST(command)
            | OperationCommand::UnrestakeVRT(command)
            | OperationCommand::UnstakeLST(command)
            | OperationCommand::StakeSOL(command)
            | OperationCommand::NormalizeLST(command)
            | OperationCommand::RestakeVST(command)
            | OperationCommand::DelegateVST(command) => command,
        }
    }
}

impl SelfExecutable for OperationCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<Option<OperationCommandEntry>> {
        self.as_inner_command().execute(ctx, accounts)
    }
}

const OPERATION_COMMAND_MAX_ACCOUNT_SIZE: usize = 24;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct OperationCommandEntry {
    pub(super) command: OperationCommand,
    #[max_len(OPERATION_COMMAND_MAX_ACCOUNT_SIZE)]
    required_accounts: Vec<OperationCommandAccountMeta>,
}

#[derive(Clone, Copy, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct OperationCommandAccountMeta {
    pub(super) pubkey: Pubkey,
    pub(super) is_writable: bool,
}

#[derive(Debug)]
#[zero_copy]
pub struct OperationCommandEntryPod {
    command: OperationCommandPod,
    required_accounts: [(Pubkey, bool); OPERATION_COMMAND_MAX_ACCOUNT_SIZE],
}

impl From<OperationCommandEntry> for OperationCommandEntryPod {
    fn from(src: OperationCommandEntry) -> Self {
        Self {
            command: src.command.into(),
            required_accounts: src
                .required_accounts
                .iter()
                .take(OPERATION_COMMAND_MAX_ACCOUNT_SIZE)
                .map(|account| (account.pubkey, account.is_writable))
                .into(),
        }
    }
}

impl From<OperationCommandEntryPod> for OperationCommandEntry {
    fn from(pod: OperationCommandEntryPod) -> Self {
        Self {
            command: pod.command.into(),
            required_accounts: pod
                .required_accounts
                .into_iter()
                .map(|(pubkey, is_writable)| OperationCommandAccountMeta {
                    pubkey,
                    is_writable,
                })
                .collect::<Vec<_>>(),
        }
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
