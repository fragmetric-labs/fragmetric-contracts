use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token::accessor::mint;
use anchor_spl::token_2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::constants::JITO_VAULT_PROGRAM_ID;
use crate::errors::ErrorCode;
use crate::modules::pricing::{Asset, TokenPricingSource, TokenValue};
use crate::utils::PDASeeds;

use super::*;

#[constant]
/// ## Version History
/// * v1: Initial Version
/// * v2: Change reserve fund structure
/// * v3: Remove `sol_fee_income_reserved_amount` field
/// * v4: Add `receipt_token_program`, .., `one_receipt_token_as_sol`, `normalized_token`, `restaking_vaults`, `operation` fields
pub const FUND_ACCOUNT_CURRENT_VERSION: u16 = 4;

const MAX_SUPPORTED_TOKENS: usize = 16;
const MAX_RESTAKING_VAULTS: usize = 2;

#[account]
#[derive(InitSpace)]
pub struct FundAccount {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    #[max_len(MAX_SUPPORTED_TOKENS)]
    pub(super) supported_tokens: Vec<SupportedToken>,
    pub(super) sol_capacity_amount: u64,
    pub(super) sol_accumulated_deposit_amount: u64,
    // TODO v0.3/operation: visibility
    pub(in crate::modules) sol_operation_reserved_amount: u64,

    pub(super) withdrawal: WithdrawalState,

    pub(super) receipt_token_program: Pubkey,
    pub(super) receipt_token_decimals: u8,
    pub(super) receipt_token_supply_amount: u64,
    pub(super) receipt_token_value: TokenValue,
    pub(super) receipt_token_value_updated_at: i64,
    pub(super) one_receipt_token_as_sol: u64,

    pub(super) normalized_token: Option<NormalizedToken>,

    #[max_len(MAX_RESTAKING_VAULTS)]
    pub(super) restaking_vaults: Vec<RestakingVault>,

    pub(super) operation: OperationState,

    reserve_account_bump: u8,
    treasury_account_bump: u8,
    _reserved: [u8; 256],
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
            self.receipt_token_mint = receipt_token_mint;
            self.withdrawal.migrate(self.data_version);
            self.data_version = 1;
        }

        if self.data_version == 1 {
            self.withdrawal.migrate(self.data_version);
            self.data_version = 2;
        }

        if self.data_version == 2 {
            self.withdrawal.migrate(self.data_version);
            self.data_version = 3;
        }

        if self.data_version == 3 {
            self.receipt_token_program = token_2022::ID;
            self.receipt_token_decimals = receipt_token_decimals;
            self.receipt_token_supply_amount = receipt_token_supply;
            self.receipt_token_value = TokenValue {
                numerator: Vec::new(),
                denominator: 0,
            };
            self.receipt_token_value_updated_at = 0;
            self.one_receipt_token_as_sol = 0;

            self.normalized_token = None;
            self.restaking_vaults = Vec::new();
            self.operation.migrate(self.data_version);

            self.reserve_account_bump =
                Pubkey::find_program_address(&self.get_reserve_account_seed_phrase(), &crate::ID).1;
            self.treasury_account_bump =
                Pubkey::find_program_address(&self.get_treasury_account_seed_phrase(), &crate::ID)
                    .1;

            for supported_token in &mut self.supported_tokens {
                supported_token.operating_amount = 0;
            }

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

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn get_reserve_account_seeds(&self) -> Vec<&[u8]> {
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
        let bump =
            Pubkey::find_program_address(&seed_phrase.each_ref().map(Vec::as_slice), &crate::ID).1;

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
        Pubkey::find_program_address(
            &self
                .get_unstaking_ticket_account_seed_phrase(pool_account, index)
                .each_ref()
                .map(Vec::as_slice),
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

    pub(super) fn get_supported_token(&self, token: &Pubkey) -> Result<&SupportedToken> {
        self.supported_tokens
            .iter()
            .find(|info| info.mint == *token)
            .ok_or_else(|| error!(ErrorCode::FundNotSupportedTokenError))
    }

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn get_supported_token_mut(
        &mut self,
        token_mint: &Pubkey,
    ) -> Result<&mut SupportedToken> {
        self.supported_tokens
            .iter_mut()
            .find(|info| info.mint == *token_mint)
            .ok_or_else(|| error!(ErrorCode::FundNotSupportedTokenError))
    }

    pub(super) fn set_sol_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        require_gte!(
            capacity_amount,
            self.sol_accumulated_deposit_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.sol_capacity_amount = capacity_amount;

        Ok(())
    }

    pub(super) fn add_supported_token(
        &mut self,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
    ) -> Result<()> {
        if self.supported_tokens.iter().any(|info| info.mint == mint) {
            err!(ErrorCode::FundAlreadySupportedTokenError)?
        }

        require_gt!(
            MAX_SUPPORTED_TOKENS,
            self.supported_tokens.len(),
            ErrorCode::FundExceededMaxSupportedTokensError
        );

        self.supported_tokens.push(SupportedToken::new(
            mint,
            program,
            decimals,
            capacity_amount,
            pricing_source,
        )?);

        Ok(())
    }

    pub(super) fn set_normalized_token(
        &mut self,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        pool: Pubkey,
        operation_reserved_amount: u64,
    ) -> Result<()> {
        if self.normalized_token.is_some() {
            err!(ErrorCode::FundNormalizedTokenAlreadySetError)?
        }

        self.normalized_token = Some(NormalizedToken::new(
            mint,
            program,
            decimals,
            pool,
            operation_reserved_amount,
        ));

        Ok(())
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
        if self
            .restaking_vaults
            .iter()
            .any(|v| v.vault == vault && v.supported_token_mint == supported_token_mint)
        {
            err!(ErrorCode::FundRestakingVaultAlreadyRegisteredError)?
        }

        require_gt!(
            MAX_RESTAKING_VAULTS,
            self.restaking_vaults.len(),
            ErrorCode::FundExceededMaxRestakingVaultsError
        );

        self.restaking_vaults.push(RestakingVault::new(
            vault,
            program,
            supported_token_mint,
            receipt_token_mint,
            receipt_token_program,
            receipt_token_decimals,
            receipt_token_operation_reserved_amount,
        )?);

        Ok(())
    }

    pub(super) fn reload_receipt_token_supply(
        &mut self,
        receipt_token_mint: &mut InterfaceAccount<Mint>,
    ) -> Result<()> {
        #[cfg(debug_assertions)]
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
            self.sol_capacity_amount,
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

    /// returns sol_withdrawal_fee
    pub(super) fn initialize_batch_withdrawal_ticket(
        &mut self,
        ticket: &mut FundBatchWithdrawalTicket,
        batch: WithdrawalBatch,
        sol_amount: u64,
        current_timestamp: i64,
    ) -> Result<u64> {
        let bump = FundBatchWithdrawalTicket::find_account_address(
            &self.receipt_token_mint,
            batch.batch_id,
        )
        .1;

        let sol_fee_amount = self.withdrawal.get_sol_fee_amount(sol_amount)?;
        let sol_user_amount = sol_amount - sol_fee_amount;

        self.sol_operation_reserved_amount -= sol_amount;
        ticket.initialize(
            bump,
            self.receipt_token_mint,
            batch.batch_id,
            batch.num_requests,
            batch.receipt_token_amount,
            sol_user_amount,
            current_timestamp,
        );
        Ok(sol_fee_amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::pricing::TokenPricingSource;
    use anchor_lang::{AccountDeserialize, Space};

    #[test]
    fn size_fund_account() {
        println!("\nfund account init size: {}", FundAccount::INIT_SPACE);
    }

    fn create_initialized_fund_account() -> FundAccount {
        let buffer = [0u8; 8 + FundAccount::INIT_SPACE];
        let mut fund = FundAccount::try_deserialize_unchecked(&mut &buffer[..]).unwrap();
        fund.migrate(0, Pubkey::new_unique(), 9, 0);
        fund
    }

    #[test]
    fn test_initialize_update_fund_account() {
        let mut fund = create_initialized_fund_account();

        assert_eq!(fund.sol_capacity_amount, 0);
        assert_eq!(fund.withdrawal.get_sol_fee_rate_as_percent(), 0.);
        assert!(fund.withdrawal.enabled);
        assert_eq!(fund.withdrawal.batch_threshold_interval_seconds, 0);

        fund.sol_accumulated_deposit_amount = 1_000_000_000_000;
        fund.set_sol_capacity_amount(0).unwrap_err();

        let interval_seconds = 60;
        fund.withdrawal
            .set_batch_threshold(interval_seconds)
            .unwrap();
        assert_eq!(
            fund.withdrawal.batch_threshold_interval_seconds,
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
            1_000_000_000,
            TokenPricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap();
        fund.add_supported_token(
            token2,
            Pubkey::default(),
            9,
            1_000_000_000,
            TokenPricingSource::MarinadeStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap();
        fund.add_supported_token(
            token1,
            Pubkey::default(),
            9,
            1_000_000_000,
            TokenPricingSource::MarinadeStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap_err();
        assert_eq!(fund.supported_tokens.len(), 2);
        assert_eq!(fund.supported_tokens[0].capacity_amount, 1_000_000_000);

        fund.supported_tokens[0].accumulated_deposit_amount = 1_000_000_000;
        fund.get_supported_token_mut(&token1)
            .unwrap()
            .set_capacity_amount(0)
            .unwrap_err();
    }

    #[test]
    fn test_deposit_sol() {
        let mut fund = create_initialized_fund_account();
        fund.set_sol_capacity_amount(100_000).unwrap();

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
            1_000,
            TokenPricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap();

        assert_eq!(fund.supported_tokens[0].operation_reserved_amount, 0);
        assert_eq!(fund.supported_tokens[0].accumulated_deposit_amount, 0);

        fund.supported_tokens[0].deposit_token(1_000).unwrap();
        assert_eq!(fund.supported_tokens[0].operation_reserved_amount, 1_000);
        assert_eq!(fund.supported_tokens[0].accumulated_deposit_amount, 1_000);

        fund.supported_tokens[0].deposit_token(1_000).unwrap_err();
    }
}
