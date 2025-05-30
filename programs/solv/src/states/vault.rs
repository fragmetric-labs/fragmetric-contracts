use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::errors::VaultError;

#[constant]
/// ## Version History
/// * v1: initial version (0x2800 = 10240 = 10KiB)
pub const VAULT_ACCOUNT_CURRENT_VERSION: u16 = 19;

pub const MAX_WITHDRAWAL_REQUESTS: usize = 80;
pub const MAX_DELEGATED_REWARD_TOKEN_MINTS: usize = 30;

#[repr(C)]
#[account(zero_copy)]
pub struct VaultAccount {
    // Header (offset = 0x0008)
    data_version: u16,
    bump: u8,
    _padding0: [u8; 5],

    /// Vault manager ensures that the vault account remains up to date.
    /// Initially, all managers are set to vault manager.
    pub(crate) vault_manager: Pubkey,
    /// Reward manager determines who can harvest rewards,
    /// accumulated in the vault's ATA.
    pub(crate) reward_manager: Pubkey,
    /// Fund manager is responsible for depositing and withdrawing VST
    /// in the vault, which directly affects the vault's TVL.
    pub(crate) fund_manager: Pubkey,
    /// Solv manager operates the vault to interact with the Solv protocol,
    /// expecting to earn yield.
    pub(crate) solv_manager: Pubkey,

    pub(crate) solv_protocol_wallet: Pubkey,
    // TODO/phase3: deprecate
    solv_protocol_withdrawal_fee_rate_bps: u16,

    _reserved0: [u8; 334],

    // VRT (offset = 0x0200)
    pub(crate) vault_receipt_token_mint: Pubkey,
    vault_receipt_token_decimals: u8,
    _padding1: [u8; 7],

    vrt_supply: u64,
    /// ∑request(state == ENQUEUED).vrt_withrawal_requested_amount
    vrt_withdrawal_enqueued_amount: u64,
    /// ∑request(state == PROCESSING).vrt_withrawal_requested_amount
    vrt_withdrawal_processing_amount: u64,
    /// ∑request(state == COMPLETED).vrt_withrawal_requested_amount
    vrt_withdrawal_completed_amount: u64,

    _reserved1: [u8; 440],

    // VST (offset = 0x0400)
    pub(crate) vault_supported_token_mint: Pubkey,
    vault_supported_token_decimals: u8,
    _padding2: [u8; 7],

    /// VST reserved amount for operation - will be deposited to the Solv protocol
    vst_operation_reserved_amount: u64,
    /// VST locked amount for withdrawal - will be locked until withdrawal is completed
    ///
    /// ## Why VST locked??
    ///
    /// If SRT is insufficient for withdrawal, insufficient amount will be taken directly from VST operation reserved.
    /// For example, assuming srt_reserve = 30, exchange_rate = 1:1.5 and trying to take 40 SRT expecting to receive 60 VST,
    /// then 10 SRT is insufficient so 10 * 1.5 = 15 VST will be taken from VST operation reserved and locked until withdrawal is completed.
    vst_withdrawal_locked_amount: u64,
    /// Ready to claim amount.
    vst_reserved_amount_to_claim: u64,
    /// Extra VST amount that exceeded the VST estimated withdrawal amount.
    vst_extra_amount_to_claim: u64,
    /// Deducted VST fee amount during withdrawal.
    vst_deducted_fee_amount: u64,
    /// Waiting for withdrawal to complete.
    /// Withdrawal fee is not applied yet, so these amount minus fee amount is the exact
    /// amount that will be able to claim when withdrawal is completed.
    vst_receivable_amount_to_claim: u64,

    _reserved2: [u8; 424],

    // SRT (offset = 0x0600)
    pub(crate) solv_receipt_token_mint: Pubkey,
    solv_receipt_token_decimals: u8,
    _padding3: [u8; 7],

    /// SRT reserved amount for operation - used to withdraw VST from the solv protocol
    srt_operation_reserved_amount: u64,
    // TODO/phase3: deprecate
    srt_operation_receivable_amount: u64,
    /// SRT locked amount for withdrawal - will be sent to the Solv protocol when withdrawal starts
    srt_withdrawal_locked_amount: u64,

    one_srt_as_micro_vst: u64,

    _reserved3: [u8; 440],

    // Withdrawal Requests (offset = 0x0800)
    withdrawal_last_created_request_id: u64,
    num_withdrawal_requests: u8,
    _padding4: [u8; 7],
    withdrawal_requests: [WithdrawalRequest; MAX_WITHDRAWAL_REQUESTS],

    _reserved4: [u8; 240],

    // Reward Delegations (offset = 0x1800)
    num_delegated_reward_token_mints: u8,
    _padding6: [u8; 7],
    delegated_reward_token_mints: [Pubkey; MAX_DELEGATED_REWARD_TOKEN_MINTS],

    _reserved6: [u8; 3128],
}

#[repr(C)]
#[zero_copy]
#[derive(Default)]
struct WithdrawalRequest {
    request_id: u64,
    vrt_withdrawal_requested_amount: u64,
    /// SRT locked amount for withdrawal - will be sent to the Solv protocol when withdrawal starts (but field remains unchanged)
    srt_withdrawal_locked_amount: u64,
    /// VST locked amount for withdrawal - will be locked until withdrawal is completed (but field remains unchanged)
    vst_withdrawal_locked_amount: u64,
    /// Total estimated amount of VST to be withdrawn by this request.
    /// Obviously this includes `vst_withdrawal_locked_amount`.
    /// First recorded as receivable amount when withdrawal starts,
    /// then recorded as reserved amount after completion, with fee amount deducted.
    vst_withdrawal_total_estimated_amount: u64,

    /// 0: enqueued
    /// 1: processing
    /// 2: completed
    state: u8,
    _reserved: [u8; 7],
}

const ENQUEUED: u8 = 0;
const PROCESSING: u8 = 1;
const COMPLETED: u8 = 2;

impl VaultAccount {
    pub const SEED: &'static [u8] = b"vault";

    pub const fn get_size() -> usize {
        8 + std::mem::size_of::<Self>()
    }

    pub fn is_initialized(&self) -> bool {
        self.data_version > 0
    }

    pub fn is_latest_version(&self) -> bool {
        self.data_version == VAULT_ACCOUNT_CURRENT_VERSION
    }

    pub fn get_bump(&self) -> u8 {
        self.bump
    }

    pub fn get_seeds(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.vault_receipt_token_mint.as_ref(),
            std::slice::from_ref(&self.bump),
        ]
    }

    pub(crate) fn initialize(
        &mut self,
        vault_manager: &Signer,
        vault_receipt_token_mint: &Account<Mint>,
        vault_supported_token_mint: &Account<Mint>,
        solv_receipt_token_mint: &Account<Mint>,
        bump: u8,
    ) -> Result<()> {
        require_eq!(vault_receipt_token_mint.supply, 0);

        if self.is_initialized() {
            err!(VaultError::InvalidAccountDataVersionError)?;
        }

        self.migrate(
            vault_manager,
            vault_receipt_token_mint,
            vault_supported_token_mint,
            solv_receipt_token_mint,
            bump,
        )
    }

    pub(crate) fn update_if_needed(
        &mut self,
        vault_manager: &Signer,
        vault_receipt_token_mint: &Account<Mint>,
        vault_supported_token_mint: &Account<Mint>,
        solv_receipt_token_mint: &Account<Mint>,
    ) -> Result<()> {
        if !self.is_initialized() {
            err!(VaultError::InvalidAccountDataVersionError)?;
        }

        self.migrate(
            vault_manager,
            vault_receipt_token_mint,
            vault_supported_token_mint,
            solv_receipt_token_mint,
            self.bump,
        )
    }

    fn migrate(
        &mut self,
        vault_manager: &Signer,
        vault_receipt_token_mint: &Account<Mint>,
        vault_supported_token_mint: &Account<Mint>,
        solv_receipt_token_mint: &Account<Mint>,
        bump: u8,
    ) -> Result<()> {
        if self.data_version == 0 {
            // Roles - initially set to vault manager
            self.vault_manager = vault_manager.key();
            self.reward_manager = vault_manager.key();
            self.fund_manager = vault_manager.key();
            self.solv_manager = vault_manager.key();

            // VRT
            self.vault_receipt_token_mint = vault_receipt_token_mint.key();
            self.vault_receipt_token_decimals = vault_receipt_token_mint.decimals;

            // VST
            self.vault_supported_token_mint = vault_supported_token_mint.key();
            self.vault_supported_token_decimals = vault_supported_token_mint.decimals;

            // SRT
            self.solv_receipt_token_mint = solv_receipt_token_mint.key();
            self.solv_receipt_token_decimals = solv_receipt_token_mint.decimals;
            self.one_srt_as_micro_vst = 10u64.pow(vault_supported_token_mint.decimals as u32 + 6);

            // Set header
            self.data_version = 1;
            self.bump = bump;
        }

        require_eq!(self.data_version, VAULT_ACCOUNT_CURRENT_VERSION);

        Ok(())
    }

    pub(crate) fn set_vault_manager(&mut self, vault_manager: Pubkey) -> Result<()> {
        self.vault_manager = vault_manager;

        Ok(())
    }

    pub(crate) fn set_reward_manager(&mut self, reward_manager: Pubkey) -> Result<()> {
        self.reward_manager = reward_manager;

        Ok(())
    }

    pub fn get_fund_manager(&self) -> Pubkey {
        self.fund_manager
    }

    pub(crate) fn set_fund_manager(&mut self, fund_manager: Pubkey) -> Result<()> {
        self.fund_manager = fund_manager;

        Ok(())
    }

    pub(crate) fn set_solv_manager(&mut self, solv_manager: Pubkey) -> Result<()> {
        self.solv_manager = solv_manager;

        Ok(())
    }

    pub(crate) fn set_solv_protocol_wallet(&mut self, solv_protocol_wallet: Pubkey) -> Result<()> {
        if self.solv_protocol_wallet != Pubkey::default() {
            err!(VaultError::SolvProtocolWalletAlreadySetError)?;
        }

        self.solv_protocol_wallet = solv_protocol_wallet;

        Ok(())
    }

    // TODO/phase3: deprecate
    pub(crate) fn set_solv_protocol_withdrawal_fee_rate(
        &mut self,
        solv_protocol_withdrawal_fee_rate_bps: u16,
    ) -> Result<()> {
        // hard limit: 10%
        if solv_protocol_withdrawal_fee_rate_bps >= 1_000 {
            err!(VaultError::InvalidSolvProtocolWithdrawalFeeRateError)?;
        }

        self.solv_protocol_withdrawal_fee_rate_bps = solv_protocol_withdrawal_fee_rate_bps;

        Ok(())
    }

    pub fn get_vrt_mint(&self) -> Pubkey {
        self.vault_receipt_token_mint
    }

    pub fn get_vrt_supply(&self) -> u64 {
        self.vrt_supply
    }

    pub fn get_vrt_withdrawal_incompleted_amount(&self) -> u64 {
        self.vrt_withdrawal_enqueued_amount + self.vrt_withdrawal_processing_amount
    }

    pub fn get_vrt_withdrawal_completed_amount(&self) -> u64 {
        self.vrt_withdrawal_completed_amount
    }

    /// VRT price = Net Asset Value (as VST) / VRT supply
    /// NAV = VST (operation reserved) + SRT (operation reserved + receivable) as VST
    pub(crate) fn mint_vrt(&mut self, vst_amount: u64) -> Result<u64> {
        let vrt_amount = self.get_vrt_amount_to_mint(vst_amount)?;

        self.vrt_supply += vrt_amount;
        self.vst_operation_reserved_amount += vst_amount;

        Ok(vrt_amount)
    }

    pub fn get_vrt_amount_to_mint(&self, vst_amount: u64) -> Result<u64> {
        let net_asset_value_as_vst = self
            .get_net_asset_value_as_vst()
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;

        if self.vrt_supply == 0 || net_asset_value_as_vst == 0 {
            // VRT price is undefined - mint as 1:1
            return Ok(vst_amount);
        }

        div_util(
            vst_amount as u128 * self.vrt_supply as u128,
            net_asset_value_as_vst,
            false,
        )
        .ok_or_else(|| error!(VaultError::CalculationArithmeticException))
    }

    pub fn get_vst_mint(&self) -> Pubkey {
        self.vault_supported_token_mint
    }

    /// NAV = VST (operation reserved) + SRT (operation reserved + receivable) as VST
    pub fn get_net_asset_value_as_vst(&self) -> Option<u64> {
        let srt_operation_reserved_amount_as_vst =
            self.get_srt_exchange_rate().get_srt_amount_as_vst(
                // TODO/phase3: deprecate srt_operation_receivable_amount
                self.srt_operation_reserved_amount + self.srt_operation_receivable_amount,
                false,
            )?;

        Some(self.vst_operation_reserved_amount + srt_operation_reserved_amount_as_vst)
    }

    pub fn get_vst_total_reserved_amount(&self) -> u64 {
        self.vst_operation_reserved_amount
            + self.vst_withdrawal_locked_amount
            + self.vst_reserved_amount_to_claim
            + self.vst_extra_amount_to_claim
    }

    pub fn get_vst_operation_reserved_amount(&self) -> u64 {
        self.vst_operation_reserved_amount
    }

    pub fn get_srt_mint(&self) -> Pubkey {
        self.solv_receipt_token_mint
    }

    pub fn get_srt_total_reserved_amount(&self) -> u64 {
        self.srt_operation_reserved_amount + self.srt_withdrawal_locked_amount
    }

    // TODO/phase3: deprecate
    pub(crate) fn get_srt_operation_receivable_amount_for_deposit(
        &self,
        vst_amount: u64,
    ) -> Result<u64> {
        self.get_srt_exchange_rate()
            .get_vst_amount_as_srt(vst_amount, false)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))
    }

    fn get_srt_exchange_rate(&self) -> SRTExchangeRate {
        SRTExchangeRate::new(self.one_srt_as_micro_vst, self.solv_receipt_token_decimals)
    }

    fn set_srt_exchange_rate(&mut self, one_srt_as_micro_vst: u64) -> Result<()> {
        // srt price must monotonically increase
        if self.one_srt_as_micro_vst > one_srt_as_micro_vst {
            err!(VaultError::InvalidSRTPriceError)?;
        }

        self.one_srt_as_micro_vst = one_srt_as_micro_vst;

        Ok(())
    }

    // TODO/phase3: deprecate
    fn is_deposit_in_progress(&self) -> bool {
        self.srt_operation_receivable_amount > 0
    }

    pub(crate) fn deposit_vst(
        &mut self,
        vst_amount: u64,
        srt_amount: u64,
        // one_srt_as_micro_vst: u64,
    ) -> Result<()> {
        // TODO/phase3: deprecate
        if self.is_deposit_in_progress() {
            err!(VaultError::DepositInProgressError)?;
        }

        let expected_srt_amount = self
            .get_srt_exchange_rate()
            .get_vst_amount_as_srt(vst_amount, false)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;

        require_gte!(srt_amount, expected_srt_amount);

        self.vst_operation_reserved_amount -= vst_amount;
        // TODO/phase3: deprecate srt_receivable_amount and replace with the code below
        // self.srt_operation_reserved_amount += srt_amount;
        // self.set_srt_exchange_rate(one_srt_as_micro_vst)?;
        self.srt_operation_receivable_amount = srt_amount;

        Ok(())
    }

    // TODO/phase3: deprecate
    pub(crate) fn resolve_srt_receivables(
        &mut self,
        srt_amount: u64,
        one_srt_as_micro_vst: u64,
    ) -> Result<()> {
        if !self.is_deposit_in_progress() {
            err!(VaultError::DepositNotInProgressError)?;
        }

        let srt_amount_as_vst =
            SRTExchangeRate::new(one_srt_as_micro_vst, self.solv_receipt_token_decimals)
                .get_srt_amount_as_vst(srt_amount, true) // provide a small tolerance here
                .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;

        let srt_operation_receivable_amount_as_vst = self
            .get_srt_exchange_rate()
            .get_srt_amount_as_vst(self.srt_operation_receivable_amount, false)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;

        require_gte!(srt_amount_as_vst, srt_operation_receivable_amount_as_vst);

        self.srt_operation_receivable_amount = 0;
        self.srt_operation_reserved_amount += srt_amount;

        self.set_srt_exchange_rate(one_srt_as_micro_vst)?;

        Ok(())
    }

    #[allow(unused)]
    fn get_withdrawal_requests_iter(&self) -> impl Iterator<Item = &WithdrawalRequest> {
        self.withdrawal_requests[..self.num_withdrawal_requests as usize].iter()
    }

    fn get_withdrawal_requests_iter_mut(&mut self) -> impl Iterator<Item = &mut WithdrawalRequest> {
        self.withdrawal_requests[..self.num_withdrawal_requests as usize].iter_mut()
    }

    // TODO/phase3: remove this comment after deprecating srt operation receivable amount
    /// returns ∆vrt_withdrawal_enqueued_amount, which "might" be less than given vrt_amount,
    /// due to srt operation receivable amount.
    pub(crate) fn enqueue_withdrawal_request(&mut self, mut vrt_amount: u64) -> Result<u64> {
        let srt_exchange_rate = self.get_srt_exchange_rate();
        let vst_operation_reserved_amount_as_srt = srt_exchange_rate
            .get_vst_amount_as_srt(self.vst_operation_reserved_amount, true)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        // TODO/phase3: deprecate srt_receivable_amount
        let net_asset_value_as_srt = vst_operation_reserved_amount_as_srt
            + self.srt_operation_reserved_amount
            + self.srt_operation_receivable_amount;

        // TODO/phase3: deprecate this block after deprecating srt_operation_receivable_amount
        {
            let net_asset_value_without_srt_receivable =
                vst_operation_reserved_amount_as_srt + self.srt_operation_reserved_amount;
            let maximum_possible_vrt_amount = if net_asset_value_as_srt == 0 {
                // When net asset value is 0, obviously srt_receivable = 0 so all vrt is possible to withdraw
                self.vrt_supply
            } else {
                div_util(
                    self.vrt_supply as u128 * net_asset_value_without_srt_receivable as u128,
                    net_asset_value_as_srt,
                    false,
                )
                .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?
            };
            vrt_amount = vrt_amount.min(maximum_possible_vrt_amount);
        }

        // Ignore empty request
        if vrt_amount == 0 {
            return Ok(0);
        }

        // Withdrawal request is full
        if self.num_withdrawal_requests as usize >= MAX_WITHDRAWAL_REQUESTS {
            err!(VaultError::ExceededMaxWithdrawalRequestsError)?;
        }

        let srt_withdrawal_required_amount = div_util(
            net_asset_value_as_srt as u128 * vrt_amount as u128,
            self.vrt_supply, // We know vrt supply > 0 because we're trying to withdraw
            true,
        )
        .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;

        let srt_withdrawal_locked_amount =
            srt_withdrawal_required_amount.min(self.srt_operation_reserved_amount);

        // Insufficient amount will be taken directly from VST operation reserved
        let insufficient_srt_amount = srt_withdrawal_required_amount - srt_withdrawal_locked_amount;
        let vst_withdrawal_locked_amount = if vst_operation_reserved_amount_as_srt == 0 {
            // When vst operation reserved amount as srt is 0, obviously insufficient srt amount = 0
            0
        } else {
            div_util(
                self.vst_operation_reserved_amount as u128 * insufficient_srt_amount as u128,
                vst_operation_reserved_amount_as_srt,
                false,
            )
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?
        };

        // VST estimation
        let vst_withdrawal_estimated_amount_from_srt = srt_exchange_rate
            .get_srt_amount_as_vst(srt_withdrawal_locked_amount, false)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        let vst_withdrawal_total_estimated_amount =
            vst_withdrawal_estimated_amount_from_srt + vst_withdrawal_locked_amount;

        // Enqueue
        self.withdrawal_last_created_request_id += 1;
        self.withdrawal_requests[self.num_withdrawal_requests as usize].initialize(
            self.withdrawal_last_created_request_id,
            vrt_amount,
            srt_withdrawal_locked_amount,
            vst_withdrawal_locked_amount,
            vst_withdrawal_total_estimated_amount,
        );
        self.num_withdrawal_requests += 1;

        // Burn VRT
        self.vrt_supply -= vrt_amount;

        // Update accountings
        self.vrt_withdrawal_enqueued_amount += vrt_amount;

        self.vst_operation_reserved_amount -= vst_withdrawal_locked_amount;
        self.vst_withdrawal_locked_amount += vst_withdrawal_locked_amount;

        self.srt_operation_reserved_amount -= srt_withdrawal_locked_amount;
        self.srt_withdrawal_locked_amount += srt_withdrawal_locked_amount;

        Ok(vrt_amount)
    }

    /// returns srt_amount_to_withdraw
    pub(crate) fn start_withdrawal_requests(&mut self) -> Result<u64> {
        let srt_amount_to_withdraw = self.srt_withdrawal_locked_amount;

        // Start
        let mut vst_receivable_amount_to_claim = 0;
        for request in self
            .get_withdrawal_requests_iter_mut()
            .skip_while(|request| request.state != ENQUEUED)
        {
            request.state = PROCESSING;

            vst_receivable_amount_to_claim += request.vst_withdrawal_total_estimated_amount;
        }

        // Update accountings
        self.vrt_withdrawal_processing_amount += self.vrt_withdrawal_enqueued_amount;
        self.vrt_withdrawal_enqueued_amount = 0;

        self.vst_receivable_amount_to_claim += vst_receivable_amount_to_claim;

        self.srt_withdrawal_locked_amount = 0;

        Ok(srt_amount_to_withdraw)
    }

    pub(crate) fn complete_withdrawal_requests(
        &mut self,
        srt_amount: u64,
        vst_amount: u64,
        one_srt_as_micro_vst: u64,
    ) -> Result<()> {
        // TODO/phase3: deprecate
        if self.is_deposit_in_progress() {
            err!(VaultError::DepositInProgressError)?;
        }

        // Validate vst_amount
        // TODO/phase3: deprecate
        let solv_protocol_withdrawal_fee_rate =
            SolvProtocolWithdrawalFeeRate(self.solv_protocol_withdrawal_fee_rate_bps);

        let srt_amount_as_vst =
            SRTExchangeRate::new(one_srt_as_micro_vst, self.solv_receipt_token_decimals)
                .get_srt_amount_as_vst(srt_amount, false)
                .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        let vst_withdrawal_fee_amount = solv_protocol_withdrawal_fee_rate
            .get_vst_withdrawal_fee(srt_amount_as_vst)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        let vst_withrawal_amount = srt_amount_as_vst - vst_withdrawal_fee_amount;

        require_gte!(vst_amount, vst_withrawal_amount);

        // Complete
        let mut vrt_withdrawal_requested_amount = 0;
        let mut srt_withdrawal_locked_amount = 0;
        let mut vst_withdrawal_locked_amount = 0;
        let mut vst_withdrawal_total_estimated_amount = 0;
        for request in self
            .get_withdrawal_requests_iter_mut()
            .skip_while(|request| request.state == COMPLETED)
            .take_while(|request| request.state == PROCESSING)
        {
            if srt_withdrawal_locked_amount + request.srt_withdrawal_locked_amount > srt_amount {
                break;
            }

            request.state = COMPLETED;

            vrt_withdrawal_requested_amount += request.vrt_withdrawal_requested_amount;
            srt_withdrawal_locked_amount += request.srt_withdrawal_locked_amount;
            vst_withdrawal_locked_amount += request.vst_withdrawal_locked_amount;
            vst_withdrawal_total_estimated_amount += request.vst_withdrawal_total_estimated_amount;
        }

        // Check exact amount
        require_eq!(srt_withdrawal_locked_amount, srt_amount);

        // Apply fee
        let vst_estimated_solv_withdrawal_amount =
            vst_withdrawal_total_estimated_amount - vst_withdrawal_locked_amount;
        let vst_estimated_solv_protocol_fee_amount = solv_protocol_withdrawal_fee_rate
            .get_vst_withdrawal_fee(vst_estimated_solv_withdrawal_amount)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        let vst_estimated_solv_withdrawal_amount_without_fee =
            vst_estimated_solv_withdrawal_amount - vst_estimated_solv_protocol_fee_amount;

        // Update accountings
        self.vrt_withdrawal_processing_amount -= vrt_withdrawal_requested_amount;
        self.vrt_withdrawal_completed_amount += vrt_withdrawal_requested_amount;

        self.vst_withdrawal_locked_amount -= vst_withdrawal_locked_amount;
        self.vst_reserved_amount_to_claim +=
            vst_estimated_solv_withdrawal_amount_without_fee + vst_withdrawal_locked_amount;
        self.vst_extra_amount_to_claim +=
            vst_amount - vst_estimated_solv_withdrawal_amount_without_fee;
        self.vst_deducted_fee_amount += vst_estimated_solv_protocol_fee_amount;
        self.vst_receivable_amount_to_claim -= vst_withdrawal_total_estimated_amount;

        self.set_srt_exchange_rate(one_srt_as_micro_vst)?;

        Ok(())
    }

    /// returns claimed_vst_amount
    pub(crate) fn claim_vst(&mut self) -> Result<u64> {
        let vst_amount = self.get_vst_claimable_amount();

        // Clear completed withdrawal requests
        let num_completed_requests = self
            .get_withdrawal_requests_iter_mut()
            .take_while(|request| request.state == COMPLETED)
            .map(|request| *request = Default::default())
            .count();
        self.withdrawal_requests[..self.num_withdrawal_requests as usize]
            .rotate_left(num_completed_requests);
        self.num_withdrawal_requests -= num_completed_requests as u8;

        // Update accountings
        self.vrt_withdrawal_completed_amount = 0;
        self.vst_reserved_amount_to_claim = 0;
        self.vst_extra_amount_to_claim = 0;
        self.vst_deducted_fee_amount = 0;

        Ok(vst_amount)
    }

    pub fn get_vst_claimable_amount(&self) -> u64 {
        self.vst_reserved_amount_to_claim + self.vst_extra_amount_to_claim
    }

    fn get_delegated_reward_token_mints_iter(&self) -> impl Iterator<Item = &Pubkey> {
        self.delegated_reward_token_mints[..self.num_delegated_reward_token_mints as usize].iter()
    }

    pub(crate) fn add_delegated_reward_token_mint(
        &mut self,
        reward_token_mint: Pubkey,
    ) -> Result<()> {
        // already registered delegated reward token
        if self
            .get_delegated_reward_token_mints_iter()
            .any(|mint| *mint == reward_token_mint.key())
        {
            return Ok(());
        }

        // validate eligible token
        if reward_token_mint.key() == self.vault_receipt_token_mint {
            err!(VaultError::NonDelegableRewardTokenMintError)?;
        }

        if reward_token_mint.key() == self.vault_supported_token_mint {
            err!(VaultError::NonDelegableRewardTokenMintError)?;
        }

        if reward_token_mint.key() == self.solv_receipt_token_mint {
            err!(VaultError::NonDelegableRewardTokenMintError)?;
        }

        if self.num_delegated_reward_token_mints as usize >= MAX_DELEGATED_REWARD_TOKEN_MINTS {
            err!(VaultError::ExceededMaxDelegatedRewardTokensError)?;
        }

        self.delegated_reward_token_mints[self.num_delegated_reward_token_mints as usize] =
            reward_token_mint.key();
        self.num_delegated_reward_token_mints += 1;

        Ok(())
    }
}

impl WithdrawalRequest {
    fn initialize(
        &mut self,
        request_id: u64,
        vrt_withdrawal_requested_amount: u64,
        srt_withdrawal_reserved_amount: u64,
        vst_withdrawal_reserved_amount: u64,
        vst_withdrawal_total_estimated_amount: u64,
    ) {
        self.request_id = request_id;
        self.vrt_withdrawal_requested_amount = vrt_withdrawal_requested_amount;
        self.srt_withdrawal_locked_amount = srt_withdrawal_reserved_amount;
        self.vst_withdrawal_locked_amount = vst_withdrawal_reserved_amount;
        self.vst_withdrawal_total_estimated_amount = vst_withdrawal_total_estimated_amount;
        self.state = ENQUEUED;
    }
}

struct SRTExchangeRate {
    one_srt_as_micro_vst: u64,
    one_srt: u64,
}

impl SRTExchangeRate {
    fn new(one_srt_as_micro_vst: u64, srt_decimals: u8) -> Self {
        Self {
            one_srt_as_micro_vst,
            one_srt: 10u64.pow(srt_decimals as u32 + 6),
        }
    }

    fn get_srt_amount_as_vst(&self, srt_amount: u64, round_up: bool) -> Option<u64> {
        div_util(
            srt_amount as u128 * self.one_srt_as_micro_vst as u128,
            self.one_srt,
            round_up,
        )
    }

    fn get_vst_amount_as_srt(&self, vst_amount: u64, round_up: bool) -> Option<u64> {
        div_util(
            vst_amount as u128 * self.one_srt as u128,
            self.one_srt_as_micro_vst,
            round_up,
        )
    }
}

struct SolvProtocolWithdrawalFeeRate(u16);

impl SolvProtocolWithdrawalFeeRate {
    fn get_vst_withdrawal_fee(&self, vst_amount: u64) -> Option<u64> {
        div_util(vst_amount as u128 * self.0 as u128, 10_000u64, true)
    }
}

/// n > 0 && d > 0
fn div_util<T1, T2>(numerator: T1, denominator: T2, round_up: bool) -> Option<u64>
where
    u128: From<T1> + From<T2>,
{
    let mut numerator = u128::from(numerator);
    let denominator = u128::from(denominator);

    if round_up {
        numerator += denominator - 1;
    }

    u64::try_from(numerator / denominator).ok()
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use bytemuck::Zeroable;
    use proptest::prelude::*;

    use super::*;

    impl VaultAccount {
        fn dummy() -> Self {
            let mut vault = VaultAccount::zeroed();
            vault.vault_receipt_token_mint = Pubkey::new_unique();
            vault.vault_supported_token_mint = Pubkey::new_unique();
            vault.solv_receipt_token_mint = Pubkey::new_unique();
            vault.solv_receipt_token_decimals = 8;
            vault.one_srt_as_micro_vst = 10u64.pow(8 + 6);
            vault
        }

        fn assert_invariants(&self) -> anyhow::Result<()> {
            let vrt_withdrawal_enqueued_amount: u64 = self
                .get_withdrawal_requests_iter()
                .filter(|request| request.state == ENQUEUED)
                .map(|request| request.vrt_withdrawal_requested_amount)
                .sum();

            if self.vrt_withdrawal_enqueued_amount != vrt_withdrawal_enqueued_amount {
                return Err(anyhow!(
                    "VRT withdrawal enqueued amount({}) != ∑request(state == ENQUEUED).vrt_withdrawal_requested_amount({})",
                    self.vrt_withdrawal_enqueued_amount,
                    vrt_withdrawal_enqueued_amount,
                ));
            }

            let vrt_withdrawal_processing_amount: u64 = self
                .get_withdrawal_requests_iter()
                .filter(|request| request.state == PROCESSING)
                .map(|request| request.vrt_withdrawal_requested_amount)
                .sum();

            if self.vrt_withdrawal_processing_amount != vrt_withdrawal_processing_amount {
                return Err(anyhow!(
                    "VRT withdrawal processing amount({}) != ∑request(state == PROCESSING).vrt_withdrawal_requested_amount({})",
                    self.vrt_withdrawal_processing_amount,
                    vrt_withdrawal_processing_amount,
                ));
            }

            let vrt_withdrawal_completed_amount: u64 = self
                .get_withdrawal_requests_iter()
                .filter(|request| request.state == COMPLETED)
                .map(|request| request.vrt_withdrawal_requested_amount)
                .sum();

            if self.vrt_withdrawal_completed_amount != vrt_withdrawal_completed_amount {
                return Err(anyhow!(
                    "VRT withdrawal completed amount({}) != ∑request(state == COMPLETED).vrt_withdrawal_requested_amount({})",
                    self.vrt_withdrawal_completed_amount,
                    vrt_withdrawal_completed_amount,
                ));
            }

            let vst_withdrawal_locked_amount: u64 = self
                .get_withdrawal_requests_iter()
                .filter(|request| request.state != COMPLETED)
                .map(|request| request.vst_withdrawal_locked_amount)
                .sum();

            if self.vst_withdrawal_locked_amount != vst_withdrawal_locked_amount {
                return Err(anyhow!(
                    "VST withdrawal locked amount({}) != ∑request(state != COMPLETED).vst_withdrawal_reserved_amount({})",
                    self.vst_withdrawal_locked_amount,
                    vst_withdrawal_locked_amount,
                ));
            }

            let srt_withdrawal_locked_amount: u64 = self
                .get_withdrawal_requests_iter()
                .filter(|request| request.state == ENQUEUED)
                .map(|request| request.srt_withdrawal_locked_amount)
                .sum();

            if self.srt_withdrawal_locked_amount != srt_withdrawal_locked_amount {
                return Err(anyhow!(
                    "SRT withdrawal locked amount({}) != ∑request(state == ENQUEUED).srt_withdrawal_reserved_amount({})",
                    self.srt_withdrawal_locked_amount,
                    srt_withdrawal_locked_amount,
                ));
            }

            let vst_receivable_amount_to_claim: u64 = self
                .get_withdrawal_requests_iter()
                .filter(|request| request.state == PROCESSING)
                .map(|request| request.vst_withdrawal_total_estimated_amount)
                .sum();

            if self.vst_receivable_amount_to_claim != vst_receivable_amount_to_claim {
                return Err(anyhow!(
                    "VST receivable amount to claim({}) != ∑request(state == PROCESSING).vst_withdrawal_total_estimated_amount({})",
                    self.vst_receivable_amount_to_claim,
                    vst_receivable_amount_to_claim,
                ));
            }

            let vst_reserved_amount_to_claim_plus_deducted_fee: u64 = self
                .get_withdrawal_requests_iter()
                .filter(|request| request.state == COMPLETED)
                .map(|request| request.vst_withdrawal_total_estimated_amount)
                .sum();

            if self.vst_reserved_amount_to_claim + self.vst_deducted_fee_amount
                != vst_reserved_amount_to_claim_plus_deducted_fee
            {
                return Err(anyhow!(
                    "VST reserved amount to claim({}) + VST deducted fee amount({}) != ∑request(state == COMPLETED).vst_withdrawal_total_estimated_amount({})",
                    self.vst_reserved_amount_to_claim,
                    self.vst_deducted_fee_amount,
                    vst_reserved_amount_to_claim_plus_deducted_fee,
                ));
            }

            Ok(())
        }
    }

    #[test]
    fn test_account_size() {
        assert_eq!(VaultAccount::get_size(), 1024 * 10);
    }

    proptest! {
        #[test]
        fn test_initial_mint_amount(vst_amount in 0..u64::MAX) {
            let mut vault = VaultAccount::dummy();

            // Initial mint
            let vrt_amount = vault.mint_vrt(vst_amount).unwrap();
            assert_eq!(vrt_amount, vst_amount);

            assert_eq!(vault.vrt_supply, vrt_amount);
            assert_eq!(vault.vst_operation_reserved_amount, vst_amount);

            vault.assert_invariants().unwrap();
        }
    }

    // TODO/phase3: deprecate
    proptest! {
        #[test]
        fn test_resolve_srt_receivables(vst_amount in 1..u64::MAX) {
            let mut vault = VaultAccount::dummy();
            vault.mint_vrt(vst_amount).unwrap();
            vault.deposit_vst(vst_amount, vst_amount).unwrap();
            vault.resolve_srt_receivables(vst_amount, 100_000_000_000_000).unwrap();

            assert_eq!(vault.srt_operation_receivable_amount, 0);
        }
    }

    #[test]
    fn test_invalid_srt_exchange_rate_while_deposit() {
        let mut vault = VaultAccount::dummy();
        vault.mint_vrt(1_000_000).unwrap();

        vault.deposit_vst(1_000_000, 1_000_000).unwrap();
        vault
            .resolve_srt_receivables(2_000_000, 50_000_000_000_000)
            .unwrap_err();
    }

    #[test]
    fn test_valid_srt_exchnage_rate_while_deposit() {
        let mut vault = VaultAccount::dummy();
        vault.mint_vrt(1_000_000).unwrap();

        vault.deposit_vst(1_000_000, 1_000_000).unwrap();
        vault
            .resolve_srt_receivables(500_000, 200_000_000_000_000)
            .unwrap();

        vault.assert_invariants().unwrap();
    }

    #[test]
    fn test_simple_withdraw_procedure() {
        let mut vault = VaultAccount::dummy();
        let vrt_supply = vault.mint_vrt(1_000_000).unwrap();
        vault.deposit_vst(1_000_000, 1_000_000).unwrap();
        // 1 SRT = 2.18919457342449 => 1_000_000 VST = 456_789.xxx SRT => 456_789
        vault
            .resolve_srt_receivables(456_789, 218_919_457_342_449)
            .unwrap();
        vault.assert_invariants().unwrap();

        let srt_amount = 456_789;
        assert_eq!(vault.srt_operation_reserved_amount, srt_amount);

        vault.enqueue_withdrawal_request(500_000).unwrap();
        vault.enqueue_withdrawal_request(500_000).unwrap();
        vault.assert_invariants().unwrap();

        assert_eq!(
            vault.vrt_supply + vault.vrt_withdrawal_enqueued_amount,
            vrt_supply,
        );

        assert_eq!(vault.get_srt_total_reserved_amount(), srt_amount);
        assert_eq!(vault.srt_operation_reserved_amount, 0);
        assert_eq!(vault.num_withdrawal_requests, 2);
        assert_eq!(
            vault.withdrawal_requests[0].srt_withdrawal_locked_amount,
            228_395 // ceil(456_789 / 2)
        );
        assert_eq!(
            vault.withdrawal_requests[1].srt_withdrawal_locked_amount,
            228_394
        );

        vault.start_withdrawal_requests().unwrap();
        vault.assert_invariants().unwrap();

        assert!(vault
            .get_withdrawal_requests_iter()
            .all(|request| request.state > 0));

        assert_eq!(vault.vrt_withdrawal_enqueued_amount, 0);
        assert_eq!(vault.srt_withdrawal_locked_amount, 0);
        assert_eq!(vault.vst_receivable_amount_to_claim, 999_999);

        // 1 SRT = 2.18919457342449 => 456_789 SRT = 999_999.xxx VRT => 999_999
        vault
            .complete_withdrawal_requests(456_789, 1_000_000, 218_919_457_342_449)
            .unwrap();
        vault.assert_invariants().unwrap();

        assert!(vault
            .get_withdrawal_requests_iter()
            .take(2)
            .all(|request| request.state == COMPLETED));

        assert_eq!(vault.vst_reserved_amount_to_claim, 999_999);
        assert_eq!(vault.vst_receivable_amount_to_claim, 0);
        assert_eq!(vault.vst_extra_amount_to_claim, 1);
    }

    #[test]
    fn test_withdraw_procedure_partial_complete() {
        let mut vault: VaultAccount = VaultAccount::dummy();
        vault.mint_vrt(1_000_000).unwrap();
        vault.deposit_vst(1_000_000, 1_000_000).unwrap();
        // 1 SRT = 2.18919457342449 => 1_000_000 VST = 456_789.xxx SRT => 456_789
        vault
            .resolve_srt_receivables(456_789, 218_919_457_342_449)
            .unwrap();
        vault.assert_invariants().unwrap();

        vault.enqueue_withdrawal_request(500_000).unwrap();
        vault.enqueue_withdrawal_request(500_000).unwrap();
        vault.assert_invariants().unwrap();

        vault.start_withdrawal_requests().unwrap();
        vault.assert_invariants().unwrap();

        // (Expected) 1 SRT = 2.18919457342449 => 228_395 SRT = 500001.xxx VRT => 500_001
        // (Actual)   1 SRT = 2.18919936600787 => 228_395 SRT = 500002.xxx VRT => 500_002 (+ 0)
        vault
            .complete_withdrawal_requests(228_395, 500_002, 218_919_936_600_787)
            .unwrap();
        vault.assert_invariants().unwrap();

        assert!(vault
            .get_withdrawal_requests_iter()
            .take(1)
            .all(|request| request.state == COMPLETED));

        assert_eq!(vault.vst_reserved_amount_to_claim, 500_001);
        assert_eq!(vault.vst_receivable_amount_to_claim, 499_998);
        assert_eq!(vault.vst_extra_amount_to_claim, 1);

        // (Expected) 1 SRT = 2.18919457342449 => 228_394 SRT = 499998.xxx VRT => 499_998
        // (Actual)   1 SRT = 2.18919936600787 => 228_394 SRT = 500000.xxx VRT => 500_000 (+ 2)
        vault
            .complete_withdrawal_requests(228_394, 500_000, 218_919_936_600_787)
            .unwrap();
        vault.assert_invariants().unwrap();

        assert!(vault
            .get_withdrawal_requests_iter()
            .take(2)
            .all(|request| request.state == COMPLETED));

        assert_eq!(vault.vst_reserved_amount_to_claim, 999_999);
        assert_eq!(vault.vst_receivable_amount_to_claim, 0);
        assert_eq!(vault.vst_extra_amount_to_claim, 3);
    }

    // TODO/phase3: deprecate
    #[test]
    fn test_complete_withdrawal_should_fail_during_deposit() {
        let mut vault: VaultAccount = VaultAccount::dummy();
        vault.mint_vrt(1_000_000).unwrap();
        vault.deposit_vst(1_000_000, 1_000_000).unwrap();
        // 1 SRT = 2.18919457342449 => 1_000_000 VST = 456_789.xxx SRT => 456_789
        vault
            .resolve_srt_receivables(456_789, 218_919_457_342_449)
            .unwrap();
        vault.assert_invariants().unwrap();

        vault.enqueue_withdrawal_request(500_000).unwrap();
        vault.enqueue_withdrawal_request(500_000).unwrap();
        vault.assert_invariants().unwrap();

        vault.start_withdrawal_requests().unwrap();
        vault.assert_invariants().unwrap();

        let vrt_amount = vault.mint_vrt(1_000_000).unwrap();
        assert_eq!(vrt_amount, 1_000_000);
        vault.deposit_vst(1_000_000, 456_789).unwrap();

        // 1 SRT = 2.18919457 => 456_789 SRT = 999_999.xxx VRT => 999_999
        vault
            .complete_withdrawal_requests(456_789, 1_000_000, 218_919_457)
            .unwrap_err();
    }

    #[test]
    fn test_enqueue_withdrawal_can_only_enqueue_maximum_possible_amount() {
        let mut vault: VaultAccount = VaultAccount::dummy();
        vault.mint_vrt(1_000_000).unwrap();
        vault.deposit_vst(1_000_000, 1_000_000).unwrap();
        // 1 SRT = 2.18919457342449 => 1_000_000 VST = 456_789.xxx SRT => 456_789
        vault
            .resolve_srt_receivables(456_789, 218_919_457_342_449)
            .unwrap();
        vault.assert_invariants().unwrap();

        let vrt_amount = vault.mint_vrt(1_000_000).unwrap();
        assert_eq!(vrt_amount, 1_000_001);
        vault.deposit_vst(1_000_000, 456_789).unwrap();

        let vrt_amount = vault.mint_vrt(1_000_000).unwrap();
        assert_eq!(vrt_amount, 1_000_001);

        let vrt_amount = vault.enqueue_withdrawal_request(3_000_000).unwrap();
        assert_eq!(vrt_amount, 2_000_002);
    }

    #[test]
    fn test_withdrawal_with_fee() {
        let mut vault: VaultAccount = VaultAccount::dummy();
        vault.mint_vrt(1_000_000).unwrap();
        vault.deposit_vst(1_000_000, 1_000_000).unwrap();
        // 1 SRT = 2.18919457342449 => 1_000_000 VST = 456_789.xxx SRT => 456_789
        vault
            .resolve_srt_receivables(456_789, 218_919_457_342_449)
            .unwrap();
        vault.assert_invariants().unwrap();

        vault.enqueue_withdrawal_request(500_000).unwrap();
        vault.enqueue_withdrawal_request(500_000).unwrap();
        vault.assert_invariants().unwrap();

        vault.start_withdrawal_requests().unwrap();
        vault.assert_invariants().unwrap();

        vault.set_solv_protocol_withdrawal_fee_rate(100).unwrap();

        // 1 SRT = 2.18919457342449 => 456_789 SRT = 999_999.xxx VRT => 999_999
        vault
            .complete_withdrawal_requests(456_789, 990_000, 218_919_457_342_449)
            .unwrap();
        vault.assert_invariants().unwrap();

        assert!(vault
            .get_withdrawal_requests_iter()
            .take(2)
            .all(|request| request.state == COMPLETED));

        assert_eq!(vault.vst_reserved_amount_to_claim, 989_999);
        assert_eq!(vault.vst_receivable_amount_to_claim, 0);
        assert_eq!(vault.vst_extra_amount_to_claim, 1);
    }

    #[test]
    fn test_delegate_reward_token() {
        let mut vault = VaultAccount::dummy();
        let reward_token_mint = Pubkey::new_unique();
        vault
            .add_delegated_reward_token_mint(reward_token_mint)
            .unwrap();
        assert_eq!(vault.num_delegated_reward_token_mints, 1);

        // Add delegation is idempotent
        vault
            .add_delegated_reward_token_mint(reward_token_mint)
            .unwrap();
        assert_eq!(vault.num_delegated_reward_token_mints, 1);

        // Cannot delegate vrt, vst, srt
        vault
            .add_delegated_reward_token_mint(vault.vault_receipt_token_mint)
            .unwrap_err();
        vault
            .add_delegated_reward_token_mint(vault.vault_supported_token_mint)
            .unwrap_err();
        vault
            .add_delegated_reward_token_mint(vault.solv_receipt_token_mint)
            .unwrap_err();
    }
}
