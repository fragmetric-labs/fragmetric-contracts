use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use spl_stake_pool::state::StakePool;
use std::cmp;

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::fund::{WeightedAllocationParticipant, WeightedAllocationStrategy};
use crate::modules::pricing::{PricingService, TokenPricingSource};
use crate::utils::{AsAccountInfo, PDASeeds};

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

impl<'info: 'a, 'a> NormalizedTokenPoolService<'info, 'a> {
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

    pub fn deserialize_pool_account(
        pool_account_info: &'info AccountInfo<'info>,
    ) -> Result<Account<'info, NormalizedTokenPoolAccount>> {
        Account::<NormalizedTokenPoolAccount>::try_from(pool_account_info)
    }

    /// returns (pubkey, writable) of [normalized_token_pool_account, normalized_token_mint, normalized_token_program]
    fn find_accounts_to_new(
        pool_account: &Account<'info, NormalizedTokenPoolAccount>,
    ) -> Vec<(Pubkey, bool)> {
        vec![
            (pool_account.key(), true),
            (pool_account.normalized_token_mint, true),
            (pool_account.normalized_token_program, false),
        ]
    }

    /// returns (pubkey, writable) of [normalized_token_pool_account, normalized_token_mint, normalized_token_program, supported_token_mint, supported_token_program, pool_supported_token_reserve_account]
    pub fn find_accounts_to_normalize_supported_token(
        pool_account_info: &'info AccountInfo<'info>,
        supported_token_mint: &'a Pubkey,
        supported_token_program: &'a Pubkey,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let pool_account = Self::deserialize_pool_account(pool_account_info)?;
        let mut accounts = Self::find_accounts_to_new(&pool_account);
        accounts.extend(vec![
            (*supported_token_mint, false),
            (*supported_token_program, false),
            (
                pool_account.find_supported_token_reserve_account_address(supported_token_mint)?,
                true,
            ),
        ]);
        Ok(accounts)
    }

    /// returns [to_normalized_token_account_amount, minted_normalized_token_amount]
    pub fn normalize_supported_token(
        &mut self,
        // fixed
        supported_token_mint: &InterfaceAccount<'info, Mint>,
        supported_token_program: &Interface<'info, TokenInterface>,
        pool_supported_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,

        // variant
        to_normalized_token_account: &mut InterfaceAccount<'info, TokenAccount>,
        from_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        from_supported_token_account_signer: &AccountInfo<'info>,
        from_supported_token_account_signer_seeds: &[&[&[u8]]],

        supported_token_amount: u64,

        pricing_service: &mut PricingService,
    ) -> Result<(u64, u64)> {
        require_keys_eq!(
            pool_supported_token_reserve_account.key(),
            self.normalized_token_pool_account
                .get_supported_token(&supported_token_mint.key())?
                .reserve_account
        );
        require_gt!(supported_token_amount, 0);

        let to_normalized_token_account_amount_before = to_normalized_token_account.amount;

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
                    to: pool_supported_token_reserve_account.to_account_info(),
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

        to_normalized_token_account.reload()?;
        let to_normalized_token_account_amount = to_normalized_token_account.amount;
        let minted_normalized_token_amount =
            to_normalized_token_account_amount - to_normalized_token_account_amount_before;

        msg!("NORMALIZE#: pool_token_mint={}, supported_token_mint={}, normalized_supported_token_amount={}, to_normalized_token_account_amount={}, minted_normalized_token_amount={}", self.normalized_token_mint.key(), supported_token_mint.key(), supported_token_amount, to_normalized_token_account_amount, minted_normalized_token_amount);

        Ok((
            to_normalized_token_account_amount,
            minted_normalized_token_amount,
        ))
    }

    pub fn denormalize_supported_token(
        &mut self,
        // fixed
        supported_token_mint: &InterfaceAccount<'info, Mint>,
        supported_token_program: &Interface<'info, TokenInterface>,
        pool_supported_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,

        // variant
        to_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        from_normalized_token_account: &InterfaceAccount<'info, TokenAccount>,
        from_normalized_token_account_signer: &AccountInfo<'info>,
        from_normalized_token_account_signer_seeds: &[&[&[u8]]],

        normalized_token_amount: u64,

        pricing_service: &mut PricingService,
    ) -> Result<()> {
        require_keys_eq!(
            pool_supported_token_reserve_account.key(),
            self.normalized_token_pool_account
                .get_supported_token(&supported_token_mint.key())?
                .reserve_account
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
                    from: pool_supported_token_reserve_account.to_account_info(),
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
        // variant
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
        let normalized_token_supply_amount = self
            .normalized_token_pool_account
            .normalized_token_supply_amount;

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
                        supported_token_total_value_as_sol,
                        normalized_token_amount,
                        normalized_token_supply_amount,
                    )?;
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
        // fixed
        supported_token_mint: &InterfaceAccount<'info, Mint>,
        supported_token_program: &Interface<'info, TokenInterface>,
        pool_supported_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,

        // variant
        withdrawal_account: &mut Account<'info, NormalizedTokenWithdrawalAccount>,
        withdrawal_authority_signer: &Signer<'info>,
        to_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        to_rent_lamports_account: &UncheckedAccount<'info>,
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
                    from: pool_supported_token_reserve_account.to_account_info(),
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
