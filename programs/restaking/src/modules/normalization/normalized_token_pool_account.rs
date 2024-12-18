use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::modules::normalization::{ClaimableToken, NormalizedTokenWithdrawalAccount};
use crate::modules::pricing::{PricingService, TokenPricingSource, TokenValue};
use crate::utils::PDASeeds;

#[constant]
/// ## Version History
/// * v1: Initial Version
/// * v2: Add `normalized_token_decimals`, .., `one_normalized_token_as_sol` fields
pub const NORMALIZED_TOKEN_POOL_ACCOUNT_CURRENT_VERSION: u16 = 2;

pub(super) const MAX_SUPPORTED_TOKENS_SIZE: usize = 10;

#[account]
#[derive(InitSpace)]
pub struct NormalizedTokenPoolAccount {
    data_version: u16,
    bump: u8,
    pub normalized_token_mint: Pubkey,
    pub(super) normalized_token_program: Pubkey,
    #[max_len(MAX_SUPPORTED_TOKENS_SIZE)]
    pub(super) supported_tokens: Vec<SupportedToken>,

    pub(super) normalized_token_decimals: u8,
    pub(super) normalized_token_supply_amount: u64,
    pub(super) normalized_token_value: TokenValue,
    pub(super) normalized_token_value_updated_slot: u64,
    pub(super) one_normalized_token_as_sol: u64,

    _reserved: [u8; 128],
}

impl PDASeeds<2> for NormalizedTokenPoolAccount {
    const SEED: &'static [u8] = b"nt_pool";

    fn get_seed_phrase(&self) -> [&[u8]; 2] {
        [Self::SEED, self.normalized_token_mint.as_ref()]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl NormalizedTokenPoolAccount {
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
            self.supported_tokens
                .iter_mut()
                .for_each(|supported_token| {
                    supported_token.decimals = 9;
                    supported_token.withdrawal_reserved_amount = 0;

                    supported_token.pricing_source = match supported_token.mint {
                        MAINNET_BSOL_MINT_ADDRESS => TokenPricingSource::SPLStakePool {
                            address: MAINNET_BSOL_STAKE_POOL_ADDRESS,
                        },
                        MAINNET_MSOL_MINT_ADDRESS => TokenPricingSource::MarinadeStakePool {
                            address: MAINNET_MSOL_STAKE_POOL_ADDRESS,
                        },
                        MAINNET_JITOSOL_MINT_ADDRESS => TokenPricingSource::SPLStakePool {
                            address: MAINNET_JITOSOL_STAKE_POOL_ADDRESS,
                        },
                        MAINNET_BNSOL_MINT_ADDRESS => TokenPricingSource::SPLStakePool {
                            address: MAINNET_BNSOL_STAKE_POOL_ADDRESS,
                        },
                        #[allow(unreachable_patterns)]
                        DEVNET_BSOL_MINT_ADDRESS => TokenPricingSource::SPLStakePool {
                            address: DEVNET_BSOL_STAKE_POOL_ADDRESS,
                        },
                        #[allow(unreachable_patterns)]
                        DEVNET_MSOL_MINT_ADDRESS => TokenPricingSource::MarinadeStakePool {
                            address: DEVNET_MSOL_STAKE_POOL_ADDRESS,
                        },
                        _ => panic!("normalized token pool pricing source migration failed"),
                    }
                });

            // Later we will remove bSOL(and ATA) - via remove supported token instruction
            // // Mainnet BSOL is removed HERE
            // #[cfg(feature = "mainnet")]
            // {
            //     self.supported_tokens = std::mem::take(&mut self.supported_tokens)
            //         .into_iter()
            //         .filter(|supported_token| {
            //             supported_token.mint != MAINNET_BSOL_MINT_ADDRESS || {
            //                 assert_eq!(supported_token.locked_amount, 0);
            //                 false
            //             }
            //         })
            //         .collect();
            // }

            self.data_version = 2;
        }
    }

    // TODO: remove?
    pub(in crate::modules) fn has_supported_token(&self, token: &Pubkey) -> bool {
        let supported_token_mint_list: Vec<&Pubkey> = self
            .supported_tokens
            .iter()
            .map(|token| &token.mint)
            .collect();
        supported_token_mint_list.contains(&token)
    }

    #[inline(always)]
    pub(super) fn initialize(&mut self, bump: u8, normalized_token_mint: &InterfaceAccount<Mint>) {
        self.migrate(
            bump,
            normalized_token_mint.key(),
            *normalized_token_mint.to_account_info().owner,
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

    pub fn find_account_address_by_token_mint(normalized_token_mint: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[
                NormalizedTokenPoolAccount::SEED,
                normalized_token_mint.to_bytes().as_ref(),
            ],
            &crate::ID,
        )
        .0
    }

    pub(super) fn add_new_supported_token(
        &mut self,
        supported_token_mint: Pubkey,
        supported_token_program: Pubkey,
        supported_token_decimals: u8,
        supported_token_lock_account: Pubkey,
        supported_token_pricing_source: TokenPricingSource,
    ) -> Result<()> {
        if self
            .supported_tokens
            .iter()
            .any(|token| token.mint == supported_token_mint)
        {
            err!(ErrorCode::NormalizedTokenPoolAlreadySupportedTokenError)?
        }

        require_gt!(
            MAX_SUPPORTED_TOKENS_SIZE,
            self.supported_tokens.len(),
            ErrorCode::NormalizedTokenPoolExceededMaxSupportedTokensError
        );

        self.supported_tokens.push(SupportedToken::new(
            supported_token_mint,
            supported_token_program,
            supported_token_decimals,
            supported_token_lock_account,
            supported_token_pricing_source,
        ));

        Ok(())
    }

    #[inline]
    pub(super) fn get_supported_tokens_iter(&self) -> impl Iterator<Item = &SupportedToken> {
        self.supported_tokens.iter()
    }

    #[inline]
    pub(super) fn get_supported_tokens_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut SupportedToken> {
        self.supported_tokens.iter_mut()
    }

    pub(super) fn get_supported_token(
        &self,
        supported_token_mint: &Pubkey,
    ) -> Result<&SupportedToken> {
        self.get_supported_tokens_iter()
            .find(|token| token.mint == *supported_token_mint)
            .ok_or_else(|| error!(ErrorCode::NormalizedTokenPoolNotSupportedTokenError))
    }

    pub(super) fn get_supported_token_mut(
        &mut self,
        supported_token_mint: &Pubkey,
    ) -> Result<&mut SupportedToken> {
        self.get_supported_tokens_iter_mut()
            .find(|token| token.mint == *supported_token_mint)
            .ok_or_else(|| error!(ErrorCode::NormalizedTokenPoolNotSupportedTokenError))
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
pub(super) struct SupportedToken {
    pub mint: Pubkey,
    pub program: Pubkey,
    pub decimals: u8,
    pub lock_account: Pubkey,
    pub locked_amount: u64,
    pub withdrawal_reserved_amount: u64,
    pub one_token_as_sol: u64,
    pub pricing_source: TokenPricingSource,
    _reserved: [u8; 46],
}

impl SupportedToken {
    fn new(
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        lock_account: Pubkey,
        pricing_source: TokenPricingSource,
    ) -> Self {
        Self {
            mint,
            program,
            decimals,
            lock_account,
            locked_amount: 0,
            withdrawal_reserved_amount: 0,
            one_token_as_sol: 0,
            pricing_source,
            _reserved: [0; 46],
        }
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
