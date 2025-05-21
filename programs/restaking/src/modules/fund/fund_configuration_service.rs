use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_2022::spl_token_2022;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::normalization;
use crate::modules::normalization::{NormalizedTokenPoolAccount, NormalizedTokenPoolService};
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking;
use crate::modules::reward;
use crate::modules::swap::TokenSwapSource;
use crate::utils::{AccountLoaderExt, SystemProgramExt};

use super::*;

pub struct FundConfigurationService<'a, 'info> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    fund_account: &'a mut AccountLoader<'info, FundAccount>,
}

impl Drop for FundConfigurationService<'_, '_> {
    fn drop(&mut self) {
        self.fund_account.exit(&crate::ID).unwrap();
    }
}

impl<'a, 'info> FundConfigurationService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        fund_account: &'a mut AccountLoader<'info, FundAccount>,
    ) -> Result<Self> {
        Ok(Self {
            receipt_token_mint,
            fund_account,
        })
    }

    pub fn process_initialize_fund_account(
        &mut self,
        receipt_token_program: &Program<'info, Token2022>,
        receipt_token_mint_current_authority: &Signer<'info>,
        fund_account_bump: u8,

        payer: &Signer<'info>,
        system_program: &Program<'info, System>,
        fund_reserve_account: &SystemAccount<'info>,
        fund_treasury_account: &SystemAccount<'info>,
    ) -> Result<()> {
        // fund possible SOL accounts to ensure rent-exempt
        let min_lamports_for_system_account = Rent::get()?.minimum_balance(0);
        anchor_lang::system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: payer.to_account_info(),
                    to: fund_reserve_account.to_account_info(),
                },
            ),
            min_lamports_for_system_account,
        )?;
        anchor_lang::system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: payer.to_account_info(),
                    to: fund_treasury_account.to_account_info(),
                },
            ),
            min_lamports_for_system_account,
        )?;

        // initialize header or entire buffer
        if self.fund_account.as_ref().data_len() < 8 + std::mem::size_of::<FundAccount>() {
            self.fund_account
                .initialize_zero_copy_header(fund_account_bump)?;
        } else {
            self.fund_account
                .load_init()?
                .initialize(fund_account_bump, self.receipt_token_mint)?;
        }

        // set token mint authority
        if self.receipt_token_mint.mint_authority.unwrap_or_default() != self.fund_account.key() {
            anchor_spl::token_2022::set_authority(
                CpiContext::new(
                    receipt_token_program.to_account_info(),
                    anchor_spl::token_2022::SetAuthority {
                        current_authority: receipt_token_mint_current_authority.to_account_info(),
                        account_or_mint: self.receipt_token_mint.to_account_info(),
                    },
                ),
                spl_token_2022::instruction::AuthorityType::MintTokens,
                Some(self.fund_account.key()),
            )?;
        }

        Ok(())
    }

    pub fn process_update_fund_account_if_needed(
        &self,
        payer: &Signer<'info>,
        system_program: &Program<'info, System>,
        desired_account_size: Option<u32>,
    ) -> Result<()> {
        let min_account_size = 8 + std::mem::size_of::<FundAccount>();
        let target_account_size = desired_account_size
            .map(|size| std::cmp::max(size as usize, min_account_size))
            .unwrap_or(min_account_size);

        let new_account_size = system_program.expand_account_size_if_needed(
            self.fund_account.as_ref(),
            payer,
            &[],
            target_account_size,
            None,
        )?;

        if new_account_size >= min_account_size {
            self.fund_account
                .load_mut()?
                .update_if_needed(self.receipt_token_mint)?;
        }

        Ok(())
    }

    pub fn process_set_address_lookup_table_account(
        &mut self,
        address_lookup_table_account: Option<Pubkey>,
    ) -> Result<()> {
        self.fund_account
            .load_mut()?
            .set_address_lookup_table_account(address_lookup_table_account);

        Ok(())
    }

    pub fn process_add_supported_token(
        &mut self,
        fund_supported_token_reserve_account: &InterfaceAccount<TokenAccount>,
        supported_token_mint: &InterfaceAccount<Mint>,
        pricing_source: TokenPricingSource,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<events::FundManagerUpdatedFund> {
        require_keys_eq!(
            fund_supported_token_reserve_account.owner,
            self.fund_account.load()?.get_reserve_account_address()?,
        );
        require_keys_eq!(
            fund_supported_token_reserve_account.mint,
            supported_token_mint.key()
        );

        self.fund_account.load_mut()?.add_supported_token(
            supported_token_mint.key(),
            *AsRef::<AccountInfo>::as_ref(supported_token_mint).owner,
            supported_token_mint.decimals,
            pricing_source,
            fund_supported_token_reserve_account.amount,
        )?;

        // validate pricing source
        let mut fund_service = FundService::new(self.receipt_token_mint, self.fund_account)?;
        fund_service.update_pricing_source_addresses()?;
        fund_service.new_pricing_service(pricing_sources, true)?;
        drop(fund_service);

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_remove_supported_token(
        &mut self,
        supported_token_mint: &InterfaceAccount<Mint>,
        normalized_token_mint: Option<&InterfaceAccount<Mint>>,
        normalized_token_pool_account: Option<&Account<normalization::NormalizedTokenPoolAccount>>,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<events::FundManagerUpdatedFund> {
        let mut fund_account = self.fund_account.load_mut()?;

        // NSOL must not support this token
        if let Some(normalized_token) = fund_account.get_normalized_token() {
            let normalized_token_mint = normalized_token_mint
                .ok_or_else(|| error!(error::ErrorCode::ConstraintAccountIsNone))?;
            let normalized_token_pool_account = normalized_token_pool_account
                .ok_or_else(|| error!(error::ErrorCode::ConstraintAccountIsNone))?;

            normalization::NormalizedTokenPoolService::validate_normalized_token_pool(
                normalized_token_pool_account,
                normalized_token_mint,
            )?;

            require_keys_eq!(normalized_token.mint, normalized_token_mint.key());

            if normalized_token_pool_account.has_supported_token(&supported_token_mint.key()) {
                err!(ErrorCode::FundSupportedTokenInUseError)?;
            }
        }

        fund_account.remove_supported_token(&supported_token_mint.key())?;

        // validate pricing
        let old_receipt_token_price = fund_account.one_receipt_token_as_sol;
        drop(fund_account);

        let mut fund_service = FundService::new(self.receipt_token_mint, self.fund_account)?;
        fund_service.update_pricing_source_addresses()?;
        fund_service.new_pricing_service(pricing_sources, true)?;
        drop(fund_service);

        let new_receipt_token_price = self.fund_account.load()?.one_receipt_token_as_sol;

        require_gte!(new_receipt_token_price, old_receipt_token_price);

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_set_normalized_token(
        &mut self,
        fund_normalized_token_reserve_account: &InterfaceAccount<TokenAccount>,
        normalized_token_mint: &InterfaceAccount<Mint>,
        normalized_token_pool: &Account<NormalizedTokenPoolAccount>,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<events::FundManagerUpdatedFund> {
        require_keys_eq!(
            fund_normalized_token_reserve_account.owner,
            self.fund_account.load()?.get_reserve_account_address()?
        );
        require_keys_eq!(
            fund_normalized_token_reserve_account.mint,
            normalized_token_mint.key()
        );

        // validate accounts
        NormalizedTokenPoolService::validate_normalized_token_pool(
            normalized_token_pool,
            normalized_token_mint,
        )?;

        // set normalized token and validate pricing source
        self.fund_account.load_mut()?.set_normalized_token(
            normalized_token_mint.key(),
            *AsRef::<AccountInfo>::as_ref(normalized_token_mint).owner,
            normalized_token_mint.decimals,
            normalized_token_pool.key(),
            fund_normalized_token_reserve_account.amount,
        )?;

        // do pricing as a validation
        let mut fund_service = FundService::new(self.receipt_token_mint, self.fund_account)?;
        fund_service.update_pricing_source_addresses()?;
        fund_service.new_pricing_service(pricing_sources, true)?;
        drop(fund_service);

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_add_restaking_vault(
        &mut self,
        fund_vault_receipt_token_account: &InterfaceAccount<TokenAccount>,

        vault: &UncheckedAccount,
        vault_supported_token_mint: &InterfaceAccount<Mint>,
        vault_receipt_token_mint: &InterfaceAccount<Mint>,

        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<events::FundManagerUpdatedFund> {
        let receipt_token_pricing_source = restaking::validate_vault(
            vault,
            vault_supported_token_mint.as_ref(),
            vault_receipt_token_mint.as_ref(),
        )?;

        self.fund_account.load_mut()?.add_restaking_vault(
            vault.key(),
            *vault.owner,
            vault_supported_token_mint.key(),
            vault_receipt_token_mint.key(),
            *AsRef::<AccountInfo>::as_ref(vault_receipt_token_mint).owner,
            vault_receipt_token_mint.decimals,
            receipt_token_pricing_source,
            fund_vault_receipt_token_account.amount,
        )?;

        // validate pricing source
        let mut fund_service = FundService::new(self.receipt_token_mint, self.fund_account)?;
        fund_service.update_pricing_source_addresses()?;
        fund_service.new_pricing_service(pricing_sources, true)?;
        drop(fund_service);

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_add_restaking_vault_delegation(
        &mut self,
        vault_operator_delegation: &UncheckedAccount,
        vault: &UncheckedAccount,
        operator: &UncheckedAccount,
    ) -> Result<events::FundManagerUpdatedFund> {
        let (delegation_index, delegated_amount, undelegating_amount) =
            restaking::validate_vault_operator_delegation(
                vault_operator_delegation,
                vault,
                operator,
            )?;

        self.fund_account
            .load_mut()?
            .get_restaking_vault_mut(vault.key)?
            .add_delegation(
                operator.key(),
                delegation_index,
                delegated_amount,
                undelegating_amount,
            )?;

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_set_wrapped_token(
        &mut self,
        wrapped_token_mint: &InterfaceAccount<'info, Mint>,
        wrapped_token_mint_current_authority: &Signer<'info>,
        wrapped_token_program: &Program<'info, Token>,
        fund_wrap_account: &SystemAccount,
        receipt_token_wrap_account: &InterfaceAccount<TokenAccount>,
        reward_account: &AccountLoader<reward::RewardAccount>,
        fund_wrap_account_reward_account: &AccountLoader<reward::UserRewardAccount>,
    ) -> Result<events::FundManagerUpdatedFund> {
        require_keys_eq!(
            *AsRef::<AccountInfo>::as_ref(wrapped_token_mint).owner,
            wrapped_token_program.key(),
        );

        require_keys_eq!(
            receipt_token_wrap_account.mint,
            self.receipt_token_mint.key(),
        );
        require_keys_eq!(
            receipt_token_wrap_account.owner,
            self.fund_account.load()?.get_wrap_account_address()?
        );

        require_eq!(
            wrapped_token_mint.decimals,
            self.receipt_token_mint.decimals,
        );

        // Must be pegged 1 to 1
        require_eq!(wrapped_token_mint.supply, receipt_token_wrap_account.amount);

        // validate accounts
        reward::UserRewardService::validate_user_reward_account(
            self.receipt_token_mint,
            fund_wrap_account,
            reward_account,
            fund_wrap_account_reward_account,
        )?;

        // set wrapped token
        self.fund_account.load_mut()?.set_wrapped_token(
            wrapped_token_mint.key(),
            wrapped_token_program.key(),
            wrapped_token_mint.decimals,
            wrapped_token_mint.supply,
        )?;

        // set token mint authority
        if wrapped_token_mint.mint_authority.unwrap_or_default() != self.fund_account.key() {
            anchor_spl::token::set_authority(
                CpiContext::new(
                    wrapped_token_program.to_account_info(),
                    anchor_spl::token::SetAuthority {
                        current_authority: wrapped_token_mint_current_authority.to_account_info(),
                        account_or_mint: wrapped_token_mint.to_account_info(),
                    },
                ),
                anchor_spl::token::spl_token::instruction::AuthorityType::MintTokens,
                Some(self.fund_account.key()),
            )?;
        }

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_update_fund_strategy(
        &mut self,
        deposit_enabled: bool,
        donation_enabled: bool,
        withdrawal_enabled: bool,
        transfer_enabled: bool,
        withdrawal_fee_rate_bps: u16,
        withdrawal_batch_threshold_interval_seconds: i64,
    ) -> Result<events::FundManagerUpdatedFund> {
        self.fund_account
            .load_mut()?
            .set_deposit_enabled(deposit_enabled)
            .set_donation_enabled(donation_enabled)
            .set_withdrawal_enabled(withdrawal_enabled)
            .set_transfer_enabled(transfer_enabled)
            .set_withdrawal_fee_rate_bps(withdrawal_fee_rate_bps)?
            .set_withdrawal_batch_threshold(withdrawal_batch_threshold_interval_seconds)?;

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_update_sol_strategy(
        &mut self,
        sol_depositable: bool,
        sol_accumulated_deposit_capacity_amount: u64,
        sol_accumulated_deposit_amount: Option<u64>,
        sol_withdrawable: bool,
        sol_withdrawal_normal_reserve_rate_bps: u16,
        sol_withdrawal_normal_reserve_max_amount: u64,
    ) -> Result<events::FundManagerUpdatedFund> {
        let mut fund_account = self.fund_account.load_mut()?;

        if let Some(sol_accumulated_deposit_amount) = sol_accumulated_deposit_amount {
            fund_account
                .sol
                .set_accumulated_deposit_amount(sol_accumulated_deposit_amount)?;
        }
        fund_account
            .sol
            .set_accumulated_deposit_capacity_amount(sol_accumulated_deposit_capacity_amount)?
            .set_depositable(sol_depositable)
            .set_withdrawable(sol_withdrawable)
            .set_normal_reserve_rate_bps(sol_withdrawal_normal_reserve_rate_bps)?
            .set_normal_reserve_max_amount(sol_withdrawal_normal_reserve_max_amount);

        // all underlying assets should be able to be either withdrawn directly or withdrawn as SOL through unstaking or swap.
        require!(
            fund_account.sol.withdrawable == 1
                || fund_account
                    .get_supported_tokens_iter()
                    .all(|supported_token| supported_token.token.withdrawable == 1),
            ErrorCode::FundInvalidConfigurationUpdateError
        );

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_update_supported_token_strategy(
        &mut self,
        token_mint: &Pubkey,
        token_depositable: bool,
        token_accumulated_deposit_capacity_amount: u64,
        token_accumulated_deposit_amount: Option<u64>,
        token_withdrawable: bool,
        token_withdrawal_normal_reserve_rate_bps: u16,
        token_withdrawal_normal_reserve_max_amount: u64,
        token_rebalancing_amount: Option<u64>,
        sol_allocation_weight: u64,
        sol_allocation_capacity_amount: u64,
    ) -> Result<events::FundManagerUpdatedFund> {
        let mut fund_account = self.fund_account.load_mut()?;
        let sol_withdrawable = fund_account.sol.withdrawable == 1;
        let supported_token = fund_account.get_supported_token_mut(token_mint)?;

        if let Some(token_accumulated_deposit_amount) = token_accumulated_deposit_amount {
            supported_token
                .token
                .set_accumulated_deposit_amount(token_accumulated_deposit_amount)?;
        }
        supported_token
            .token
            .set_depositable(token_depositable)
            .set_accumulated_deposit_capacity_amount(token_accumulated_deposit_capacity_amount)?
            .set_withdrawable(token_withdrawable)
            .set_normal_reserve_rate_bps(token_withdrawal_normal_reserve_rate_bps)?
            .set_normal_reserve_max_amount(token_withdrawal_normal_reserve_max_amount);

        if let Some(token_rebalancing_amount) = token_rebalancing_amount {
            supported_token.set_rebalancing_strategy(token_rebalancing_amount)?;
        }
        supported_token
            .set_sol_allocation_strategy(sol_allocation_weight, sol_allocation_capacity_amount)?;

        // given underlying asset should be able to be either withdrawn directly or withdrawn as SOL through unstaking or swap.
        require!(
            sol_withdrawable || supported_token.token.withdrawable == 1,
            ErrorCode::FundInvalidConfigurationUpdateError
        );

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_update_restaking_vault_strategy(
        &mut self,
        vault: &Pubkey,
        sol_allocation_weight: u64,
        sol_allocation_capacity_amount: u64,
    ) -> Result<events::FundManagerUpdatedFund> {
        self.fund_account
            .load_mut()?
            .get_restaking_vault_mut(vault)?
            .set_sol_allocation_strategy(sol_allocation_weight, sol_allocation_capacity_amount)?;

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_update_restaking_vault_delegation_strategy(
        &mut self,
        vault: &Pubkey,
        operator: &Pubkey,
        token_allocation_weight: u64,
        token_allocation_capacity_amount: u64,
        token_redelegating_amount: Option<u64>,
    ) -> Result<events::FundManagerUpdatedFund> {
        let mut fund_account = self.fund_account.load_mut()?;
        let delegation = fund_account
            .get_restaking_vault_mut(vault)?
            .get_delegation_mut(operator)?;

        delegation.set_supported_token_allocation_strategy(
            token_allocation_weight,
            token_allocation_capacity_amount,
        )?;
        if let Some(token_amount) = token_redelegating_amount {
            delegation.set_supported_token_redelegating_amount(token_amount)?;
        }

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_add_restaking_vault_compounding_reward_token(
        &mut self,
        vault: &Pubkey,
        compounding_reward_token_mint: Pubkey,
    ) -> Result<events::FundManagerUpdatedFund> {
        self.fund_account
            .load_mut()?
            .get_restaking_vault_mut(vault)?
            .add_compounding_reward_token(compounding_reward_token_mint)?;

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_remove_restaking_vault_compounding_reward_token(
        &mut self,
        vault: &Pubkey,
        compounding_reward_token_mint: &Pubkey,
    ) -> Result<events::FundManagerUpdatedFund> {
        self.fund_account
            .load_mut()?
            .get_restaking_vault_mut(vault)?
            .remove_compounding_reward_token(compounding_reward_token_mint)?;

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_add_restaking_vault_distributing_reward_token(
        &mut self,
        vault: &Pubkey,
        distributing_reward_token_mint: Pubkey,
    ) -> Result<events::FundManagerUpdatedFund> {
        self.fund_account
            .load_mut()?
            .get_restaking_vault_mut(vault)?
            .add_distributing_reward_token(distributing_reward_token_mint)?;

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_update_restaking_vault_distributing_reward_token_harvest_threshold(
        &mut self,
        vault: &Pubkey,
        distributing_reward_token_mint: &Pubkey,
        harvest_threshold_min_amount: u64,
        harvest_threshold_max_amount: u64,
        harvest_threshold_interval_seconds: i64,
    ) -> Result<events::FundManagerUpdatedFund> {
        self.fund_account
            .load_mut()?
            .get_restaking_vault_mut(vault)?
            .get_distributing_reward_token_mut(distributing_reward_token_mint)?
            .update_harvest_threshold(
                harvest_threshold_min_amount,
                harvest_threshold_max_amount,
                harvest_threshold_interval_seconds,
            )?;

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_remove_restaking_vault_distributing_reward_token(
        &mut self,
        vault: &Pubkey,
        distributing_reward_token_mint: &Pubkey,
    ) -> Result<events::FundManagerUpdatedFund> {
        self.fund_account
            .load_mut()?
            .get_restaking_vault_mut(vault)?
            .remove_distributing_reward_token(distributing_reward_token_mint)?;

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_add_wrapped_token_holder(
        &mut self,
        wrapped_token_holder: &InterfaceAccount<TokenAccount>,
        reward_account: &AccountLoader<reward::RewardAccount>,
        wrapped_token_holder_reward_account: &AccountLoader<reward::UserRewardAccount>,
    ) -> Result<events::FundManagerUpdatedFund> {
        reward::UserRewardService::validate_user_reward_account(
            self.receipt_token_mint,
            wrapped_token_holder.as_ref(),
            reward_account,
            wrapped_token_holder_reward_account,
        )?;

        let mut fund_account = self.fund_account.load_mut()?;
        let wrapped_token = fund_account
            .get_wrapped_token_mut()
            .ok_or_else(|| error!(ErrorCode::FundWrappedTokenNotSetError))?;

        require_keys_eq!(wrapped_token_holder.mint, wrapped_token.mint);

        wrapped_token.add_holder(wrapped_token_holder.key())?;

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_remove_wrapped_token_holder(
        &self,
        fund_wrap_account: &SystemAccount,
        wrapped_token_holder: &InterfaceAccount<TokenAccount>,
        reward_account: &AccountLoader<'info, reward::RewardAccount>,
        fund_wrap_account_reward_account: &AccountLoader<reward::UserRewardAccount>,
        wrapped_token_holder_reward_account: &AccountLoader<reward::UserRewardAccount>,
    ) -> Result<events::FundManagerUpdatedFund> {
        reward::UserRewardService::validate_user_reward_account(
            self.receipt_token_mint,
            fund_wrap_account,
            reward_account,
            fund_wrap_account_reward_account,
        )?;

        reward::UserRewardService::validate_user_reward_account(
            self.receipt_token_mint,
            wrapped_token_holder.as_ref(),
            reward_account,
            wrapped_token_holder_reward_account,
        )?;

        let mut fund_account = self.fund_account.load_mut()?;
        let wrapped_token = fund_account
            .get_wrapped_token_mut()
            .ok_or_else(|| error!(ErrorCode::FundWrappedTokenNotSetError))?;

        let (old_wrapped_token_holder_amount, old_wrapped_token_retained_amount) =
            wrapped_token.remove_holder(&wrapped_token_holder.key())?;

        // update reward
        let reward_service = reward::RewardService::new(self.receipt_token_mint, reward_account)?;

        // holder lost `old_wrapped_token_holder_amount`
        reward_service.update_reward_pools_token_allocation(
            Some(wrapped_token_holder_reward_account),
            None,
            old_wrapped_token_holder_amount,
            None,
        )?;

        // fund_wrap_account gained ∆wrapped_token_retained_amount
        if wrapped_token.retained_amount > old_wrapped_token_retained_amount {
            let wrapped_token_retained_amount_delta =
                wrapped_token.retained_amount - old_wrapped_token_retained_amount;
            reward_service.update_reward_pools_token_allocation(
                None,
                Some(fund_wrap_account_reward_account),
                wrapped_token_retained_amount_delta,
                None,
            )?;
        }

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_add_token_swap_strategy(
        &mut self,
        from_token_mint: Pubkey,
        to_token_mint: Pubkey,
        swap_source: TokenSwapSource,
    ) -> Result<events::FundManagerUpdatedFund> {
        self.fund_account.load_mut()?.add_token_swap_strategy(
            from_token_mint,
            to_token_mint,
            swap_source,
        )?;

        self.create_fund_manager_updated_fund_event()
    }

    fn create_fund_manager_updated_fund_event(&self) -> Result<events::FundManagerUpdatedFund> {
        Ok(events::FundManagerUpdatedFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: self.fund_account.key(),
        })
    }
}
