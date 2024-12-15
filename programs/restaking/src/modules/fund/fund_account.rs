use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token::accessor::mint;
use anchor_spl::token_2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use bytemuck::{Pod, Zeroable};

use crate::constants::JITO_VAULT_PROGRAM_ID;
use crate::errors::ErrorCode;
use crate::modules::pricing::{
    Asset, PricingService, TokenPricingSource, TokenValue, TokenValuePod,
};
use crate::utils::{get_proportional_amount, PDASeeds, ZeroCopyHeader};

use super::*;

#[constant]
/// ## Version History
/// * v4: migrate to new layout including new fields using bytemuck. (35376 ~= 35KB)
pub const FUND_ACCOUNT_CURRENT_VERSION: u16 = 4;

pub const FUND_WITHDRAWAL_FEE_RATE_BPS_LIMIT: u16 = 500;
pub const FUND_ACCOUNT_MAX_QUEUED_WITHDRAWAL_BATCHES: usize = 10;
pub const FUND_ACCOUNT_MAX_SUPPORTED_TOKENS: usize = 10;
pub const FUND_ACCOUNT_MAX_RESTAKING_VAULTS: usize = 4;

#[account(zero_copy)]
#[repr(C)]
pub struct FundAccount {
    data_version: u16,
    bump: u8,
    reserve_account_bump: u8,
    treasury_account_bump: u8,
    _padding: [u8; 10],

    /// irreversible receipt token configuration
    pub(super) transfer_enabled: u8,

    /// informative
    pub receipt_token_mint: Pubkey,
    pub(super) receipt_token_program: Pubkey,
    pub(super) receipt_token_decimals: u8,
    _padding2: [u8; 7],
    pub(super) receipt_token_supply_amount: u64,
    pub(super) one_receipt_token_as_sol: u64,
    pub(super) receipt_token_value_updated_slot: u64,
    pub(super) receipt_token_value: TokenValuePod,

    /// configurations: SOL deposit & withdrawal
    pub(super) sol_accumulated_deposit_capacity_amount: u64,
    pub(super) sol_accumulated_deposit_amount: u64,
    _padding3: [u8; 5],
    pub(super) sol_withdrawable: u8,
    pub(super) sol_normal_reserve_rate_bps: u16,
    pub(super) sol_normal_reserve_max_amount: u64,

    /// configurations: withdrawal
    pub(super) withdrawal_batch_threshold_interval_seconds: i64,
    pub(super) withdrawal_fee_rate_bps: u16,
    pub(super) withdrawal_enabled: u8,
    _padding4: [u8; 5],

    withdrawal_last_created_request_id: u64,
    pub(super) withdrawal_last_processed_batch_id: u64,
    withdrawal_last_batch_enqueued_at: i64,
    withdrawal_last_batch_processed_at: i64,
    withdrawal_pending_batch: WithdrawalBatch,
    _padding5: [u8; 15],
    withdrawal_num_queued_batches: u8,
    withdrawal_queued_batches: [WithdrawalBatch; FUND_ACCOUNT_MAX_QUEUED_WITHDRAWAL_BATCHES],
    _reserved: [u8; 64],

    /// informative: reserved amount that users can claim for processed withdrawal requests, which is not accounted for as an asset of the fund.
    pub sol_withdrawal_user_reserved_amount: u64,

    /// asset: A receivable that the fund may charge the users requesting withdrawals.
    /// It is accrued during either the preparation of the withdrawal obligation or rebalancing of LST (fee from unstaking, unrestaking).
    /// And it shall be settled by the withdrawal fee normally. But it also can be written off by an authorized operation.
    /// Then it costs the rebalancing expense to the capital of the fund itself as an operation cost instead of charging the users requesting withdrawals.
    pub(super) sol_operation_receivable_amount: u64,

    /// asset
    pub(super) sol_operation_reserved_amount: u64,

    /// asset & configuration: underlying assets
    _padding6: [u8; 15],
    num_supported_tokens: u8,
    supported_tokens: [SupportedToken; FUND_ACCOUNT_MAX_SUPPORTED_TOKENS],

    /// asset & configuration: optional basket of underlying assets
    normalized_token: NormalizedToken,

    /// asset & configurations: investments
    _padding7: [u8; 15],
    num_restaking_vaults: u8,
    restaking_vaults: [RestakingVault; FUND_ACCOUNT_MAX_RESTAKING_VAULTS],

    /// fund operation state
    pub(super) operation: OperationState,
}

impl PDASeeds<2> for FundAccount {
    const SEED: &'static [u8] = b"fund";

    fn get_seed_phrase(&self) -> [&[u8]; 2] {
        [Self::SEED, self.receipt_token_mint.as_ref()]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
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
    ) {
        if self.data_version == 0 {
            self.bump = bump;
            self.reserve_account_bump =
                Pubkey::find_program_address(&self.get_reserve_account_seed_phrase(), &crate::ID).1;
            self.treasury_account_bump =
                Pubkey::find_program_address(&self.get_treasury_account_seed_phrase(), &crate::ID)
                    .1;
            self.receipt_token_mint = receipt_token_mint;
            self.receipt_token_program = token_2022::ID;
            self.receipt_token_decimals = receipt_token_decimals;
            self.receipt_token_supply_amount = receipt_token_supply;
            self.withdrawal_last_created_request_id = 10_000;
            self.withdrawal_pending_batch.batch_id = 10_001;
            self.data_version = 4;
        }
    }

    #[inline(always)]
    pub(super) fn initialize(&mut self, bump: u8, receipt_token_mint: &InterfaceAccount<Mint>) {
        self.migrate(
            bump,
            receipt_token_mint.key(),
            receipt_token_mint.decimals,
            receipt_token_mint.supply,
        );
    }

    #[inline(always)]
    pub(super) fn update_if_needed(&mut self, receipt_token_mint: &InterfaceAccount<Mint>) {
        self.initialize(self.bump, receipt_token_mint);
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

    pub(super) fn get_reserve_account_seeds(&self) -> Vec<&[u8]> {
        let mut seeds = Vec::with_capacity(3);
        seeds.extend(self.get_reserve_account_seed_phrase());
        seeds.push(std::slice::from_ref(&self.reserve_account_bump));
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

    pub(super) fn get_treasury_account_seeds(&self) -> Vec<&[u8]> {
        let mut seeds = Vec::with_capacity(3);
        seeds.extend(self.get_treasury_account_seed_phrase());
        seeds.push(std::slice::from_ref(&self.treasury_account_bump));
        seeds
    }

    pub(super) fn get_treasury_account_address(&self) -> Result<Pubkey> {
        Ok(
            Pubkey::create_program_address(&self.get_treasury_account_seeds(), &crate::ID)
                .map_err(|_| ProgramError::InvalidSeeds)?,
        )
    }

    pub(super) fn find_supported_token_account_address(&self, token: &Pubkey) -> Result<Pubkey> {
        let supported_token = self.get_supported_token(token)?;
        Ok(
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &self.find_account_address()?,
                &supported_token.mint,
                &supported_token.program,
            ),
        )
    }

    pub const UNSTAKING_TICKET_SEED: &'static [u8] = b"unstaking_ticket";

    #[inline(always)]
    fn get_unstaking_ticket_account_seed_phrase(
        &self,
        pool_account: &Pubkey,
        index: u8,
    ) -> [Vec<u8>; 4] {
        [
            Self::UNSTAKING_TICKET_SEED.to_vec(),
            self.receipt_token_mint.as_ref().to_vec(),
            pool_account.as_ref().to_vec(),
            vec![index],
        ]
    }

    /// usage:
    /// ```rs
    /// let seeds: Vec<Vec<u8>> = get_unstaking_ticket_account_seeds();
    /// let seeds_ref: &[&[u8]] = seeds.iter().map(Vec::as_slice).collect::<Vec<_>>().as_slice();
    /// // ...
    /// ctx.with_signer_seeds(&[seeds_ref])
    /// ```
    pub(super) fn get_unstaking_ticket_account_seeds(
        &self,
        pool_account: &Pubkey,
        index: u8,
    ) -> Vec<Vec<u8>> {
        let seed_phrase = self.get_unstaking_ticket_account_seed_phrase(pool_account, index);
        let bump = Pubkey::find_program_address(
            &std::array::from_fn::<_, 4, _>(|i| seed_phrase[i].as_slice()),
            &crate::ID,
        )
        .1;

        let mut seeds = Vec::with_capacity(5);
        seeds.extend(seed_phrase);
        seeds.push(vec![bump]);
        seeds
    }

    pub(super) fn find_unstaking_ticket_account_address(
        &self,
        pool_account: &Pubkey,
        index: u8,
    ) -> (Pubkey, u8) {
        let seed_phrase = self.get_unstaking_ticket_account_seed_phrase(pool_account, index);
        Pubkey::find_program_address(
            &std::array::from_fn::<_, 4, _>(|i| seed_phrase[i].as_slice()),
            &crate::ID,
        )
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
        self.supported_tokens[..self.num_supported_tokens as usize].iter()
    }

    #[inline]
    pub fn get_supported_tokens_iter_mut(&mut self) -> impl Iterator<Item = &mut SupportedToken> {
        self.supported_tokens[..self.num_supported_tokens as usize].iter_mut()
    }

    pub(super) fn get_supported_token(&self, token: &Pubkey) -> Result<&SupportedToken> {
        self.get_supported_tokens_iter()
            .find(|supported_token| supported_token.mint == *token)
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

    pub(super) fn set_sol_accumulated_deposit_amount(&mut self, sol_amount: u64) -> Result<()> {
        require_gte!(
            self.sol_accumulated_deposit_capacity_amount,
            sol_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.sol_accumulated_deposit_amount = sol_amount;

        Ok(())
    }

    pub(super) fn set_sol_accumulated_deposit_capacity_amount(
        &mut self,
        sol_amount: u64,
    ) -> Result<()> {
        require_gte!(
            sol_amount,
            self.sol_accumulated_deposit_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.sol_accumulated_deposit_capacity_amount = sol_amount;

        Ok(())
    }

    #[inline(always)]
    pub fn get_withdrawal_fee_rate_as_percent(&self) -> f32 {
        self.withdrawal_fee_rate_bps as f32 / 100.0
    }

    #[inline(always)]
    pub(super) fn get_withdrawal_fee_amount(&self, amount: u64) -> Result<u64> {
        get_proportional_amount(amount, self.withdrawal_fee_rate_bps as u64, 10_000)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
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
    pub(super) fn set_withdrawal_enabled(&mut self, enabled: bool) {
        self.withdrawal_enabled = if enabled { 1 } else { 0 };
    }

    pub(super) fn set_withdrawal_batch_threshold(&mut self, interval_seconds: i64) -> Result<()> {
        require_gte!(interval_seconds, 0);

        self.withdrawal_batch_threshold_interval_seconds = interval_seconds;

        Ok(())
    }

    #[inline(always)]
    pub(super) fn set_sol_normal_reserve_max_amount(&mut self, sol_amount: u64) {
        self.sol_normal_reserve_max_amount = sol_amount;
    }

    pub(super) fn set_sol_normal_reserve_rate_bps(&mut self, reserve_rate_bps: u16) -> Result<()> {
        require_gte!(
            10_00, // 10%
            reserve_rate_bps,
            ErrorCode::FundInvalidUpdateError
        );

        self.sol_normal_reserve_rate_bps = reserve_rate_bps;

        Ok(())
    }

    #[inline(always)]
    pub(super) fn set_sol_withdrawable(&mut self, withdrawable: bool) {
        self.sol_withdrawable = if withdrawable { 1 } else { 0 };
    }

    pub(super) fn create_withdrawal_request(
        &mut self,
        receipt_token_amount: u64,
        supported_token_mint: Option<Pubkey>,
        current_timestamp: i64,
    ) -> Result<WithdrawalRequest> {
        if self.withdrawal_enabled == 0 {
            err!(ErrorCode::FundWithdrawalDisabledError)?
        }

        // TODO: check supported_token_mint

        self.withdrawal_last_created_request_id += 1;
        let request_id = self.withdrawal_last_created_request_id;

        let request = WithdrawalRequest::new(
            self.withdrawal_pending_batch.batch_id,
            request_id,
            receipt_token_amount,
            supported_token_mint,
            current_timestamp,
        );
        self.withdrawal_pending_batch.add_request(&request)?;

        Ok(request)
    }

    pub(super) fn cancel_withdrawal_request(&mut self, request: WithdrawalRequest) -> Result<()> {
        // assert creation
        require_gte!(
            self.withdrawal_last_created_request_id,
            request.request_id,
            ErrorCode::FundWithdrawalRequestNotFoundError
        );

        // assert not queued yet
        require_eq!(
            request.batch_id,
            self.withdrawal_pending_batch.batch_id,
            ErrorCode::FundWithdrawalRequestAlreadyQueuedError
        );

        self.withdrawal_pending_batch.remove_request(&request)
    }

    /// returns [enqueued]
    pub(super) fn enqueue_withdrawal_pending_batch(
        &mut self,
        current_timestamp: i64,
        forced: bool,
    ) -> bool {
        if !((forced
            || current_timestamp - self.withdrawal_last_batch_enqueued_at
                >= self.withdrawal_batch_threshold_interval_seconds)
            && self.withdrawal_num_queued_batches
                < FUND_ACCOUNT_MAX_QUEUED_WITHDRAWAL_BATCHES as u8
            && self.withdrawal_pending_batch.num_requests > 0)
        {
            return false;
        }

        let next_batch_id = self.withdrawal_pending_batch.batch_id + 1;
        let mut pending_batch = std::mem::replace(&mut self.withdrawal_pending_batch, {
            let mut new_pending_batch = WithdrawalBatch::zeroed();
            new_pending_batch.initialize(next_batch_id);
            new_pending_batch
        });
        pending_batch.enqueued_at = current_timestamp;

        self.withdrawal_last_batch_enqueued_at = current_timestamp;
        self.withdrawal_queued_batches[self.withdrawal_num_queued_batches as usize] = pending_batch;
        self.withdrawal_num_queued_batches += 1;

        true
    }

    pub(super) fn dequeue_withdrawal_batches(
        &mut self,
        mut count: usize,
        current_timestamp: i64,
    ) -> Result<Vec<WithdrawalBatch>> {
        if count == 0 {
            return Ok(vec![]);
        }
        require_gte!(self.withdrawal_num_queued_batches as usize, count);

        self.withdrawal_last_processed_batch_id =
            self.withdrawal_queued_batches[count - 1].batch_id;
        self.withdrawal_last_batch_processed_at = current_timestamp;
        let processing_batches = self.withdrawal_queued_batches[..count].to_vec();

        for i in 0..self.withdrawal_num_queued_batches as usize {
            if i < (self.withdrawal_num_queued_batches as usize) - count {
                self.withdrawal_queued_batches[i] = self.withdrawal_queued_batches[i + count];
            } else {
                self.withdrawal_queued_batches[i] = WithdrawalBatch::zeroed()
            }
        }
        self.withdrawal_num_queued_batches -= count as u8;

        Ok(processing_batches)
    }

    pub(super) fn get_withdrawal_queued_batches_iter(
        &self,
    ) -> impl Iterator<Item = &WithdrawalBatch> {
        self.withdrawal_queued_batches[..self.withdrawal_num_queued_batches as usize].iter()
    }

    pub(super) fn get_withdrawal_queued_batches_iter_to_process(
        &self,
        current_timestamp: i64,
        forced: bool,
    ) -> impl Iterator<Item = &WithdrawalBatch> {
        let available = forced
            || current_timestamp - self.withdrawal_last_batch_processed_at
                >= self.withdrawal_batch_threshold_interval_seconds;
        self.get_withdrawal_queued_batches_iter()
            .filter(move |_| available)
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
        self.restaking_vaults[..self.num_restaking_vaults as usize].iter()
    }

    #[inline]
    pub(super) fn get_restaking_vaults_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut RestakingVault> {
        self.restaking_vaults[..self.num_restaking_vaults as usize].iter_mut()
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

    pub(super) fn deposit_sol(&mut self, amount: u64) -> Result<()> {
        let new_sol_accumulated_deposit_amount = self
            .sol_accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        require_gte!(
            self.sol_accumulated_deposit_capacity_amount,
            new_sol_accumulated_deposit_amount,
            ErrorCode::FundExceededSOLCapacityAmountError
        );

        self.sol_accumulated_deposit_amount = new_sol_accumulated_deposit_amount;
        self.sol_operation_reserved_amount = self
            .sol_operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
pub(super) struct WithdrawalBatch {
    pub batch_id: u64,
    pub num_requests: u64,
    pub receipt_token_amount: u64,
    enqueued_at: i64,
    _reserved: [u8; 32],
}

impl WithdrawalBatch {
    fn initialize(&mut self, batch_id: u64) {
        self.batch_id = batch_id;
        self.num_requests = 0;
        self.receipt_token_amount = 0;
        self.enqueued_at = 0;
    }

    fn add_request(&mut self, request: &WithdrawalRequest) -> Result<()> {
        self.num_requests += 1;
        self.receipt_token_amount += request.receipt_token_amount;

        Ok(())
    }

    fn remove_request(&mut self, request: &WithdrawalRequest) -> Result<()> {
        self.num_requests -= 1;
        self.receipt_token_amount -= request.receipt_token_amount;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::pricing::TokenPricingSource;
    use anchor_lang::{solana_program, AccountDeserialize};

    #[test]
    fn size_fund_account() {
        let size = 8 + std::mem::size_of::<FundAccount>();
        println!(
            "\nfund account size={}, version={}",
            size, FUND_ACCOUNT_CURRENT_VERSION
        );
        println!(
            "fund operation_state size={}",
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
        fund.migrate(0, Pubkey::new_unique(), 9, 0);
        fund
    }

    #[test]
    fn test_initialize_update_fund_account() {
        let mut fund = create_initialized_fund_account();

        fund.sol_accumulated_deposit_amount = 1_000_000_000_000;
        fund.set_sol_accumulated_deposit_capacity_amount(0)
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
            fund.supported_tokens[0].accumulated_deposit_capacity_amount,
            1_000_000_000
        );

        fund.supported_tokens[0].accumulated_deposit_amount = 1_000_000_000;
        fund.get_supported_token_mut(&token1)
            .unwrap()
            .set_accumulated_deposit_capacity_amount(0)
            .unwrap_err();
    }

    #[test]
    fn test_deposit_sol() {
        let mut fund = create_initialized_fund_account();
        fund.set_sol_accumulated_deposit_capacity_amount(100_000)
            .unwrap();

        assert_eq!(fund.sol_operation_reserved_amount, 0);
        assert_eq!(fund.sol_accumulated_deposit_amount, 0);

        fund.deposit_sol(100_000).unwrap();
        assert_eq!(fund.sol_operation_reserved_amount, 100_000);
        assert_eq!(fund.sol_accumulated_deposit_amount, 100_000);

        fund.deposit_sol(100_000).unwrap_err();
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
            .set_accumulated_deposit_capacity_amount(1_000)
            .unwrap();

        assert_eq!(fund.supported_tokens[0].operation_reserved_amount, 0);
        assert_eq!(fund.supported_tokens[0].accumulated_deposit_amount, 0);

        fund.supported_tokens[0].deposit_token(1_000).unwrap();
        assert_eq!(fund.supported_tokens[0].operation_reserved_amount, 1_000);
        assert_eq!(fund.supported_tokens[0].accumulated_deposit_amount, 1_000);

        fund.supported_tokens[0].deposit_token(1_000).unwrap_err();
    }

    #[test]
    fn withdrawal_test() {
        let mut fund = create_initialized_fund_account();
        assert_eq!(fund.withdrawal_pending_batch.batch_id, 10001);
        assert_eq!(fund.withdrawal_last_created_request_id, 10000);

        assert!(fund.create_withdrawal_request(10, None, 0).is_err());

        fund.set_withdrawal_enabled(true);
        fund.set_withdrawal_batch_threshold(1).unwrap();

        let req1 = fund.create_withdrawal_request(10, None, 0).unwrap();
        assert_eq!(req1.batch_id, fund.withdrawal_pending_batch.batch_id);
        assert_eq!(fund.withdrawal_pending_batch.batch_id, 10_001);
        assert_eq!(req1.request_id, 10_001);
        assert_eq!(fund.withdrawal_last_created_request_id, 10_001);
        assert_eq!(fund.withdrawal_pending_batch.num_requests, 1);

        let req2 = fund.create_withdrawal_request(20, None, 0).unwrap();
        assert_eq!(req2.batch_id, fund.withdrawal_pending_batch.batch_id);
        assert_eq!(req2.request_id, 10_002);
        assert_eq!(fund.withdrawal_last_created_request_id, 10_002);
        assert_eq!(fund.withdrawal_pending_batch.num_requests, 2);

        fund.cancel_withdrawal_request(req2.clone()).unwrap();
        assert_eq!(fund.withdrawal_last_created_request_id, 10_002);
        assert_eq!(fund.withdrawal_pending_batch.num_requests, 1);

        let req3 = fund.create_withdrawal_request(20, None, 0).unwrap();
        assert_eq!(req3.batch_id, fund.withdrawal_pending_batch.batch_id);
        assert_eq!(req3.request_id, 10_003);
        assert_eq!(fund.withdrawal_last_created_request_id, 10_003);
        assert_eq!(fund.withdrawal_pending_batch.num_requests, 2);

        assert!(!fund.enqueue_withdrawal_pending_batch(0, false));
        assert!(fund.enqueue_withdrawal_pending_batch(1, false));

        assert!(fund.cancel_withdrawal_request(req3).is_err());
        assert_eq!(fund.withdrawal_pending_batch.batch_id, 10_002);
        assert_eq!(fund.withdrawal_pending_batch.num_requests, 0);
        assert_eq!(fund.withdrawal_pending_batch.receipt_token_amount, 0);
        assert_eq!(fund.withdrawal_queued_batches[0].batch_id, 10_001);
        assert_eq!(fund.withdrawal_queued_batches[0].num_requests, 2);
        assert_eq!(fund.withdrawal_queued_batches[0].receipt_token_amount, 30);
        assert_eq!(fund.withdrawal_queued_batches[0].enqueued_at, 1);
        assert_eq!(fund.withdrawal_last_batch_enqueued_at, 1);
        assert_eq!(fund.get_withdrawal_queued_batches_iter().count(), 1);
        assert_eq!(
            fund.get_withdrawal_queued_batches_iter_to_process(0, false)
                .count(),
            0
        );
        assert_eq!(
            fund.get_withdrawal_queued_batches_iter_to_process(0, true)
                .count(),
            1
        );
        assert_eq!(
            fund.get_withdrawal_queued_batches_iter_to_process(1, false)
                .count(),
            1
        );

        assert!(!fund.enqueue_withdrawal_pending_batch(1, false));
        assert!(!fund.enqueue_withdrawal_pending_batch(1, true));

        let req4 = fund.create_withdrawal_request(30, None, 2).unwrap();
        assert_eq!(req4.batch_id, fund.withdrawal_pending_batch.batch_id);
        assert_eq!(req4.request_id, 10_004);
        assert_eq!(fund.withdrawal_last_created_request_id, 10_004);
        assert_eq!(fund.withdrawal_pending_batch.num_requests, 1);

        assert!(fund.enqueue_withdrawal_pending_batch(2, false));
        assert_eq!(fund.get_withdrawal_queued_batches_iter().count(), 2);
        assert_eq!(fund.withdrawal_pending_batch.batch_id, 10_003);
        assert_eq!(fund.withdrawal_pending_batch.num_requests, 0);
        assert_eq!(fund.withdrawal_pending_batch.receipt_token_amount, 0);
        assert_eq!(fund.withdrawal_queued_batches[0].batch_id, 10_001);
        assert_eq!(fund.withdrawal_queued_batches[0].num_requests, 2);
        assert_eq!(fund.withdrawal_queued_batches[0].receipt_token_amount, 30);
        assert_eq!(fund.withdrawal_queued_batches[0].enqueued_at, 1);
        assert_eq!(fund.withdrawal_queued_batches[1].batch_id, 10_002);
        assert_eq!(fund.withdrawal_queued_batches[1].num_requests, 1);
        assert_eq!(fund.withdrawal_queued_batches[1].receipt_token_amount, 30);
        assert_eq!(fund.withdrawal_queued_batches[1].enqueued_at, 2);
        assert_eq!(fund.withdrawal_last_batch_enqueued_at, 2);
        assert_eq!(fund.get_withdrawal_queued_batches_iter().count(), 2);
        assert_eq!(
            fund.get_withdrawal_queued_batches_iter_to_process(0, false)
                .count(),
            0
        );
        assert_eq!(
            fund.get_withdrawal_queued_batches_iter_to_process(0, true)
                .count(),
            2
        );
        assert_eq!(
            fund.get_withdrawal_queued_batches_iter_to_process(2, false)
                .count(),
            2
        );

        assert!(fund.dequeue_withdrawal_batches(3, 3).is_err());
        let dequeued_batches = fund.dequeue_withdrawal_batches(2, 3).unwrap();
        assert_eq!(dequeued_batches.len(), 2);
        assert_eq!(dequeued_batches[0].batch_id, 10_001);
        assert_eq!(dequeued_batches[0].num_requests, 2);
        assert_eq!(dequeued_batches[0].receipt_token_amount, 30);
        assert_eq!(dequeued_batches[0].enqueued_at, 1);
        assert_eq!(dequeued_batches[1].batch_id, 10_002);
        assert_eq!(dequeued_batches[1].num_requests, 1);
        assert_eq!(dequeued_batches[1].receipt_token_amount, 30);
        assert_eq!(dequeued_batches[1].enqueued_at, 2);
        assert_eq!(fund.withdrawal_last_batch_processed_at, 3);
        assert_eq!(fund.withdrawal_last_processed_batch_id, 10_002);

        assert_eq!(fund.get_withdrawal_queued_batches_iter().count(), 0);
        assert_eq!(
            fund.get_withdrawal_queued_batches_iter_to_process(4, false)
                .count(),
            0
        );
        assert_eq!(
            fund.get_withdrawal_queued_batches_iter_to_process(4, true)
                .count(),
            0
        );

        // println!("{:?}", state);
    }
}
