use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::modules::pricing::{PricingService, TokenPricingSource};
use crate::utils::{AccountExt, PDASeeds};

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

        from_supported_token_account_signer: &AccountInfo<'info>,
        from_supported_token_account_signer_seeds: &[&[&[u8]]],

        supported_token_amount: u64,

        pricing_service: &mut PricingService,
    ) -> Result<()> {
        require_keys_eq!(
            pool_supported_token_account.key(),
            self.normalized_token_pool_account
                .get_supported_token(&supported_token_mint.key())?
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
            .get_supported_token_mut(&supported_token_mint.key())?
            .lock_token(supported_token_amount)?;

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                supported_token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: from_supported_token_account.to_account_info(),
                    mint: supported_token_mint.to_account_info(),
                    to: pool_supported_token_account.to_account_info(),
                    authority: from_supported_token_account_signer.to_account_info(),
                },
                from_supported_token_account_signer_seeds,
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
                &[self.normalized_token_pool_account.get_seeds().as_ref()],
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

        from_normalized_token_account_signer: AccountInfo<'info>,
        from_normalized_token_account_signer_seeds: &[&[&[u8]]],

        normalized_token_amount: u64,

        pricing_service: &mut PricingService,
    ) -> Result<()> {
        require_keys_eq!(
            pool_supported_token_account.key(),
            self.normalized_token_pool_account
                .get_supported_token(&supported_token_mint.key())?
                .lock_account
        );
        require_gt!(normalized_token_amount, 0);

        let normalized_token_amount_as_sol = pricing_service
            .get_token_amount_as_sol(&self.normalized_token_mint.key(), normalized_token_amount)?;
        let supported_token_amount = pricing_service
            .get_sol_amount_as_token(&supported_token_mint.key(), normalized_token_amount_as_sol)?;

        self.normalized_token_pool_account
            .get_supported_token_mut(&supported_token_mint.key())?
            .unlock_token(supported_token_amount)?;

        anchor_spl::token_interface::burn(
            CpiContext::new_with_signer(
                self.normalized_token_program.to_account_info(),
                anchor_spl::token_interface::Burn {
                    mint: self.normalized_token_mint.to_account_info(),
                    from: from_normalized_token_account.to_account_info(),
                    authority: from_normalized_token_account_signer.to_account_info(),
                },
                from_normalized_token_account_signer_seeds,
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
                &[self.normalized_token_pool_account.get_seeds().as_ref()],
            ),
            supported_token_amount,
            supported_token_mint.decimals,
        )?;

        self.normalized_token_pool_account
            .reload_normalized_token_supply(self.normalized_token_mint)?;

        self.update_asset_values(pricing_service)?;

        Ok(())
    }

    pub fn process_initialize_withdrawal_ticket(
        &mut self,
        withdrawal_ticket: &mut Account<'info, NormalizedTokenWithdrawalTicketAccount>,
        withdrawal_ticket_bump: u8,
        from_normalized_token_account: &InterfaceAccount<'info, TokenAccount>,
        from_normalized_token_account_signer: &Signer<'info>,

        pricing_service: &mut PricingService,
    ) -> Result<()> {
        withdrawal_ticket.initialize(
            withdrawal_ticket_bump,
            from_normalized_token_account_signer.key(),
            self.normalized_token_mint.key(),
            self.normalized_token_pool_account.key(),
        );
        require!(
            withdrawal_ticket.is_latest_version(),
            ErrorCode::InvalidDataVersionError
        );

        // calculate claimable amount for each supported tokens to withdraw them proportionally relative to the current composition ratio.
        let normalized_token_amount = from_normalized_token_account.amount;
        require_gt!(normalized_token_amount, 0);

        let normalized_token_amount_as_sol = pricing_service
            .get_token_amount_as_sol(&self.normalized_token_mint.key(), normalized_token_amount)?;
        let pool_total_value_as_sol = pricing_service
            .get_token_total_value_as_sol(&self.normalized_token_mint.key())?
            .0;

        let mut claimable_tokens_value_as_sol = 0u64;
        let claimable_tokens = self
            .normalized_token_pool_account
            .supported_tokens
            .iter_mut()
            .map(|supported_token| {
                let supported_token_total_value_as_sol = pricing_service.get_token_amount_as_sol(
                    &supported_token.mint,
                    supported_token.locked_amount,
                )?;
                let supported_token_claimable_amount_as_sol =
                    crate::utils::get_proportional_amount(
                        normalized_token_amount_as_sol,
                        supported_token_total_value_as_sol,
                        pool_total_value_as_sol,
                    )
                    .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
                claimable_tokens_value_as_sol += supported_token_claimable_amount_as_sol;

                let supported_token_claimable_amount = pricing_service.get_sol_amount_as_token(
                    &supported_token.mint,
                    supported_token_claimable_amount_as_sol,
                )?;

                supported_token
                    .unlock_withdrawal_reserved_token(supported_token_claimable_amount)?;

                Ok(if supported_token_claimable_amount > 0 {
                    Some(ClaimableToken::new(
                        supported_token.mint,
                        supported_token.program,
                        supported_token_claimable_amount,
                    ))
                } else {
                    None
                })
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .filter_map(|x| x)
            .collect::<Vec<_>>();

        // examine the value of the ticket; there can be cutting for the amount of [claimable_tokens.len()] lamports at maximum during evaluation.
        require_gte!(
            normalized_token_amount_as_sol,
            claimable_tokens_value_as_sol
        );
        require_gte!(
            claimable_tokens.len() as u64,
            normalized_token_amount_as_sol - claimable_tokens_value_as_sol
        );

        // finalize the ticket state.
        withdrawal_ticket.set_claimable_tokens(
            normalized_token_amount,
            claimable_tokens,
            self.current_timestamp,
        )?;

        withdrawal_ticket.exit(&crate::ID)?;

        // burn given normalized token amount
        anchor_spl::token_interface::burn(
            CpiContext::new(
                self.normalized_token_program.to_account_info(),
                anchor_spl::token_interface::Burn {
                    mint: self.normalized_token_mint.to_account_info(),
                    from: from_normalized_token_account.to_account_info(),
                    authority: from_normalized_token_account_signer.to_account_info(),
                },
            ),
            normalized_token_amount,
        )?;

        // sync pool account data and pricing information
        self.normalized_token_pool_account
            .reload_normalized_token_supply(self.normalized_token_mint)?;

        self.update_asset_values(pricing_service)?;

        Ok(())
    }

    pub fn process_claim_withdrawal_ticket(
        &mut self,
        withdrawal_ticket: &mut Account<'info, NormalizedTokenWithdrawalTicketAccount>,
        pool_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        to_supported_token_account: &InterfaceAccount<'info, TokenAccount>,

        supported_token_mint: &InterfaceAccount<'info, Mint>,
        supported_token_program: &Interface<'info, TokenInterface>,

        withdrawal_authority_signer: &Signer<'info>,
    ) -> Result<()> {
        withdrawal_ticket.update_if_needed();
        require!(
            withdrawal_ticket.is_latest_version(),
            ErrorCode::InvalidDataVersionError
        );
        require_keys_eq!(
            withdrawal_ticket.normalized_token_pool,
            self.normalized_token_pool_account.key()
        );
        require_keys_eq!(
            withdrawal_ticket.withdrawal_authority,
            withdrawal_authority_signer.key()
        );

        // transfer claimable supported token
        let claimable_token =
            withdrawal_ticket.get_claimable_token_mut(&supported_token_mint.key())?;

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                supported_token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: pool_supported_token_account.to_account_info(),
                    mint: supported_token_mint.to_account_info(),
                    to: to_supported_token_account.to_account_info(),
                    authority: self.normalized_token_pool_account.to_account_info(),
                },
                &[self.normalized_token_pool_account.get_seeds().as_ref()],
            ),
            claimable_token.claimable_amount,
            supported_token_mint.decimals,
        )?;

        // mark the token amount as settled.
        let supported_token = self
            .normalized_token_pool_account
            .get_supported_token_mut(&supported_token_mint.key())?;
        supported_token.settle_withdrawal_reserved_token(claimable_token.claimable_amount)?;
        claimable_token.settle()?;
        withdrawal_ticket.exit(&crate::ID)?;

        // close the ticket account after all tokens are settled.
        if withdrawal_ticket.is_settled() {
            let mut withdrawal_ticket_account_info = withdrawal_ticket.to_account_info();
            let withdrawal_ticket_lamports = withdrawal_ticket_account_info.lamports();
            **withdrawal_authority_signer.lamports.borrow_mut() += withdrawal_ticket_lamports;
            **withdrawal_ticket_account_info.lamports.borrow_mut() = 0;
            
            let mut data = withdrawal_ticket_account_info.try_borrow_mut_data()?;
            data.fill(0);
        }

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
