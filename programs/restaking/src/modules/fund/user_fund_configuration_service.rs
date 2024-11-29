use anchor_lang::prelude::*;
use anchor_spl::token_interface::*;

use crate::modules::fund::*;

pub struct UserFundConfigurationService<'info: 'a, 'a> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    user: &'a Signer<'info>,
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
        #[cfg(debug_assertions)]
        require_keys_eq!(user.key(), user_fund_account.user);

        Ok(Self {
            receipt_token_mint,
            user,
            user_fund_account,
        })
    }

    pub fn process_initialize_user_fund_account(
        &mut self,
        bump: u8,
        user_receipt_token_account: &InterfaceAccount<'info, TokenAccount>,
    ) -> Result<()> {
        self.user_fund_account.initialize(
            bump,
            self.receipt_token_mint,
            user_receipt_token_account,
        );
        Ok(())
    }

    pub fn process_update_user_fund_account_if_needed(
        &mut self,
        user_receipt_token_account: &InterfaceAccount<'info, TokenAccount>,
    ) -> Result<()> {
        self.user_fund_account
            .update_if_needed(self.receipt_token_mint, user_receipt_token_account);
        Ok(())
    }
}
