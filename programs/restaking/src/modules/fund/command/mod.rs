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

impl OperationCommand {
    fn discriminant(&self) -> u8 {
        match self {
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
        }
    }

    fn serialize_as_pod(&self, pod: &mut OperationCommandPod) -> Result<()> {
        pod.clear();
        pod.discriminant = self.discriminant();
        self.serialize(&mut pod.buffer.as_mut_slice())?;
        Ok(())
    }
}

const OPERATION_COMMAND_BUFFER_SIZE: usize = 2023;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
pub struct OperationCommandPod {
    discriminant: u8,
    buffer: [u8; OPERATION_COMMAND_BUFFER_SIZE],
}

impl OperationCommandPod {
    fn clear(&mut self) {
        self.discriminant = 0;
        self.buffer.fill(0);
    }

    fn try_deserialize(&self) -> Result<Option<OperationCommand>> {
        Ok({
            if self.discriminant == 0 {
                None
            } else {
                let command = OperationCommand::deserialize(&mut self.buffer.as_slice())?;
                if self.discriminant == command.discriminant() {
                    Some(command)
                } else {
                    Err(Error::from(ProgramError::InvalidAccountData))?
                }
            }
        })
    }
}

#[derive(Clone, Copy, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct OperationCommandAccountMeta {
    pub(super) pubkey: Pubkey,
    pub(super) is_writable: bool,
}

impl OperationCommandAccountMeta {
    pub fn serialize_as_pod(&self, pod: &mut OperationCommandAccountMetaPod) {
        pod.pubkey = self.pubkey;
        pod.is_writable = if self.is_writable { 1 } else { 0 };
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
pub struct OperationCommandAccountMetaPod {
    pubkey: Pubkey,
    is_writable: u8,
    _padding: [u8; 7],
}

impl OperationCommandAccountMetaPod {
    pub fn deserialize(&self) -> OperationCommandAccountMeta {
        OperationCommandAccountMeta {
            pubkey: self.pubkey,
            is_writable: self.is_writable == 1,
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

impl OperationCommandEntry {
    pub fn serialize_as_pod(&self, pod: &mut OperationCommandEntryPod) -> Result<()> {
        pod.num_required_accounts = self.required_accounts.len() as u8;
        for (i, account_meta) in self
            .required_accounts
            .iter()
            .take(OPERATION_COMMAND_MAX_ACCOUNT_SIZE)
            .enumerate()
        {
            account_meta.serialize_as_pod(&mut pod.required_accounts[i]);
        }
        self.command.serialize_as_pod(&mut pod.command)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
pub struct OperationCommandEntryPod {
    num_required_accounts: u8,
    _padding: [u8; 7],
    required_accounts: [OperationCommandAccountMetaPod; OPERATION_COMMAND_MAX_ACCOUNT_SIZE],
    command: OperationCommandPod,
}

impl OperationCommandEntryPod {
    pub fn is_none(&self) -> bool {
        self.command.discriminant == 0
    }

    pub fn clear(&mut self) {
        self.command.clear();
        self.num_required_accounts = 0;
        self.required_accounts
            .fill(OperationCommandAccountMetaPod::zeroed());
    }

    pub fn try_deserialize(&self) -> Result<Option<OperationCommandEntry>> {
        Ok({
            let command = self.command.try_deserialize()?;
            command.map(|command| OperationCommandEntry {
                command,
                required_accounts: self
                    .required_accounts
                    .iter()
                    .take(self.num_required_accounts as usize)
                    .map(|account_meta_pod| account_meta_pod.deserialize())
                    .collect::<Vec<_>>(),
            })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_command_buffer() {
        println!(
            "\ncommand buffer_size={}, init_size={}",
            OPERATION_COMMAND_BUFFER_SIZE,
            OperationCommand::INIT_SPACE,
        );
        assert_eq!(
            OPERATION_COMMAND_BUFFER_SIZE > OperationCommand::INIT_SPACE,
            true
        );
    }
}
