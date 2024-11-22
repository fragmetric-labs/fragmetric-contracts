use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use anchor_spl::token_interface::*;

use crate::events;
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
        self.fund_account.initialize(
            bump,
            self.receipt_token_mint,
        );

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

    pub fn process_update_withdrawal_enabled_flag(&mut self, enabled: bool) -> Result<()> {
        self.fund_account
            .withdrawal
            .set_withdrawal_enabled_flag(enabled);

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

    pub fn process_update_batch_processing_threshold(
        &mut self,
        amount: Option<u64>,
        duration: Option<i64>,
    ) -> Result<()> {
        self.fund_account
            .withdrawal
            .set_batch_processing_threshold(amount, duration);

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

    fn emit_fund_manager_updated_fund_event(&self) -> Result<()> {
        emit!(events::FundManagerUpdatedFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: FundAccountInfo::from(self.fund_account),
        });

        Ok(())
    }
}
