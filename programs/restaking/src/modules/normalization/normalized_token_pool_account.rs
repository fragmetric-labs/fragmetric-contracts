use anchor_lang::prelude::*;
use std::slice::Iter;

use crate::errors::ErrorCode;
use crate::utils::PDASeeds;

const MAX_SUPPORTED_TOKENS: usize = 10;

#[account]
#[derive(InitSpace)]

pub struct NormalizedTokenPoolAccount {
    data_version: u16,
    bump: u8,
    pub normalized_token_mint: Pubkey,
    normalized_token_program: Pubkey,
    #[max_len(MAX_SUPPORTED_TOKENS)]
    pub(super) supported_tokens: Vec<SupportedToken>,
    _reserved: [u8; 128],
}

impl PDASeeds<2> for NormalizedTokenPoolAccount {
    const SEED: &'static [u8] = b"nt_pool";

    fn get_seeds(&self) -> [&[u8]; 2] {
        [Self::SEED, self.normalized_token_mint.as_ref()]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl NormalizedTokenPoolAccount {
    pub(super) fn initialize(
        &mut self,
        bump: u8,
        normalized_token_mint: Pubkey,
        normalized_token_program: Pubkey,
    ) {
        if self.data_version == 0 {
            self.bump = bump;
            self.normalized_token_mint = normalized_token_mint;
            self.normalized_token_program = normalized_token_program;
            self.data_version = 1;
        }
    }

    pub(super) fn add_new_supported_token(
        &mut self,
        supported_token_mint: Pubkey,
        supported_token_program: Pubkey,
        supported_token_lock_account: Pubkey,
    ) -> Result<()> {
        if self
            .supported_tokens
            .iter()
            .any(|token| token.mint == supported_token_mint)
        {
            err!(ErrorCode::NormalizedTokenPoolAlreadySupportedTokenError)?
        }

        require_gt!(
            MAX_SUPPORTED_TOKENS,
            self.supported_tokens.len(),
            ErrorCode::NormalizedTokenPoolExceededMaxSupportedTokensError
        );

        self.supported_tokens.push(SupportedToken::new(
            supported_token_mint,
            supported_token_program,
            supported_token_lock_account,
        ));

        Ok(())
    }

    pub(super) fn get_supported_tokens_locked_amount(&self) -> Vec<(Pubkey, u64)> {
        self.supported_tokens
            .iter()
            .map(|token| (token.mint, token.locked_amount))
            .collect()
    }

    pub(super) fn get_supported_token(
        &self,
        supported_token_mint: Pubkey,
    ) -> Result<&SupportedToken> {
        self.supported_tokens
            .iter()
            .find(|token| token.mint == supported_token_mint)
            .ok_or_else(|| error!(ErrorCode::NormalizedTokenPoolNotSupportedTokenError))
    }

    pub(super) fn get_supported_token_mut(
        &mut self,
        supported_token_mint: Pubkey,
    ) -> Result<&mut SupportedToken> {
        self.supported_tokens
            .iter_mut()
            .find(|token| token.mint == supported_token_mint)
            .ok_or_else(|| error!(ErrorCode::NormalizedTokenPoolNotSupportedTokenError))
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub(super) struct SupportedToken {
    mint: Pubkey,
    program: Pubkey,
    lock_account: Pubkey,
    locked_amount: u64,
    _reserved: [u8; 64],
}

impl SupportedToken {
    fn new(mint: Pubkey, program: Pubkey, lock_account: Pubkey) -> Self {
        Self {
            mint,
            program,
            lock_account,
            locked_amount: 0,
            _reserved: [0; 64],
        }
    }

    pub(super) fn get_mint(&self) -> Pubkey {
        self.mint
    }

    pub(super) fn get_locked_amount(&self) -> u64 {
        self.locked_amount
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
            .ok_or_else(|| error!(ErrorCode::NormalizedTokenPoolNotEnoughLockedToken))?;

        Ok(())
    }
}
