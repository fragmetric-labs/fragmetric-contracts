use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use anchor_spl::token_interface::*;

use crate::events;
use crate::modules::fund::*;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::{fund, pricing};

pub struct FundConfigurationService<'info> {
    _phantom: std::marker::PhantomData<&'info ()>,
}

impl<'info> FundConfigurationService<'info> {
    pub fn process_initialize_fund_account(
        receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
        fund_account: &mut Account<'info, FundAccount>,
        
        receipt_token_program: &Program<'info, Token2022>,
        receipt_token_mint_current_authority: &Signer<'info>,
        bump: u8,
    ) -> Result<()> {
        fund_account.initialize(
            bump,
            receipt_token_mint.key(),
            receipt_token_mint.decimals,
        );

        // set token mint authority
        token_2022::set_authority(
            CpiContext::new(
                receipt_token_program.to_account_info(),
                token_2022::SetAuthority {
                    current_authority: receipt_token_mint_current_authority.to_account_info(),
                    account_or_mint: receipt_token_mint.to_account_info(),
                },
            ),
            spl_token_2022::instruction::AuthorityType::MintTokens,
            Some(fund_account.key()),
        )
    }

    pub fn process_update_fund_account_if_needed(
        receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
        fund_account: &mut Account<'info, FundAccount>,
    ) -> Result<()> {
        fund_account.update_if_needed(receipt_token_mint);
        Ok(())
    }

    pub fn process_update_sol_capacity_amount(
        receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
        fund_account: &mut Account<'info, FundAccount>,
        
        capacity_amount: u64,
    ) -> Result<()> {
        fund_account.set_sol_capacity_amount(capacity_amount)?;
        Self::emit_fund_manager_updated_fund_event(
            receipt_token_mint,
            fund_account,
        )
    }

    pub fn process_update_supported_token_capacity_amount(
        receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
        fund_account: &mut Account<'info, FundAccount>,
        
        token_mint: &Pubkey,
        capacity_amount: u64,
    ) -> Result<()> {
        fund_account
            .get_supported_token_mut(token_mint)?
            .set_capacity_amount(capacity_amount)?;

        Self::emit_fund_manager_updated_fund_event(
            receipt_token_mint,
            fund_account,
        )
    }

    pub fn process_update_withdrawal_enabled_flag(
        receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
        fund_account: &mut Account<'info, FundAccount>,
        
        enabled: bool,
    ) -> Result<()> {
        fund_account
            .withdrawal
            .set_withdrawal_enabled_flag(enabled);

        Self::emit_fund_manager_updated_fund_event(
            receipt_token_mint,
            fund_account,
        )
    }

    pub fn process_update_sol_withdrawal_fee_rate(
        receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
        fund_account: &mut Account<'info, FundAccount>,
        
        sol_withdrawal_fee_rate: u16,
    ) -> Result<()> {
        fund_account
            .withdrawal
            .set_sol_withdrawal_fee_rate(sol_withdrawal_fee_rate)?;

        Self::emit_fund_manager_updated_fund_event(
            receipt_token_mint,
            fund_account,
        )
    }

    pub fn process_update_batch_processing_threshold(
        receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
        fund_account: &mut Account<'info, FundAccount>,
        
        amount: Option<u64>,
        duration: Option<i64>,
    ) -> Result<()> {
        fund_account
            .withdrawal
            .set_batch_processing_threshold(amount, duration);

        Self::emit_fund_manager_updated_fund_event(
            receipt_token_mint,
            fund_account,
        )
    }

    pub fn process_add_supported_token(
        receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
        fund_account: &mut Account<'info, FundAccount>,
        
        fund_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        supported_token_mint: &InterfaceAccount<Mint>,
        supported_token_program: &Interface<TokenInterface>,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        require_keys_eq!(fund_supported_token_account.owner, fund_account.key(),);
        require_keys_eq!(
            fund_supported_token_account.mint,
            supported_token_mint.key()
        );
        require_keys_eq!(
            *supported_token_mint.to_account_info().owner,
            supported_token_program.key()
        );

        fund_account.add_supported_token(
            supported_token_mint.key(),
            supported_token_program.key(),
            supported_token_mint.decimals,
            capacity_amount,
            pricing_source,
        )?;

        // validate pricing source
        FundService::new(receipt_token_mint, fund_account)?
            .new_pricing_service(&pricing_sources)?;

        Self::emit_fund_manager_updated_fund_event(
            receipt_token_mint,
            fund_account,
        )
    }

    fn emit_fund_manager_updated_fund_event(
        receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
        fund_account: &mut Account<'info, FundAccount>,
    ) -> Result<()> {
        emit!(events::FundManagerUpdatedFund {
            receipt_token_mint: receipt_token_mint.key(),
            fund_account: FundAccountInfo::from(fund_account, receipt_token_mint),
        });

        Ok(())
    }
}
