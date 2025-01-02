use anchor_lang::prelude::*;
use anchor_spl::token::accessor::amount;
use anchor_spl::token::Token;
use anchor_spl::token_2022;
use anchor_spl::token_interface::*;

use crate::modules::normalization::{NormalizedTokenPoolAccount, NormalizedTokenPoolService};
use crate::modules::pricing::TokenPricingSource;
use crate::utils::AccountLoaderExt;
use crate::{errors, events};

use super::*;

pub struct FundConfigurationService<'info: 'a, 'a> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    fund_account: &'a mut AccountLoader<'info, FundAccount>,
}

impl Drop for FundConfigurationService<'_, '_> {
    fn drop(&mut self) {
        self.fund_account.exit(&crate::ID).unwrap();
    }
}

impl<'info: 'a, 'a> FundConfigurationService<'info, 'a> {
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
        fund_reserve_account: &SystemAccount<'info>,
        fund_account_bump: u8,
    ) -> Result<()> {
        if self.fund_account.as_ref().data_len() < 8 + std::mem::size_of::<FundAccount>() {
            self.fund_account
                .initialize_zero_copy_header(fund_account_bump)?;
        } else {
            self.fund_account.load_init()?.initialize(
                fund_account_bump,
                self.receipt_token_mint,
                fund_reserve_account.lamports(),
            );
        }

        // set token mint authority
        if self.receipt_token_mint.mint_authority.unwrap_or_default() != self.fund_account.key() {
            token_2022::set_authority(
                CpiContext::new(
                    receipt_token_program.to_account_info(),
                    token_2022::SetAuthority {
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
        fund_reserve_account: &SystemAccount<'info>,
        desired_account_size: Option<u32>,
    ) -> Result<()> {
        self.fund_account.expand_account_size_if_needed(
            payer,
            system_program,
            desired_account_size,
        )?;

        if self.fund_account.as_ref().data_len() >= 8 + std::mem::size_of::<FundAccount>() {
            self.fund_account
                .load_mut()?
                .update_if_needed(self.receipt_token_mint, fund_reserve_account.lamports());
        }

        Ok(())
    }

    pub fn process_add_supported_token(
        &mut self,
        fund_supported_token_reserve_account: &InterfaceAccount<TokenAccount>,
        supported_token_mint: &InterfaceAccount<Mint>,
        supported_token_program: &Interface<TokenInterface>,
        pricing_source: TokenPricingSource,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<events::FundManagerUpdatedFund> {
        require_keys_eq!(
            fund_supported_token_reserve_account.owner,
            self.fund_account.key()
        );
        require_keys_eq!(
            fund_supported_token_reserve_account.mint,
            supported_token_mint.key()
        );
        require_keys_eq!(
            *supported_token_mint.to_account_info().owner,
            supported_token_program.key()
        );

        self.fund_account.load_mut()?.add_supported_token(
            supported_token_mint.key(),
            supported_token_program.key(),
            supported_token_mint.decimals,
            pricing_source,
            fund_supported_token_reserve_account.amount,
        )?;

        // validate pricing source
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .new_pricing_service(pricing_sources)?;

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_set_normalized_token(
        &mut self,
        fund_normalized_token_account: &InterfaceAccount<TokenAccount>,
        normalized_token_mint: &mut InterfaceAccount<'info, Mint>,
        normalized_token_program: &Program<'info, Token>,
        normalized_token_pool: &mut Account<'info, NormalizedTokenPoolAccount>,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<events::FundManagerUpdatedFund> {
        require_keys_eq!(fund_normalized_token_account.owner, self.fund_account.key());
        require_keys_eq!(
            fund_normalized_token_account.mint,
            normalized_token_mint.key()
        );
        require_keys_eq!(
            *normalized_token_mint.to_account_info().owner,
            normalized_token_program.key()
        );

        // validate accounts
        NormalizedTokenPoolService::new(
            normalized_token_pool,
            normalized_token_mint,
            normalized_token_program,
        )?;

        // set normalized token and validate pricing source
        self.fund_account.load_mut()?.set_normalized_token(
            normalized_token_mint.key(),
            normalized_token_program.key(),
            normalized_token_mint.decimals,
            normalized_token_pool.key(),
            fund_normalized_token_account.amount,
        )?;

        // do pricing as a validation
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .new_pricing_service(pricing_sources)?;

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_add_restaking_vault(
        &mut self,
        fund_vault_supported_token_account: &InterfaceAccount<TokenAccount>,
        fund_vault_receipt_token_account: &InterfaceAccount<TokenAccount>,

        vault_supported_token_mint: &InterfaceAccount<Mint>,
        vault_supported_token_program: &Interface<TokenInterface>,

        vault: &UncheckedAccount,
        vault_program: &UncheckedAccount,
        vault_receipt_token_mint: &InterfaceAccount<Mint>,
        vault_receipt_token_program: &Interface<TokenInterface>,

        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<events::FundManagerUpdatedFund> {
        require_keys_eq!(
            fund_vault_supported_token_account.owner,
            self.fund_account.key()
        );
        require_keys_eq!(
            fund_vault_supported_token_account.mint,
            vault_supported_token_mint.key()
        );
        require_keys_eq!(
            *vault_supported_token_mint.to_account_info().owner,
            vault_supported_token_program.key()
        );

        require_keys_eq!(
            fund_vault_receipt_token_account.owner,
            self.fund_account.key()
        );
        require_keys_eq!(
            fund_vault_receipt_token_account.mint,
            vault_receipt_token_mint.key()
        );
        require_keys_eq!(
            *fund_vault_receipt_token_account.to_account_info().owner,
            vault_receipt_token_program.key()
        );

        // TODO: add more vault validation since we do not check vault address anymore
        require_keys_eq!(*vault.to_account_info().owner, vault_program.key());

        // TODO: add more vault receipt token mint validation since we do not check mint address anymore

        self.fund_account.load_mut()?.add_restaking_vault(
            vault.key(),
            vault_program.key(),
            vault_supported_token_mint.key(),
            vault_receipt_token_mint.key(),
            vault_receipt_token_program.key(),
            vault_receipt_token_mint.decimals,
            fund_vault_receipt_token_account.amount,
        )?;

        // validate pricing source
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .new_pricing_service(pricing_sources)?;

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_add_restaking_delegation(
        &mut self,
        vault: &UncheckedAccount,
        vault_program: &UncheckedAccount,
        vault_operator: &UncheckedAccount,
    ) -> Result<events::FundManagerUpdatedFund> {
        {
            let mut fund_account = self.fund_account.load_mut()?;
            let restaking_vault = fund_account.get_restaking_vault_mut(vault.key)?;

            require_keys_eq!(restaking_vault.program, vault_program.key());

            // TODO: need some validation?
            restaking_vault.add_delegation(vault_operator.key)?;
        }

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_update_fund_strategy(
        &mut self,
        deposit_enabled: bool,
        withdrawal_enabled: bool,
        withdrawal_fee_rate_bps: u16,
        withdrawal_batch_threshold_interval_seconds: i64,
    ) -> Result<events::FundManagerUpdatedFund> {
        {
            let mut fund_account = self.fund_account.load_mut()?;
            fund_account.set_deposit_enabled(deposit_enabled);
            fund_account.set_withdrawal_enabled(withdrawal_enabled);
            fund_account.set_withdrawal_fee_rate_bps(withdrawal_fee_rate_bps)?;
            fund_account
                .set_withdrawal_batch_threshold(withdrawal_batch_threshold_interval_seconds)?;
        }

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
        {
            let mut fund_account = self.fund_account.load_mut()?;

            fund_account.sol.set_depositable(sol_depositable);
            fund_account
                .sol
                .set_accumulated_deposit_capacity_amount(sol_accumulated_deposit_capacity_amount)?;
            if let Some(sol_accumulated_deposit_amount) = sol_accumulated_deposit_amount {
                fund_account
                    .sol
                    .set_accumulated_deposit_capacity_amount(sol_accumulated_deposit_amount)?;
            }

            fund_account.sol.set_withdrawable(sol_withdrawable);
            fund_account
                .sol
                .set_normal_reserve_rate_bps(sol_withdrawal_normal_reserve_rate_bps)?;
            fund_account
                .sol
                .set_normal_reserve_max_amount(sol_withdrawal_normal_reserve_max_amount);

            // all underlying assets should be able to be either withdrawn directly or withdrawn as SOL through unstaking or swap.
            require!(
                fund_account.sol.withdrawable == 1
                    || fund_account
                        .get_supported_tokens_iter()
                        .all(|supported_token| supported_token.token.withdrawable == 1),
                errors::ErrorCode::FundInvalidConfigurationUpdateError
            );
        }

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
        {
            let mut fund_account = self.fund_account.load_mut()?;
            let sol_withdrawable = fund_account.sol.withdrawable == 1;
            let supported_token = fund_account.get_supported_token_mut(token_mint)?;

            supported_token.token.set_depositable(token_depositable);
            supported_token
                .token
                .set_accumulated_deposit_capacity_amount(
                    token_accumulated_deposit_capacity_amount,
                )?;

            if let Some(token_accumulated_deposit_amount) = token_accumulated_deposit_amount {
                supported_token
                    .token
                    .set_accumulated_deposit_amount(token_accumulated_deposit_amount)?;
            }
            supported_token.token.set_withdrawable(token_withdrawable);
            supported_token
                .token
                .set_normal_reserve_rate_bps(token_withdrawal_normal_reserve_rate_bps)?;
            supported_token
                .token
                .set_normal_reserve_max_amount(token_withdrawal_normal_reserve_max_amount);

            if let Some(token_rebalancing_amount) = token_rebalancing_amount {
                supported_token.set_rebalancing_strategy(token_rebalancing_amount)?;
            }
            supported_token.set_sol_allocation_strategy(
                sol_allocation_weight,
                sol_allocation_capacity_amount,
            )?;

            // given underlying asset should be able to be either withdrawn directly or withdrawn as SOL through unstaking or swap.
            require!(
                sol_withdrawable || supported_token.token.withdrawable == 1,
                errors::ErrorCode::FundInvalidConfigurationUpdateError
            );
        }

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_update_restaking_vault_strategy(
        &mut self,
        vault: &Pubkey,
        sol_allocation_weight: u64,
        sol_allocation_capacity_amount: u64,
    ) -> Result<events::FundManagerUpdatedFund> {
        {
            let mut fund_account = self.fund_account.load_mut()?;
            let vault = fund_account.get_restaking_vault_mut(vault)?;
            vault.set_sol_allocation_strategy(
                sol_allocation_weight,
                sol_allocation_capacity_amount,
            )?;
        }

        self.create_fund_manager_updated_fund_event()
    }

    pub fn process_add_restaking_vault_operator(
        &mut self,
        vault: &Pubkey,
        operator: &Pubkey,
    ) -> Result<events::FundManagerUpdatedFund> {
        {
            let mut fund_account = self.fund_account.load_mut()?;
            fund_account
                .get_restaking_vault_mut(vault)?
                .add_delegation(operator)?;
        }

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
        {
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
        }

        self.create_fund_manager_updated_fund_event()
    }

    fn create_fund_manager_updated_fund_event(&self) -> Result<events::FundManagerUpdatedFund> {
        Ok(events::FundManagerUpdatedFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: self.fund_account.key(),
        })
    }
}
