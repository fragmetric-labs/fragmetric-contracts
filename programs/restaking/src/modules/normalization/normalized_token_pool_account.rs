use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token_interface::Mint;

use crate::errors::ErrorCode;
use crate::modules::pricing::{TokenPricingSource, TokenValue};
use crate::utils::PDASeeds;

#[constant]
/// ## Version History
/// * v1: Initial Version
/// * v2: Add `normalized_token_decimals`, .., `one_normalized_token_as_sol` fields
pub const NORMALIZED_TOKEN_POOL_ACCOUNT_CURRENT_VERSION: u16 = 2;

const NORMALIZED_TOKEN_POOL_ACCOUNT_MAX_SUPPORTED_TOKENS_SIZE: usize = 30;

#[account]
#[derive(InitSpace)]
pub struct NormalizedTokenPoolAccount {
    data_version: u16,
    bump: u8,
    pub(crate) normalized_token_mint: Pubkey,
    pub(crate) normalized_token_program: Pubkey,
    #[max_len(NORMALIZED_TOKEN_POOL_ACCOUNT_MAX_SUPPORTED_TOKENS_SIZE)]
    pub(super) supported_tokens: Vec<NormalizedSupportedToken>,

    pub(super) normalized_token_decimals: u8,
    pub(crate) normalized_token_supply_amount: u64,
    pub(super) normalized_token_value: TokenValue,
    pub(super) normalized_token_value_updated_slot: u64,
    pub(super) one_normalized_token_as_sol: u64,

    _reserved: [u8; 128],
}

impl PDASeeds<3> for NormalizedTokenPoolAccount {
    const SEED: &'static [u8] = b"nt_pool";

    fn get_bump(&self) -> u8 {
        self.bump
    }

    fn get_seeds(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.normalized_token_mint.as_ref(),
            std::slice::from_ref(&self.bump),
        ]
    }
}

impl NormalizedTokenPoolAccount {
    pub const MAX_SUPPORTED_TOKENS_SIZE: usize =
        NORMALIZED_TOKEN_POOL_ACCOUNT_MAX_SUPPORTED_TOKENS_SIZE;

    fn migrate(
        &mut self,
        bump: u8,
        normalized_token_mint: Pubkey,
        normalized_token_program: Pubkey,
        normalized_token_decimals: u8,
        normalized_token_supply_amount: u64,
    ) {
        if self.data_version == 0 {
            self.bump = bump;
            self.normalized_token_mint = normalized_token_mint;
            self.normalized_token_program = normalized_token_program;
            self.data_version = 1;
        }
        if self.data_version == 1 {
            self.normalized_token_decimals = normalized_token_decimals;
            self.normalized_token_supply_amount = normalized_token_supply_amount;
            self.normalized_token_value = TokenValue {
                numerator: Vec::new(),
                denominator: 0,
            };
            self.normalized_token_value_updated_slot = 0;
            self.one_normalized_token_as_sol = 0;

            self.data_version = 2;
        }
    }

    #[inline(always)]
    pub(super) fn initialize(&mut self, bump: u8, normalized_token_mint: &InterfaceAccount<Mint>) {
        self.migrate(
            bump,
            normalized_token_mint.key(),
            *AsRef::<AccountInfo>::as_ref(normalized_token_mint).owner,
            normalized_token_mint.decimals,
            normalized_token_mint.supply,
        );
    }

    #[inline(always)]
    pub(super) fn update_if_needed(&mut self, normalized_token_mint: &InterfaceAccount<Mint>) {
        self.initialize(self.bump, normalized_token_mint);
    }

    #[inline(always)]
    pub fn is_latest_version(&self) -> bool {
        self.data_version == NORMALIZED_TOKEN_POOL_ACCOUNT_CURRENT_VERSION
    }

    pub fn find_account_address(normalized_token_mint: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[
                NormalizedTokenPoolAccount::SEED,
                normalized_token_mint.as_ref(),
            ],
            &crate::ID,
        )
        .0
    }

    pub fn find_supported_token_reserve_account_address(
        &self,
        supported_token_mint: &Pubkey,
    ) -> Result<Pubkey> {
        let supported_token = self.get_supported_token(supported_token_mint)?;
        Ok(supported_token.find_reserve_account_address(&self.normalized_token_mint))
    }

    pub(super) fn add_new_supported_token(
        &mut self,
        supported_token_mint: Pubkey,
        supported_token_program: Pubkey,
        supported_token_decimals: u8,
        supported_token_reserve_account: Pubkey,
        supported_token_pricing_source: TokenPricingSource,
    ) -> Result<()> {
        if self.has_supported_token(&supported_token_mint) {
            err!(ErrorCode::NormalizedTokenPoolAlreadySupportedTokenError)?
        }

        require_gt!(
            NORMALIZED_TOKEN_POOL_ACCOUNT_MAX_SUPPORTED_TOKENS_SIZE,
            self.supported_tokens.len(),
            ErrorCode::NormalizedTokenPoolExceededMaxSupportedTokensError
        );

        self.supported_tokens.push(NormalizedSupportedToken::new(
            supported_token_mint,
            supported_token_program,
            supported_token_decimals,
            supported_token_reserve_account,
            supported_token_pricing_source,
        ));

        Ok(())
    }

    pub(crate) fn get_supported_token(
        &self,
        supported_token_mint: &Pubkey,
    ) -> Result<&NormalizedSupportedToken> {
        self.supported_tokens
            .iter()
            .find(|token| token.mint == *supported_token_mint)
            .ok_or_else(|| error!(ErrorCode::NormalizedTokenPoolNotSupportedTokenError))
    }

    pub(super) fn get_supported_token_mut(
        &mut self,
        supported_token_mint: &Pubkey,
    ) -> Result<&mut NormalizedSupportedToken> {
        self.supported_tokens
            .iter_mut()
            .find(|token| token.mint == *supported_token_mint)
            .ok_or_else(|| error!(ErrorCode::NormalizedTokenPoolNotSupportedTokenError))
    }

    pub fn has_supported_token(&self, supported_token_mint: &Pubkey) -> bool {
        self.supported_tokens
            .iter()
            .any(|token| token.mint == *supported_token_mint)
    }

    pub(super) fn reload_normalized_token_supply(
        &mut self,
        normalized_token_mint: &mut InterfaceAccount<Mint>,
    ) -> Result<()> {
        require_keys_eq!(self.normalized_token_mint, normalized_token_mint.key());

        normalized_token_mint.reload()?;
        self.normalized_token_supply_amount = normalized_token_mint.supply;

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub(crate) struct NormalizedSupportedToken {
    pub mint: Pubkey,
    pub program: Pubkey,
    pub reserve_account: Pubkey,
    pub locked_amount: u64,
    pub decimals: u8,
    pub withdrawal_reserved_amount: u64,
    pub one_token_as_sol: u64,
    pub pricing_source: TokenPricingSource,
    _reserved: [u8; 14],
}

impl NormalizedSupportedToken {
    fn new(
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        reserve_account: Pubkey,
        pricing_source: TokenPricingSource,
    ) -> Self {
        Self {
            mint,
            program,
            decimals,
            reserve_account,
            locked_amount: 0,
            withdrawal_reserved_amount: 0,
            one_token_as_sol: 0,
            pricing_source,
            _reserved: [0; 14],
        }
    }

    pub(super) fn find_reserve_account_address(&self, normalized_token_mint: &Pubkey) -> Pubkey {
        spl_associated_token_account::get_associated_token_address_with_program_id(
            &NormalizedTokenPoolAccount::find_account_address(normalized_token_mint),
            &self.mint,
            &self.program,
        )
    }

    pub(super) fn lock_token(&mut self, token_amount: u64) -> Result<()> {
        self.locked_amount = self
            .locked_amount
            .checked_add(token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    pub(super) fn unlock_token(&mut self, token_amount: u64) -> Result<()> {
        self.locked_amount = self
            .locked_amount
            .checked_sub(token_amount)
            .ok_or_else(|| {
                error!(ErrorCode::NormalizedTokenPoolNotEnoughSupportedTokenException)
            })?;

        Ok(())
    }

    pub(super) fn allocate_locked_token_to_withdrawal_reserved(
        &mut self,
        token_amount: u64,
    ) -> Result<()> {
        self.locked_amount = self
            .locked_amount
            .checked_sub(token_amount)
            .ok_or_else(|| {
                error!(ErrorCode::NormalizedTokenPoolNotEnoughSupportedTokenException)
            })?;
        self.withdrawal_reserved_amount += token_amount;

        Ok(())
    }

    pub(super) fn settle_withdrawal_reserved_token(&mut self, token_amount: u64) -> Result<()> {
        self.withdrawal_reserved_amount = self
            .withdrawal_reserved_amount
            .checked_sub(token_amount)
            .ok_or_else(|| {
                error!(ErrorCode::NormalizedTokenPoolNotEnoughSupportedTokenException)
            })?;

        Ok(())
    }
}
