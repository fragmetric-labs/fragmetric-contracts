mod cmd10_stake_sol;
mod cmd11_normalize_st;
mod cmd12_restake_vst;
mod cmd13_delegate_vst;
mod cmd14_harvest_reward;
mod cmd1_initialize;
mod cmd2_enqueue_withdrawal_batch;
mod cmd3_claim_unrestaked_vst;
mod cmd4_denormalize_nt;
mod cmd5_undelegate_vst;
mod cmd6_unrestake_vrt;
mod cmd7_claim_unstaked_sol;
mod cmd8_unstake_lst;
mod cmd9_process_withdrawal_batch;

pub use cmd10_stake_sol::*;
pub use cmd11_normalize_st::*;
pub use cmd12_restake_vst::*;
pub use cmd13_delegate_vst::*;
pub use cmd14_harvest_reward::*;
pub use cmd1_initialize::*;
pub use cmd2_enqueue_withdrawal_batch::*;
pub use cmd3_claim_unrestaked_vst::*;
pub use cmd4_denormalize_nt::*;
pub use cmd5_undelegate_vst::*;
pub use cmd6_unrestake_vrt::*;
pub use cmd7_claim_unstaked_sol::*;
pub use cmd8_unstake_lst::*;
pub use cmd9_process_withdrawal_batch::*;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use bytemuck::Zeroable;

use crate::modules::fund;

use super::*;

// propagate common accounts and values to all commands
pub struct OperationCommandContext<'info: 'a, 'a> {
    pub(super) operator: &'a Signer<'info>,
    pub(super) receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    pub(super) fund_account: &'a mut AccountLoader<'info, FundAccount>,
    pub(super) system_program: &'a Program<'info, System>,
}

// enum to hold all command variants
#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum OperationCommand {
    Initialize(InitializeCommand),
    EnqueueWithdrawalBatch(EnqueueWithdrawalBatchCommand),
    ClaimUnrestakedVST(ClaimUnrestakedVSTCommand),
    DenormalizeNT(DenormalizeNTCommand),
    UndelegateVST(UndelegateVSTCommand),
    UnrestakeVRT(UnrestakeVRTCommand),
    ClaimUnstakedSOL(ClaimUnstakedSOLCommand),
    UnstakeLST(UnstakeLSTCommand),
    ProcessWithdrawalBatch(ProcessWithdrawalBatchCommand),
    StakeSOL(StakeSOLCommand),
    NormalizeST(NormalizeSTCommand),
    RestakeVST(RestakeVSTCommand),
    DelegateVST(DelegateVSTCommand),
    HarvestReward(HarvestRewardCommand),
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum OperationCommandResult {
    Initialize(InitializeCommandResult),
    EnqueueWithdrawalBatch(EnqueueWithdrawalBatchCommandResult),
    ClaimUnrestakedVST(ClaimUnrestakedVSTCommandResult),
    DenormalizeNT(DenormalizeNTCommandResult),
    UndelegateVST(UndelegateVSTCommandResult),
    UnrestakeVRT(UnrestakeVRTCommandResult),
    ClaimUnstakedSOL(ClaimUnstakedSOLCommandResult),
    UnstakeLST(UnstakeLSTCommandResult),
    ProcessWithdrawalBatch(ProcessWithdrawalBatchCommandResult),
    StakeSOL(StakeSOLCommandResult),
    NormalizeST(NormalizeSTCommandResult),
    RestakeVST(RestakeVSTCommandResult),
    DelegateVST(DelegateVSTCommandResult),
    HarvestReward(HarvestRewardCommandResult),
}

// cmd1
impl From<InitializeCommand> for OperationCommand {
    fn from(command: InitializeCommand) -> Self {
        Self::Initialize(command)
    }
}

impl From<InitializeCommandResult> for OperationCommandResult {
    fn from(result: InitializeCommandResult) -> Self {
        Self::Initialize(result)
    }
}

// cmd2
impl From<EnqueueWithdrawalBatchCommand> for OperationCommand {
    fn from(command: EnqueueWithdrawalBatchCommand) -> Self {
        Self::EnqueueWithdrawalBatch(command)
    }
}

impl From<EnqueueWithdrawalBatchCommandResult> for OperationCommandResult {
    fn from(result: EnqueueWithdrawalBatchCommandResult) -> Self {
        Self::EnqueueWithdrawalBatch(result)
    }
}

// cmd3
impl From<ClaimUnrestakedVSTCommand> for OperationCommand {
    fn from(command: ClaimUnrestakedVSTCommand) -> Self {
        Self::ClaimUnrestakedVST(command)
    }
}

impl From<ClaimUnrestakedVSTCommandResult> for OperationCommandResult {
    fn from(result: ClaimUnrestakedVSTCommandResult) -> Self {
        Self::ClaimUnrestakedVST(result)
    }
}

// cmd4
impl From<DenormalizeNTCommand> for OperationCommand {
    fn from(command: DenormalizeNTCommand) -> Self {
        Self::DenormalizeNT(command)
    }
}

impl From<DenormalizeNTCommandResult> for OperationCommandResult {
    fn from(result: DenormalizeNTCommandResult) -> Self {
        Self::DenormalizeNT(result)
    }
}

// cmd5
impl From<UndelegateVSTCommand> for OperationCommand {
    fn from(command: UndelegateVSTCommand) -> Self {
        Self::UndelegateVST(command)
    }
}

impl From<UndelegateVSTCommandResult> for OperationCommandResult {
    fn from(result: UndelegateVSTCommandResult) -> Self {
        Self::UndelegateVST(result)
    }
}

// cmd6
impl From<UnrestakeVRTCommand> for OperationCommand {
    fn from(command: UnrestakeVRTCommand) -> Self {
        Self::UnrestakeVRT(command)
    }
}

impl From<UnrestakeVRTCommandResult> for OperationCommandResult {
    fn from(result: UnrestakeVRTCommandResult) -> Self {
        Self::UnrestakeVRT(result)
    }
}

// cmd7
impl From<ClaimUnstakedSOLCommand> for OperationCommand {
    fn from(command: ClaimUnstakedSOLCommand) -> Self {
        Self::ClaimUnstakedSOL(command)
    }
}

impl From<ClaimUnstakedSOLCommandResult> for OperationCommandResult {
    fn from(result: ClaimUnstakedSOLCommandResult) -> Self {
        Self::ClaimUnstakedSOL(result)
    }
}

// cmd8
impl From<UnstakeLSTCommand> for OperationCommand {
    fn from(command: UnstakeLSTCommand) -> Self {
        Self::UnstakeLST(command)
    }
}

impl From<UnstakeLSTCommandResult> for OperationCommandResult {
    fn from(result: UnstakeLSTCommandResult) -> Self {
        Self::UnstakeLST(result)
    }
}

// cmd9
impl From<ProcessWithdrawalBatchCommand> for OperationCommand {
    fn from(command: ProcessWithdrawalBatchCommand) -> Self {
        Self::ProcessWithdrawalBatch(command)
    }
}

impl From<ProcessWithdrawalBatchCommandResult> for OperationCommandResult {
    fn from(result: ProcessWithdrawalBatchCommandResult) -> Self {
        Self::ProcessWithdrawalBatch(result)
    }
}

// cmd10
impl From<StakeSOLCommand> for OperationCommand {
    fn from(command: StakeSOLCommand) -> Self {
        Self::StakeSOL(command)
    }
}

impl From<StakeSOLCommandResult> for OperationCommandResult {
    fn from(result: StakeSOLCommandResult) -> Self {
        Self::StakeSOL(result)
    }
}

// cmd11
impl From<NormalizeSTCommand> for OperationCommand {
    fn from(command: NormalizeSTCommand) -> Self {
        Self::NormalizeST(command)
    }
}

impl From<NormalizeSTCommandResult> for OperationCommandResult {
    fn from(result: NormalizeSTCommandResult) -> Self {
        Self::NormalizeST(result)
    }
}

// cmd12
impl From<RestakeVSTCommand> for OperationCommand {
    fn from(command: RestakeVSTCommand) -> Self {
        Self::RestakeVST(command)
    }
}

impl From<RestakeVSTCommandResult> for OperationCommandResult {
    fn from(result: RestakeVSTCommandResult) -> Self {
        Self::RestakeVST(result)
    }
}

// cmd13
impl From<DelegateVSTCommand> for OperationCommand {
    fn from(command: DelegateVSTCommand) -> Self {
        Self::DelegateVST(command)
    }
}

impl From<DelegateVSTCommandResult> for OperationCommandResult {
    fn from(result: DelegateVSTCommandResult) -> Self {
        Self::DelegateVST(result)
    }
}

// cmd14
impl From<HarvestRewardCommand> for OperationCommand {
    fn from(command: HarvestRewardCommand) -> Self {
        Self::HarvestReward(command)
    }
}

impl From<HarvestRewardCommandResult> for OperationCommandResult {
    fn from(result: HarvestRewardCommandResult) -> Self {
        Self::HarvestReward(result)
    }
}

impl OperationCommand {
    fn discriminant(&self) -> u8 {
        match self {
            OperationCommand::Initialize(_) => 1,
            OperationCommand::EnqueueWithdrawalBatch(_) => 2,
            OperationCommand::ClaimUnrestakedVST(_) => 3,
            OperationCommand::DenormalizeNT(_) => 4,
            OperationCommand::UndelegateVST(_) => 5,
            OperationCommand::UnrestakeVRT(_) => 6,
            OperationCommand::ClaimUnstakedSOL(_) => 7,
            OperationCommand::UnstakeLST(_) => 8,
            OperationCommand::ProcessWithdrawalBatch(_) => 9,
            OperationCommand::StakeSOL(_) => 10,
            OperationCommand::NormalizeST(_) => 11,
            OperationCommand::RestakeVST(_) => 12,
            OperationCommand::DelegateVST(_) => 13,
            OperationCommand::HarvestReward(_) => 14,
        }
    }

    fn serialize_as_pod(&self, pod: &mut OperationCommandPod) -> Result<()> {
        pod.clear();
        pod.discriminant = self.discriminant();
        self.serialize(&mut pod.buffer.as_mut_slice())?;
        Ok(())
    }
}

const FUND_ACCOUNT_OPERATION_COMMAND_BUFFER_SIZE: usize = 2535;

#[zero_copy]
#[derive(Debug)]
#[repr(C)]
pub struct OperationCommandPod {
    discriminant: u8,
    buffer: [u8; FUND_ACCOUNT_OPERATION_COMMAND_BUFFER_SIZE],
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

#[zero_copy]
#[derive(Debug)]
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

/// Technically, can contain up to 57 accounts out of 64 with reserved 6 accounts and payer.
pub const FUND_ACCOUNT_OPERATION_COMMAND_MAX_ACCOUNT_SIZE: usize = 32;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct OperationCommandEntry {
    pub(super) command: OperationCommand,
    #[max_len(FUND_ACCOUNT_OPERATION_COMMAND_MAX_ACCOUNT_SIZE)]
    pub(super) required_accounts: Vec<OperationCommandAccountMeta>,
}

impl OperationCommandEntry {
    pub fn is_safe_with_unchecked_params(&self) -> bool {
        match self.command {
            OperationCommand::Initialize(..)
            | OperationCommand::EnqueueWithdrawalBatch(..)
            | OperationCommand::ProcessWithdrawalBatch(..) => true,
            _ => false,
        }
    }

    pub fn serialize_as_pod(&self, pod: &mut OperationCommandEntryPod) -> Result<()> {
        pod.num_required_accounts = self.required_accounts.len() as u8;
        for (i, account_meta) in self
            .required_accounts
            .iter()
            .take(FUND_ACCOUNT_OPERATION_COMMAND_MAX_ACCOUNT_SIZE)
            .enumerate()
        {
            account_meta.serialize_as_pod(&mut pod.required_accounts[i]);
        }
        self.command.serialize_as_pod(&mut pod.command)
    }
}

#[zero_copy]
#[derive(Debug)]
#[repr(C)]
pub struct OperationCommandEntryPod {
    num_required_accounts: u8,
    _padding: [u8; 7],
    required_accounts:
        [OperationCommandAccountMetaPod; FUND_ACCOUNT_OPERATION_COMMAND_MAX_ACCOUNT_SIZE],
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
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        match self {
            OperationCommand::Initialize(command) => command.execute(ctx, accounts),
            OperationCommand::EnqueueWithdrawalBatch(command) => command.execute(ctx, accounts),
            OperationCommand::ClaimUnrestakedVST(command) => command.execute(ctx, accounts),
            OperationCommand::DenormalizeNT(command) => command.execute(ctx, accounts),
            OperationCommand::UndelegateVST(command) => command.execute(ctx, accounts),
            OperationCommand::UnrestakeVRT(command) => command.execute(ctx, accounts),
            OperationCommand::ClaimUnstakedSOL(command) => command.execute(ctx, accounts),
            OperationCommand::UnstakeLST(command) => command.execute(ctx, accounts),
            OperationCommand::ProcessWithdrawalBatch(command) => command.execute(ctx, accounts),
            OperationCommand::StakeSOL(command) => command.execute(ctx, accounts),
            OperationCommand::NormalizeST(command) => command.execute(ctx, accounts),
            OperationCommand::RestakeVST(command) => command.execute(ctx, accounts),
            OperationCommand::DelegateVST(command) => command.execute(ctx, accounts),
            OperationCommand::HarvestReward(command) => command.execute(ctx, accounts),
        }
    }
}

pub(super) trait SelfExecutable: Into<OperationCommand> {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )>;

    fn with_required_accounts(
        self,
        required_accounts: impl IntoIterator<Item = (Pubkey, bool)>,
    ) -> OperationCommandEntry {
        OperationCommandEntry {
            command: self.into(),
            required_accounts: required_accounts
                .into_iter()
                .take(FUND_ACCOUNT_OPERATION_COMMAND_MAX_ACCOUNT_SIZE)
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
            FUND_ACCOUNT_OPERATION_COMMAND_BUFFER_SIZE,
            OperationCommand::INIT_SPACE,
        );
        assert_eq!(
            FUND_ACCOUNT_OPERATION_COMMAND_BUFFER_SIZE > OperationCommand::INIT_SPACE,
            true
        );
    }
}
