use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::utils::PDASeeds;

use super::NORMALIZED_TOKEN_POOL_ACCOUNT_MAX_SUPPORTED_TOKENS_SIZE;

#[constant]
/// ## Version History
/// * v1: Initial Version
pub const NORMALIZED_TOKEN_WITHDRAWAL_ACCOUNT_CURRENT_VERSION: u16 = 1;

#[account]
#[derive(InitSpace)]
pub struct NormalizedTokenWithdrawalAccount {
    data_version: u16,
    bump: u8,
    pub withdrawal_authority: Pubkey,
    pub normalized_token_mint: Pubkey,
    pub(super) normalized_token_pool: Pubkey,
    pub(super) normalized_token_amount: u64,
    #[max_len(NORMALIZED_TOKEN_POOL_ACCOUNT_MAX_SUPPORTED_TOKENS_SIZE)]
    pub(super) claimable_tokens: Vec<NormalizedClaimableToken>,
    pub(super) created_at: i64,
    _reserved: [u8; 32],
}

impl PDASeeds<4> for NormalizedTokenWithdrawalAccount {
    const SEED: &'static [u8] = b"nt_withdrawal";

    fn get_bump(&self) -> u8 {
        self.bump
    }

    fn get_seeds(&self) -> [&[u8]; 4] {
        [
            Self::SEED,
            self.normalized_token_mint.as_ref(),
            self.withdrawal_authority.as_ref(),
            std::slice::from_ref(&self.bump),
        ]
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
        claimable_tokens: Vec<NormalizedClaimableToken>,
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
    ) -> Result<&mut NormalizedClaimableToken> {
        self.claimable_tokens
            .iter_mut()
            .find(|token| token.mint == *supported_token_mint && !token.claimed)
            .ok_or_else(|| error!(ErrorCode::NormalizedTokenPoolNonClaimableTokenError))
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub(super) struct NormalizedClaimableToken {
    pub mint: Pubkey,
    pub program: Pubkey,
    pub claimable_amount: u64,
    pub claimed: bool,
}

impl NormalizedClaimableToken {
    pub(super) fn new(mint: Pubkey, program: Pubkey, claimable_amount: u64) -> Self {
        Self {
            mint,
            program,
            claimable_amount,
            claimed: false,
        }
    }

    pub(super) fn settle(&mut self) -> Result<()> {
        require!(
            !self.claimed,
            ErrorCode::NormalizedTokenPoolNonClaimableTokenError
        );
        self.claimed = true;
        Ok(())
    }
}
