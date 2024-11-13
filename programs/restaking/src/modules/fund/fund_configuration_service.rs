use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use anchor_spl::token_interface::*;

use crate::events;
use crate::modules::fund::*;
use crate::modules::pricing::TokenPricingSource;

pub struct FundConfigurationService<'info, 'a>
where
    'info: 'a,
{
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    fund_account: &'a mut Account<'info, FundAccount>,
}

impl<'info, 'a> FundConfigurationService<'info, 'a> {
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
        receipt_token_program: &'a Program<'info, Token2022>,
        admin: &Signer<'info>,
        bump: u8,
    ) -> Result<()> {
        self.fund_account
            .initialize(bump, self.receipt_token_mint.key());

        // set token mint authority
        token_2022::set_authority(
            CpiContext::new(
                receipt_token_program.to_account_info(),
                token_2022::SetAuthority {
                    current_authority: admin.to_account_info(),
                    account_or_mint: self.receipt_token_mint.to_account_info(),
                },
            ),
            spl_token_2022::instruction::AuthorityType::MintTokens,
            Some(self.fund_account.key()),
        )
    }

    pub fn process_update_fund_account_if_needed(&mut self) -> Result<()> {
        self.fund_account
            .update_if_needed(self.receipt_token_mint.key());
        Ok(())
    }

    pub fn process_update_sol_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        self.fund_account.set_sol_capacity_amount(capacity_amount)?;
        self.emit_fund_manager_updated_fund_event()
    }

    pub fn process_update_supported_token_capacity_amount(
        &mut self,
        token: Pubkey,
        capacity_amount: u64,
    ) -> Result<()> {
        self.fund_account
            .get_supported_token_mut(token)?
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
        supported_token_mint: &InterfaceAccount<Mint>,
        supported_token_program: &Interface<TokenInterface>,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        self.fund_account.add_supported_token(
            supported_token_mint.key(),
            supported_token_program.key(),
            supported_token_mint.decimals,
            capacity_amount,
            pricing_source,
        )?;

        // TODO: get pricing service or?
        // validate pricing source
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .update_asset_prices(pricing_sources)?;

        self.emit_fund_manager_updated_fund_event()
    }

    fn emit_fund_manager_updated_fund_event(&self) -> Result<()> {
        emit!(events::FundManagerUpdatedFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: FundAccountInfo::from(self.fund_account, self.receipt_token_mint),
        });

        Ok(())
    }
}
