use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use anchor_spl::token_interface::*;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::events;
use crate::modules::fund::*;
use crate::modules::pricing::TokenPricingSource;

pub struct FundUserConfigurationService<'info, 'a> where 'info : 'a {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    user: &'a Signer<'info>,
    user_fund_account: &'a mut Account<'info, UserFundAccount>,
}

impl<'info, 'a> FundUserConfigurationService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        user: &'a Signer<'info>,
        user_fund_account: &'a mut Account<'info, UserFundAccount>,
    ) -> Result<Self> {
        Ok(Self {
            receipt_token_mint,
            user,
            user_fund_account,
        })
    }

    pub fn process_initialize_user_fund_account(
        &mut self,
        bump: u8,
    ) -> Result<()> {
        self.user_fund_account.initialize(bump, self.receipt_token_mint.key(), self.user.key());
        Ok(())
    }

    pub fn process_update_user_fund_account_if_needed(
        &mut self,
    ) -> Result<()> {
        self.user_fund_account.update_if_needed(self.receipt_token_mint.key(), self.user.key());
        Ok(())
    }
}

