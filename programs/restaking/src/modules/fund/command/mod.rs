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
use bytemuck::{Pod, Zeroable};

use crate::modules::fund;

// propagate common accounts and values to all commands
pub struct OperationCommandContext<'info: 'a, 'a> {
    pub(super) operator: &'a Signer<'info>,
    pub(super) receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    pub(super) fund_account: &'a mut AccountLoader<'info, fund::FundAccount>,
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

// TODO: check this size
const OPERATION_COMMAND_BUFFER_SIZE: usize = 319;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
pub struct OperationCommandPod {
    discriminant: u8,
    buffer: [u8; OPERATION_COMMAND_BUFFER_SIZE],
}

impl From<OperationCommand> for OperationCommandPod {
    fn from(src: OperationCommand) -> Self {
        let mut pod = Self {
            discriminant: match src {
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
            },
            buffer: [0; OPERATION_COMMAND_BUFFER_SIZE],
        };
        src.serialize(&mut &mut pod.buffer[..]).unwrap();

        pod
    }
}

impl TryFrom<&OperationCommandPod> for OperationCommand {
    type Error = anchor_lang::error::Error;

    fn try_from(pod: &OperationCommandPod) -> Result<OperationCommand> {
        Ok(match pod.discriminant {
            1 => InitializeCommand::try_from_slice(&pod.buffer[..])?.into(),
            2 => ClaimUnstakedSOLCommand::try_from_slice(&pod.buffer[..])?.into(),
            3 => EnqueueWithdrawalBatchCommand::try_from_slice(&pod.buffer[..])?.into(),
            4 => ClaimUnstakedSOLCommand::try_from_slice(&pod.buffer[..])?.into(),
            5 => ClaimUnrestakedVSTCommand::try_from_slice(&pod.buffer[..])?.into(),
            6 => DenormalizeNTCommand::try_from_slice(&pod.buffer[..])?.into(),
            7 => UndelegateVSTCommand::try_from_slice(&pod.buffer[..])?.into(),
            8 => UnrestakeVRTCommand::try_from_slice(&pod.buffer[..])?.into(),
            9 => UnstakeLSTCommand::try_from_slice(&pod.buffer[..])?.into(),
            10 => StakeSOLCommand::try_from_slice(&pod.buffer[..])?.into(),
            11 => NormalizeLSTCommand::try_from_slice(&pod.buffer[..])?.into(),
            12 => RestakeVSTCommand::try_from_slice(&pod.buffer[..])?.into(),
            13 => DelegateVSTCommand::try_from_slice(&pod.buffer[..])?.into(),
            _ => Err(Error::from(ProgramError::InvalidAccountData))?,
        })
    }
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

#[derive(Clone, Copy, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct OperationCommandAccountMeta {
    pub(super) pubkey: Pubkey,
    pub(super) is_writable: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug, Default)]
#[repr(C)]
pub struct OperationCommandAccountMetaPod {
    pubkey: Pubkey,
    is_writable: u8,
    _padding: [u8; 7],
}

impl From<OperationCommandAccountMeta> for OperationCommandAccountMetaPod {
    fn from(src: OperationCommandAccountMeta) -> Self {
        Self {
            pubkey: src.pubkey,
            is_writable: if src.is_writable { 1 } else { 0 },
            _padding: [0; 7],
        }
    }
}

impl From<&OperationCommandAccountMetaPod> for OperationCommandAccountMeta {
    fn from(pod: &OperationCommandAccountMetaPod) -> Self {
        Self {
            pubkey: pod.pubkey,
            is_writable: pod.is_writable == 1,
        }
    }
}

const OPERATION_COMMAND_MAX_ACCOUNT_SIZE: usize = 24;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct OperationCommandEntry {
    pub(super) command: OperationCommand,
    #[max_len(OPERATION_COMMAND_MAX_ACCOUNT_SIZE)]
    pub(super) required_accounts: Vec<OperationCommandAccountMeta>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
pub struct OperationCommandEntryPod {
    num_required_accounts: u8,
    _padding: [u8; 7],
    required_accounts: [OperationCommandAccountMetaPod; OPERATION_COMMAND_MAX_ACCOUNT_SIZE],
    command: OperationCommandPod,
}

impl Default for OperationCommandEntryPod {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl From<OperationCommandEntry> for OperationCommandEntryPod {
    fn from(src: OperationCommandEntry) -> Self {
        let mut pod: OperationCommandEntryPod = OperationCommandEntryPod::zeroed();
        pod.num_required_accounts = src.required_accounts.len() as u8;
        for (i, account_meta) in src
            .required_accounts
            .into_iter()
            .take(OPERATION_COMMAND_MAX_ACCOUNT_SIZE)
            .enumerate()
        {
            pod.required_accounts[i] = account_meta.into();
        }
        pod.command = src.command.into();
        pod
    }
}

impl OperationCommandEntryPod {
    pub fn is_none(&self) -> bool {
        self.command.discriminant == 0
    }
}

impl TryFrom<&OperationCommandEntryPod> for OperationCommandEntry {
    type Error = anchor_lang::error::Error;

    fn try_from(pod: &OperationCommandEntryPod) -> Result<OperationCommandEntry> {
        Ok(OperationCommandEntry {
            command: (&pod.command).try_into()?,
            required_accounts: pod
                .required_accounts
                .iter()
                .take(pod.num_required_accounts as usize)
                .map(Into::into)
                .collect::<Vec<_>>(),
        })
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
                .take(OPERATION_COMMAND_MAX_ACCOUNT_SIZE)
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
