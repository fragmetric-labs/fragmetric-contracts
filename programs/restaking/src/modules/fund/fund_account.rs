use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token_2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use bytemuck::Zeroable;
use std::ops::Neg;

use crate::errors::ErrorCode;
use crate::modules::pricing::{
    Asset, PricingService, TokenPricingSource, TokenValue, TokenValuePod,
};
use crate::utils::{get_proportional_amount, PDASeeds, ZeroCopyHeader};

use super::*;

#[constant]
/// ## Version History
/// * v15: migrate to new layout including new fields using bytemuck. (150584 ~= 148KB)
pub const FUND_ACCOUNT_CURRENT_VERSION: u16 = 15;

pub const FUND_WITHDRAWAL_FEE_RATE_BPS_LIMIT: u16 = 500;
pub const FUND_ACCOUNT_MAX_SUPPORTED_TOKENS: usize = 30;
pub const FUND_ACCOUNT_MAX_RESTAKING_VAULTS: usize = 30;

#[account(zero_copy)]
#[repr(C)]
pub struct FundAccount {
    data_version: u16,
    bump: u8,
    reserve_account_bump: u8,
    treasury_account_bump: u8,
    _padding: [u8; 9],
    pub(super) transfer_enabled: u8,

    address_lookup_table_enabled: u8,
    address_lookup_table_account: Pubkey,

    // informative
    reserve_account: Pubkey,
    treasury_account: Pubkey,

    /// receipt token information
    pub receipt_token_mint: Pubkey,
    pub(super) receipt_token_program: Pubkey,
    pub(super) receipt_token_decimals: u8,
    _padding2: [u8; 7],
    pub(super) receipt_token_supply_amount: u64,
    pub(super) one_receipt_token_as_sol: u64,
    pub(super) receipt_token_value_updated_slot: u64,
    pub(super) receipt_token_value: TokenValuePod,

    /// global withdrawal configurations
    pub(super) withdrawal_batch_threshold_interval_seconds: i64,
    pub(super) withdrawal_fee_rate_bps: u16,
    pub(super) withdrawal_enabled: u8,
    pub(super) deposit_enabled: u8,
    pub(super) donation_enabled: u8,
    _padding4: [u8; 3],

    /// SOL deposit & withdrawal
    pub(super) sol: AssetState,

    /// underlying assets
    _padding6: [u8; 15],
    num_supported_tokens: u8,
    supported_tokens: [SupportedToken; FUND_ACCOUNT_MAX_SUPPORTED_TOKENS],

    /// optional basket of underlying assets
    normalized_token: NormalizedToken,

    /// investments
    _padding7: [u8; 15],
    num_restaking_vaults: u8,
    restaking_vaults: [RestakingVault; FUND_ACCOUNT_MAX_RESTAKING_VAULTS],

    /// fund operation state
    pub(super) operation: OperationState,
}

impl PDASeeds<3> for FundAccount {
    const SEED: &'static [u8] = b"fund";

    fn get_bump(&self) -> u8 {
        self.bump
    }

    fn get_seeds(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.receipt_token_mint.as_ref(),
            std::slice::from_ref(&self.bump),
        ]
    }
}

impl ZeroCopyHeader for FundAccount {
    fn get_bump_offset() -> usize {
        2
    }
}

impl FundAccount {
    fn migrate(
        &mut self,
        bump: u8,
        receipt_token_mint: Pubkey,
        receipt_token_decimals: u8,
        receipt_token_supply: u64,
        sol_operation_reserved_amount: u64,
    ) {
        if self.data_version == 0 {
            self.receipt_token_mint = receipt_token_mint;
            self.receipt_token_program = token_2022::ID;
            self.receipt_token_decimals = receipt_token_decimals;
            self.receipt_token_supply_amount = receipt_token_supply;
            self.sol.initialize(None, sol_operation_reserved_amount);

            // Must calculate addresses at last
            self.bump = bump;
            (self.reserve_account, self.reserve_account_bump) =
                Pubkey::find_program_address(&self.get_reserve_account_seed_phrase(), &crate::ID);
            (self.treasury_account, self.treasury_account_bump) =
                Pubkey::find_program_address(&self.get_treasury_account_seed_phrase(), &crate::ID);

            self.data_version = 15;
        }
    }

    #[inline(always)]
    pub(super) fn initialize(
        &mut self,
        bump: u8,
        receipt_token_mint: &InterfaceAccount<Mint>,
        sol_operation_reserved_amount: u64,
    ) {
        self.migrate(
            bump,
            receipt_token_mint.key(),
            receipt_token_mint.decimals,
            receipt_token_mint.supply,
            sol_operation_reserved_amount,
        );
    }

    #[inline(always)]
    pub(super) fn update_if_needed(
        &mut self,
        receipt_token_mint: &InterfaceAccount<Mint>,
        sol_operation_reserved_amount: u64,
    ) {
        self.initialize(self.bump, receipt_token_mint, sol_operation_reserved_amount);
    }

    #[inline(always)]
    pub fn is_latest_version(&self) -> bool {
        self.data_version == FUND_ACCOUNT_CURRENT_VERSION
    }

    pub(super) fn find_account_address(&self) -> Result<Pubkey> {
        Ok(
            Pubkey::create_program_address(&self.get_seeds(), &crate::ID)
                .map_err(|_| ProgramError::InvalidSeeds)?,
        )
    }

    pub const RESERVE_SEED: &'static [u8] = b"fund_reserve";

    #[inline(always)]
    fn get_reserve_account_seed_phrase(&self) -> [&[u8]; 2] {
        [Self::RESERVE_SEED, self.receipt_token_mint.as_ref()]
    }

    pub(super) fn get_reserve_account_seeds(&self) -> [&[u8]; 3] {
        let mut seeds = <[_; 3]>::default();
        seeds[..2].copy_from_slice(&self.get_reserve_account_seed_phrase());
        seeds[2] = std::slice::from_ref(&self.reserve_account_bump);
        seeds
    }

    pub(super) fn get_reserve_account_address(&self) -> Result<Pubkey> {
        Ok(
            Pubkey::create_program_address(&self.get_reserve_account_seeds(), &crate::ID)
                .map_err(|_| ProgramError::InvalidSeeds)?,
        )
    }

    pub const TREASURY_SEED: &'static [u8] = b"fund_treasury";

    #[inline(always)]
    fn get_treasury_account_seed_phrase(&self) -> [&[u8]; 2] {
        [Self::TREASURY_SEED, self.receipt_token_mint.as_ref()]
    }

    pub(super) fn get_treasury_account_seeds(&self) -> [&[u8]; 3] {
        let mut seeds = <[_; 3]>::default();
        seeds[..2].copy_from_slice(&self.get_treasury_account_seed_phrase());
        seeds[2] = std::slice::from_ref(&self.treasury_account_bump);
        seeds
    }

    pub(super) fn get_treasury_account_address(&self) -> Result<Pubkey> {
        Ok(
            Pubkey::create_program_address(&self.get_treasury_account_seeds(), &crate::ID)
                .map_err(|_| ProgramError::InvalidSeeds)?,
        )
    }

    pub(super) fn find_supported_token_reserve_account_address(
        &self,
        token: &Pubkey,
    ) -> Result<Pubkey> {
        let supported_token = self.get_supported_token(token)?;
        Ok(
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &self.get_reserve_account_address()?,
                &supported_token.mint,
                &supported_token.program,
            ),
        )
    }

    pub(super) fn find_supported_token_treasury_account_address(
        &self,
        token: &Pubkey,
    ) -> Result<Pubkey> {
        let supported_token = self.get_supported_token(token)?;
        Ok(
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &self.get_treasury_account_address()?,
                &supported_token.mint,
                &supported_token.program,
            ),
        )
    }

    pub(super) fn find_normalized_token_reserve_account_address(&self) -> Result<Pubkey> {
        let normalized_token = self
            .get_normalized_token()
            .ok_or_else(|| error!(ErrorCode::FundNormalizedTokenNotSetError))?;
        Ok(
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &self.get_reserve_account_address()?,
                &normalized_token.mint,
                &normalized_token.program,
            ),
        )
    }

    pub(super) fn find_vault_receipt_token_reserve_account_address(
        &self,
        vault: &Pubkey,
    ) -> Result<Pubkey> {
        let restaking_vault = self.get_restaking_vault(vault)?;
        Ok(
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &self.get_reserve_account_address()?,
                &restaking_vault.receipt_token_mint,
                &restaking_vault.receipt_token_program,
            ),
        )
    }

    pub(super) fn find_stake_account_address<'a>(
        fund_account: &'a Pubkey,
        pool_account: &'a Pubkey,
        index: u8,
    ) -> FundStakeAccountAddress<'a> {
        FundStakeAccountAddress::new(fund_account, pool_account, index)
    }

    pub(super) fn find_unstaking_ticket_account_address<'a>(
        fund_account: &'a Pubkey,
        pool_account: &'a Pubkey,
        index: u8,
    ) -> FundUnstakingTicketAddress<'a> {
        FundUnstakingTicketAddress::new(fund_account, pool_account, index)
    }

    pub(super) fn find_receipt_token_lock_account_address(&self) -> Result<Pubkey> {
        Ok(
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &self.find_account_address()?,
                &self.receipt_token_mint,
                &self.receipt_token_program,
            ),
        )
    }

    #[inline]
    pub fn get_supported_tokens_iter(&self) -> impl Iterator<Item = &SupportedToken> {
        self.supported_tokens
            .iter()
            .take(self.num_supported_tokens as usize)
    }

    #[inline]
    pub fn get_supported_tokens_iter_mut(&mut self) -> impl Iterator<Item = &mut SupportedToken> {
        self.supported_tokens
            .iter_mut()
            .take(self.num_supported_tokens as usize)
    }

    pub(super) fn get_supported_token(&self, token_mint: &Pubkey) -> Result<&SupportedToken> {
        self.get_supported_tokens_iter()
            .find(|supported_token| supported_token.mint == *token_mint)
            .ok_or_else(|| error!(ErrorCode::FundNotSupportedTokenError))
    }

    pub(super) fn get_supported_token_mut(
        &mut self,
        token_mint: &Pubkey,
    ) -> Result<&mut SupportedToken> {
        self.get_supported_tokens_iter_mut()
            .find(|supported_token| supported_token.mint == *token_mint)
            .ok_or_else(|| error!(ErrorCode::FundNotSupportedTokenError))
    }

    pub(super) fn set_address_lookup_table_account(
        &mut self,
        address_lookup_table_account: &Option<Pubkey>,
    ) {
        if let Some(address) = address_lookup_table_account {
            self.address_lookup_table_enabled = 1;
            self.address_lookup_table_account = *address;
        } else {
            self.address_lookup_table_enabled = 0;
            self.address_lookup_table_account = Pubkey::default();
        }
    }

    #[inline(always)]
    pub(super) fn get_withdrawal_fee_amount(&self, amount: u64) -> Result<u64> {
        get_proportional_amount(amount, self.withdrawal_fee_rate_bps as u64, 10_000)
    }

    pub(super) fn set_withdrawal_fee_rate_bps(&mut self, fee_rate_bps: u16) -> Result<()> {
        require_gte!(
            FUND_WITHDRAWAL_FEE_RATE_BPS_LIMIT,
            fee_rate_bps,
            ErrorCode::FundInvalidWithdrawalFeeRateError
        );

        self.withdrawal_fee_rate_bps = fee_rate_bps;

        Ok(())
    }

    #[inline(always)]
    pub(super) fn set_deposit_enabled(&mut self, enabled: bool) {
        self.deposit_enabled = if enabled { 1 } else { 0 };
    }

    #[inline(always)]
    pub(super) fn set_donation_enabled(&mut self, enabled: bool) {
        self.donation_enabled = if enabled { 1 } else { 0 };
    }

    #[inline(always)]
    pub(super) fn set_withdrawal_enabled(&mut self, enabled: bool) {
        self.withdrawal_enabled = if enabled { 1 } else { 0 };
    }

    pub(super) fn set_withdrawal_batch_threshold(&mut self, interval_seconds: i64) -> Result<()> {
        require_gte!(interval_seconds, 0);

        self.withdrawal_batch_threshold_interval_seconds = interval_seconds;

        Ok(())
    }

    pub(super) fn add_supported_token(
        &mut self,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        pricing_source: TokenPricingSource,
        operation_reserved_amount: u64,
    ) -> Result<()> {
        if self
            .get_supported_tokens_iter()
            .any(|info| info.mint == mint)
        {
            err!(ErrorCode::FundAlreadySupportedTokenError)?
        }

        require_gt!(
            FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
            self.num_supported_tokens as usize,
            ErrorCode::FundExceededMaxSupportedTokensError
        );

        let mut supported_token = SupportedToken::zeroed();
        supported_token.initialize(
            mint,
            program,
            decimals,
            pricing_source,
            operation_reserved_amount,
        )?;
        self.supported_tokens[self.num_supported_tokens as usize] = supported_token;
        self.num_supported_tokens += 1;

        Ok(())
    }

    #[inline]
    pub(super) fn get_normalized_token(&self) -> Option<&NormalizedToken> {
        if self.normalized_token.enabled == 1 {
            Some(&self.normalized_token)
        } else {
            None
        }
    }

    #[inline]
    pub(super) fn get_normalized_token_pool_address(&self) -> Option<Pubkey> {
        (self.normalized_token.enabled == 1)
            .then(|| self.normalized_token.pricing_source.address())
            .flatten()
    }

    #[inline]
    pub(super) fn get_normalized_token_mut(&mut self) -> Option<&mut NormalizedToken> {
        if self.normalized_token.enabled == 1 {
            Some(&mut self.normalized_token)
        } else {
            None
        }
    }

    pub(super) fn set_normalized_token(
        &mut self,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        pool: Pubkey,
        operation_reserved_amount: u64,
    ) -> Result<()> {
        if self.normalized_token.enabled != 0 {
            err!(ErrorCode::FundNormalizedTokenAlreadySetError)?
        }

        let normalized_token = &mut self.normalized_token;
        normalized_token.initialize(mint, program, decimals, pool, operation_reserved_amount)
    }

    #[inline]
    pub(super) fn get_restaking_vaults_iter(&self) -> impl Iterator<Item = &RestakingVault> {
        self.restaking_vaults
            .iter()
            .take(self.num_restaking_vaults as usize)
    }

    #[inline]
    pub(super) fn get_restaking_vaults_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut RestakingVault> {
        self.restaking_vaults
            .iter_mut()
            .take(self.num_restaking_vaults as usize)
    }

    pub(super) fn get_restaking_vault(&self, vault: &Pubkey) -> Result<&RestakingVault> {
        self.get_restaking_vaults_iter()
            .find(|restaking_vault| restaking_vault.vault == *vault)
            .ok_or_else(|| error!(ErrorCode::FundRestakingVaultNotFoundError))
    }

    pub(super) fn get_restaking_vault_mut(
        &mut self,
        vault: &Pubkey,
    ) -> Result<&mut RestakingVault> {
        self.get_restaking_vaults_iter_mut()
            .find(|restaking_vault| restaking_vault.vault == *vault)
            .ok_or_else(|| error!(ErrorCode::FundRestakingVaultNotFoundError))
    }

    pub(super) fn add_restaking_vault(
        &mut self,
        vault: Pubkey,
        program: Pubkey,
        supported_token_mint: Pubkey,
        receipt_token_mint: Pubkey,
        receipt_token_program: Pubkey,
        receipt_token_decimals: u8,
        receipt_token_operation_reserved_amount: u64,
    ) -> Result<()> {
        if self.get_restaking_vaults_iter().any(|v| v.vault == vault) {
            err!(ErrorCode::FundRestakingVaultAlreadyRegisteredError)?
        }

        require_gt!(
            FUND_ACCOUNT_MAX_RESTAKING_VAULTS,
            self.num_restaking_vaults as usize,
            ErrorCode::FundExceededMaxRestakingVaultsError
        );

        let mut restaking_vault = RestakingVault::zeroed();
        restaking_vault.initialize(
            vault,
            program,
            supported_token_mint,
            receipt_token_mint,
            receipt_token_program,
            receipt_token_decimals,
            receipt_token_operation_reserved_amount,
        )?;
        self.restaking_vaults[self.num_restaking_vaults as usize] = restaking_vault;
        self.num_restaking_vaults += 1;

        Ok(())
    }

    pub(super) fn reload_receipt_token_supply(
        &mut self,
        receipt_token_mint: &mut InterfaceAccount<Mint>,
    ) -> Result<()> {
        require_keys_eq!(self.receipt_token_mint, receipt_token_mint.key());

        receipt_token_mint.reload()?;
        self.receipt_token_supply_amount = receipt_token_mint.supply;

        Ok(())
    }

    #[inline]
    pub(super) fn get_asset_state(
        &self,
        supported_token_mint: Option<Pubkey>,
    ) -> Result<&AssetState> {
        Ok(match supported_token_mint {
            Some(supported_token_mint) => &self.get_supported_token(&supported_token_mint)?.token,
            None => &self.sol,
        })
    }

    #[inline]
    pub(super) fn get_asset_state_mut(
        &mut self,
        supported_token_mint: Option<Pubkey>,
    ) -> Result<&mut AssetState> {
        Ok(match supported_token_mint {
            Some(supported_token_mint) => {
                &mut self.get_supported_token_mut(&supported_token_mint)?.token
            }
            None => &mut self.sol,
        })
    }

    pub(super) fn get_asset_states_iter(&self) -> impl Iterator<Item = &AssetState> {
        std::iter::once(&self.sol).chain(self.get_supported_tokens_iter().map(|v| &v.token))
    }

    pub(super) fn get_asset_states_iter_mut(&mut self) -> impl Iterator<Item = &mut AssetState> {
        let sol = &mut self.sol;
        let tokens = self
            .supported_tokens
            .iter_mut()
            .take(self.num_supported_tokens as usize)
            .map(|v| &mut v.token);
        std::iter::once(sol).chain(tokens)
    }

    /// returns [deposited_amount]
    pub(super) fn deposit(
        &mut self,
        supported_token_mint: Option<Pubkey>,
        asset_amount: u64,
    ) -> Result<u64> {
        if self.deposit_enabled == 0 {
            err!(ErrorCode::FundDepositDisabledError)?
        }
        self.get_asset_state_mut(supported_token_mint)?
            .deposit(asset_amount)
    }

    /// returns [deposited_amount, offsetted_receivable_amount]
    pub(super) fn donate(
        &mut self,
        supported_token_mint: Option<Pubkey>,
        asset_amount: u64,
        offset_receivable: bool,
    ) -> Result<(u64, u64)> {
        if self.deposit_enabled == 0 {
            err!(ErrorCode::FundDepositDisabledError)?
        } else if self.donation_enabled == 0 {
            err!(ErrorCode::FundDonationDisabledError)?
        }
        self.get_asset_state_mut(supported_token_mint)?
            .donate(asset_amount, offset_receivable)
    }

    /// requested receipt_token_amount can be reduced based on the status of the underlying asset.
    /// asset value should be up-to-date before call this.
    pub(super) fn create_withdrawal_request(
        &mut self,
        supported_token_mint: Option<Pubkey>,
        mut receipt_token_amount: u64,
        current_timestamp: i64,
    ) -> Result<WithdrawalRequest> {
        if self.withdrawal_enabled == 0 {
            err!(ErrorCode::FundWithdrawalDisabledError)?
        }

        let asset = self.get_asset_state_mut(supported_token_mint)?;
        if asset.withdrawable_value_as_receipt_token_amount == 0 {
            err!(ErrorCode::FundWithdrawalReserveExhaustedSupportedAsset)?
        }
        receipt_token_amount =
            receipt_token_amount.min(asset.withdrawable_value_as_receipt_token_amount);
        asset.create_withdrawal_request(receipt_token_amount, current_timestamp)
    }

    /// asset value should be updated after call this to estimate fresh withdrawable_value_as_receipt_token_amount.
    pub(super) fn cancel_withdrawal_request(&mut self, request: &WithdrawalRequest) -> Result<()> {
        self.get_asset_state_mut(request.supported_token_mint)?
            .cancel_withdrawal_request(request)
    }

    /// receipt token amount in the queued withdrawal batches for all assets.
    pub(super) fn get_total_receipt_token_withdrawal_obligated_amount(&self) -> u64 {
        self.get_asset_states_iter()
            .map(|asset| asset.get_receipt_token_withdrawal_obligated_amount())
            .sum()
    }

    /// receipt token amount in the queued withdrawal batches for an asset.
    pub(super) fn get_asset_receipt_token_withdrawal_obligated_amount(
        &self,
        supported_token_mint: Option<Pubkey>,
    ) -> Result<u64> {
        Ok(self
            .get_asset_state(supported_token_mint)?
            .get_receipt_token_withdrawal_obligated_amount())
    }

    /// represents the surplus or shortage amount after fulfilling the withdrawal obligations for the given asset.
    pub(super) fn get_asset_net_operation_reserved_amount(
        &self,
        supported_token_mint: Option<Pubkey>,
        with_normal_reserve: bool,
        pricing_service: &PricingService,
    ) -> Result<i128> {
        self.get_asset_state(supported_token_mint)?
            .get_net_operation_reserved_amount(
                &self.receipt_token_mint.key(),
                &self.receipt_token_value.try_deserialize()?,
                with_normal_reserve,
                pricing_service,
            )
    }

    /// get total asset amount, so it includes cash, receivable, normalized, restaked amount.
    pub(super) fn get_asset_total_amount(
        &self,
        supported_token_mint: Option<Pubkey>,
    ) -> Result<u64> {
        Ok(self
            .get_asset_state(supported_token_mint)?
            .get_total_amount(&self.receipt_token_value.try_deserialize()?))
    }

    /// get total asset amount, so it includes cash, receivable, normalized, restaked amount.
    pub(super) fn get_asset_total_amount_as_sol(
        &self,
        supported_token_mint: Option<Pubkey>,
        pricing_service: &PricingService,
    ) -> Result<u64> {
        let asset_total_amount = self.get_asset_total_amount(supported_token_mint)?;
        Ok(match supported_token_mint {
            Some(mint) => pricing_service.get_token_amount_as_sol(&mint, asset_total_amount)?,
            None => asset_total_amount,
        })
    }

    /// get total receivable value of the fund, as converted in the given asset.
    pub(super) fn get_total_operation_receivable_amount_as_asset(
        &self,
        supported_token_mint: Option<Pubkey>,
        pricing_service: &PricingService,
    ) -> Result<u64> {
        let total_operation_receivable_amount_as_sol = self
            .get_asset_states_iter()
            .map(|asset| {
                Ok(match asset.get_token_mint_and_program() {
                    Some((token_mint, _)) => pricing_service
                        .get_token_amount_as_sol(&token_mint, asset.operation_receivable_amount)?,
                    None => asset.operation_receivable_amount,
                })
            })
            .collect::<Result<Vec<_>>>()?
            .iter()
            .sum::<u64>();

        let asset = self.get_asset_state(supported_token_mint)?;
        Ok(match asset.get_token_mint_and_program() {
            Some((token_mint, _)) => pricing_service
                .get_sol_amount_as_token(&token_mint, total_operation_receivable_amount_as_sol)?,
            None => total_operation_receivable_amount_as_sol,
        })
    }

    pub(super) fn get_total_unstaking_obligated_amount_as_sol(
        &self,
        pricing_service: &PricingService,
    ) -> Result<u64> {
        let sol_net_operation_reserved_amount =
            self.get_asset_net_operation_reserved_amount(None, true, &pricing_service)?;
        Ok(
            u64::try_from(sol_net_operation_reserved_amount.min(0).neg())?.saturating_sub(
                self.get_supported_tokens_iter()
                    .map(|supported_token| supported_token.pending_unstaking_amount_as_sol)
                    .sum(),
            ),
        )
    }
}

pub(super) struct FundStakeAccountAddress<'a> {
    stake_account_address: Pubkey,
    fund_account: &'a Pubkey,
    pool_account: &'a Pubkey,
    index: u8,
    bump: u8,
}

impl<'a> FundStakeAccountAddress<'a> {
    fn new(fund_account: &'a Pubkey, pool_account: &'a Pubkey, index: u8) -> Self {
        let (stake_account_address, bump) = Pubkey::find_program_address(
            &[fund_account.as_ref(), pool_account.as_ref(), &[index]],
            &crate::ID,
        );
        Self {
            stake_account_address,
            fund_account,
            pool_account,
            index,
            bump,
        }
    }

    pub(super) fn get_seeds(&self) -> [&[u8]; 4] {
        [
            self.fund_account.as_ref(),
            self.pool_account.as_ref(),
            std::slice::from_ref(&self.index),
            std::slice::from_ref(&self.bump),
        ]
    }
}

impl std::ops::Deref for FundStakeAccountAddress<'_> {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.stake_account_address
    }
}

impl AsRef<Pubkey> for FundStakeAccountAddress<'_> {
    fn as_ref(&self) -> &Pubkey {
        &self.stake_account_address
    }
}

pub(super) struct FundUnstakingTicketAddress<'a> {
    unstaking_ticket_address: Pubkey,
    fund_account: &'a Pubkey,
    pool_account: &'a Pubkey,
    index: u8,
    bump: u8,
}

impl<'a> FundUnstakingTicketAddress<'a> {
    pub const SEED: &'static [u8] = b"unstaking_ticket";

    fn new(fund_account: &'a Pubkey, pool_account: &'a Pubkey, index: u8) -> Self {
        let (unstaking_ticket_address, bump) = Pubkey::find_program_address(
            &[
                Self::SEED,
                fund_account.as_ref(),
                pool_account.as_ref(),
                &[index],
            ],
            &crate::ID,
        );
        Self {
            unstaking_ticket_address,
            fund_account,
            pool_account,
            index,
            bump,
        }
    }

    pub(super) fn get_seeds(&self) -> [&[u8]; 5] {
        [
            Self::SEED,
            self.fund_account.as_ref(),
            self.pool_account.as_ref(),
            std::slice::from_ref(&self.index),
            std::slice::from_ref(&self.bump),
        ]
    }
}

impl std::ops::Deref for FundUnstakingTicketAddress<'_> {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.unstaking_ticket_address
    }
}

impl AsRef<Pubkey> for FundUnstakingTicketAddress<'_> {
    fn as_ref(&self) -> &Pubkey {
        &self.unstaking_ticket_address
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::pricing::TokenPricingSource;
    use anchor_lang::solana_program;

    #[test]
    fn size_fund_account() {
        let size = 8 + std::mem::size_of::<FundAccount>();
        println!(
            "\nfund account size={}, version={}",
            size, FUND_ACCOUNT_CURRENT_VERSION
        );
        println!(
            "supported_token size={}",
            std::mem::size_of::<SupportedToken>()
        );
        println!(
            "operation_state size={}",
            std::mem::size_of::<OperationState>()
        );
        assert_eq!(
            size < solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE
                * (FUND_ACCOUNT_CURRENT_VERSION as usize),
            true
        );

        assert_eq!(std::mem::size_of::<FundAccount>() % 8, 0);
        assert_eq!(std::mem::align_of::<FundAccount>(), 8);

        assert_eq!(std::mem::size_of::<WithdrawalBatch>() % 8, 0);
        assert_eq!(std::mem::align_of::<WithdrawalBatch>(), 8);

        assert_eq!(std::mem::size_of::<SupportedToken>() % 8, 0);
        assert_eq!(std::mem::align_of::<SupportedToken>(), 8);

        assert_eq!(std::mem::size_of::<NormalizedToken>() % 8, 0);
        assert_eq!(std::mem::align_of::<NormalizedToken>(), 8);

        assert_eq!(std::mem::size_of::<RestakingVault>() % 8, 0);
        assert_eq!(std::mem::align_of::<RestakingVault>(), 8);

        assert_eq!(std::mem::size_of::<OperationState>() % 8, 0);
        assert_eq!(std::mem::align_of::<OperationState>(), 8);
    }

    fn create_initialized_fund_account() -> FundAccount {
        let buffer = [0u8; 8 + std::mem::size_of::<FundAccount>()];
        let mut fund = FundAccount::try_deserialize_unchecked(&mut &buffer[..]).unwrap();
        fund.migrate(0, Pubkey::new_unique(), 9, 0, 0);
        fund
    }

    #[test]
    fn test_initialize_update_fund_account() {
        let mut fund = create_initialized_fund_account();

        fund.sol.accumulated_deposit_amount = 1_000_000_000_000;
        fund.sol
            .set_accumulated_deposit_capacity_amount(0)
            .unwrap_err();

        let interval_seconds = 60;
        fund.set_withdrawal_batch_threshold(interval_seconds)
            .unwrap();
        assert_eq!(
            fund.withdrawal_batch_threshold_interval_seconds,
            interval_seconds
        );
    }

    #[test]
    fn test_update_token() {
        let mut fund = create_initialized_fund_account();

        let token1 = Pubkey::new_unique();
        let token2 = Pubkey::new_unique();

        fund.add_supported_token(
            token1,
            Pubkey::default(),
            9,
            TokenPricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
            0,
        )
        .unwrap();
        fund.get_supported_token_mut(&token1)
            .unwrap()
            .token
            .set_accumulated_deposit_capacity_amount(1_000_000_000)
            .unwrap();

        fund.add_supported_token(
            token2,
            Pubkey::default(),
            9,
            TokenPricingSource::MarinadeStakePool {
                address: Pubkey::new_unique(),
            },
            0,
        )
        .unwrap();
        fund.get_supported_token_mut(&token2)
            .unwrap()
            .token
            .set_accumulated_deposit_capacity_amount(1_000_000_000)
            .unwrap();

        fund.add_supported_token(
            token1,
            Pubkey::default(),
            9,
            TokenPricingSource::MarinadeStakePool {
                address: Pubkey::new_unique(),
            },
            0,
        )
        .unwrap_err();
        assert_eq!(fund.num_supported_tokens, 2);
        assert_eq!(
            fund.supported_tokens[0]
                .token
                .accumulated_deposit_capacity_amount,
            1_000_000_000
        );

        fund.supported_tokens[0].token.accumulated_deposit_amount = 1_000_000_000;
        fund.get_supported_token_mut(&token1)
            .unwrap()
            .token
            .set_accumulated_deposit_capacity_amount(0)
            .unwrap_err();
    }

    #[test]
    fn test_deposit_sol() {
        let mut fund = create_initialized_fund_account();
        fund.sol
            .set_accumulated_deposit_capacity_amount(100_000)
            .unwrap();

        assert_eq!(fund.sol.operation_reserved_amount, 0);
        assert_eq!(fund.sol.accumulated_deposit_amount, 0);

        fund.deposit(None, 100_000).unwrap_err();
        fund.set_deposit_enabled(true);
        fund.deposit(None, 100_000).unwrap_err();
        fund.sol.set_depositable(true);
        fund.deposit(None, 100_000).unwrap();
        assert_eq!(fund.sol.operation_reserved_amount, 100_000);
        assert_eq!(fund.sol.accumulated_deposit_amount, 100_000);

        fund.donate(None, 100_000, false).unwrap_err();
        fund.set_donation_enabled(true);
        fund.sol
            .set_accumulated_deposit_capacity_amount(200_000)
            .unwrap();
        fund.donate(None, 100_000, false).unwrap();

        fund.sol.deposit(100_000).unwrap_err();
    }

    #[test]
    fn test_deposit_token() {
        let mut fund = create_initialized_fund_account();

        fund.add_supported_token(
            Pubkey::new_unique(),
            Pubkey::default(),
            9,
            TokenPricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
            0,
        )
        .unwrap();
        fund.supported_tokens[0]
            .token
            .set_accumulated_deposit_capacity_amount(1_000)
            .unwrap();

        assert_eq!(fund.supported_tokens[0].token.operation_reserved_amount, 0);
        assert_eq!(fund.supported_tokens[0].token.accumulated_deposit_amount, 0);

        fund.deposit(Some(fund.supported_tokens[0].mint), 1_000)
            .unwrap_err();
        fund.set_deposit_enabled(true);
        fund.supported_tokens[0].token.set_depositable(true);
        fund.deposit(Some(fund.supported_tokens[0].mint), 1_000)
            .unwrap();
        assert_eq!(
            fund.supported_tokens[0].token.operation_reserved_amount,
            1_000
        );
        assert_eq!(
            fund.supported_tokens[0].token.accumulated_deposit_amount,
            1_000
        );

        fund.supported_tokens[0].token.deposit(1_000).unwrap_err();
    }
}
