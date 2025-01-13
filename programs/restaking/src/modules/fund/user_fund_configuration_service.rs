use anchor_lang::prelude::*;
use anchor_spl::token_interface::*;

use crate::utils::{AccountInfoExt, AsAccountInfo, PDASeeds, SystemProgramExt};
use crate::{events, modules};

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
    pub fn process_create_user_fund_account_idempotent<'c>(
        system_program: &'c Program<'info, System>,
        receipt_token_mint: &'c mut InterfaceAccount<'info, Mint>,

        user: &'c Signer<'info>,
        user_receipt_token_account: &'c InterfaceAccount<'info, TokenAccount>,
        user_fund_account: &'c mut UncheckedAccount<'info>,
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
            )?
            .process_initialize_user_fund_account(
                user_fund_account_bump,
                user_receipt_token_account,
            )?;

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
            )?
            .process_update_user_fund_account_if_needed(user_receipt_token_account)?;

            Ok(event)
        }
    }

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
