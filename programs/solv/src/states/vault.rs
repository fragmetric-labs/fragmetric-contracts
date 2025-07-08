use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::constants;
use crate::errors::VaultError;

#[constant]
/// ## Version History
/// * v1: deprecated
/// * v2: initial version (0x2800 = 10240 = 10KiB)
pub const VAULT_ACCOUNT_CURRENT_VERSION: u16 = 2;

pub const MAX_WITHDRAWAL_REQUESTS: usize = 60;
pub const MAX_DELEGATED_REWARD_TOKEN_MINTS: usize = 30;

const SOLV_PROTOCOL_MAX_EXTRA_FEE_AMOUNT: u64 = 20_000;

#[repr(C)]
#[account(zero_copy)]
#[cfg_attr(test, derive(Debug))]
pub struct VaultAccount {
    // Header (offset = 0x0008)
    pub(crate) data_version: u16,
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

    // TODO/phase3: deprecate
    pub(crate) solv_protocol_wallet: Pubkey,
    // TODO/phase3: deprecate
    solv_protocol_deposit_fee_rate_bps: u16,
    // TODO/phase3: deprecate
    solv_protocol_withdrawal_fee_rate_bps: u16,

    _reserved0: [u8; 332],

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

    /// informative VRT redemption rate.
    one_vrt_as_micro_vst: u64,

    _reserved1: [u8; 432],

    // VST (offset = 0x0400)
    pub(crate) vault_supported_token_mint: Pubkey,
    vault_supported_token_decimals: u8,
    _padding2: [u8; 7],

    /// VST reserved amount for operation - will be deposited to the Solv protocol
    vst_operation_reserved_amount: u64,
    /// VST receivable amount for operation - offsetted by fund manager OR via donation
    ///
    /// ## Where does VST receivables come from?
    ///
    /// During deposit & withdraw from solv protocol, there is a protocol fee.
    /// It will prevent the NAV loss considering fee as receivable.
    /// Receivables will then be offsetted when fund manager withdraws VST, as a withdrawal fee.
    ///
    /// Another scenario is when solv manager adjusts SRT exchange rate (to lower price).
    /// By human fault, SRT exchange rate might be set higher (for example, missed protocol extra fee).
    /// In this case, by adjusting SRT exchange rate, decreased net asset value will be VST receivable.
    vst_operation_receivable_amount: u64,
    /// VST locked amount for withdrawal - will be locked until withdrawal is completed
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

    _reserved2: [u8; 416],

    // SRT (offset = 0x0600)
    pub(crate) solv_receipt_token_mint: Pubkey,
    solv_receipt_token_decimals: u8,
    _padding3: [u8; 7],

    /// SRT reserved amount for operation - used to withdraw VST from the solv protocol
    srt_operation_reserved_amount: u64,
    /// SRT receivable amount for operation - will be offsetted when deposit completes to solv protocol
    srt_operation_receivable_amount: u64,
    /// SRT locked amount for withdrawal - will be sent to the Solv protocol when withdrawal starts
    srt_withdrawal_locked_amount: u64,

    /// SRT redemption rate being used for vault net asset value appreciation.
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

const BPS: u16 = 10_000;
const MICRO: u64 = 1_000_000;

#[repr(C)]
#[zero_copy]
#[derive(Default)]
#[cfg_attr(test, derive(Debug))]
struct WithdrawalRequest {
    request_id: u64,
    vrt_withdrawal_requested_amount: u64,
    /// Locked SRT amount for withdrawal - will be sent to the Solv protocol when withdrawal starts (but field remains unchanged)
    srt_withdrawal_locked_amount: u64,
    /// Locked VST amount for withdrawal - will be locked until withdrawal is completed (but field remains unchanged)
    vst_withdrawal_locked_amount: u64,
    /// Total estimated amount of VST to be withdrawn by this request.
    /// First recorded as `vst_receivable_amount_to_claim` when withdrawal starts,
    /// then after withdrawal completes, the actual VST withdrawn amount is recorded as
    /// `vst_reserved_amount_to_claim` + `vst_extra_amount_to_claim` + `vst_deducted_fee_amount`.
    /// Deducted solv protocol withdrawal fee is added to `vst_deducted_fee_amount`.
    ///
    /// ```txt
    /// `vst_withdrawal_total_estimated_amount` = `srt_withdrawal_locked_amount` as VST + `vst_withdrawal_locked_amount` + `vst_deducted_fee_amount`
    /// ```
    vst_withdrawal_total_estimated_amount: u64,
    /// SRT price when requeest is enqueued - will be used for price validation
    one_srt_as_micro_vst: u64,

    /// 0: enqueued
    /// 1: processing
    /// 2: completed
    state: u8,
    _reserved: [u8; 15],
}

const WITHDRAWAL_REQUEST_STATE_ENQUEUED: u8 = 0;
const WITHDRAWAL_REQUEST_STATE_PROCESSING: u8 = 1;
const WITHDRAWAL_REQUEST_STATE_COMPLETED: u8 = 2;

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
            require_eq!(
                [
                    constants::ZBTC_MINT_ADDRESS,
                    constants::CBBTC_MINT_ADDRESS,
                    constants::WBTC_MINT_ADDRESS
                ]
                .contains(&vault_supported_token_mint.key()),
                true
            );
            require_eq!(vault_receipt_token_mint.decimals, 8);
            require_eq!(vault_receipt_token_mint.supply, 0);
            require_eq!(vault_supported_token_mint.decimals, 8);
            require_eq!(solv_receipt_token_mint.decimals, 8);

            // Roles - initially set to vault manager
            self.vault_manager = vault_manager.key();
            self.reward_manager = vault_manager.key();
            self.fund_manager = vault_manager.key();
            self.solv_manager = vault_manager.key();

            // VRT
            self.vault_receipt_token_mint = vault_receipt_token_mint.key();
            self.vault_receipt_token_decimals = vault_receipt_token_mint.decimals;
            self.one_vrt_as_micro_vst = 10u64.pow(vault_receipt_token_mint.decimals as u32 + 6);

            // VST
            self.vault_supported_token_mint = vault_supported_token_mint.key();
            self.vault_supported_token_decimals = vault_supported_token_mint.decimals;

            // SRT
            self.solv_receipt_token_mint = solv_receipt_token_mint.key();
            self.solv_receipt_token_decimals = solv_receipt_token_mint.decimals;
            self.one_srt_as_micro_vst = 10u64.pow(solv_receipt_token_mint.decimals as u32 + 6);

            // Set header
            self.bump = bump;
            self.data_version = 2; // skip account version number 1, which is a totally reset version.
        }

        require_eq!(self.data_version, VAULT_ACCOUNT_CURRENT_VERSION);

        Ok(())
    }

    pub fn get_vault_manager(&self) -> Pubkey {
        self.vault_manager
    }

    pub(crate) fn set_vault_manager(&mut self, vault_manager: Pubkey) -> Result<()> {
        self.vault_manager = vault_manager;

        Ok(())
    }

    pub fn get_reward_manager(&self) -> Pubkey {
        self.reward_manager
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

    pub fn get_solv_manager(&self) -> Pubkey {
        self.solv_manager
    }

    pub(crate) fn set_solv_manager(&mut self, solv_manager: Pubkey) -> Result<()> {
        self.solv_manager = solv_manager;

        Ok(())
    }

    // TODO/phase3: deprecate
    pub(crate) fn set_solv_protocol_wallet(&mut self, solv_protocol_wallet: Pubkey) -> Result<()> {
        if self.solv_protocol_wallet != Pubkey::default() {
            err!(VaultError::SolvProtocolWalletAlreadySetError)?;
        }

        self.solv_protocol_wallet = solv_protocol_wallet;

        Ok(())
    }

    pub fn get_withdrawal_fee_rate_bps(&self) -> u16 {
        self.solv_protocol_deposit_fee_rate_bps + self.solv_protocol_withdrawal_fee_rate_bps
    }

    // TODO/phase3: deprecate
    pub(crate) fn set_solv_protocol_deposit_fee_rate_bps(
        &mut self,
        fee_rate_bps: u16,
    ) -> Result<&mut Self> {
        // hard limit: 10%
        if fee_rate_bps >= 1_000 {
            err!(VaultError::InvalidSolvProtocolDepositFeeRateError)?;
        }

        self.solv_protocol_deposit_fee_rate_bps = fee_rate_bps;

        Ok(self)
    }

    // TODO/phase3: deprecate
    pub(crate) fn set_solv_protocol_withdrawal_fee_rate_bps(
        &mut self,
        fee_rate_bps: u16,
    ) -> Result<&mut Self> {
        // hard limit: 10%
        if fee_rate_bps >= 1_000 {
            err!(VaultError::InvalidSolvProtocolWithdrawalFeeRateError)?;
        }

        self.solv_protocol_withdrawal_fee_rate_bps = fee_rate_bps;

        Ok(self)
    }

    pub fn get_vrt_mint(&self) -> Pubkey {
        self.vault_receipt_token_mint
    }

    pub fn get_vrt_supply(&self) -> u64 {
        self.vrt_supply
    }

    /// ENQUEUED + PROCESSING
    pub fn get_vrt_withdrawal_incompleted_amount(&self) -> u64 {
        self.vrt_withdrawal_enqueued_amount + self.vrt_withdrawal_processing_amount
    }

    /// COMPLETE
    pub fn get_vrt_withdrawal_completed_amount(&self) -> u64 {
        self.vrt_withdrawal_completed_amount
    }

    /// * VRT price = NAV / VRT supply
    /// * NAV = VST (operation reserved + receivable) + floor(SRT (operation reserved) as VST) + floor(SRT (operation receivable) as VST)
    fn update_vrt_exchange_rate(&mut self) -> Result<()> {
        let net_asset_value_as_vst = self
            .get_net_asset_value_as_vst()
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;

        self.one_vrt_as_micro_vst = if self.vrt_supply == 0 || net_asset_value_as_vst == 0 {
            10u64.pow(self.vault_receipt_token_decimals as u32 + 6)
        } else {
            div_util(
                10u64.pow(self.vault_receipt_token_decimals as u32 + 6) as u128
                    * net_asset_value_as_vst as u128,
                self.vrt_supply,
                false,
            )
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?
        };

        Ok(())
    }

    /// informative VRT redepmtion rate
    pub fn get_one_vrt_as_micro_vst(&self) -> u64 {
        self.one_vrt_as_micro_vst
    }

    /// VRT mint amount = floor(VST * VRT supply / NAV) ?? VST
    /// * VRT price = NAV / VRT supply
    /// * NAV = VST (operation reserved + receivable) + floor(SRT (operation reserved) as VST) + floor(SRT (operation receivable) as VST)
    pub(crate) fn mint_vrt(&mut self, vst_amount: u64) -> Result<u64> {
        let vrt_amount = self.get_vrt_amount_to_mint(vst_amount)?;

        self.vrt_supply += vrt_amount;
        self.vst_operation_reserved_amount += vst_amount;

        self.update_vrt_exchange_rate()?;

        msg!(
            "Mint VRT: vst_amount={}, vrt_amount={}",
            vst_amount,
            vrt_amount
        );

        Ok(vrt_amount)
    }

    /// VRT mint amount = floor(VST * VRT supply / NAV) ?? VST
    /// * VRT price = NAV / VRT supply
    /// * NAV = VST (operation reserved + receivable) + floor(SRT (operation reserved) as VST) + floor(SRT (operation receivable) as VST)
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

    /// NAV = VST (operation reserved + receivable) + floor(SRT (operation reserved) as VST) + floor(SRT (operation receivable) as VST)
    pub fn get_net_asset_value_as_vst(&self) -> Option<u64> {
        let srt_exchange_rate = self.get_srt_exchange_rate();
        let srt_operation_reserved_amount_as_vst =
            srt_exchange_rate.get_srt_amount_as_vst(self.srt_operation_reserved_amount, false)?;
        let srt_operation_receivable_amount_as_vst =
            srt_exchange_rate.get_srt_amount_as_vst(self.srt_operation_receivable_amount, false)?;

        Some(
            self.vst_operation_reserved_amount
                + self.vst_operation_receivable_amount
                + srt_operation_reserved_amount_as_vst
                + srt_operation_receivable_amount_as_vst,
        )
    }

    /// NAV = VST (operation reserved + receivable) + floor(SRT (operation reserved) as VST) + floor(SRT (operation receivable) as VST)
    pub fn get_net_asset_value_as_micro_vst(&self) -> Option<u64> {
        let srt_exchange_rate = self.get_srt_exchange_rate();
        let srt_operation_reserved_amount_as_micro_vst = srt_exchange_rate.get_srt_amount_as_vst(
            self.srt_operation_reserved_amount.checked_mul(MICRO)?,
            false,
        )?;
        let srt_operation_receivable_amount_as_micro_vst = srt_exchange_rate
            .get_srt_amount_as_vst(
                self.srt_operation_receivable_amount.checked_mul(MICRO)?,
                false,
            )?;

        let vst_operation_reserved_amount_as_micro =
            self.vst_operation_reserved_amount.checked_mul(MICRO)?;
        let vst_operation_receivable_amount_as_micro =
            self.vst_operation_receivable_amount.checked_mul(MICRO)?;

        vst_operation_reserved_amount_as_micro
            .checked_add(vst_operation_receivable_amount_as_micro)?
            .checked_add(srt_operation_reserved_amount_as_micro_vst)?
            .checked_add(srt_operation_receivable_amount_as_micro_vst)
    }

    /// Minimum amount of VST required in vault token account
    pub(crate) fn get_vst_total_reserved_amount(&self) -> u64 {
        self.vst_operation_reserved_amount
            + self.vst_withdrawal_locked_amount
            + self.vst_reserved_amount_to_claim
            + self.vst_extra_amount_to_claim
    }

    pub fn get_vst_operation_reserved_amount(&self) -> u64 {
        self.vst_operation_reserved_amount
    }

    pub fn get_vst_deducted_fee_amount(&self) -> u64 {
        self.vst_deducted_fee_amount
    }

    pub fn get_vst_estimated_amount_from_last_withdrawal_request(&self) -> Result<u64> {
        if self.num_withdrawal_requests == 0 {
            err!(VaultError::WithdrawalRequestNotFoundError)?;
        }
        Ok(
            self.withdrawal_requests[self.num_withdrawal_requests as usize - 1]
                .vst_withdrawal_total_estimated_amount,
        )
    }

    pub fn get_vst_total_estimated_amount_from_completed_withdrawal_requests(&self) -> u64 {
        self.vst_reserved_amount_to_claim + self.vst_deducted_fee_amount
    }

    /// Minimum amount of SRT required in vault token account
    pub(crate) fn get_srt_total_reserved_amount(&self) -> u64 {
        self.srt_operation_reserved_amount + self.srt_withdrawal_locked_amount
    }

    pub(crate) fn get_one_srt_as_micro_vst(&self) -> u64 {
        self.one_srt_as_micro_vst
    }

    fn get_srt_exchange_rate(&self) -> SRTExchangeRate {
        SRTExchangeRate::new(self.one_srt_as_micro_vst, self.solv_receipt_token_decimals)
    }

    pub(crate) fn refresh_srt_exchange_rate_with_validation(
        &mut self,
        new_one_srt_as_micro_vst: u64,
        heuristic_validation: bool,
    ) -> Result<()> {
        if self.is_deposit_in_progress() {
            err!(VaultError::DepositInProgressError)?;
        }

        // srt price must monotonically increase
        if self.one_srt_as_micro_vst > new_one_srt_as_micro_vst {
            err!(VaultError::InvalidSRTPriceError)?;
        }

        // TODO: deprecate this heuristic assertion
        if heuristic_validation
            && new_one_srt_as_micro_vst - self.one_srt_as_micro_vst > self.one_srt_as_micro_vst / 10
        {
            err!(VaultError::InvalidSRTPriceError)?;
        }

        let old_one_srt_as_micro_vst = self.one_srt_as_micro_vst;
        self.one_srt_as_micro_vst = new_one_srt_as_micro_vst;
        self.update_vrt_exchange_rate()?;

        msg!(
            "Refresh SRT exchange rate: old={}, new = {}",
            old_one_srt_as_micro_vst,
            new_one_srt_as_micro_vst,
        );

        Ok(())
    }

    /// returns ∆vst_operation_receivable_amount
    pub(crate) fn adjust_srt_exchange_rate_with_extra_vst_receivables(
        &mut self,
        new_one_srt_as_micro_vst: u64,
        heuristic_validation: bool,
    ) -> Result<u64> {
        if self.is_deposit_in_progress() {
            err!(VaultError::DepositInProgressError)?;
        }

        if new_one_srt_as_micro_vst > self.one_srt_as_micro_vst {
            err!(VaultError::InvalidSRTPriceError)?;
        }

        // TODO: deprecate this heuristic assertion
        if heuristic_validation
            && self.one_srt_as_micro_vst - new_one_srt_as_micro_vst > self.one_srt_as_micro_vst / 10
        {
            err!(VaultError::InvalidSRTPriceError)?;
        }

        let srt_operation_reserved_amount_as_vst = self
            .get_srt_exchange_rate()
            .get_srt_amount_as_vst(self.srt_operation_reserved_amount, false)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        let post_srt_operation_reserved_amount_as_vst =
            SRTExchangeRate::new(new_one_srt_as_micro_vst, self.solv_receipt_token_decimals)
                .get_srt_amount_as_vst(self.srt_operation_reserved_amount, false)
                .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        let extra_vst_receivable_amount =
            srt_operation_reserved_amount_as_vst - post_srt_operation_reserved_amount_as_vst;

        self.vst_operation_receivable_amount += extra_vst_receivable_amount;
        let old_one_srt_as_micro_vst = self.one_srt_as_micro_vst;
        self.one_srt_as_micro_vst = new_one_srt_as_micro_vst;

        msg!(
            "Adjust SRT exchange rate: old={}, new={}, extra_vst_receivable_amount={}",
            old_one_srt_as_micro_vst,
            new_one_srt_as_micro_vst,
            extra_vst_receivable_amount,
        );

        Ok(extra_vst_receivable_amount)
    }

    fn is_deposit_in_progress(&self) -> bool {
        self.srt_operation_receivable_amount > 0
    }

    /// Protocol extra fee is not accounted here - we don't know exact amount now
    /// * VST protocol deposit fee = ceil(VST * solv_protocol_deposit_fee_rate)  
    /// * SRT expected = ceil((VST - protocol fee) as SRT)
    ///
    /// returns (srt_expected_amount, solv_protocol_deposit_fee_amount_as_vst)
    pub(crate) fn deposit_vst(&mut self, vst_amount: u64) -> Result<(u64, u64)> {
        if self.is_deposit_in_progress() {
            err!(VaultError::DepositInProgressError)?;
        }

        require_gte!(self.vst_operation_reserved_amount, vst_amount);

        let solv_protocol_deposit_fee_amount_as_vst = div_util(
            vst_amount as u128 * self.solv_protocol_deposit_fee_rate_bps as u128,
            BPS,
            true,
        )
        .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        let srt_estimated_amount = self
            .get_srt_exchange_rate()
            .get_vst_amount_as_srt(vst_amount - solv_protocol_deposit_fee_amount_as_vst, true)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;

        self.vst_operation_reserved_amount -= vst_amount;
        self.vst_operation_receivable_amount += solv_protocol_deposit_fee_amount_as_vst;
        self.srt_operation_receivable_amount = srt_estimated_amount;

        msg!(
            "Deposit VST: vst_amount={}, deposit_fee_amount_as_vst={}, srt_estimated_amount={}",
            vst_amount,
            solv_protocol_deposit_fee_amount_as_vst,
            srt_estimated_amount,
        );

        Ok((
            srt_estimated_amount,
            solv_protocol_deposit_fee_amount_as_vst,
        ))
    }

    /// Offset VST receivables with VST donation.
    /// Cannot donate more than VST receivable.
    pub(crate) fn donate_vst(&mut self, mut vst_amount: u64) -> Result<u64> {
        vst_amount = std::cmp::min(vst_amount, self.vst_operation_receivable_amount);

        self.vst_operation_reserved_amount += vst_amount;
        self.vst_operation_receivable_amount -= vst_amount;

        msg!("Donate VST: vst_amount={}", vst_amount);

        Ok(vst_amount)
    }

    /// Offset VST receivables with SRT donation.
    /// Cannot donate more than VST receivable.
    pub(crate) fn donate_srt(&mut self, mut srt_amount: u64) -> Result<u64> {
        let srt_exchange_rate = self.get_srt_exchange_rate();

        let maximum_possible_srt_donation_amount = srt_exchange_rate
            .get_vst_amount_as_srt(self.vst_operation_receivable_amount, false)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        srt_amount = std::cmp::min(srt_amount, maximum_possible_srt_donation_amount);

        let srt_operation_reserved_amount_as_vst = srt_exchange_rate
            .get_srt_amount_as_vst(self.srt_operation_reserved_amount, false)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        let post_srt_operation_reserved_amount_as_vst = srt_exchange_rate
            .get_srt_amount_as_vst(self.srt_operation_reserved_amount + srt_amount, false)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        let offsetting_vst_operation_receivable_amount =
            post_srt_operation_reserved_amount_as_vst - srt_operation_reserved_amount_as_vst;

        // Validation
        require_gte!(
            self.vst_operation_receivable_amount,
            offsetting_vst_operation_receivable_amount,
        );

        self.vst_operation_receivable_amount -= offsetting_vst_operation_receivable_amount;
        self.srt_operation_reserved_amount += srt_amount;

        msg!(
            "Donate SRT: srt_amount={}, offsetted_vst_operation_receivable_amount={}",
            srt_amount,
            offsetting_vst_operation_receivable_amount,
        );

        Ok(srt_amount)
    }

    /// VST protocol extra fee = max(floor(SRT receivable as VST (w/ old price)) - floor(SRT as VST (w/ new price)), 0) ≤ HARD LIMIT(20000)
    ///
    /// returns (srt_operation_reserved_amount, solv_protocol_extra_fee_amount_as_vst)
    pub(crate) fn offset_srt_receivables(
        &mut self,
        srt_amount: u64,
        new_one_srt_as_micro_vst: u64,
        heuristic_validation: bool,
    ) -> Result<(u64, u64)> {
        if !self.is_deposit_in_progress() {
            err!(VaultError::DepositNotInProgressError)?;
        }

        let srt_amount_as_vst =
            SRTExchangeRate::new(new_one_srt_as_micro_vst, self.solv_receipt_token_decimals)
                .get_srt_amount_as_vst(srt_amount, false)
                .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        let srt_operation_receivable_amount_as_vst = self
            .get_srt_exchange_rate()
            .get_srt_amount_as_vst(self.srt_operation_receivable_amount, false)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;

        let solv_protocol_extra_fee_amount_as_vst =
            srt_operation_receivable_amount_as_vst.saturating_sub(srt_amount_as_vst);

        if solv_protocol_extra_fee_amount_as_vst > SOLV_PROTOCOL_MAX_EXTRA_FEE_AMOUNT {
            err!(VaultError::InvalidSolvProtocolExtraFeeAmountError)?;
        }

        let offsetted_srt_operation_receivable_amount = self.srt_operation_receivable_amount;
        self.vst_operation_receivable_amount += solv_protocol_extra_fee_amount_as_vst;
        self.srt_operation_reserved_amount += srt_amount;
        self.srt_operation_receivable_amount = 0;

        let old_one_srt_as_micro_vst = self.one_srt_as_micro_vst;
        self.refresh_srt_exchange_rate_with_validation(
            new_one_srt_as_micro_vst,
            heuristic_validation,
        )?;

        msg!(
            "Offset SRT receivables: received_srt_amount={}, new_one_srt_as_micro_vst={}, offsetted_srt_operation_receivable_amount={}, old_one_srt_as_micro_vst={}, deducted_extra_fee_amount_as_vst={}",
            srt_amount,
            new_one_srt_as_micro_vst,
            offsetted_srt_operation_receivable_amount,
            old_one_srt_as_micro_vst,
            solv_protocol_extra_fee_amount_as_vst,
        );

        Ok((
            self.srt_operation_reserved_amount,
            solv_protocol_extra_fee_amount_as_vst,
        ))
    }

    fn get_withdrawal_requests_iter_mut(&mut self) -> impl Iterator<Item = &mut WithdrawalRequest> {
        self.withdrawal_requests[..self.num_withdrawal_requests as usize].iter_mut()
    }

    /// returns (∆vrt_withdrawal_enqueued_amount, ∆vst_withdrawal_estimated_amount).
    /// vrt_withdrawal_enqueued_amount "might" be less than given vrt_amount,
    /// due to srt operation receivable amount.
    pub(crate) fn enqueue_withdrawal_request(&mut self, mut vrt_amount: u64) -> Result<(u64, u64)> {
        let srt_exchange_rate = self.get_srt_exchange_rate();
        let srt_operation_reserved_amount_as_vst = srt_exchange_rate
            .get_srt_amount_as_vst(self.srt_operation_reserved_amount, false)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        let srt_operation_receivable_amount_as_vst = srt_exchange_rate
            .get_srt_amount_as_vst(self.srt_operation_receivable_amount, false)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        let net_asset_value_as_vst = self.vst_operation_reserved_amount
            + self.vst_operation_receivable_amount
            + srt_operation_reserved_amount_as_vst
            + srt_operation_receivable_amount_as_vst;

        // First, adjust VRT withdrawal request amount if needed, due to srt_operation_receivable_amount
        let net_asset_value_without_srt_receivable_as_vst =
            net_asset_value_as_vst - srt_operation_receivable_amount_as_vst;
        let maximum_possible_vst_withdrawal_amount = if net_asset_value_as_vst == 0 {
            // When net asset value is 0, obviously srt_receivable = 0 so all vrt is possible to withdraw
            self.vrt_supply
        } else {
            div_util(
                self.vrt_supply as u128 * net_asset_value_without_srt_receivable_as_vst as u128,
                net_asset_value_as_vst,
                false,
            )
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?
        };
        vrt_amount = std::cmp::min(vrt_amount, maximum_possible_vst_withdrawal_amount);

        // Ignore empty request
        if vrt_amount == 0 {
            return Ok((0, 0));
        }

        // Withdrawal request is full
        if self.num_withdrawal_requests as usize >= MAX_WITHDRAWAL_REQUESTS {
            err!(VaultError::ExceededMaxWithdrawalRequestsError)?;
        }

        // Calculate target VST withdrawal amount,
        // then calculate how to prepare required VST for withdrawal.
        // First option is to pay with VST reserved.
        // Second option is to take as withdrawal fee, then offset VST receivable.
        // Last option is to withdraw SRT.
        let target_vst_withdrawal_amount = div_util(
            vrt_amount as u128 * net_asset_value_as_vst as u128,
            self.vrt_supply, // We know vrt_supply > 0 because vrt_amount > 0
            false,
        )
        .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;

        // First, calculate fair amount of target VST offsetting receivable (will be taken as vst withdrawl fee)
        let target_vst_offsetting_receivable_amount = if net_asset_value_as_vst == 0 {
            0
        } else {
            div_util(
                self.vst_operation_receivable_amount as u128 * target_vst_withdrawal_amount as u128,
                net_asset_value_as_vst,
                false,
            )
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?
        };

        // Then, pay with VST reserved as much as possible, in order to avoid solv protocol withdrawal fee
        let vst_withdrawal_locked_amount = std::cmp::min(
            self.vst_operation_reserved_amount,
            target_vst_withdrawal_amount - target_vst_offsetting_receivable_amount,
        );

        // Finally, insufficient amount will be filled by withdrawing SRT
        let insufficient_vst_amount = target_vst_withdrawal_amount
            - target_vst_offsetting_receivable_amount
            - vst_withdrawal_locked_amount;

        let srt_withdrawal_locked_amount;
        let srt_withdrawal_locked_amount_as_vst;
        let post_srt_operation_reserved_amount_as_vst;
        let vst_withdrawal_fee_amount;
        let vst_offsetting_receivable_amount;

        if insufficient_vst_amount == 0 {
            // No insufficient amount, so no need to withdraw SRT
            srt_withdrawal_locked_amount = 0;
            srt_withdrawal_locked_amount_as_vst = 0;
            post_srt_operation_reserved_amount_as_vst = srt_operation_reserved_amount_as_vst;
            vst_withdrawal_fee_amount = target_vst_offsetting_receivable_amount;
            vst_offsetting_receivable_amount = target_vst_offsetting_receivable_amount;
        } else if insufficient_vst_amount >= srt_operation_reserved_amount_as_vst {
            // Even full SRT withdrawal is not enough to fill insufficient amount,
            // so  additionally offset VST receivable, which will be taken as VST withdrawal fee.
            srt_withdrawal_locked_amount = self.srt_operation_reserved_amount;
            srt_withdrawal_locked_amount_as_vst = srt_operation_reserved_amount_as_vst;
            post_srt_operation_reserved_amount_as_vst = 0;

            let insufficient_vst_amount =
                insufficient_vst_amount - srt_withdrawal_locked_amount_as_vst;
            vst_withdrawal_fee_amount =
                target_vst_offsetting_receivable_amount + insufficient_vst_amount;
            vst_offsetting_receivable_amount =
                target_vst_offsetting_receivable_amount + insufficient_vst_amount;
        } else {
            // Calculate (almost) exact amount of SRT to withdraw, yet there exists small diff due to round down.
            srt_withdrawal_locked_amount = srt_exchange_rate
                .get_vst_amount_as_srt(insufficient_vst_amount, false)
                .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
            srt_withdrawal_locked_amount_as_vst = srt_exchange_rate
                .get_srt_amount_as_vst(srt_withdrawal_locked_amount, false)
                .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
            post_srt_operation_reserved_amount_as_vst = srt_exchange_rate
                .get_srt_amount_as_vst(
                    self.srt_operation_reserved_amount - srt_withdrawal_locked_amount,
                    false,
                )
                .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;

            // Due to round down, estimated VST withdrawal amount is little less than target VST withdrawal amount.
            // Treat that diff as extra withdrawal fee so the estimation equals to target withdrawal amount.
            let diff = insufficient_vst_amount - srt_withdrawal_locked_amount_as_vst;
            vst_withdrawal_fee_amount = target_vst_offsetting_receivable_amount + diff;

            // Due to round down, net asset value will decrease slightly less than target VST withdrawal amount.
            // Resolve that diff by additionally offsetting VST receivable, as much as possible.
            // Still, if there is insufficient VST receivable, net asset value will remain higher than expected.
            let net_asset_value_decreased_amount_by_srt_withdrawal =
                srt_operation_reserved_amount_as_vst - post_srt_operation_reserved_amount_as_vst;
            let diff = insufficient_vst_amount - net_asset_value_decreased_amount_by_srt_withdrawal;
            vst_offsetting_receivable_amount = std::cmp::min(
                self.vst_operation_receivable_amount,
                target_vst_offsetting_receivable_amount + diff,
            );
        }

        let net_asset_value_decreased_amount_by_srt_withdrawal =
            srt_operation_reserved_amount_as_vst - post_srt_operation_reserved_amount_as_vst;
        let net_asset_value_decreasing_amount = vst_withdrawal_locked_amount
            + vst_offsetting_receivable_amount
            + net_asset_value_decreased_amount_by_srt_withdrawal;
        let vst_withdrawal_total_estimated_amount = vst_withdrawal_locked_amount
            + vst_withdrawal_fee_amount
            + srt_withdrawal_locked_amount_as_vst;

        // Validation
        let tolerance = srt_exchange_rate
            .get_srt_amount_as_vst(1, true)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        require_gte!(
            net_asset_value_decreasing_amount,
            target_vst_withdrawal_amount.saturating_sub(tolerance),
        );
        require_gte!(
            target_vst_withdrawal_amount,
            net_asset_value_decreasing_amount,
        );
        require_eq!(
            target_vst_withdrawal_amount,
            vst_withdrawal_total_estimated_amount,
        );

        // Enqueue
        self.withdrawal_last_created_request_id += 1;
        self.withdrawal_requests[self.num_withdrawal_requests as usize].initialize(
            self.withdrawal_last_created_request_id,
            vrt_amount,
            srt_withdrawal_locked_amount,
            vst_withdrawal_locked_amount,
            vst_withdrawal_total_estimated_amount,
            self.one_srt_as_micro_vst,
        );
        self.num_withdrawal_requests += 1;

        // Burn VRT
        self.vrt_supply -= vrt_amount;

        // Update accountings
        self.vrt_withdrawal_enqueued_amount += vrt_amount;

        self.vst_operation_reserved_amount -= vst_withdrawal_locked_amount;
        self.vst_operation_receivable_amount -= vst_offsetting_receivable_amount;
        self.vst_withdrawal_locked_amount += vst_withdrawal_locked_amount;

        self.srt_operation_reserved_amount -= srt_withdrawal_locked_amount;
        self.srt_withdrawal_locked_amount += srt_withdrawal_locked_amount;

        self.update_vrt_exchange_rate()?;

        msg!(
            "Enqueue withdrawal request: burnt_vrt_amount={}, vrt_withdrawal_enqueued_amount={}, vst_withdrawal_locked_amount={}, vst_withdrawal_fee_amount={}, srt_withdrawal_locked_amount={}",
            vrt_amount,
            self.vrt_withdrawal_enqueued_amount,
            vst_withdrawal_locked_amount,
            vst_withdrawal_fee_amount,
            srt_withdrawal_locked_amount,
        );

        Ok((vrt_amount, vst_withdrawal_total_estimated_amount))
    }

    /// returns (srt_amount_to_withdraw, vst_estimated_amount_to_receive)
    pub(crate) fn confirm_withdrawal_requests(&mut self) -> Result<(u64, u64)> {
        let srt_amount_to_withdraw = self.srt_withdrawal_locked_amount;
        // NOTE: this estimation is quite inprecise, so only used for event emission & logging.
        let vst_estimated_amount_to_receive = self
            .get_srt_exchange_rate()
            .get_srt_amount_as_vst(srt_amount_to_withdraw, false)
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;

        // Start
        let mut vst_receivable_amount_to_claim = 0;
        for request in self
            .get_withdrawal_requests_iter_mut()
            .skip_while(|request| request.state != WITHDRAWAL_REQUEST_STATE_ENQUEUED)
        {
            request.state = WITHDRAWAL_REQUEST_STATE_PROCESSING;

            vst_receivable_amount_to_claim += request.vst_withdrawal_total_estimated_amount;
        }

        // Update accountings
        self.vrt_withdrawal_processing_amount += self.vrt_withdrawal_enqueued_amount;
        self.vrt_withdrawal_enqueued_amount = 0;

        self.vst_receivable_amount_to_claim += vst_receivable_amount_to_claim;

        self.srt_withdrawal_locked_amount = 0;

        msg!(
            "Confirm withdrawal requests: srt_amount_to_withdraw={}, vst_estimated_amount_to_receive={}, vst_receivable_amount_to_claim={}, total_vst_receivable_amount_to_claim={}, vrt_withdrawal_processing_amount={}",
            srt_amount_to_withdraw,
            vst_estimated_amount_to_receive,
            vst_receivable_amount_to_claim,
            self.vst_receivable_amount_to_claim,
            self.vrt_withdrawal_processing_amount,
        );

        Ok((srt_amount_to_withdraw, vst_estimated_amount_to_receive))
    }

    /// returns (∆vst_reserved_amount_to_claim, ∆vst_extra_amount_to_claim, ∆vst_deducted_fee_amount)
    pub(crate) fn complete_withdrawal_requests(
        &mut self,
        srt_amount: u64,
        vst_amount: u64,
        old_one_srt_as_micro_vst: u64, // SRT price which request processed at
        heuristic_validation: bool,
    ) -> Result<(u64, u64, u64)> {
        if self.is_deposit_in_progress() {
            err!(VaultError::DepositInProgressError)?;
        }

        // Validate vst_amount
        if srt_amount == 0 {
            // Since there might exist requests without SRT withdrawal required,
            // so completion with 0 SRT is allowed.
            // In this case, obviously VST amount should be 0.
            require_eq!(vst_amount, 0);
        } else {
            let srt_amount_as_vst =
                SRTExchangeRate::new(old_one_srt_as_micro_vst, self.solv_receipt_token_decimals)
                    .get_srt_amount_as_vst(srt_amount, false)
                    .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
            let solv_protocol_withdrawal_fee_amount_as_vst = div_util(
                srt_amount_as_vst as u128 * self.solv_protocol_withdrawal_fee_rate_bps as u128,
                BPS,
                true,
            )
            .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
            let expected_vst_amount =
                srt_amount_as_vst - solv_protocol_withdrawal_fee_amount_as_vst;

            let solv_protocol_extra_fee_amount_as_vst =
                expected_vst_amount.saturating_sub(vst_amount);

            if solv_protocol_extra_fee_amount_as_vst > SOLV_PROTOCOL_MAX_EXTRA_FEE_AMOUNT {
                err!(VaultError::InvalidSolvProtocolExtraFeeAmountError)?;
            }
        }

        // Complete
        let mut vrt_withdrawal_requested_amount = 0;
        let mut srt_withdrawal_locked_amount = 0;
        let mut vst_withdrawal_locked_amount = 0;
        let mut vst_withdrawal_total_estimated_amount = 0;
        for request in self
            .get_withdrawal_requests_iter_mut()
            .skip_while(|request| request.state == WITHDRAWAL_REQUEST_STATE_COMPLETED)
            .take_while(|request| request.state == WITHDRAWAL_REQUEST_STATE_PROCESSING)
        {
            if srt_withdrawal_locked_amount + request.srt_withdrawal_locked_amount > srt_amount {
                break;
            }

            if request.one_srt_as_micro_vst > old_one_srt_as_micro_vst {
                err!(VaultError::InvalidSRTPriceError)?;
            }

            request.state = WITHDRAWAL_REQUEST_STATE_COMPLETED;

            vrt_withdrawal_requested_amount += request.vrt_withdrawal_requested_amount;
            srt_withdrawal_locked_amount += request.srt_withdrawal_locked_amount;
            vst_withdrawal_locked_amount += request.vst_withdrawal_locked_amount;
            vst_withdrawal_total_estimated_amount += request.vst_withdrawal_total_estimated_amount;
        }

        require_gt!(vrt_withdrawal_requested_amount, 0);

        // Check exact amount
        require_eq!(srt_withdrawal_locked_amount, srt_amount);

        // Apply withdrawal fee
        let vst_withdrawal_fee_amount = div_util(
            vst_withdrawal_total_estimated_amount as u128
                * self.get_withdrawal_fee_rate_bps() as u128,
            BPS,
            true,
        )
        .ok_or_else(|| error!(VaultError::CalculationArithmeticException))?;
        let vst_withdrawal_estimated_amount_without_fee =
            vst_withdrawal_total_estimated_amount - vst_withdrawal_fee_amount;

        // Calculate surplus or shortage
        let vst_withdrawal_amount_without_fee = vst_withdrawal_locked_amount + vst_amount;

        let vst_reserved_amount_to_claim;
        let vst_extra_amount_to_claim;
        let vst_deducted_fee_amount;

        if vst_withdrawal_amount_without_fee >= vst_withdrawal_estimated_amount_without_fee {
            // Surplus as extra claimable amount
            let surplus =
                vst_withdrawal_amount_without_fee - vst_withdrawal_estimated_amount_without_fee;
            vst_reserved_amount_to_claim = vst_withdrawal_estimated_amount_without_fee;
            vst_extra_amount_to_claim = surplus;
            vst_deducted_fee_amount = vst_withdrawal_fee_amount;
        } else {
            // Shortage as extra fee
            let shortage =
                vst_withdrawal_estimated_amount_without_fee - vst_withdrawal_amount_without_fee;
            vst_reserved_amount_to_claim = vst_withdrawal_amount_without_fee;
            vst_extra_amount_to_claim = 0;
            vst_deducted_fee_amount = vst_withdrawal_fee_amount + shortage;
        }

        // Update accountings
        self.vrt_withdrawal_processing_amount -= vrt_withdrawal_requested_amount;
        self.vrt_withdrawal_completed_amount += vrt_withdrawal_requested_amount;

        self.vst_withdrawal_locked_amount -= vst_withdrawal_locked_amount;
        self.vst_reserved_amount_to_claim += vst_reserved_amount_to_claim;
        self.vst_extra_amount_to_claim += vst_extra_amount_to_claim;
        self.vst_deducted_fee_amount += vst_deducted_fee_amount;
        self.vst_receivable_amount_to_claim -= vst_withdrawal_total_estimated_amount;

        // TODO: deprecate this heuristic validation
        if old_one_srt_as_micro_vst < self.one_srt_as_micro_vst
            && heuristic_validation
            && self.one_srt_as_micro_vst - old_one_srt_as_micro_vst > self.one_srt_as_micro_vst / 10
        {
            err!(VaultError::InvalidSRTPriceError)?;
        }

        msg!(
            "Complete withdrawal requests: burnt_srt_amount={}, received_vst_amount={}, vst_withdrawal_unlocked_amount={}, vst_deducted_fee_amount={}, vst_extra_claimable_amount={}, total_vst_reserved_amount_to_claim={}, total_vst_extra_amount_to_claim={}, total_vst_deducted_fee_amount={}, vrt_withdrawal_completed_amount={}",
            srt_amount,
            vst_amount,
            vst_withdrawal_locked_amount,
            vst_deducted_fee_amount,
            vst_extra_amount_to_claim,
            self.vst_reserved_amount_to_claim,
            self.vst_extra_amount_to_claim,
            self.vst_deducted_fee_amount,
            self.vrt_withdrawal_completed_amount,
        );

        Ok((
            vst_reserved_amount_to_claim,
            vst_extra_amount_to_claim,
            vst_deducted_fee_amount,
        ))
    }

    /// returns (vrt_burnt_amount, vst_claimed_amount, vst_extra_amount, vst_deducted_fee_amount)
    pub(crate) fn claim_vst(&mut self) -> Result<(u64, u64, u64, u64)> {
        let vrt_burnt_amount = self.vrt_withdrawal_completed_amount;
        let vst_claimed_amount = self.vst_reserved_amount_to_claim;
        let vst_extra_amount = self.vst_extra_amount_to_claim;
        let vst_deducted_fee_amount = self.vst_deducted_fee_amount;

        // Clear completed withdrawal requests
        let num_completed_requests = self
            .get_withdrawal_requests_iter_mut()
            .take_while(|request| request.state == WITHDRAWAL_REQUEST_STATE_COMPLETED)
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

        msg!(
            "Claim VST: vrt_burnt_amount={}, vst_claimed_amount={}, vst_extra_amount={}, vst_deducted_fee_amount={}",
            vrt_burnt_amount,
            vst_claimed_amount,
            vst_extra_amount,
            vst_deducted_fee_amount,
        );

        Ok((
            vrt_burnt_amount,
            vst_claimed_amount,
            vst_extra_amount,
            vst_deducted_fee_amount,
        ))
    }

    pub fn get_vst_total_claimable_amount(&self) -> u64 {
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
        srt_withdrawal_locked_amount: u64,
        vst_withdrawal_locked_amount: u64,
        vst_withdrawal_total_estimated_amount: u64,
        one_srt_as_micro_vst: u64,
    ) {
        self.request_id = request_id;
        self.vrt_withdrawal_requested_amount = vrt_withdrawal_requested_amount;
        self.srt_withdrawal_locked_amount = srt_withdrawal_locked_amount;
        self.vst_withdrawal_locked_amount = vst_withdrawal_locked_amount;
        self.vst_withdrawal_total_estimated_amount = vst_withdrawal_total_estimated_amount;
        self.one_srt_as_micro_vst = one_srt_as_micro_vst;
        self.state = WITHDRAWAL_REQUEST_STATE_ENQUEUED;
    }
}

struct SRTExchangeRate {
    one_srt_as_micro_vst: u64,
    one_srt_as_micro: u64,
}

impl SRTExchangeRate {
    fn new(one_srt_as_micro_vst: u64, srt_decimals: u8) -> Self {
        Self {
            one_srt_as_micro_vst,
            one_srt_as_micro: 10u64.pow(srt_decimals as u32 + 6),
        }
    }

    fn get_srt_amount_as_vst(&self, srt_amount: u64, round_up: bool) -> Option<u64> {
        div_util(
            srt_amount as u128 * self.one_srt_as_micro_vst as u128,
            self.one_srt_as_micro,
            round_up,
        )
    }

    fn get_vst_amount_as_srt(&self, vst_amount: u64, round_up: bool) -> Option<u64> {
        div_util(
            vst_amount as u128 * self.one_srt_as_micro as u128,
            self.one_srt_as_micro_vst,
            round_up,
        )
    }
}

/// d > 0
fn div_util<T>(mut numerator: u128, denominator: T, round_up: bool) -> Option<u64>
where
    u128: From<T>,
{
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
    use rust_decimal::Decimal;

    use super::*;

    impl VaultAccount {
        fn dummy() -> Self {
            let mut vault = Self::zeroed();
            vault.solv_protocol_deposit_fee_rate_bps = 10;
            vault.solv_protocol_withdrawal_fee_rate_bps = 10;
            vault.vault_receipt_token_mint = Pubkey::new_unique();
            vault.vault_supported_token_mint = Pubkey::new_unique();
            vault.solv_receipt_token_mint = Pubkey::new_unique();
            vault.solv_receipt_token_decimals = 8;
            vault.one_srt_as_micro_vst = 10u64.pow(8 + 6);
            vault
        }

        fn srt_price_as_decimal(&self) -> Decimal {
            Decimal::new(
                self.one_srt_as_micro_vst as i64,
                self.solv_receipt_token_decimals as u32 + 6,
            )
        }

        fn vrt_price_as_decimal(&self) -> Decimal {
            let nav = self.get_net_asset_value_as_vst().unwrap();
            let supply = self.vrt_supply;
            Decimal::new(nav as i64, 0) / Decimal::new(supply as i64, 0)
        }

        fn get_withdrawal_requests_iter(&self) -> impl Iterator<Item = &WithdrawalRequest> {
            self.withdrawal_requests[..self.num_withdrawal_requests as usize].iter()
        }

        fn assert_invariants(&self) -> anyhow::Result<()> {
            // vrt_withdrawal_enqueued_amount = ∑request(state = ENQUEUED).vrt_withdrawal_requested_amount
            let vrt_withdrawal_enqueued_amount: u64 = self
                .get_withdrawal_requests_iter()
                .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_ENQUEUED)
                .map(|request| request.vrt_withdrawal_requested_amount)
                .sum();

            if self.vrt_withdrawal_enqueued_amount != vrt_withdrawal_enqueued_amount {
                return Err(anyhow!(
                    "VRT withdrawal enqueued amount({}) != ∑request(state == ENQUEUED).vrt_withdrawal_requested_amount({})",
                    self.vrt_withdrawal_enqueued_amount,
                    vrt_withdrawal_enqueued_amount,
                ));
            }

            // vrt_withdrawal_processing_amount = ∑request(state = PROCESSING).vrt_withdrawal_requested_amount
            let vrt_withdrawal_processing_amount: u64 = self
                .get_withdrawal_requests_iter()
                .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_PROCESSING)
                .map(|request| request.vrt_withdrawal_requested_amount)
                .sum();

            if self.vrt_withdrawal_processing_amount != vrt_withdrawal_processing_amount {
                return Err(anyhow!(
                    "VRT withdrawal processing amount({}) != ∑request(state == PROCESSING).vrt_withdrawal_requested_amount({})",
                    self.vrt_withdrawal_processing_amount,
                    vrt_withdrawal_processing_amount,
                ));
            }

            // vrt_withdrawal_completed_amount = ∑request(state = COMPLETED).vrt_withdrawal_requested_amount
            let vrt_withdrawal_completed_amount: u64 = self
                .get_withdrawal_requests_iter()
                .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_COMPLETED)
                .map(|request| request.vrt_withdrawal_requested_amount)
                .sum();

            if self.vrt_withdrawal_completed_amount != vrt_withdrawal_completed_amount {
                return Err(anyhow!(
                    "VRT withdrawal completed amount({}) != ∑request(state == COMPLETED).vrt_withdrawal_requested_amount({})",
                    self.vrt_withdrawal_completed_amount,
                    vrt_withdrawal_completed_amount,
                ));
            }

            // vst_withdrawal_locked_amount = ∑request(state != COMPLETED).vst_withdrawal_locked_amount
            let vst_withdrawal_locked_amount: u64 = self
                .get_withdrawal_requests_iter()
                .filter(|request| request.state != WITHDRAWAL_REQUEST_STATE_COMPLETED)
                .map(|request| request.vst_withdrawal_locked_amount)
                .sum();

            if self.vst_withdrawal_locked_amount != vst_withdrawal_locked_amount {
                return Err(anyhow!(
                    "VST withdrawal locked amount({}) != ∑request(state != COMPLETED).vst_withdrawal_locked_amount({})",
                    self.vst_withdrawal_locked_amount,
                    vst_withdrawal_locked_amount,
                ));
            }

            // srt_withdrawal_locked_amount = ∑request(state == ENQUEUED).srt_withdrawal_reserved_amount
            let srt_withdrawal_locked_amount: u64 = self
                .get_withdrawal_requests_iter()
                .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_ENQUEUED)
                .map(|request| request.srt_withdrawal_locked_amount)
                .sum();

            if self.srt_withdrawal_locked_amount != srt_withdrawal_locked_amount {
                return Err(anyhow!(
                    "SRT withdrawal locked amount({}) != ∑request(state == ENQUEUED).srt_withdrawal_reserved_amount({})",
                    self.srt_withdrawal_locked_amount,
                    srt_withdrawal_locked_amount,
                ));
            }

            // vst_receivable_amount_to_claim = ∑request(state == PROCESSING).vst_withdrawal_total_estimated_amount
            let vst_receivable_amount_to_claim: u64 = self
                .get_withdrawal_requests_iter()
                .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_PROCESSING)
                .map(|request| request.vst_withdrawal_total_estimated_amount)
                .sum();

            if self.vst_receivable_amount_to_claim != vst_receivable_amount_to_claim {
                return Err(anyhow!(
                    "VST receivable amount to claim({}) != ∑request(state == PROCESSING).vst_withdrawal_total_estimated_amount({})",
                    self.vst_receivable_amount_to_claim,
                    vst_receivable_amount_to_claim,
                ));
            }

            // vst_reserved_amount_to_claim + vst_deducted_fee_amount = ∑request(state == COMPLETED).vst_withdrawal_total_estimated_amount
            let vst_reserved_amount_to_claim_plus_deducted_fee: u64 = self
                .get_withdrawal_requests_iter()
                .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_COMPLETED)
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

            // VRT supply = 0 <=> Net Asset Value = 0
            let nav = self
                .get_net_asset_value_as_vst()
                .ok_or_else(|| anyhow!("Invalid NAV"))?;

            if self.vrt_supply == 0 && nav > 0 {
                return Err(anyhow!("VRT supply = 0 but NAV({}) > 0", nav));
            }

            if self.vrt_supply > 0 && nav == 0 {
                return Err(anyhow!("VRT supply({}) > 0 but NAV = 0", self.vrt_supply));
            }

            // VRT price ≥ 1 (might be different based on decimals)
            if nav < self.vrt_supply {
                return Err(anyhow!("VRT price({}) < 1", self.vrt_price_as_decimal()));
            }

            // SRT price ≥ 1 (might be different based on decimals)
            if self.one_srt_as_micro_vst < 10u64.pow(self.solv_receipt_token_decimals as u32 + 6) {
                return Err(anyhow!("SRT price({}) < 1", self.srt_price_as_decimal()));
            }

            Ok(())
        }

        fn assert_price_increased(&self, old: &Self) -> anyhow::Result<()> {
            // ∆SRT price ≥ 0
            if self.one_srt_as_micro_vst < old.one_srt_as_micro_vst {
                return Err(anyhow!(
                    "SRT price({}) decreased, previously {}",
                    self.srt_price_as_decimal(),
                    old.srt_price_as_decimal(),
                ));
            }

            // ∆VRT price ≥ 0
            let nav = self
                .get_net_asset_value_as_vst()
                .ok_or_else(|| anyhow!("Invalid NAV"))?;
            let old_nav = old
                .get_net_asset_value_as_vst()
                .ok_or_else(|| anyhow!("Invalid old NAV"))?;
            if nav as u128 * (old.vrt_supply as u128) < old_nav as u128 * self.vrt_supply as u128 {
                return Err(anyhow!(
                    "VRT price({}) decreased, previously {}",
                    self.vrt_price_as_decimal(),
                    old.vrt_price_as_decimal(),
                ));
            }

            Ok(())
        }
    }

    #[test]
    fn test_account_size() {
        assert_eq!(VaultAccount::get_size(), 1024 * 10);
    }

    prop_compose! {
        fn vault()
            (vrt_supply in 1..u64::MAX)
            (
                vrt_supply in Just(vrt_supply),
                extra_vst in 0..vrt_supply.min(u64::MAX - vrt_supply + 1),
                one_srt_as_micro_vst in 100_000_000_000_000u64..200_000_000_000_000,
            )
        -> VaultAccount {
            let mut vault = VaultAccount::dummy();
            vault.vrt_supply = vrt_supply;
            vault.vst_operation_reserved_amount = vrt_supply + extra_vst;
            vault.one_srt_as_micro_vst = one_srt_as_micro_vst;
            vault
        }
    }

    prop_compose! {
        fn vault_and_vrt_withdrawal_amount()
            (vault in vault())
            (
                mut vault in Just(vault),
                vrt_amount in 0..=vault.vrt_supply,
                vst_receivable in 0..=vault.vst_operation_reserved_amount,
            )
        -> (VaultAccount, u64) {
            vault.vst_operation_reserved_amount -= vst_receivable;
            vault.vst_operation_receivable_amount += vst_receivable;
            (vault, vrt_amount)
        }
    }

    proptest! {
        #[test]
        fn test_initial_mint_vrt(
            vst_amount in 0..u64::MAX
        ) {
            let mut vault = VaultAccount::dummy();
            let old_vault = vault.clone();

            // Price = 1
            let vrt_amount = vault.mint_vrt(vst_amount).unwrap();
            assert_eq!(vrt_amount, vst_amount);

            assert_eq!(
                vault.vrt_supply,
                old_vault.vrt_supply + vrt_amount,
            );
            assert_eq!(
                vault.vst_operation_reserved_amount,
                old_vault.vst_operation_reserved_amount + vst_amount,
            );

            vault.assert_invariants().unwrap();
            vault.assert_price_increased(&old_vault).unwrap();
        }

        #[test]
        fn test_mint_vrt(
            mut vault in vault(),
            mut vst_amount in 0..u64::MAX,
        ) {
            vst_amount = vst_amount.min(u64::MAX - vault.vst_operation_reserved_amount);
            let old_vault = vault.clone();

            // VRT Price ≥ 1
            let vrt_amount = vault.mint_vrt(vst_amount).unwrap();
            assert!(vrt_amount <= vst_amount);

            assert_eq!(
                vault.vrt_supply,
                old_vault.vrt_supply + vrt_amount,
            );
            assert_eq!(
                vault.vst_operation_reserved_amount,
                old_vault.vst_operation_reserved_amount + vst_amount,
            );

            vault.assert_invariants().unwrap();
            vault.assert_price_increased(&old_vault).unwrap();
        }

        #[test]
        fn test_deposit_vst(
            mut vault in vault(),
        ) {
            let old_vault = vault.clone();

            // SRT Price ≥ 1
            vault.deposit_vst(vault.vst_operation_reserved_amount).unwrap();
            assert!(vault.srt_operation_receivable_amount <= old_vault.vst_operation_reserved_amount - vault.vst_operation_receivable_amount);

            assert_eq!(vault.vst_operation_reserved_amount, 0);

            vault.assert_invariants().unwrap();
            vault.assert_price_increased(&old_vault).unwrap();
        }

        #[test]
        fn test_offset_srt_receivables_no_price_increase(
            mut vault in vault(),
            extra_fee_amount_as_srt in 0..SOLV_PROTOCOL_MAX_EXTRA_FEE_AMOUNT / 3,
        ) {
            vault
                .deposit_vst(vault.vst_operation_reserved_amount)
                .unwrap();
            let old_vault = vault.clone();

            let srt_amount = vault.srt_operation_receivable_amount - extra_fee_amount_as_srt;
            let new_one_srt_as_micro_vst = vault.one_srt_as_micro_vst;

            vault.offset_srt_receivables(
                srt_amount,
                new_one_srt_as_micro_vst,
                true,
            )
            .unwrap();

            let vst_operation_receivable_amount_delta = vault.vst_operation_receivable_amount
                - old_vault.vst_operation_receivable_amount;
            let extra_fee_amount_as_vst = old_vault
                .get_srt_exchange_rate()
                .get_srt_amount_as_vst(extra_fee_amount_as_srt, false)
                .unwrap();

            assert_eq!(
                vault.srt_operation_reserved_amount,
                old_vault.srt_operation_reserved_amount + srt_amount,
            );
            assert_eq!(
                vault.srt_operation_receivable_amount,
                0,
            );
            assert!(vst_operation_receivable_amount_delta >= extra_fee_amount_as_vst);
            assert!(vst_operation_receivable_amount_delta <= extra_fee_amount_as_vst + 1);

            vault.assert_invariants().unwrap();
            vault.assert_price_increased(&old_vault).unwrap();
        }

        #[test]
        fn test_offset_srt_receivables_with_price_increase(
            mut vault in vault(),
            extra_fee_amount_as_srt in 0..SOLV_PROTOCOL_MAX_EXTRA_FEE_AMOUNT / 3,
        ) {
            vault
                .deposit_vst(vault.vst_operation_reserved_amount)
                .unwrap();
            let old_vault = vault.clone();

            let srt_amount = ((vault.srt_operation_receivable_amount as u128 * 20 / 21) as u64)
                .saturating_sub(extra_fee_amount_as_srt);
            let new_one_srt_as_micro_vst =
                ((vault.one_srt_as_micro_vst as u128 + 19) * 21 / 20) as u64;

            vault
                .offset_srt_receivables(srt_amount, new_one_srt_as_micro_vst, true)
                .unwrap();

            let vst_operation_receivable_amount_delta =
                vault.vst_operation_receivable_amount - old_vault.vst_operation_receivable_amount;
            let extra_fee_amount_as_vst = vault
                .get_srt_exchange_rate()
                .get_srt_amount_as_vst(extra_fee_amount_as_srt + 1, false)
                .unwrap();

            assert_eq!(
                vault.srt_operation_reserved_amount,
                old_vault.srt_operation_reserved_amount + srt_amount,
            );
            assert_eq!(vault.srt_operation_receivable_amount, 0);
            assert!(vst_operation_receivable_amount_delta <= extra_fee_amount_as_vst + 2);

            vault.assert_invariants().unwrap();
            vault.assert_price_increased(&old_vault).unwrap();
        }

        #[test]
        fn test_enqueue_withdrawal_request(
            mut vault in vault(),
        ) {
            for (i, vrt_amount) in [vault.vrt_supply / 2, vault.vrt_supply - vault.vrt_supply / 2].into_iter().enumerate() {
                let old_vault = vault.clone();

                vault.enqueue_withdrawal_request(vrt_amount).unwrap();

                assert_eq!(vault.num_withdrawal_requests, i as u8 + 1);
                assert_eq!(
                    vault
                        .get_withdrawal_requests_iter()
                        .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_ENQUEUED)
                        .count(),
                    i + 1,
                );

                vault.assert_invariants().unwrap();
                vault.assert_price_increased(&old_vault).unwrap();
            }
        }

        #[test]
        fn test_enqueue_withdrawal_request_adjusts_vrt_amount_due_to_srt_receivable(
            mut vault in vault(),
        ) {
            vault
                .deposit_vst(vault.vst_operation_reserved_amount / 2)
                .unwrap();
            let old_vault = vault.clone();

            let expected_vrt_amount = div_util(
                vault.vrt_supply as u128
                    * (vault.vst_operation_reserved_amount + vault.vst_operation_receivable_amount)
                        as u128,
                vault.get_net_asset_value_as_vst().unwrap(),
                false,
            )
            .unwrap();
            let expected_vst_amount = div_util(
                expected_vrt_amount as u128 * vault.get_net_asset_value_as_vst().unwrap() as u128,
                vault.vrt_supply,
                false,
            )
            .unwrap();
            let (vrt_amount, _) = vault.enqueue_withdrawal_request(vault.vrt_supply).unwrap();

            assert_eq!(vrt_amount, expected_vrt_amount);
            assert_eq!(vault.vrt_supply + expected_vrt_amount, old_vault.vrt_supply);
            assert_eq!(vault.vrt_withdrawal_enqueued_amount, vrt_amount);
            assert_eq!(
                vault
                    .get_vst_estimated_amount_from_last_withdrawal_request()
                    .unwrap(),
                expected_vst_amount,
            );

            vault.assert_invariants().unwrap();
            vault.assert_price_increased(&old_vault).unwrap();
        }

        #[test]
        fn test_enqueue_withdrawal_request_adjusts_offsetting_vrt_receivable_due_to_srt_receivable(
            mut vault in vault(),
        ) {
            vault.solv_protocol_deposit_fee_rate_bps = 5000; // 50%
            vault
                .deposit_vst(vault.vst_operation_reserved_amount / 2)
                .unwrap();
            let old_vault = vault.clone();

            let expected_vrt_amount = div_util(
                vault.vrt_supply as u128
                    * (vault.vst_operation_reserved_amount + vault.vst_operation_receivable_amount)
                        as u128,
                vault.get_net_asset_value_as_vst().unwrap(),
                false,
            )
            .unwrap();
            let expected_vst_amount = div_util(
                expected_vrt_amount as u128 * vault.get_net_asset_value_as_vst().unwrap() as u128,
                vault.vrt_supply,
                false,
            )
            .unwrap();
            let (vrt_amount, _) = vault.enqueue_withdrawal_request(vault.vrt_supply).unwrap();

            assert_eq!(vrt_amount, expected_vrt_amount);
            assert_eq!(vault.vrt_supply + expected_vrt_amount, old_vault.vrt_supply);
            assert_eq!(vault.vrt_withdrawal_enqueued_amount, vrt_amount);
            assert_eq!(
                vault
                    .get_vst_estimated_amount_from_last_withdrawal_request()
                    .unwrap(),
                expected_vst_amount,
            );

            let tolerance = old_vault.vst_operation_reserved_amount
                + old_vault.vst_operation_receivable_amount
                - div_util(
                    vrt_amount as u128 * old_vault.get_net_asset_value_as_vst().unwrap() as u128,
                    old_vault.vrt_supply,
                    false,
                )
                .unwrap();
            assert!(vault.vst_operation_receivable_amount <= tolerance);

            vault.assert_invariants().unwrap();
            vault.assert_price_increased(&old_vault).unwrap();
        }

        #[test]
        fn test_enqueue_withdrawal_request_without_srt_operation_reserved(
            (mut vault, vrt_amount) in vault_and_vrt_withdrawal_amount(),
        ) {
            let old_vault = vault.clone();

            let expected_vst_amount = div_util(
                vrt_amount as u128 * vault.get_net_asset_value_as_vst().unwrap() as u128,
                vault.vrt_supply,
                false,
            )
            .unwrap();
            let (post_vrt_amount, _) = vault.enqueue_withdrawal_request(vrt_amount).unwrap();

            assert_eq!(post_vrt_amount, vrt_amount);
            assert_eq!(vault.vrt_supply + vrt_amount, old_vault.vrt_supply);
            assert_eq!(vault.vrt_withdrawal_enqueued_amount, post_vrt_amount);
            assert_eq!(
                vault
                    .get_vst_estimated_amount_from_last_withdrawal_request()
                    .unwrap(),
                expected_vst_amount,
            );

            vault.assert_invariants().unwrap();
            vault.assert_price_increased(&old_vault).unwrap();
        }

        #[test]
        fn test_enqueue_withdrawal_request_with_srt_operation_reserved(
            (mut vault, vrt_amount) in vault_and_vrt_withdrawal_amount(),
        ) {
            vault.solv_protocol_deposit_fee_rate_bps = 0;
            vault
                .deposit_vst(vault.vst_operation_reserved_amount / 2)
                .unwrap();
            vault
                .offset_srt_receivables(
                    vault.srt_operation_receivable_amount,
                    vault.one_srt_as_micro_vst,
                    true,
                )
                .unwrap();
            let old_vault = vault.clone();

            let expected_vst_amount = div_util(
                vrt_amount as u128 * vault.get_net_asset_value_as_vst().unwrap() as u128,
                vault.vrt_supply,
                false,
            )
            .unwrap();
            let (post_vrt_amount, _) = vault.enqueue_withdrawal_request(vrt_amount).unwrap();

            assert_eq!(post_vrt_amount, vrt_amount);
            assert_eq!(vault.vrt_supply + vrt_amount, old_vault.vrt_supply);
            assert_eq!(vault.vrt_withdrawal_enqueued_amount, post_vrt_amount);
            assert_eq!(
                vault
                    .get_vst_estimated_amount_from_last_withdrawal_request()
                    .unwrap(),
                expected_vst_amount,
            );

            vault.assert_invariants().unwrap();
            vault.assert_price_increased(&old_vault).unwrap();
        }

        #[test]
        fn test_confirm_withdrawal_request(
            mut vault in vault(),
        ) {
            vault.enqueue_withdrawal_request(vault.vrt_supply / 2).unwrap();
            vault.enqueue_withdrawal_request(vault.vrt_supply).unwrap();
            let old_vault = vault.clone();

            vault.confirm_withdrawal_requests().unwrap();

            assert_eq!(vault.srt_withdrawal_locked_amount, 0);
            assert_eq!(vault.num_withdrawal_requests, 2);
            assert_eq!(
                vault
                    .get_withdrawal_requests_iter()
                    .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_ENQUEUED)
                    .count(),
                0,
            );
            assert_eq!(
                vault
                    .get_withdrawal_requests_iter()
                    .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_PROCESSING)
                    .count(),
                2,
            );

            vault.assert_invariants().unwrap();
            vault.assert_price_increased(&old_vault).unwrap();
        }

        #[test]
        fn test_complete_withdrawal_request_one_by_one(
            mut vault in vault(),
            extra_fee_amount_as_vst in 0..=SOLV_PROTOCOL_MAX_EXTRA_FEE_AMOUNT,
        ) {
            vault.solv_protocol_deposit_fee_rate_bps = 0;
            vault.solv_protocol_withdrawal_fee_rate_bps = 0;
            vault
                .deposit_vst(vault.vst_operation_reserved_amount)
                .unwrap();
            vault
                .offset_srt_receivables(
                    vault.srt_operation_receivable_amount,
                    vault.one_srt_as_micro_vst,
                    true,
                )
                .unwrap();
            vault
                .enqueue_withdrawal_request(vault.vrt_supply / 2)
                .unwrap();
            vault.enqueue_withdrawal_request(vault.vrt_supply).unwrap();
            vault.confirm_withdrawal_requests().unwrap();

            let mut shortage = 0;
            for i in 0..2 {
                let old_vault = vault.clone();

                let srt_amount = vault.withdrawal_requests[i].srt_withdrawal_locked_amount;
                let vst_amount = vault
                    .get_srt_exchange_rate()
                    .get_srt_amount_as_vst(srt_amount, false)
                    .unwrap()
                    .saturating_sub(extra_fee_amount_as_vst);
                shortage += vault.withdrawal_requests[i].vst_withdrawal_total_estimated_amount
                    - vst_amount;

                vault
                    .complete_withdrawal_requests(
                        srt_amount,
                        vst_amount,
                        vault.one_srt_as_micro_vst,
                        true,
                    )
                    .unwrap();

                assert_eq!(vault.num_withdrawal_requests, 2);
                assert_eq!(
                    vault
                        .get_withdrawal_requests_iter()
                        .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_ENQUEUED)
                        .count(),
                    0,
                );
                assert_eq!(
                    vault
                        .get_withdrawal_requests_iter()
                        .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_PROCESSING)
                        .count(),
                    1 - i,
                );
                assert_eq!(
                    vault
                        .get_withdrawal_requests_iter()
                        .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_COMPLETED)
                        .count(),
                    1 + i,
                );

                assert_eq!(
                    vault.vst_deducted_fee_amount,
                    shortage,
                );

                vault.assert_invariants().unwrap();
                vault.assert_price_increased(&old_vault).unwrap();
            }
        }

        #[test]
        fn test_complete_withdrawal_request_bulk(
            mut vault in vault(),
            extra_fee_amount_as_vst in 0..=SOLV_PROTOCOL_MAX_EXTRA_FEE_AMOUNT,
        ) {
            vault.solv_protocol_deposit_fee_rate_bps = 0;
            vault.solv_protocol_withdrawal_fee_rate_bps = 0;
            vault
                .deposit_vst(vault.vst_operation_reserved_amount)
                .unwrap();
            vault
                .offset_srt_receivables(
                    vault.srt_operation_receivable_amount,
                    vault.one_srt_as_micro_vst,
                    true,
                )
                .unwrap();
            vault
                .enqueue_withdrawal_request(vault.vrt_supply / 2)
                .unwrap();
            vault.enqueue_withdrawal_request(vault.vrt_supply).unwrap();
            vault.confirm_withdrawal_requests().unwrap();
            let old_vault = vault.clone();

            let srt_amount = vault
                .get_withdrawal_requests_iter()
                .map(|request| request.srt_withdrawal_locked_amount)
                .sum();
            let vst_amount = vault
                .get_srt_exchange_rate()
                .get_srt_amount_as_vst(srt_amount, false)
                .unwrap()
                .saturating_sub(extra_fee_amount_as_vst);
            let shortage = vault.vst_receivable_amount_to_claim - vst_amount;

            vault
                .complete_withdrawal_requests(
                    srt_amount,
                    vst_amount,
                    vault.one_srt_as_micro_vst,
                    true,
                )
                .unwrap();

            assert_eq!(vault.num_withdrawal_requests, 2);
            assert_eq!(
                vault
                    .get_withdrawal_requests_iter()
                    .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_ENQUEUED)
                    .count(),
                0,
            );
            assert_eq!(
                vault
                    .get_withdrawal_requests_iter()
                    .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_PROCESSING)
                    .count(),
                0,
            );
            assert_eq!(
                vault
                    .get_withdrawal_requests_iter()
                    .filter(|request| request.state == WITHDRAWAL_REQUEST_STATE_COMPLETED)
                    .count(),
                2,
            );

            assert_eq!(
                vault.vst_deducted_fee_amount,
                shortage,
            );

            vault.assert_invariants().unwrap();
            vault.assert_price_increased(&old_vault).unwrap();
        }

        #[test]
        fn test_complete_withdrawal_request_with_higher_price(
            mut vault in vault(),
            extra_fee_amount_as_vst in 0..=SOLV_PROTOCOL_MAX_EXTRA_FEE_AMOUNT,
        ) {
            if vault.vst_operation_reserved_amount > (1 << 62) {
                vault.vst_operation_reserved_amount = 1 << 62;
            }
            vault.solv_protocol_deposit_fee_rate_bps = 0;
            vault.solv_protocol_withdrawal_fee_rate_bps = 0;
            vault
                .deposit_vst(vault.vst_operation_reserved_amount)
                .unwrap();
            vault
                .offset_srt_receivables(
                    vault.srt_operation_receivable_amount,
                    vault.one_srt_as_micro_vst,
                    true,
                )
                .unwrap();
            vault.enqueue_withdrawal_request(vault.vrt_supply).unwrap();
            vault.confirm_withdrawal_requests().unwrap();
            let old_vault = vault.clone();

            let new_one_srt_as_micro_vst = (vault.one_srt_as_micro_vst as u128 * 21 / 20) as u64;
            let new_srt_exchange_rate = SRTExchangeRate::new(new_one_srt_as_micro_vst, 8);

            let srt_amount = vault.withdrawal_requests[0].srt_withdrawal_locked_amount;
            let vst_amount = new_srt_exchange_rate
                .get_srt_amount_as_vst(srt_amount, false)
                .unwrap()
                .saturating_sub(extra_fee_amount_as_vst);
            let surplus_or_shortage =
                vst_amount as i128 - vault.vst_receivable_amount_to_claim as i128;

            vault
                .complete_withdrawal_requests(
                    srt_amount,
                    vst_amount,
                    new_one_srt_as_micro_vst,
                    true,
                )
                .unwrap();

            if surplus_or_shortage >= 0 {
                let surplus = surplus_or_shortage as u64;
                assert_eq!(vault.vst_extra_amount_to_claim, surplus);
            } else {
                let shortage = -surplus_or_shortage as u64;
                assert_eq!(vault.vst_deducted_fee_amount, shortage);
            }

            vault.assert_invariants().unwrap();
            vault.assert_price_increased(&old_vault).unwrap();
        }

        // If there is no extra fee then total withdrawal fee ≤ solv deposit + withdrawal fee
        #[test]
        fn test_complete_withdrawal_request_with_fee(
            mut vault in vault(),
        ) {
            vault
                .deposit_vst(vault.vst_operation_reserved_amount)
                .unwrap();
            vault
                .offset_srt_receivables(
                    vault.srt_operation_receivable_amount,
                    vault.one_srt_as_micro_vst,
                    true,
                )
                .unwrap();
            vault.enqueue_withdrawal_request(vault.vrt_supply).unwrap();
            vault.confirm_withdrawal_requests().unwrap();
            let old_vault = vault.clone();

            let srt_amount = vault.withdrawal_requests[0].srt_withdrawal_locked_amount;
            let vst_amount_with_fee = vault
                .get_srt_exchange_rate()
                .get_srt_amount_as_vst(srt_amount, false)
                .unwrap();
            let solv_protocol_withdrawal_fee_amount_as_vst = div_util(
                vst_amount_with_fee as u128 * vault.solv_protocol_withdrawal_fee_rate_bps as u128,
                BPS,
                true,
            )
            .unwrap();
            let vst_amount = vst_amount_with_fee - solv_protocol_withdrawal_fee_amount_as_vst;

            vault
                .complete_withdrawal_requests(
                    srt_amount,
                    vst_amount,
                    vault.one_srt_as_micro_vst,
                    true,
                )
                .unwrap();

            let vst_withdrawal_fee = div_util(
                vault.withdrawal_requests[0].vst_withdrawal_total_estimated_amount as u128
                    * vault.get_withdrawal_fee_rate_bps() as u128,
                BPS,
                true,
            )
            .unwrap();

            assert_eq!(vault.vst_deducted_fee_amount, vst_withdrawal_fee);

            vault.assert_invariants().unwrap();
            vault.assert_price_increased(&old_vault).unwrap();
        }
    }

    #[test]
    fn test_deposit_vst_fails_while_deposit_in_progress() {
        let mut vault = VaultAccount::dummy();
        vault.mint_vrt(100).unwrap();
        vault.deposit_vst(50).unwrap();
        vault.deposit_vst(50).unwrap_err();
    }

    #[test]
    fn test_offset_srt_receivables_invalid_extra_fee_amount() {
        let mut vault = VaultAccount::dummy();
        vault.solv_protocol_deposit_fee_rate_bps = 0;

        vault
            .mint_vrt(SOLV_PROTOCOL_MAX_EXTRA_FEE_AMOUNT + 1)
            .unwrap();
        vault
            .deposit_vst(SOLV_PROTOCOL_MAX_EXTRA_FEE_AMOUNT + 1)
            .unwrap();
        vault
            .offset_srt_receivables(1, vault.one_srt_as_micro_vst, true)
            .unwrap();

        vault.mint_vrt(SOLV_PROTOCOL_MAX_EXTRA_FEE_AMOUNT).unwrap();
        vault
            .deposit_vst(SOLV_PROTOCOL_MAX_EXTRA_FEE_AMOUNT)
            .unwrap();
        vault
            .offset_srt_receivables(0, vault.one_srt_as_micro_vst, true)
            .unwrap();

        vault
            .mint_vrt(SOLV_PROTOCOL_MAX_EXTRA_FEE_AMOUNT + 1)
            .unwrap();
        vault
            .deposit_vst(SOLV_PROTOCOL_MAX_EXTRA_FEE_AMOUNT + 1)
            .unwrap();
        vault
            .offset_srt_receivables(0, vault.one_srt_as_micro_vst, true)
            .unwrap_err();
    }

    #[test]
    fn test_offset_srt_receivables_invalid_srt_price() {
        let mut vault = VaultAccount::dummy();

        vault.mint_vrt(100).unwrap();
        vault.deposit_vst(100).unwrap();
        vault
            .offset_srt_receivables(200, 50_000_000_000_000, true)
            .unwrap_err();
    }

    #[test]
    fn test_complete_withdrawal_fails_while_deposit_in_progress() {
        let mut vault = VaultAccount::dummy();
        vault.solv_protocol_deposit_fee_rate_bps = 0;
        vault.solv_protocol_withdrawal_fee_rate_bps = 0;
        let vrt_amount = vault.mint_vrt(1_000_000).unwrap();
        vault
            .deposit_vst(vault.vst_operation_reserved_amount / 2)
            .unwrap();
        vault
            .offset_srt_receivables(
                vault.srt_operation_receivable_amount,
                vault.one_srt_as_micro_vst,
                true,
            )
            .unwrap();
        vault
            .deposit_vst(vault.vst_operation_reserved_amount)
            .unwrap();
        vault.enqueue_withdrawal_request(vrt_amount).unwrap();
        vault.confirm_withdrawal_requests().unwrap();

        vault
            .complete_withdrawal_requests(
                vault.srt_withdrawal_locked_amount,
                vault.vst_receivable_amount_to_claim,
                vault.one_srt_as_micro_vst,
                true,
            )
            .unwrap_err();
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
