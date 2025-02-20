use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::modules::reward::*;

use super::*;

pub struct FundWrapAccountRewardConfigurationService<'a, 'info> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    fund_account: &'a AccountLoader<'info, FundAccount>,
}

impl<'a, 'info> FundWrapAccountRewardConfigurationService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        fund_account: &'a AccountLoader<'info, FundAccount>,
    ) -> Result<Self> {
        Ok(Self {
            receipt_token_mint,
            fund_account,
        })
    }

    pub fn process_initialize_fund_wrap_account_reward_account(
        &self,
        fund_wrap_account: &SystemAccount,
        receipt_token_wrap_account: &InterfaceAccount<'info, TokenAccount>,
        reward_account: &mut AccountLoader<'info, RewardAccount>,
        fund_wrap_account_reward_account: &mut AccountLoader<'info, UserRewardAccount>,
        fund_wrap_account_reward_account_bump: u8,
    ) -> Result<()> {
        UserRewardConfigurationService::new_with_user_seeds(
            self.receipt_token_mint,
            fund_wrap_account,
            &self.fund_account.load()?.get_wrap_account_seeds(),
            receipt_token_wrap_account,
            reward_account,
            fund_wrap_account_reward_account,
        )?
        .process_initialize_user_reward_account(fund_wrap_account_reward_account_bump)?;

        Ok(())
    }

    pub fn process_update_fund_wrap_account_reward_account_if_needed(
        &self,
        payer: &Signer<'info>,
        system_program: &Program<'info, System>,
        fund_wrap_account: &SystemAccount,
        receipt_token_wrap_account: &InterfaceAccount<'info, TokenAccount>,
        reward_account: &mut AccountLoader<'info, RewardAccount>,
        fund_wrap_account_reward_account: &mut AccountLoader<'info, UserRewardAccount>,
        desired_account_size: Option<u32>,
    ) -> Result<()> {
        UserRewardConfigurationService::new_with_user_seeds(
            self.receipt_token_mint,
            fund_wrap_account,
            &self.fund_account.load()?.get_wrap_account_seeds(),
            receipt_token_wrap_account,
            reward_account,
            fund_wrap_account_reward_account,
        )?
        .process_update_user_reward_account_if_needed(
            payer,
            system_program,
            desired_account_size,
        )?;

        Ok(())
    }
}
