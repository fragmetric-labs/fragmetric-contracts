use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::pricing::{PricingService, TokenPricingSource};
use crate::utils::{AsAccountInfo, PDASeeds};

use super::*;

pub struct NormalizedTokenPoolService<'a, 'info> {
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

impl<'a, 'info: 'a> NormalizedTokenPoolService<'a, 'info> {
    pub fn new(
        normalized_token_pool_account: &'a mut Account<'info, NormalizedTokenPoolAccount>,
        normalized_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        normalized_token_program: &'a Program<'info, Token>,
    ) -> Result<Self> {
        Self::validate_normalized_token_pool(normalized_token_pool_account, normalized_token_mint)?;

        let clock = Clock::get()?;
        Ok(Self {
            normalized_token_pool_account,
            normalized_token_mint,
            normalized_token_program,
            current_slot: clock.slot,
            current_timestamp: clock.unix_timestamp,
        })
    }

    pub fn validate_normalized_token_pool(
        normalized_token_pool_account: &Account<NormalizedTokenPoolAccount>,
        normalized_token_mint: &InterfaceAccount<Mint>,
    ) -> Result<()> {
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
            Token::id(),
        );

        Ok(())
    }

    #[inline(always)]
    pub(super) fn deserialize_pool_account(
        pool_account_info: &'info AccountInfo<'info>,
    ) -> Result<Account<'info, NormalizedTokenPoolAccount>> {
        Account::try_from(pool_account_info)
    }

    /// * pool_account(writable)
    /// * pool_token_mint(writable)
    /// * pool_token_program
    fn find_accounts_to_new(
        pool_account: &Account<NormalizedTokenPoolAccount>,
    ) -> [(Pubkey, bool); 3] {
        [
            (pool_account.key(), true),
            (pool_account.normalized_token_mint, true),
            (pool_account.normalized_token_program, false),
        ]
    }

    /// * (0) pool_account(writable)
    /// * (1) pool_token_mint(writable)
    /// * (2) pool_token_program
    /// * (3) supported_token_mint
    /// * (4) supported_token_program
    /// * (5) supported_token_reserve_account(writable)
    #[inline(never)]
    pub(in crate::modules) fn find_accounts_to_normalize_supported_token(
        pool_account_info: &'info AccountInfo<'info>,
        supported_token_mint: &Pubkey,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let pool_account = &Self::deserialize_pool_account(pool_account_info)?;
        let supported_token = pool_account.get_supported_token(supported_token_mint)?;

        let accounts = Self::find_accounts_to_new(pool_account).into_iter().chain([
            (supported_token.mint, false),
            (supported_token.program, false),
            (
                supported_token.find_reserve_account_address(&pool_account.normalized_token_mint),
                true,
            ),
        ]);

        Ok(accounts)
    }

    /// * (0) pool_account(writable)
    /// * (1) pool_token_mint(writable)
    /// * (2) pool_token_program
    /// * (3) supported_token_mint
    /// * (4) supported_token_program
    /// * (5) supported_token_reserve_account(writable)
    pub(in crate::modules) fn find_accounts_to_denormalize_supported_token(
        pool_account_info: &'info AccountInfo<'info>,
        supported_token_mint: &Pubkey,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let pool_account = &Self::deserialize_pool_account(pool_account_info)?;
        let supported_token = pool_account.get_supported_token(supported_token_mint)?;

        let accounts = Self::find_accounts_to_new(pool_account).into_iter().chain([
            (supported_token.mint, false),
            (supported_token.program, false),
            (
                supported_token.find_reserve_account_address(&pool_account.normalized_token_mint),
                true,
            ),
        ]);

        Ok(accounts)
    }

    /// returns [to_normalized_token_account_amount, minted_normalized_token_amount]
    pub(in crate::modules) fn normalize_supported_token(
        &mut self,
        // fixed
        supported_token_mint: &InterfaceAccount<'info, Mint>,
        supported_token_program: &Interface<'info, TokenInterface>,
        supported_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,

        // variant
        to_normalized_token_account: &mut InterfaceAccount<'info, TokenAccount>,
        from_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        from_supported_token_account_signer: &AccountInfo<'info>,
        from_supported_token_account_signer_seeds: &[&[&[u8]]],

        supported_token_amount: u64,

        pricing_service: &mut PricingService,
    ) -> Result<(u64, u64)> {
        require_keys_eq!(
            supported_token_reserve_account.key(),
            self.normalized_token_pool_account
                .get_supported_token(&supported_token_mint.key())?
                .reserve_account
        );
        require_gt!(supported_token_amount, 0);
        require_gte!(from_supported_token_account.amount, supported_token_amount);

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
                    to: supported_token_reserve_account.to_account_info(),
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

        self.sync_pool_account_data_and_pricing(pricing_service)?;

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

    pub(in crate::modules) fn denormalize_supported_token(
        &mut self,
        // fixed
        supported_token_mint: &InterfaceAccount<'info, Mint>,
        supported_token_program: &Interface<'info, TokenInterface>,
        supported_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,

        // variant
        to_supported_token_account: &mut InterfaceAccount<'info, TokenAccount>,
        from_normalized_token_account: &InterfaceAccount<'info, TokenAccount>,
        from_normalized_token_account_signer: &AccountInfo<'info>,
        from_normalized_token_account_signer_seeds: &[&[&[u8]]],

        normalized_token_amount: u64,

        pricing_service: &mut PricingService,
    ) -> Result<(u64, u64)> {
        require_keys_eq!(
            supported_token_reserve_account.key(),
            self.normalized_token_pool_account
                .get_supported_token(&supported_token_mint.key())?
                .reserve_account
        );
        require_gt!(normalized_token_amount, 0);
        require_gte!(
            from_normalized_token_account.amount,
            normalized_token_amount
        );

        let to_supported_token_account_amount_before = to_supported_token_account.amount;

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
                    from: supported_token_reserve_account.to_account_info(),
                    mint: supported_token_mint.to_account_info(),
                    to: to_supported_token_account.to_account_info(),
                    authority: self.normalized_token_pool_account.to_account_info(),
                },
                &[self.normalized_token_pool_account.get_seeds().as_ref()],
            ),
            supported_token_amount,
            supported_token_mint.decimals,
        )?;

        self.sync_pool_account_data_and_pricing(pricing_service)?;

        to_supported_token_account.reload()?;
        let to_supported_token_account_amount = to_supported_token_account.amount;
        let denormalized_supported_token_amount =
            to_supported_token_account_amount - to_supported_token_account_amount_before;

        msg!("DENORMALIZE#: pool_token_mint={}, supported_token_mint={}, burnt_normalized_token_amount={}, to_supported_token_account_amount={}, denormalized_supported_token_amount={}", self.normalized_token_mint.key(), supported_token_mint.key(), normalized_token_amount, to_supported_token_account_amount, denormalized_supported_token_amount);

        Ok((
            to_supported_token_account_amount,
            denormalized_supported_token_amount,
        ))
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
                let supported_token_locked_amount_as_sol = pricing_service
                    .get_token_amount_as_sol(
                        &supported_token.mint,
                        supported_token.locked_amount,
                    )?;
                let supported_token_claimable_amount_as_sol =
                    crate::utils::get_proportional_amount(
                        supported_token_locked_amount_as_sol,
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

                Ok((supported_token_claimable_amount > 0).then(|| {
                    NormalizedClaimableToken::new(
                        supported_token.mint,
                        supported_token.program,
                        supported_token_claimable_amount,
                    )
                }))
            })
            .filter_map(Result::transpose)
            .collect::<Result<Vec<_>>>()?;

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
        self.sync_pool_account_data_and_pricing(pricing_service)?;

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
        withdrawal_authority: &Signer<'info>,
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
            withdrawal_authority.key()
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
        self.normalized_token_pool_account
            .get_supported_token_mut(&supported_token_mint.key())?
            .settle_withdrawal_reserved_token(claimable_token.claimable_amount)?;
        claimable_token.settle()?;

        // close the withdrawal account after all tokens are settled.
        if withdrawal_account.is_settled() {
            withdrawal_account.close(to_rent_lamports_account.to_account_info())?;
        }

        Ok(())
    }

    fn sync_pool_account_data_and_pricing(
        &mut self,
        pricing_service: &mut PricingService,
    ) -> Result<()> {
        self.normalized_token_pool_account
            .reload_normalized_token_supply(self.normalized_token_mint)?;
        self.update_asset_values(pricing_service)
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
        let mut pricing_service = if pricing_sources
            .iter()
            .find(|source| source.key() == self.normalized_token_pool_account.key())
            .is_some()
        {
            PricingService::new(pricing_sources)
        } else {
            PricingService::new(
                pricing_sources
                    .iter()
                    .chain([self.normalized_token_pool_account.as_account_info()]),
            )
        };

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
            .one_normalized_token_as_sol = pricing_service
            .get_one_token_amount_as_sol(
                normalized_token_mint_key,
                self.normalized_token_mint.decimals,
            )?
            .unwrap_or_default();

        for supported_token in self
            .normalized_token_pool_account
            .supported_tokens
            .iter_mut()
        {
            supported_token.one_token_as_sol = pricing_service
                .get_one_token_amount_as_sol(&supported_token.mint, supported_token.decimals)?
                .unwrap_or_default();
        }

        pricing_service.flatten_token_value(
            normalized_token_mint_key,
            &mut self.normalized_token_pool_account.normalized_token_value,
        )?;

        self.normalized_token_pool_account
            .normalized_token_value_updated_slot = self.current_slot;

        Ok(())
    }
}
