use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token_2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::errors::ErrorCode;
use crate::modules::pricing::{Asset, TokenPricingSource, TokenValue};
use crate::utils::PDASeeds;

use super::*;

#[constant]
/// ## Version History
/// * v1: Initial Version
/// * v2: Change reserve fund structure
/// * v3: Remove `sol_fee_income_reserved_amount` field
/// * v4: Add `receipt_token_program`, .., `one_receipt_token_as_sol`, `restaking_vaults`, `operation` fields
pub const FUND_ACCOUNT_CURRENT_VERSION: u16 = 4;

const MAX_SUPPORTED_TOKENS: usize = 16;
const MAX_RESTAKING_VAULTS: usize = 4;

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
    _reserved: [u8; 256],
}

impl PDASeeds<2> for FundAccount {
    const SEED: &'static [u8] = b"fund";

    fn get_seeds(&self) -> [&[u8]; 2] {
        [Self::SEED, self.receipt_token_mint.as_ref()]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl FundAccount {
    pub const RESERVE_SEED: &'static [u8] = b"fund_reserve";
    pub const TREASURY_SEED: &'static [u8] = b"fund_treasury";

    pub(super) fn find_account_address(&self) -> Result<Pubkey> {
        Ok(
            Pubkey::create_program_address(&self.get_signer_seeds(), &crate::ID)
                .map_err(|_| ProgramError::InvalidSeeds)?,
        )
    }

    pub(super) fn get_reserve_account_seeds(&self) -> Vec<&[u8]> {
        vec![Self::RESERVE_SEED, self.receipt_token_mint.as_ref()]
    }

    pub(super) fn find_reserve_account_address(&self) -> (Pubkey, u8) {
        Pubkey::find_program_address(&self.get_reserve_account_seeds(), &crate::ID)
    }

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn find_reserve_account_seeds(&self) -> Vec<&[u8]> {
        [
            self.get_reserve_account_seeds(),
            // TODO v0.3/general: leak??
            vec![std::slice::from_ref(Box::leak(Box::new(
                self.find_reserve_account_address().1,
            )))],
        ]
        .concat()
    }

    pub(super) fn find_treasury_account_address(&self) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::TREASURY_SEED, self.receipt_token_mint.as_ref()],
            &crate::ID,
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

    pub(super) fn find_receipt_token_lock_account_address(&self) -> Result<Pubkey> {
        Ok(
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &self.find_account_address()?,
                &self.receipt_token_mint,
                &self.receipt_token_program,
            ),
        )
    }

    pub(super) fn update(
        &mut self,
        bump: u8,
        receipt_token_mint: Pubkey,
        receipt_token_decimals: u8,
        receipt_token_supply: u64,
    ) {
        if self.data_version == 0 {
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.withdrawal.initialize(self.data_version);
            self.data_version = 1;
        }

        if self.data_version == 1 {
            self.withdrawal.initialize(self.data_version);
            self.data_version = 2;
        }

        if self.data_version == 2 {
            self.withdrawal.initialize(self.data_version);
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
            self.operation.initialize(self.data_version);

            self.data_version = 4;
        }
    }

    #[inline(always)]
    pub(super) fn initialize(&mut self, bump: u8, receipt_token_mint: &InterfaceAccount<Mint>) {
        self.update(
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

        let token_info =
            SupportedToken::new(mint, program, decimals, capacity_amount, pricing_source);
        self.supported_tokens.push(token_info);

        Ok(())
    }

    pub(super) fn set_normalized_token(
        &mut self,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        pool: Pubkey,
    ) -> Result<()> {
        if self.normalized_token.is_some() {
            err!(ErrorCode::FundNormalizedTokenAlreadySet)?
        }

        self.normalized_token = Some(NormalizedToken::new(mint, program, decimals, pool));

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
}
