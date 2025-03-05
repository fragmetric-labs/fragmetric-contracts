use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

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
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,

    user: &'a Signer<'info>,
    user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
    user_wrapped_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
    user_fund_account: &'a mut UncheckedAccount<'info>,
    user_reward_account: &'a mut UncheckedAccount<'info>,

    fund_wrap_account: &'a SystemAccount<'info>,
    receipt_token_wrap_account: &'a mut InterfaceAccount<'info, TokenAccount>,
    fund_wrap_account_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
}

impl<'a, 'info> UserFundWrapService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        receipt_token_program: &'a Program<'info, Token2022>,
        wrapped_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        wrapped_token_program: &'a Program<'info, Token>,
        fund_account: &'a mut AccountLoader<'info, FundAccount>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,

        user: &'a Signer<'info>,
        user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
        user_wrapped_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
        user_fund_account: &'a mut UncheckedAccount<'info>,
        user_reward_account: &'a mut UncheckedAccount<'info>,

        fund_wrap_account: &'a SystemAccount<'info>,
        receipt_token_wrap_account: &'a mut InterfaceAccount<'info, TokenAccount>,
        fund_wrap_account_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    ) -> Result<Self> {
        require_keys_eq!(
            wrapped_token_mint.key(),
            fund_account
                .load()?
                .get_wrapped_token()
                .map(|wrapped_token| wrapped_token.mint)
                .unwrap_or_default(),
        );

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
        let receipt_token_supply_before = self.receipt_token_mint.supply;

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
                &[&self.fund_account.load()?.get_seeds()],
            ),
            amount,
        )?;

        self.fund_account
            .load_mut()?
            .reload_receipt_token_supply(self.receipt_token_mint)?;
        let receipt_token_supply = self.receipt_token_mint.supply;
        require_eq!(receipt_token_supply, receipt_token_supply_before);

        let mut user_fund_account_option = self
            .user_fund_account
            .as_account_info()
            .parse_optional_account_boxed::<UserFundAccount>()?;
        if let Some(user_fund_account) = &mut user_fund_account_option {
            user_fund_account.reload_receipt_token_amount(self.user_receipt_token_account)?;
            user_fund_account.exit(&crate::ID)?;
        }

        let mut user_reward_account_option =
            self.user_reward_account
                .as_account_info()
                .parse_optional_account_loader::<UserRewardAccount>()?;
        RewardService::new(self.receipt_token_mint, self.reward_account)?
            .update_reward_pools_token_allocation(
                user_reward_account_option.as_mut(),
                Some(self.fund_wrap_account_reward_account),
                amount,
                None,
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
                &[&self.fund_account.load()?.get_seeds()],
            ),
            amount,
        )?;

        self.fund_account
            .load_mut()?
            .reload_wrapped_token_supply(self.wrapped_token_mint)?;

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

        Ok(Some(self.process_wrap_receipt_token(
            target_balance - self.user_wrapped_token_account.amount,
        )?))
    }

    pub fn process_unwrap_receipt_token(
        &mut self,
        amount: u64,
    ) -> Result<events::UserUnwrappedReceiptToken> {
        require_gte!(self.user_wrapped_token_account.amount, amount);
        let receipt_token_supply_before = self.receipt_token_mint.supply;

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

        self.fund_account
            .load_mut()?
            .reload_wrapped_token_supply(self.wrapped_token_mint)?;

        // first, burn wrapped receipt token (use burn/mint instead of transfer to avoid circular CPI through transfer hook)
        anchor_spl::token_2022::burn(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                anchor_spl::token_2022::Burn {
                    mint: self.receipt_token_mint.to_account_info(),
                    from: self.receipt_token_wrap_account.to_account_info(),
                    authority: self.fund_wrap_account.to_account_info(),
                },
                &[&self.fund_account.load()?.get_wrap_account_seeds()],
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
                &[&self.fund_account.load()?.get_seeds()],
            ),
            amount,
        )?;

        self.fund_account
            .load_mut()?
            .reload_receipt_token_supply(self.receipt_token_mint)?;
        let receipt_token_supply = self.receipt_token_mint.supply;
        require_eq!(receipt_token_supply, receipt_token_supply_before);

        let mut user_fund_account_option = self
            .user_fund_account
            .as_account_info()
            .parse_optional_account_boxed::<UserFundAccount>()?;
        if let Some(user_fund_account) = &mut user_fund_account_option {
            user_fund_account.reload_receipt_token_amount(self.user_receipt_token_account)?;
            user_fund_account.exit(&crate::ID)?;
        }

        let mut user_reward_account_option =
            self.user_reward_account
                .as_account_info()
                .parse_optional_account_loader::<UserRewardAccount>()?;
        RewardService::new(self.receipt_token_mint, self.reward_account)?
            .update_reward_pools_token_allocation(
                Some(self.fund_wrap_account_reward_account),
                user_reward_account_option.as_mut(),
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
