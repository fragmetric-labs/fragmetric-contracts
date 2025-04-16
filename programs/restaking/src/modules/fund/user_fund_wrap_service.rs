use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::reward::*;
use crate::utils::{AccountInfoExt, AsAccountInfo, PDASeeds};

use super::*;

pub struct UserFundWrapService<'a, 'info> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    receipt_token_program: &'a Program<'info, Token2022>,
    wrapped_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    wrapped_token_program: &'a Program<'info, Token>,
    fund_account: &'a mut AccountLoader<'info, FundAccount>,
    reward_account: &'a AccountLoader<'info, RewardAccount>,

    user: &'a Signer<'info>,
    user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
    user_wrapped_token_account: &'a InterfaceAccount<'info, TokenAccount>,
    user_fund_account: &'a mut UncheckedAccount<'info>,
    user_reward_account: &'a UncheckedAccount<'info>,

    fund_wrap_account: &'a SystemAccount<'info>,
    receipt_token_wrap_account: &'a InterfaceAccount<'info, TokenAccount>,
    fund_wrap_account_reward_account: &'a AccountLoader<'info, UserRewardAccount>,
}

impl<'a, 'info> UserFundWrapService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        receipt_token_program: &'a Program<'info, Token2022>,
        wrapped_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        wrapped_token_program: &'a Program<'info, Token>,
        fund_account: &'a mut AccountLoader<'info, FundAccount>,
        reward_account: &'a AccountLoader<'info, RewardAccount>,

        user: &'a Signer<'info>,
        user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
        user_wrapped_token_account: &'a InterfaceAccount<'info, TokenAccount>,
        user_fund_account: &'a mut UncheckedAccount<'info>,
        user_reward_account: &'a UncheckedAccount<'info>,

        fund_wrap_account: &'a SystemAccount<'info>,
        receipt_token_wrap_account: &'a InterfaceAccount<'info, TokenAccount>,
        fund_wrap_account_reward_account: &'a AccountLoader<'info, UserRewardAccount>,
    ) -> Result<Self> {
        let wrapped_token_mint_address = *fund_account
            .load()?
            .get_wrapped_token_mint_address()
            .ok_or_else(|| error!(ErrorCode::FundWrappedTokenNotSetError))?;
        require_keys_eq!(wrapped_token_mint.key(), wrapped_token_mint_address);

        Ok(Self {
            receipt_token_mint,
            receipt_token_program,
            wrapped_token_mint,
            wrapped_token_program,
            fund_account,
            reward_account,
            user,
            user_receipt_token_account,
            user_wrapped_token_account,
            user_fund_account,
            user_reward_account,
            fund_wrap_account,
            receipt_token_wrap_account,
            fund_wrap_account_reward_account,
        })
    }

    pub fn process_wrap_receipt_token(
        &mut self,
        amount: u64,
    ) -> Result<events::UserWrappedReceiptToken> {
        require_gte!(self.user_receipt_token_account.amount, amount);

        let fund_account = self.fund_account.load()?;

        // first, burn user receipt token (use burn/mint instead of transfer to avoid circular CPI through transfer hook)
        anchor_spl::token_2022::burn(
            CpiContext::new(
                self.receipt_token_program.to_account_info(),
                anchor_spl::token_2022::Burn {
                    mint: self.receipt_token_mint.to_account_info(),
                    from: self.user_receipt_token_account.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            amount,
        )?;

        // then, mint receipt token to fund's wrap account
        anchor_spl::token_2022::mint_to(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                anchor_spl::token_2022::MintTo {
                    mint: self.receipt_token_mint.to_account_info(),
                    to: self.receipt_token_wrap_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[&fund_account.get_seeds()],
            ),
            amount,
        )?;

        // mint wrapped token to user
        anchor_spl::token::mint_to(
            CpiContext::new_with_signer(
                self.wrapped_token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: self.wrapped_token_mint.to_account_info(),
                    to: self.user_wrapped_token_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[&fund_account.get_seeds()],
            ),
            amount,
        )?;

        drop(fund_account);
        let mut fund_account = self.fund_account.load_mut()?;

        // update receipt token
        fund_account.reload_receipt_token_supply(self.receipt_token_mint)?;

        let mut user_fund_account_option = self
            .user_fund_account
            .as_account_info()
            .parse_optional_account_boxed::<UserFundAccount>()?;
        if let Some(user_fund_account) = &mut user_fund_account_option {
            user_fund_account.reload_receipt_token_amount(self.user_receipt_token_account)?;
            user_fund_account.exit(&crate::ID)?;
        }

        // update wrapped token
        let wrapped_token = fund_account
            .get_wrapped_token_mut()
            .ok_or_else(|| error!(ErrorCode::FundWrappedTokenNotSetError))?;
        let old_wrapped_token_retained_amount =
            wrapped_token.reload_supply(self.wrapped_token_mint)?;

        // update reward
        let reward_service = RewardService::new(self.receipt_token_mint, self.reward_account)?;

        // user lost `amount`
        let user_reward_account_option = self
            .user_reward_account
            .as_account_info()
            .parse_optional_account_loader::<UserRewardAccount>()?;
        reward_service.update_reward_pools_token_allocation(
            user_reward_account_option.as_ref(),
            None,
            amount,
            None,
        )?;

        // fund_wrap_account gained ∆wrapped_token_retained_amount
        if wrapped_token.retained_amount > old_wrapped_token_retained_amount {
            let wrapped_token_retained_amount_delta =
                wrapped_token.retained_amount - old_wrapped_token_retained_amount;
            reward_service.update_reward_pools_token_allocation(
                None,
                Some(self.fund_wrap_account_reward_account),
                wrapped_token_retained_amount_delta,
                None,
            )?;
        }

        // event
        Ok(events::UserWrappedReceiptToken {
            receipt_token_mint: self.receipt_token_mint.key(),
            wrapped_token_mint: self.wrapped_token_mint.key(),
            fund_account: self.fund_account.key(),
            user: self.user.key(),
            user_receipt_token_account: self.user_receipt_token_account.key(),
            user_wrapped_token_account: self.user_wrapped_token_account.key(),
            updated_user_fund_account: user_fund_account_option.map(|account| account.key()),
            updated_user_reward_account: user_reward_account_option.map(|account| account.key()),
            updated_fund_wrap_account_reward_account: self.fund_wrap_account_reward_account.key(),
            wrapped_receipt_token_amount: amount,
        })
    }

    pub fn process_wrap_receipt_token_if_needed(
        &mut self,
        target_balance: u64,
    ) -> Result<Option<events::UserWrappedReceiptToken>> {
        if self.user_wrapped_token_account.amount >= target_balance {
            return Ok(None);
        }

        let required_amount = target_balance - self.user_wrapped_token_account.amount;
        Ok(Some(self.process_wrap_receipt_token(required_amount)?))
    }

    pub fn process_unwrap_receipt_token(
        &mut self,
        amount: u64,
    ) -> Result<events::UserUnwrappedReceiptToken> {
        require_gte!(self.user_wrapped_token_account.amount, amount);

        let fund_account = self.fund_account.load()?;

        // burn wrapped token from user
        anchor_spl::token::burn(
            CpiContext::new(
                self.wrapped_token_program.to_account_info(),
                anchor_spl::token::Burn {
                    mint: self.wrapped_token_mint.to_account_info(),
                    from: self.user_wrapped_token_account.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            amount,
        )?;

        // first, burn wrapped receipt token (use burn/mint instead of transfer to avoid circular CPI through transfer hook)
        anchor_spl::token_2022::burn(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                anchor_spl::token_2022::Burn {
                    mint: self.receipt_token_mint.to_account_info(),
                    from: self.receipt_token_wrap_account.to_account_info(),
                    authority: self.fund_wrap_account.to_account_info(),
                },
                &[&fund_account.get_wrap_account_seeds()],
            ),
            amount,
        )?;

        // then, mint receipt token to fund's wrap account
        anchor_spl::token_2022::mint_to(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                anchor_spl::token_2022::MintTo {
                    mint: self.receipt_token_mint.to_account_info(),
                    to: self.user_receipt_token_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[&fund_account.get_seeds()],
            ),
            amount,
        )?;

        drop(fund_account);
        let mut fund_account = self.fund_account.load_mut()?;

        // update receipt token
        fund_account.reload_receipt_token_supply(self.receipt_token_mint)?;

        let mut user_fund_account_option = self
            .user_fund_account
            .as_account_info()
            .parse_optional_account_boxed::<UserFundAccount>()?;
        if let Some(user_fund_account) = &mut user_fund_account_option {
            user_fund_account.reload_receipt_token_amount(self.user_receipt_token_account)?;
            user_fund_account.exit(&crate::ID)?;
        }

        // update wrapped token
        let wrapped_token = fund_account
            .get_wrapped_token_mut()
            .ok_or_else(|| error!(ErrorCode::FundWrappedTokenNotSetError))?;
        let old_wrapped_token_retained_amount =
            wrapped_token.reload_supply(self.wrapped_token_mint)?;

        // update reward
        let reward_service = RewardService::new(self.receipt_token_mint, self.reward_account)?;

        // fund_wrap_account lost ∆wrapped_token_retained_amount
        if old_wrapped_token_retained_amount > wrapped_token.retained_amount {
            let wrapped_token_retained_amount_delta =
                old_wrapped_token_retained_amount - wrapped_token.retained_amount;
            reward_service.update_reward_pools_token_allocation(
                Some(self.fund_wrap_account_reward_account),
                None,
                wrapped_token_retained_amount_delta,
                None,
            )?;
        }

        // user gained `amount`
        let user_reward_account_option = self
            .user_reward_account
            .as_account_info()
            .parse_optional_account_loader::<UserRewardAccount>()?;
        reward_service.update_reward_pools_token_allocation(
            None,
            user_reward_account_option.as_ref(),
            amount,
            None,
        )?;

        // event
        Ok(events::UserUnwrappedReceiptToken {
            receipt_token_mint: self.receipt_token_mint.key(),
            wrapped_token_mint: self.wrapped_token_mint.key(),
            fund_account: self.fund_account.key(),
            user: self.user.key(),
            user_receipt_token_account: self.user_receipt_token_account.key(),
            user_wrapped_token_account: self.user_wrapped_token_account.key(),
            updated_user_fund_account: user_fund_account_option.map(|account| account.key()),
            updated_user_reward_account: user_reward_account_option.map(|account| account.key()),
            updated_fund_wrap_account_reward_account: self.fund_wrap_account_reward_account.key(),
            unwrapped_receipt_token_amount: amount,
        })
    }
}
