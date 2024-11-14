use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::modules::pricing;
use crate::modules::pricing::PricingService;
use crate::utils::PDASeeds;

use super::*;

pub struct NormalizedTokenPoolService<'info, 'a>
where
    'info: 'a,
{
    normalized_token_pool_account: &'a mut Account<'info, NormalizedTokenPoolAccount>,
    normalized_token_mint: &'a InterfaceAccount<'info, Mint>,
    normalized_token_program: &'a Program<'info, Token>,
}

impl Drop for NormalizedTokenPoolService<'_, '_> {
    fn drop(&mut self) {
        self.normalized_token_pool_account.exit(&crate::ID).unwrap();
    }
}

impl<'info, 'a> NormalizedTokenPoolService<'info, 'a> {
    pub fn new(
        normalized_token_pool_account: &'a mut Account<'info, NormalizedTokenPoolAccount>,
        normalized_token_mint: &'a InterfaceAccount<'info, Mint>,
        normalized_token_program: &'a Program<'info, Token>,
    ) -> Result<Self> {
        Ok(Self {
            normalized_token_pool_account,
            normalized_token_mint,
            normalized_token_program,
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

        pricing_service: &PricingService,
    ) -> Result<()> {
        require_keys_eq!(
            pool_supported_token_account.owner,
            self.normalized_token_pool_account.key()
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

        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                supported_token_program.to_account_info(),
                token_interface::TransferChecked {
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

        token_interface::mint_to(
            CpiContext::new_with_signer(
                self.normalized_token_program.to_account_info(),
                token_interface::MintTo {
                    mint: self.normalized_token_mint.to_account_info(),
                    to: to_normalized_token_account.to_account_info(),
                    authority: self.normalized_token_pool_account.to_account_info(),
                },
                &[self
                    .normalized_token_pool_account
                    .get_signer_seeds()
                    .as_ref()],
            ),
            normalized_token_amount,
        )?;

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

        pricing_service: &PricingService,
    ) -> Result<()> {
        require_keys_eq!(
            pool_supported_token_account.owner,
            self.normalized_token_pool_account.key()
        );
        require_gt!(normalized_token_amount, 0);

        let normalized_token_amount_as_sol = pricing_service
            .get_token_amount_as_sol(&self.normalized_token_mint.key(), normalized_token_amount)?;
        let supported_token_amount = pricing_service
            .get_sol_amount_as_token(&supported_token_mint.key(), normalized_token_amount_as_sol)?;

        self.normalized_token_pool_account
            .get_supported_token_mut(supported_token_mint.key())?
            .unlock_token(supported_token_amount)?;

        token_interface::burn(
            CpiContext::new_with_signer(
                self.normalized_token_program.to_account_info(),
                token_interface::Burn {
                    mint: self.normalized_token_mint.to_account_info(),
                    from: from_normalized_token_account.to_account_info(),
                    authority: signer.to_account_info(),
                },
                signer_seeds,
            ),
            normalized_token_amount,
        )?;

        token_interface::transfer_checked(
            CpiContext::new_with_signer(
                supported_token_program.to_account_info(),
                token_interface::TransferChecked {
                    from: pool_supported_token_account.to_account_info(),
                    mint: supported_token_mint.to_account_info(),
                    to: to_supported_token_account.to_account_info(),
                    authority: self.normalized_token_pool_account.to_account_info(),
                },
                &[self
                    .normalized_token_pool_account
                    .get_signer_seeds()
                    .as_ref()],
            ),
            supported_token_amount,
            supported_token_mint.decimals,
        )
    }
}
