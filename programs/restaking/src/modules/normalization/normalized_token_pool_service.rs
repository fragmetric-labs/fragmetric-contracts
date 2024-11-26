use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::modules::pricing::{PricingService, TokenPricingSource};
use crate::utils::PDASeeds;

use super::*;

pub struct NormalizedTokenPoolService<'info: 'a, 'a> {
    normalized_token_pool_account: &'a mut Account<'info, NormalizedTokenPoolAccount>,
    normalized_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    normalized_token_program: &'a Program<'info, Token>,
    current_timestamp: i64,
}

impl Drop for NormalizedTokenPoolService<'_, '_> {
    fn drop(&mut self) {
        self.normalized_token_pool_account.exit(&crate::ID).unwrap();
    }
}

impl<'info, 'a> NormalizedTokenPoolService<'info, 'a> {
    pub fn new(
        normalized_token_pool_account: &'a mut Account<'info, NormalizedTokenPoolAccount>,
        normalized_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        normalized_token_program: &'a Program<'info, Token>,
    ) -> Result<Self> {
        require!(
            normalized_token_pool_account.is_latest_version(),
            ErrorCode::InvalidDataVersionError
        );
        require_keys_eq!(
            normalized_token_pool_account.normalized_token_mint,
            normalized_token_mint.key(),
        );
        require_keys_eq!(
            normalized_token_pool_account.normalized_token_program,
            normalized_token_program.key(),
        );

        let clock = Clock::get()?;
        Ok(Self {
            normalized_token_pool_account,
            normalized_token_mint,
            normalized_token_program,
            current_timestamp: clock.unix_timestamp,
        })
    }

    pub(in crate::modules) fn normalize_supported_token(
        &mut self,
        to_normalized_token_account: &InterfaceAccount<'info, TokenAccount>,
        from_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        pool_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        supported_token_mint: &InterfaceAccount<'info, Mint>,
        supported_token_program: &Interface<'info, TokenInterface>,

        signer: &AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],

        supported_token_amount: u64,

        pricing_service: &mut PricingService,
    ) -> Result<()> {
        require_keys_eq!(
            pool_supported_token_account.key(),
            self.normalized_token_pool_account
                .get_supported_token(supported_token_mint.key())?
                .lock_account
        );
        require_gt!(supported_token_amount, 0);

        let supported_token_amount_as_sol = pricing_service
            .get_token_amount_as_sol(&supported_token_mint.key(), supported_token_amount)?;
        let normalized_token_amount = pricing_service.get_sol_amount_as_token(
            &self.normalized_token_mint.key(),
            supported_token_amount_as_sol,
        )?;

        self.normalized_token_pool_account
            .get_supported_token_mut(supported_token_mint.key())?
            .lock_token(supported_token_amount)?;

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                supported_token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: from_supported_token_account.to_account_info(),
                    mint: supported_token_mint.to_account_info(),
                    to: pool_supported_token_account.to_account_info(),
                    authority: signer.to_account_info(),
                },
                signer_seeds,
            ),
            supported_token_amount,
            supported_token_mint.decimals,
        )?;

        anchor_spl::token_interface::mint_to(
            CpiContext::new_with_signer(
                self.normalized_token_program.to_account_info(),
                anchor_spl::token_interface::MintTo {
                    mint: self.normalized_token_mint.to_account_info(),
                    to: to_normalized_token_account.to_account_info(),
                    authority: self.normalized_token_pool_account.to_account_info(),
                },
                &[self
                    .normalized_token_pool_account
                    .get_seeds()
                    .as_ref()],
            ),
            normalized_token_amount,
        )?;

        self.normalized_token_pool_account
            .reload_normalized_token_supply(self.normalized_token_mint)?;

        self.update_asset_values(pricing_service)?;

        Ok(())
    }

    pub(in crate::modules) fn denormalize_supported_token(
        &mut self,
        from_normalized_token_account: &InterfaceAccount<'info, TokenAccount>,
        to_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        pool_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        supported_token_mint: &InterfaceAccount<'info, Mint>,
        supported_token_program: &Interface<'info, TokenInterface>,

        signer: AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],

        normalized_token_amount: u64,

        pricing_service: &mut PricingService,
    ) -> Result<()> {
        require_keys_eq!(
            pool_supported_token_account.key(),
            self.normalized_token_pool_account
                .get_supported_token(supported_token_mint.key())?
                .lock_account
        );
        require_gt!(normalized_token_amount, 0);

        let normalized_token_amount_as_sol = pricing_service
            .get_token_amount_as_sol(&self.normalized_token_mint.key(), normalized_token_amount)?;
        let supported_token_amount = pricing_service
            .get_sol_amount_as_token(&supported_token_mint.key(), normalized_token_amount_as_sol)?;

        self.normalized_token_pool_account
            .get_supported_token_mut(supported_token_mint.key())?
            .unlock_token(supported_token_amount)?;

        anchor_spl::token_interface::burn(
            CpiContext::new_with_signer(
                self.normalized_token_program.to_account_info(),
                anchor_spl::token_interface::Burn {
                    mint: self.normalized_token_mint.to_account_info(),
                    from: from_normalized_token_account.to_account_info(),
                    authority: signer.to_account_info(),
                },
                signer_seeds,
            ),
            normalized_token_amount,
        )?;

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                supported_token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: pool_supported_token_account.to_account_info(),
                    mint: supported_token_mint.to_account_info(),
                    to: to_supported_token_account.to_account_info(),
                    authority: self.normalized_token_pool_account.to_account_info(),
                },
                &[self
                    .normalized_token_pool_account
                    .get_seeds()
                    .as_ref()],
            ),
            supported_token_amount,
            supported_token_mint.decimals,
        )?;

        self.normalized_token_pool_account
            .reload_normalized_token_supply(self.normalized_token_mint)?;

        self.update_asset_values(pricing_service)?;

        Ok(())
    }

    fn update_asset_values(&mut self, pricing_service: &mut PricingService) -> Result<()> {
        // ensure any update on fund account written before do pricing
        self.normalized_token_pool_account.exit(&crate::ID)?;

        // update pool asset values
        let normalized_token_mint_key = &self.normalized_token_mint.key();
        pricing_service.resolve_token_pricing_source(
            normalized_token_mint_key,
            &TokenPricingSource::FragmetricNormalizedTokenPool {
                address: self.normalized_token_pool_account.key(),
            },
        )?;

        // the values being written below are informative, only for event emission.
        self.normalized_token_pool_account
            .one_normalized_token_as_sol = pricing_service.get_token_amount_as_sol(
            normalized_token_mint_key,
            10u64
                .checked_pow(self.normalized_token_mint.decimals as u32)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
        )?;

        self.normalized_token_pool_account.normalized_token_value =
            pricing_service.get_token_total_value_as_atomic(normalized_token_mint_key)?;

        self.normalized_token_pool_account
            .normalized_token_value_updated_at = self.current_timestamp;

        Ok(())
    }
}
