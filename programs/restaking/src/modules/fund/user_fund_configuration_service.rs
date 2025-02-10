use anchor_lang::prelude::*;
use anchor_spl::token_interface::*;

use crate::utils::{AccountInfoExt, AsAccountInfo, PDASeeds, SystemProgramExt};
use crate::{events, modules};

use super::*;

pub struct UserFundConfigurationService<'a, 'info> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    user_fund_account: &'a mut Account<'info, UserFundAccount>,
    user_receipt_token_account: &'a InterfaceAccount<'info, TokenAccount>,
}

impl Drop for UserFundConfigurationService<'_, '_> {
    fn drop(&mut self) {
        self.user_fund_account.exit(&crate::ID).unwrap();
    }
}

impl<'a, 'info> UserFundConfigurationService<'a, 'info> {
    pub fn process_create_user_fund_account_idempotent(
        system_program: &Program<'info, System>,
        receipt_token_mint: &mut InterfaceAccount<'info, Mint>,

        user: &Signer<'info>,
        user_receipt_token_account: &InterfaceAccount<'info, TokenAccount>,
        user_fund_account: &mut UncheckedAccount<'info>,
        user_fund_account_bump: u8,

        _desired_account_size: Option<u32>, // reserved
    ) -> Result<Option<events::UserCreatedOrUpdatedFundAccount>> {
        if !user_fund_account.is_initialized() {
            system_program.initialize_account(
                &user_fund_account,
                user,
                &[&[
                    UserFundAccount::SEED,
                    receipt_token_mint.key().as_ref(),
                    user.key().as_ref(),
                    &[user_fund_account_bump],
                ]],
                8 + UserFundAccount::INIT_SPACE,
                None,
                &crate::ID,
            )?;

            let mut user_fund_account_parsed = Account::<UserFundAccount>::try_from_unchecked(
                user_fund_account.as_account_info(),
            )?;

            let event = UserFundConfigurationService::new(
                receipt_token_mint,
                &user,
                &mut user_fund_account_parsed,
                user_receipt_token_account,
            )?
            .process_initialize_user_fund_account(user_fund_account_bump)?;

            Ok(event)
        } else {
            system_program.expand_account_size_if_needed(
                user_fund_account,
                user,
                &[],
                8 + UserFundAccount::INIT_SPACE,
                None,
            )?;

            let mut user_fund_account_parsed =
                Account::<UserFundAccount>::try_from(user_fund_account.as_account_info())?;

            require_eq!(user_fund_account_bump, user_fund_account_parsed.get_bump());

            let event = UserFundConfigurationService::new(
                receipt_token_mint,
                user,
                &mut user_fund_account_parsed,
                user_receipt_token_account,
            )?
            .process_update_user_fund_account_if_needed()?;

            Ok(event)
        }
    }

    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        user: &Signer,
        user_fund_account: &'a mut Account<'info, UserFundAccount>,
        user_receipt_token_account: &'a InterfaceAccount<'info, TokenAccount>,
    ) -> Result<Self> {
        require_keys_eq!(user_receipt_token_account.owner, user.key());

        Ok(Self {
            receipt_token_mint,
            user_fund_account,
            user_receipt_token_account,
        })
    }

    pub fn process_initialize_user_fund_account(
        &mut self,
        user_fund_account_bump: u8,
    ) -> Result<Option<events::UserCreatedOrUpdatedFundAccount>> {
        if self.user_fund_account.initialize(
            user_fund_account_bump,
            self.receipt_token_mint,
            self.user_receipt_token_account,
        ) {
            Ok(Some(events::UserCreatedOrUpdatedFundAccount {
                receipt_token_mint: self.receipt_token_mint.key(),
                user_fund_account: self.user_fund_account.key(),
                receipt_token_amount: self.user_receipt_token_account.amount,
                created: true,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn process_update_user_fund_account_if_needed(
        &mut self,
    ) -> Result<Option<events::UserCreatedOrUpdatedFundAccount>> {
        let initializing = self.user_fund_account.is_initializing();
        if self
            .user_fund_account
            .update_if_needed(self.receipt_token_mint, self.user_receipt_token_account)
        {
            Ok(Some(events::UserCreatedOrUpdatedFundAccount {
                receipt_token_mint: self.receipt_token_mint.key(),
                user_fund_account: self.user_fund_account.key(),
                receipt_token_amount: self.user_receipt_token_account.amount,
                created: initializing,
            }))
        } else {
            Ok(None)
        }
    }
}
