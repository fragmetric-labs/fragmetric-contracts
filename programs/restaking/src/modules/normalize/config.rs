use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::utils::PDASeeds;

const MAX_SUPPORTED_TOKENS: usize = 10;

#[account]
#[derive(InitSpace)]
pub struct NormalizedTokenPoolConfig {
    data_version: u16,
    bump: u8,
    pub normalized_token_mint: Pubkey,
    normalized_token_program: Pubkey,
    pub normalized_token_authority: Pubkey,
    /// buffer account
    normalized_token_account: Pubkey,
    #[max_len(MAX_SUPPORTED_TOKENS)]
    supported_tokens: Vec<SupportedTokenConfig>,
}

impl PDASeeds<1> for NormalizedTokenPoolConfig {
    const SEED: &'static [u8] = b"normalized_token_pool_config";

    fn get_seeds(&self) -> [&[u8]; 1] {
        [self.normalized_token_mint.as_ref()]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl NormalizedTokenPoolConfig {
    pub(super) fn initialize(
        &mut self,
        bump: u8,
        normalized_token_mint: Pubkey,
        normalized_token_program: Pubkey,
        normalized_token_authority: Pubkey,
        normalized_token_account: Pubkey,
    ) {
        self.bump = bump;
        self.normalized_token_mint = normalized_token_mint;
        self.normalized_token_program = normalized_token_program;
        self.normalized_token_authority = normalized_token_authority;
        self.normalized_token_account = normalized_token_account;
    }

    pub(super) fn add_supported_token(
        &mut self,
        supported_token_mint: Pubkey,
        supported_token_program: Pubkey,
        supported_token_account: Pubkey,
        supported_token_authority: Pubkey,
        supported_token_lock_account: Pubkey,
    ) -> Result<()> {
        require_gt!(
            MAX_SUPPORTED_TOKENS,
            self.supported_tokens.len(),
            ErrorCode::NormalizeExceededMaxSupportedTokensError
        );

        self.supported_tokens.push(SupportedTokenConfig::new(
            supported_token_mint,
            supported_token_program,
            supported_token_account,
            supported_token_authority,
            supported_token_lock_account,
        ));

        Ok(())
    }

    pub(in crate::modules) fn get_supported_tokens_locked_amount(&self) -> Vec<(Pubkey, u64)> {
        self.supported_tokens
            .iter()
            .map(|config| (config.mint, config.locked_amount))
            .collect()
    }

    fn get_supported_token_config(
        &self,
        supported_token_mint: Pubkey,
    ) -> Result<&SupportedTokenConfig> {
        self.supported_tokens
            .iter()
            .find(|config| config.mint == supported_token_mint)
            .ok_or_else(|| error!(ErrorCode::NormalizeNotSupportedTokenError))
    }

    pub(super) fn get_supported_token_config_mut(
        &mut self,
        supported_token_mint: Pubkey,
    ) -> Result<&mut SupportedTokenConfig> {
        self.supported_tokens
            .iter_mut()
            .find(|config| config.mint == supported_token_mint)
            .ok_or_else(|| error!(ErrorCode::NormalizeNotSupportedTokenError))
    }

    pub(super) fn validate_adapter_constructor_accounts(
        &self,
        accounts: &[AccountInfo],
    ) -> Result<()> {
        require_gte!(
            accounts.len(),
            4,
            ErrorCode::NormalizeInvalidAccountsProvided,
        );
        require_eq!(
            self.normalized_token_mint,
            accounts[0].key(),
            ErrorCode::NormalizeInvalidAccountsProvided,
        );
        require_eq!(
            self.normalized_token_program,
            accounts[1].key(),
            ErrorCode::NormalizeInvalidAccountsProvided,
        );
        require_eq!(
            self.normalized_token_authority,
            accounts[2].key(),
            ErrorCode::NormalizeInvalidAccountsProvided,
        );
        require_eq!(
            self.normalized_token_account,
            accounts[3].key(),
            ErrorCode::NormalizeInvalidAccountsProvided,
        );

        self.get_supported_token_config(accounts[4].key())?
            .validate_adapter_constructor_accounts(&accounts[4..])
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct SupportedTokenConfig {
    mint: Pubkey,
    program: Pubkey,
    token_account: Pubkey,
    token_authority: Pubkey,
    /// lock_authority = normalized_token_authority
    lock_account: Pubkey,
    locked_amount: u64,
}

impl SupportedTokenConfig {
    fn new(
        mint: Pubkey,
        program: Pubkey,
        token_account: Pubkey,
        token_authority: Pubkey,
        lock_account: Pubkey,
    ) -> Self {
        Self {
            mint,
            program,
            token_account,
            token_authority,
            lock_account,
            locked_amount: 0,
        }
    }

    fn validate_adapter_constructor_accounts(&self, accounts: &[AccountInfo]) -> Result<()> {
        require_eq!(
            accounts.len(),
            5,
            ErrorCode::NormalizeInvalidAccountsProvided,
        );
        require_eq!(
            self.mint,
            accounts[0].key(),
            ErrorCode::NormalizeInvalidAccountsProvided,
        );
        require_eq!(
            self.program,
            accounts[1].key(),
            ErrorCode::NormalizeInvalidAccountsProvided,
        );
        require_eq!(
            self.token_account,
            accounts[2].key(),
            ErrorCode::NormalizeInvalidAccountsProvided,
        );
        require_eq!(
            self.token_authority,
            accounts[3].key(),
            ErrorCode::NormalizeInvalidAccountsProvided,
        );
        require_eq!(
            self.lock_account,
            accounts[4].key(),
            ErrorCode::NormalizeInvalidAccountsProvided,
        );

        Ok(())
    }

    pub(super) fn lock_token(&mut self, token_amount: u64) -> Result<()> {
        self.locked_amount = self
            .locked_amount
            .checked_add(token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}
