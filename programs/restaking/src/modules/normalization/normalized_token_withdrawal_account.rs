use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::errors::ErrorCode;
use crate::utils::PDASeeds;

use super::MAX_SUPPORTED_TOKENS;

#[constant]
/// ## Version History
/// * v1: Initial Version
pub const NORMALIZED_TOKEN_WITHDRAWAL_ACCOUNT_CURRENT_VERSION: u16 = 1;

#[account]
#[derive(InitSpace)]
pub struct NormalizedTokenWithdrawalAccount {
    data_version: u16,
    bump: u8,
    pub(super) withdrawal_authority: Pubkey,
    pub normalized_token_mint: Pubkey,
    pub(super) normalized_token_pool: Pubkey,
    normalized_token_amount: u64,
    #[max_len(MAX_SUPPORTED_TOKENS)]
    claimable_tokens: Vec<ClaimableToken>,
    created_at: i64,
    _reserved: [u8; 32],
}

impl PDASeeds<3> for NormalizedTokenWithdrawalAccount {
    const SEED: &'static [u8] = b"nt_withdrawal";

    fn get_seed_phrase(&self) -> [&[u8]; 3] {
        [
            Self::SEED,
            self.normalized_token_mint.as_ref(),
            self.withdrawal_authority.as_ref(),
        ]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl NormalizedTokenWithdrawalAccount {
    fn migrate(
        &mut self,
        bump: u8,
        withdrawal_authority: Pubkey,
        normalized_token_mint: Pubkey,
        normalized_token_pool: Pubkey,
    ) {
        if self.data_version == 0 {
            self.bump = bump;
            self.withdrawal_authority = withdrawal_authority;
            self.normalized_token_mint = normalized_token_mint;
            self.normalized_token_pool = normalized_token_pool;
            self.normalized_token_amount = 0;
            self.claimable_tokens = Vec::new();
            self.created_at = 0;
            self._reserved = Default::default();
            self.data_version = 1;
        }
    }

    #[inline(always)]
    pub(super) fn initialize(
        &mut self,
        bump: u8,
        withdrawal_authority: Pubkey,
        normalized_token_mint: Pubkey,
        normalized_token_pool: Pubkey,
    ) {
        self.migrate(
            bump,
            withdrawal_authority,
            normalized_token_mint,
            normalized_token_pool,
        )
    }

    #[inline(always)]
    pub(super) fn update_if_needed(&mut self) {
        self.migrate(
            self.bump,
            self.withdrawal_authority,
            self.normalized_token_mint,
            self.normalized_token_pool,
        )
    }

    #[inline(always)]
    pub fn is_latest_version(&self) -> bool {
        self.data_version == NORMALIZED_TOKEN_WITHDRAWAL_ACCOUNT_CURRENT_VERSION
    }

    pub(super) fn set_claimable_tokens(
        &mut self,
        normalized_token_amount: u64,
        claimable_tokens: Vec<ClaimableToken>,
        current_timestamp: i64,
    ) -> Result<()> {
        require_eq!(
            self.normalized_token_amount,
            0,
            ErrorCode::NormalizedTokenPoolAlreadySettledWithdrawalAccountError
        );
        require_eq!(
            claimable_tokens
                .iter()
                .all(|supported_token| !supported_token.claimed
                    && supported_token.claimable_amount > 0),
            true
        );

        self.normalized_token_amount = normalized_token_amount;
        self.claimable_tokens = claimable_tokens;
        self.created_at = current_timestamp;
        Ok(())
    }

    pub(super) fn is_settled(&self) -> bool {
        self.claimable_tokens
            .iter()
            .all(|supported_token| supported_token.claimed)
    }

    pub(super) fn get_claimable_token_mut(
        &mut self,
        supported_token_mint: &Pubkey,
    ) -> Result<&mut ClaimableToken> {
        self.claimable_tokens
            .iter_mut()
            .find(|token| token.mint == *supported_token_mint && !token.claimed)
            .ok_or_else(|| error!(ErrorCode::NormalizedTokenPoolUnclaimableTokenError))
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub(super) struct ClaimableToken {
    mint: Pubkey,
    program: Pubkey,
    pub claimable_amount: u64,
    claimed: bool,
}

impl ClaimableToken {
    pub fn new(mint: Pubkey, program: Pubkey, claimable_amount: u64) -> Self {
        Self {
            mint,
            program,
            claimable_amount,
            claimed: false,
        }
    }

    pub fn settle(&mut self) -> Result<()> {
        require!(
            !self.claimed,
            ErrorCode::NormalizedTokenPoolUnclaimableTokenError
        );
        self.claimed = true;
        Ok(())
    }
}
