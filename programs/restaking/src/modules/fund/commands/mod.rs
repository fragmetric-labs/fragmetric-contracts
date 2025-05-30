mod cmd10_harvest_reward;
mod cmd11_stake_sol;
mod cmd12_normalize_st;
mod cmd13_restake_vst;
mod cmd14_delegate_vst;
mod cmd1_initialize;
mod cmd2_enqueue_withdrawal_batch;
mod cmd3_claim_unrestaked_vst;
mod cmd4_denormalize_nt;
mod cmd5_claim_unstaked_sol;
mod cmd6_process_withdrawal_batch;
mod cmd7_unstake_lst;
mod cmd8_unrestake_vrt;
mod cmd9_undelegate_vst;

pub use cmd10_harvest_reward::*;
pub use cmd11_stake_sol::*;
pub use cmd12_normalize_st::*;
pub use cmd13_restake_vst::*;
pub use cmd14_delegate_vst::*;
pub use cmd1_initialize::*;
pub use cmd2_enqueue_withdrawal_batch::*;
pub use cmd3_claim_unrestaked_vst::*;
pub use cmd4_denormalize_nt::*;
pub use cmd5_claim_unstaked_sol::*;
pub use cmd6_process_withdrawal_batch::*;
pub use cmd7_unstake_lst::*;
pub use cmd8_unrestake_vrt::*;
pub use cmd9_undelegate_vst::*;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::errors::ErrorCode;

use super::*;

// propagate common accounts and values to all commands
pub(super) struct OperationCommandContext<'info, 'a> {
    pub operator: &'a Signer<'info>,
    pub receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    pub fund_account: &'a mut AccountLoader<'info, FundAccount>,
    pub system_program: &'a Program<'info, System>,
}

// enum to hold all command variants
#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub enum OperationCommand {
    Initialize(InitializeCommand),
    EnqueueWithdrawalBatch(EnqueueWithdrawalBatchCommand),
    ClaimUnrestakedVST(ClaimUnrestakedVSTCommand),
    DenormalizeNT(DenormalizeNTCommand),
    ClaimUnstakedSOL(ClaimUnstakedSOLCommand),
    ProcessWithdrawalBatch(ProcessWithdrawalBatchCommand),
    UnstakeLST(UnstakeLSTCommand),
    UnrestakeVRT(UnrestakeVRTCommand),
    UndelegateVST(UndelegateVSTCommand),
    HarvestReward(HarvestRewardCommand),
    StakeSOL(StakeSOLCommand),
    NormalizeST(NormalizeSTCommand),
    RestakeVST(RestakeVSTCommand),
    DelegateVST(DelegateVSTCommand),
}

impl std::fmt::Debug for OperationCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationCommand::Initialize(command) => command.fmt(f),
            OperationCommand::EnqueueWithdrawalBatch(command) => command.fmt(f),
            OperationCommand::ClaimUnrestakedVST(command) => command.fmt(f),
            OperationCommand::DenormalizeNT(command) => command.fmt(f),
            OperationCommand::ClaimUnstakedSOL(command) => command.fmt(f),
            OperationCommand::ProcessWithdrawalBatch(command) => command.fmt(f),
            OperationCommand::UnstakeLST(command) => command.fmt(f),
            OperationCommand::UndelegateVST(command) => command.fmt(f),
            OperationCommand::UnrestakeVRT(command) => command.fmt(f),
            OperationCommand::HarvestReward(command) => command.fmt(f),
            OperationCommand::StakeSOL(command) => command.fmt(f),
            OperationCommand::NormalizeST(command) => command.fmt(f),
            OperationCommand::RestakeVST(command) => command.fmt(f),
            OperationCommand::DelegateVST(command) => command.fmt(f),
        }
    }
}

impl OperationCommand {
    pub fn type_name(&self) -> &str {
        match self {
            OperationCommand::Initialize(..) => "Initialize",
            OperationCommand::EnqueueWithdrawalBatch(..) => "EnqueueWithdrawalBatch",
            OperationCommand::ClaimUnrestakedVST(..) => "ClaimUnrestakedVST",
            OperationCommand::DenormalizeNT(..) => "DenormalizeNT",
            OperationCommand::ClaimUnstakedSOL(..) => "ClaimUnstakedSOL",
            OperationCommand::ProcessWithdrawalBatch(..) => "ProcessWithdrawalBatch",
            OperationCommand::UnstakeLST(..) => "UnstakeLST",
            OperationCommand::UndelegateVST(..) => "UndelegateVST",
            OperationCommand::UnrestakeVRT(..) => "UnrestakeVRT",
            OperationCommand::HarvestReward(..) => "HarvestReward",
            OperationCommand::StakeSOL(..) => "StakeSOL",
            OperationCommand::NormalizeST(..) => "NormalizeST",
            OperationCommand::RestakeVST(..) => "RestakeVST",
            OperationCommand::DelegateVST(..) => "DelegateVST",
        }
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub enum OperationCommandResult {
    Initialize(InitializeCommandResult),
    EnqueueWithdrawalBatch(EnqueueWithdrawalBatchCommandResult),
    ClaimUnrestakedVST(ClaimUnrestakedVSTCommandResult),
    DenormalizeNT(DenormalizeNTCommandResult),
    ClaimUnstakedSOL(ClaimUnstakedSOLCommandResult),
    ProcessWithdrawalBatch(ProcessWithdrawalBatchCommandResult),
    UnstakeLST(UnstakeLSTCommandResult),
    UnrestakeVRT(UnrestakeVRTCommandResult),
    UndelegateVST(UndelegateVSTCommandResult),
    HarvestReward(HarvestRewardCommandResult),
    StakeSOL(StakeSOLCommandResult),
    NormalizeST(NormalizeSTCommandResult),
    RestakeVST(RestakeVSTCommandResult),
    DelegateVST(DelegateVSTCommandResult),
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
    pub fn discriminant(&self) -> u8 {
        match self {
            OperationCommand::Initialize(_) => 1,
            OperationCommand::EnqueueWithdrawalBatch(_) => 2,
            OperationCommand::ClaimUnrestakedVST(_) => 3,
            OperationCommand::DenormalizeNT(_) => 4,
            OperationCommand::ClaimUnstakedSOL(_) => 5,
            OperationCommand::ProcessWithdrawalBatch(_) => 6,
            OperationCommand::UnstakeLST(_) => 7,
            OperationCommand::UnrestakeVRT(_) => 8,
            OperationCommand::UndelegateVST(_) => 9,
            OperationCommand::StakeSOL(_) => 10,
            OperationCommand::HarvestReward(_) => 11,
            OperationCommand::NormalizeST(_) => 12,
            OperationCommand::RestakeVST(_) => 13,
            OperationCommand::DelegateVST(_) => 14,
        }
    }

    pub fn is_safe_with_unchecked_params(&self) -> bool {
        match self {
            Self::Initialize(_)
            | Self::EnqueueWithdrawalBatch(_)
            | Self::ProcessWithdrawalBatch(_) => true,
            _ => false,
        }
    }

    pub fn serialize_as_pod(&self, pod: &mut OperationCommandPod) -> Result<()> {
        pod.set_none();
        pod.discriminant = self.discriminant();
        self.serialize(&mut pod.buffer.as_mut_slice())?;
        Ok(())
    }
}

const FUND_ACCOUNT_OPERATION_COMMAND_BUFFER_SIZE: usize = 3126;

/// Pod type of `Option<OperationCommand>`
#[zero_copy]
#[repr(C)]
pub struct OperationCommandPod {
    discriminant: u8,
    buffer: [u8; FUND_ACCOUNT_OPERATION_COMMAND_BUFFER_SIZE],
}

impl OperationCommandPod {
    pub fn discriminant(&self) -> Option<u8> {
        (self.discriminant != 0).then_some(self.discriminant)
    }

    pub fn is_none(&self) -> bool {
        self.discriminant == 0
    }

    pub fn set_none(&mut self) {
        self.discriminant = 0;
        self.buffer.fill(0);
    }

    pub fn try_deserialize(&self) -> Result<Option<OperationCommand>> {
        if self.discriminant == 0 {
            return Ok(None);
        }

        let command = OperationCommand::deserialize(&mut &self.buffer[..])?;
        if self.discriminant == command.discriminant() {
            Ok(Some(command))
        } else {
            Err(Error::from(ProgramError::InvalidAccountData))?
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct OperationCommandAccountMeta {
    pub pubkey: Pubkey,
    pub is_writable: bool,
}

impl From<(Pubkey, bool)> for OperationCommandAccountMeta {
    fn from((pubkey, is_writable): (Pubkey, bool)) -> Self {
        Self {
            pubkey,
            is_writable,
        }
    }
}

impl std::fmt::Debug for OperationCommandAccountMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.pubkey.fmt(f)?;
        if self.is_writable {
            f.write_str("(W)")?;
        }
        Ok(())
    }
}

impl OperationCommandAccountMeta {
    pub fn serialize_as_pod(&self, pod: &mut OperationCommandAccountMetaPod) {
        pod.pubkey = self.pubkey;
        pod.is_writable = self.is_writable as u8;
    }
}

#[zero_copy]
#[repr(C)]
/// Pod type of `OperationCommandAccountMeta`
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
const FUND_ACCOUNT_OPERATION_COMMAND_MAX_ACCOUNT_SIZE: usize = 32;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct OperationCommandEntry {
    pub command: OperationCommand,
    #[max_len(FUND_ACCOUNT_OPERATION_COMMAND_MAX_ACCOUNT_SIZE)]
    pub required_accounts: Vec<OperationCommandAccountMeta>,
}

impl Default for OperationCommandEntry {
    fn default() -> Self {
        InitializeCommand::default().without_required_accounts()
    }
}

impl OperationCommandEntry {
    pub const MAX_ACCOUNT_SIZE: usize = FUND_ACCOUNT_OPERATION_COMMAND_MAX_ACCOUNT_SIZE;

    pub fn serialize_as_pod(&self, pod: &mut OperationCommandEntryPod) -> Result<()> {
        if self.required_accounts.len() > Self::MAX_ACCOUNT_SIZE {
            err!(ErrorCode::IndexOutOfBoundsException)?;
        }

        pod.num_required_accounts = self.required_accounts.len() as u8;
        for (pod, meta) in pod
            .required_accounts
            .iter_mut()
            .zip(&self.required_accounts)
        {
            meta.serialize_as_pod(pod);
        }
        self.command.serialize_as_pod(&mut pod.command)
    }
}

/// Pod type of `Option<OperationCommandEntry>`
#[zero_copy]
#[repr(C)]
pub struct OperationCommandEntryPod {
    num_required_accounts: u8,
    _padding: [u8; 7],
    required_accounts:
        [OperationCommandAccountMetaPod; FUND_ACCOUNT_OPERATION_COMMAND_MAX_ACCOUNT_SIZE],
    command: OperationCommandPod,
}

impl OperationCommandEntryPod {
    pub fn discriminant(&self) -> Option<u8> {
        self.command.discriminant()
    }

    pub fn is_none(&self) -> bool {
        self.command.is_none()
    }

    pub fn set_none(&mut self) {
        self.command.set_none();
        self.num_required_accounts = 0;
    }

    pub fn try_deserialize(&self) -> Result<Option<OperationCommandEntry>> {
        let Some(command) = self.command.try_deserialize()? else {
            return Ok(None);
        };
        let required_accounts = self.required_accounts[..self.num_required_accounts as usize]
            .iter()
            .map(|pod| pod.deserialize())
            .collect();

        Ok(Some(OperationCommandEntry {
            command,
            required_accounts,
        }))
    }
}

impl SelfExecutable for OperationCommand {
    fn execute<'a, 'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> ExecutionResult {
        match self {
            OperationCommand::Initialize(command) => command.execute(ctx, accounts),
            OperationCommand::EnqueueWithdrawalBatch(command) => command.execute(ctx, accounts),
            OperationCommand::ClaimUnrestakedVST(command) => command.execute(ctx, accounts),
            OperationCommand::DenormalizeNT(command) => command.execute(ctx, accounts),
            OperationCommand::ClaimUnstakedSOL(command) => command.execute(ctx, accounts),
            OperationCommand::ProcessWithdrawalBatch(command) => command.execute(ctx, accounts),
            OperationCommand::UnstakeLST(command) => command.execute(ctx, accounts),
            OperationCommand::UnrestakeVRT(command) => command.execute(ctx, accounts),
            OperationCommand::UndelegateVST(command) => command.execute(ctx, accounts),
            OperationCommand::HarvestReward(command) => command.execute(ctx, accounts),
            OperationCommand::StakeSOL(command) => command.execute(ctx, accounts),
            OperationCommand::NormalizeST(command) => command.execute(ctx, accounts),
            OperationCommand::RestakeVST(command) => command.execute(ctx, accounts),
            OperationCommand::DelegateVST(command) => command.execute(ctx, accounts),
        }
    }
}

type ExecutionResult = Result<(
    Option<OperationCommandResult>,
    Option<OperationCommandEntry>,
)>;

pub(super) trait SelfExecutable: Into<OperationCommand> {
    fn execute<'a, 'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> ExecutionResult;

    fn with_required_accounts(
        self,
        required_accounts: impl IntoIterator<Item = (Pubkey, bool)>,
    ) -> OperationCommandEntry {
        OperationCommandEntry {
            command: self.into(),
            required_accounts: required_accounts.into_iter().map(Into::into).collect(),
        }
    }

    fn without_required_accounts(self) -> OperationCommandEntry {
        OperationCommandEntry {
            command: self.into(),
            required_accounts: Vec::with_capacity(0),
        }
    }
}

trait DebugStructExt {
    fn field_first_element<T: std::fmt::Debug>(&mut self, name: &str, values: &[T]) -> &mut Self;
}

impl DebugStructExt for std::fmt::DebugStruct<'_, '_> {
    fn field_first_element<T: std::fmt::Debug>(&mut self, name: &str, values: &[T]) -> &mut Self {
        if values.is_empty() {
            self
        } else {
            self.field(name, &values[0])
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
            FUND_ACCOUNT_OPERATION_COMMAND_BUFFER_SIZE >= OperationCommand::INIT_SPACE,
            true
        );
    }
}
