use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use std::cmp;

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::fund::{WeightedAllocationParticipant, WeightedAllocationStrategy};
use crate::modules::pricing::{PricingService, TokenPricingSource};
use crate::utils::{AccountExt, PDASeeds};

use super::*;

pub struct NormalizedTokenPoolService<'info: 'a, 'a> {
    normalized_token_pool_account: &'a mut Account<'info, NormalizedTokenPoolAccount>,
    normalized_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    normalized_token_program: &'a Program<'info, Token>,
    current_slot: u64,
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
            ErrorCode::InvalidAccountDataVersionError
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
            current_slot: clock.slot,
            current_timestamp: clock.unix_timestamp,
        })
    }

    // TODO: move it to fund operation side
    pub(in crate::modules) fn get_denormalize_tokens_asset(
        &self,
        pricing_service: &PricingService,
        denormalize_amount_as_sol: u64,
    ) -> Result<Vec<(Pubkey, Pubkey, u64)>> {
        // let mut participants = vec![];
        // let supported_tokens = self
        //     .normalized_token_pool_account
        //     .supported_tokens
        //     .iter()
        //     .filter_map(|t| {
        //         if t.locked_amount == 0 {
        //             None
        //         } else {
        //             let reserved_amount_as_sol = pricing_service
        //                 .get_token_amount_as_sol(&t.mint, t.locked_amount)
        //                 .unwrap();
        //             participants.push(WeightedAllocationParticipant::new(
        //                 reserved_amount_as_sol,
        //                 0,
        //                 u64::MAX,
        //             ));
        //             Some(t)
        //         }
        //     })
        //     .collect::<Vec<_>>();
        //
        // WeightedAllocationStrategy::put(&mut participants, denormalize_amount_as_sol);

        let mut supported_tokens_state = vec![];
        // for (i, supported_token) in supported_tokens.iter().enumerate() {
        //     let need_to_denormalize_amount = pricing_service.get_sol_amount_as_token(
        //         &supported_token.mint,
        //         participants[i].get_last_put_amount()?,
        //     )?;
        //
        //     supported_tokens_state.push((
        //         supported_token.mint,
        //         supported_token.program,
        //         cmp::min(supported_token.locked_amount, need_to_denormalize_amount),
        //     ));
        // }
        Ok(supported_tokens_state)
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
        // normalized_token_mint_amount will be equal to supported_token_amount_as_sol at the initial minting, so 1SOL = 1NT.
        let normalized_token_mint_amount = if self.normalized_token_mint.supply == 0 {
            supported_token_amount_as_sol
        } else {
            pricing_service.get_sol_amount_as_token(
                &self.normalized_token_mint.key(),
                supported_token_amount_as_sol,
            )?
        };

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
            normalized_token_mint_amount,
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

        from_normalized_token_account_signer: &AccountInfo<'info>,
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

    pub fn process_initialize_withdrawal_account(
        &mut self,
        withdrawal_account: &mut Account<'info, NormalizedTokenWithdrawalAccount>,
        withdrawal_account_bump: u8,
        from_normalized_token_account: &InterfaceAccount<'info, TokenAccount>,
        from_normalized_token_account_signer: &Signer<'info>,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        withdrawal_account.initialize(
            withdrawal_account_bump,
            from_normalized_token_account_signer.key(),
            self.normalized_token_mint.key(),
            self.normalized_token_pool_account.key(),
        );
        require!(
            withdrawal_account.is_latest_version(),
            ErrorCode::InvalidAccountDataVersionError
        );

        // calculate claimable amount for each supported tokens to withdraw them proportionally relative to the current composition ratio.
        let normalized_token_amount = from_normalized_token_account.amount;
        require_gt!(normalized_token_amount, 0);

        let pricing_service = &mut self.new_pricing_service(pricing_sources)?;

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

                supported_token.allocate_locked_token_to_withdrawal_reserved(
                    supported_token_claimable_amount,
                )?;

                Ok(if supported_token_claimable_amount > 0 {
                    Some(NormalizedClaimableToken::new(
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

        // during evaluation, up to [claimable_tokens.len()] lamports can be deducted.
        require_gte!(
            normalized_token_amount_as_sol,
            claimable_tokens_value_as_sol
        );
        require_gte!(
            claimable_tokens.len() as u64,
            normalized_token_amount_as_sol - claimable_tokens_value_as_sol
        );

        // finalize the withdrawal account state.
        withdrawal_account.set_claimable_tokens(
            normalized_token_amount,
            claimable_tokens,
            self.current_timestamp,
        )?;

        withdrawal_account.exit(&crate::ID)?;

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

    pub fn process_withdraw(
        &mut self,
        withdrawal_account: &mut Account<'info, NormalizedTokenWithdrawalAccount>,
        pool_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        to_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        to_rent_lamports_account: &UncheckedAccount<'info>,

        supported_token_mint: &InterfaceAccount<'info, Mint>,
        supported_token_program: &Interface<'info, TokenInterface>,

        withdrawal_authority_signer: &Signer<'info>,
    ) -> Result<()> {
        withdrawal_account.update_if_needed();
        require!(
            withdrawal_account.is_latest_version(),
            ErrorCode::InvalidAccountDataVersionError
        );
        require_keys_eq!(
            withdrawal_account.normalized_token_pool,
            self.normalized_token_pool_account.key()
        );
        require_keys_eq!(
            withdrawal_account.withdrawal_authority,
            withdrawal_authority_signer.key()
        );

        // transfer claimable supported token
        let claimable_token =
            withdrawal_account.get_claimable_token_mut(&supported_token_mint.key())?;

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
        withdrawal_account.exit(&crate::ID)?;

        // close the withdrawal account after all tokens are settled.
        if withdrawal_account.is_settled() {
            withdrawal_account.close(to_rent_lamports_account.to_account_info())?;
        }

        Ok(())
    }

    pub fn process_update_prices(
        &mut self,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<events::OperatorUpdatedNormalizedTokenPoolPrices> {
        self.new_pricing_service(pricing_sources)?;
        Ok(events::OperatorUpdatedNormalizedTokenPoolPrices {
            normalized_token_mint: self.normalized_token_mint.key(),
            normalized_token_pool_account: self.normalized_token_pool_account.key(),
        })
    }

    // create a pricing service and register pool assets' value resolver
    pub(super) fn new_pricing_service(
        &mut self,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<PricingService<'info>> {
        let mut pricing_service = PricingService::new(pricing_sources)?
            .register_token_pricing_source_account(
                self.normalized_token_pool_account.as_account_info(),
            );

        // try to update current underlying assets' price
        self.update_asset_values(&mut pricing_service)?;

        Ok(pricing_service)
    }

    fn update_asset_values(&mut self, pricing_service: &mut PricingService) -> Result<()> {
        // ensure any update on pool account written before do pricing
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

        for supported_token in self
            .normalized_token_pool_account
            .get_supported_tokens_iter_mut()
        {
            supported_token.one_token_as_sol = pricing_service.get_token_amount_as_sol(
                &supported_token.mint,
                10u64
                    .checked_pow(supported_token.decimals as u32)
                    .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
            )?;
        }

        self.normalized_token_pool_account.normalized_token_value =
            pricing_service.get_token_total_value_as_atomic(normalized_token_mint_key)?;

        self.normalized_token_pool_account
            .normalized_token_value_updated_slot = self.current_slot;

        Ok(())
    }
}
