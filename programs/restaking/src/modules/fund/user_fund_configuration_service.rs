use anchor_lang::prelude::*;
use anchor_spl::token_interface::*;

use crate::events;

use super::*;

pub struct UserFundConfigurationService<'info: 'a, 'a> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    _user: &'a Signer<'info>,
    user_fund_account: &'a mut Account<'info, UserFundAccount>,
}

impl Drop for UserFundConfigurationService<'_, '_> {
    fn drop(&mut self) {
        self.user_fund_account.exit(&crate::ID).unwrap();
    }
}

impl<'info, 'a> UserFundConfigurationService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        user: &'a Signer<'info>,
        user_fund_account: &'a mut Account<'info, UserFundAccount>,
    ) -> Result<Self> {
        Ok(Self {
            receipt_token_mint,
            _user: user,
            user_fund_account,
        })
    }

    pub fn process_initialize_user_fund_account(
        &mut self,
        user_fund_account_bump: u8,
        user_receipt_token_account: &InterfaceAccount<'info, TokenAccount>,
    ) -> Result<Option<events::UserCreatedOrUpdatedFundAccount>> {
        if self.user_fund_account.initialize(
            user_fund_account_bump,
            self.receipt_token_mint,
            user_receipt_token_account,
        ) {
            Ok(Some(events::UserCreatedOrUpdatedFundAccount {
                receipt_token_mint: self.receipt_token_mint.key(),
                user_fund_account: self.user_fund_account.key(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn process_update_user_fund_account_if_needed(
        &mut self,
        user_receipt_token_account: &InterfaceAccount<'info, TokenAccount>,
    ) -> Result<Option<events::UserCreatedOrUpdatedFundAccount>> {
        if self
            .user_fund_account
            .update_if_needed(self.receipt_token_mint, user_receipt_token_account)
        {
            Ok(Some(events::UserCreatedOrUpdatedFundAccount {
                receipt_token_mint: self.receipt_token_mint.key(),
                user_fund_account: self.user_fund_account.key(),
            }))
        } else {
            Ok(None)
        }
    }
}
