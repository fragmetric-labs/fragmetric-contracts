use anchor_lang::prelude::*;
use anchor_spl::token::accessor::amount;
use anchor_spl::token::Token;
use anchor_spl::token_2022;
use anchor_spl::token_interface::*;

use crate::events;
use crate::modules::normalization::{NormalizedTokenPoolAccount, NormalizedTokenPoolService};
use crate::modules::pricing::TokenPricingSource;

use super::*;

pub struct FundConfigurationService<'info: 'a, 'a> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    fund_account: &'a mut Account<'info, FundAccount>,
}

impl Drop for FundConfigurationService<'_, '_> {
    fn drop(&mut self) {
        self.fund_account.exit(&crate::ID).unwrap();
    }
}

impl<'info: 'a, 'a> FundConfigurationService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        fund_account: &'a mut Account<'info, FundAccount>,
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
        bump: u8,
    ) -> Result<()> {
        self.fund_account.initialize(bump, self.receipt_token_mint);

        // set token mint authority
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
        )
    }

    pub fn process_update_fund_account_if_needed(&mut self) -> Result<()> {
        self.fund_account.update_if_needed(self.receipt_token_mint);
        Ok(())
    }

    pub fn process_update_sol_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        self.fund_account.set_sol_capacity_amount(capacity_amount)?;
        self.emit_fund_manager_updated_fund_event()
    }

    pub fn process_update_supported_token_capacity_amount(
        &mut self,
        token_mint: &Pubkey,
        capacity_amount: u64,
    ) -> Result<()> {
        self.fund_account
            .get_supported_token_mut(token_mint)?
            .set_capacity_amount(capacity_amount)?;

        self.emit_fund_manager_updated_fund_event()
    }

    pub fn process_update_withdrawal_enabled(&mut self, enabled: bool) -> Result<()> {
        self.fund_account.withdrawal.set_withdrawal_enabled(enabled);

        self.emit_fund_manager_updated_fund_event()
    }

    pub fn process_update_sol_withdrawal_fee_rate(
        &mut self,
        sol_withdrawal_fee_rate: u16,
    ) -> Result<()> {
        self.fund_account
            .withdrawal
            .set_sol_withdrawal_fee_rate(sol_withdrawal_fee_rate)?;

        self.emit_fund_manager_updated_fund_event()
    }

    pub fn process_update_batch_threshold(
        &mut self,
        creation_interval_seconds: i64,
        processing_interval_seconds: i64,
    ) -> Result<()> {
        self.fund_account
            .withdrawal
            .set_batch_threshold(creation_interval_seconds, processing_interval_seconds)?;

        self.emit_fund_manager_updated_fund_event()
    }

    pub fn process_add_supported_token(
        &mut self,
        fund_supported_token_account: &InterfaceAccount<TokenAccount>,
        supported_token_mint: &InterfaceAccount<Mint>,
        supported_token_program: &Interface<TokenInterface>,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        require_keys_eq!(fund_supported_token_account.owner, self.fund_account.key());
        require_keys_eq!(
            fund_supported_token_account.mint,
            supported_token_mint.key()
        );
        require_keys_eq!(
            *supported_token_mint.to_account_info().owner,
            supported_token_program.key()
        );

        self.fund_account.add_supported_token(
            supported_token_mint.key(),
            supported_token_program.key(),
            supported_token_mint.decimals,
            capacity_amount,
            pricing_source,
        )?;

        // validate pricing source
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .new_pricing_service(pricing_sources)?;

        self.emit_fund_manager_updated_fund_event()
    }

    pub fn process_set_normalized_token(
        &mut self,
        fund_normalized_token_account: &InterfaceAccount<TokenAccount>,
        normalized_token_mint: &mut InterfaceAccount<'info, Mint>,
        normalized_token_program: &Program<'info, Token>,
        normalized_token_pool: &mut Account<'info, NormalizedTokenPoolAccount>,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<()> {
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
        self.fund_account.set_normalized_token(
            normalized_token_mint.key(),
            normalized_token_program.key(),
            normalized_token_mint.decimals,
            normalized_token_pool.key(),
            fund_normalized_token_account.amount,
        )?;

        // do pricing as a validation
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .new_pricing_service(pricing_sources)?;

        self.emit_fund_manager_updated_fund_event()
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
    ) -> Result<()> {
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

        require_keys_eq!(*vault.to_account_info().owner, vault_program.key());

        self.fund_account.add_restaking_vault(
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

        self.emit_fund_manager_updated_fund_event()
    }

    fn emit_fund_manager_updated_fund_event(&self) -> Result<()> {
        emit!(events::FundManagerUpdatedFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: FundAccountInfo::from(self.fund_account),
        });

        Ok(())
    }
}
