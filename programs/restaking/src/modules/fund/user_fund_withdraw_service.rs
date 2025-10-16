use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::{token_2022, token_interface};

use crate::modules::fund::{DepositMetadata, FundAccount, FundService, UserFundAccount};
use crate::modules::reward::{RewardAccount, RewardService, UserRewardAccount};
use crate::utils::{AccountInfoExt, AsAccountInfo, PDASeeds};
use crate::{errors, events};

pub struct UserFundWithdrawService<'a, 'info> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    receipt_token_program: &'a Program<'info, Token2022>,
    fund_account: &'a mut AccountLoader<'info, FundAccount>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,

    user: &'a Signer<'info>,
    user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
    user_fund_account: &'a mut Account<'info, UserFundAccount>,
    user_reward_account: &'a mut UncheckedAccount<'info>,

    _current_slot: u64,
    current_timestamp: i64,
}

impl Drop for UserFundWithdrawService<'_, '_> {
    fn drop(&mut self) {
        self.fund_account.exit(&crate::ID).unwrap();
        self.user_fund_account.exit(&crate::ID).unwrap();
    }
}

impl<'a, 'info> UserFundWithdrawService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        receipt_token_program: &'a Program<'info, Token2022>,
        fund_account: &'a mut AccountLoader<'info, FundAccount>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
        user: &'a Signer<'info>,
        user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
        user_fund_account: &'a mut Account<'info, UserFundAccount>,
        user_reward_account: &'a mut UncheckedAccount<'info>,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            receipt_token_program,
            fund_account,
            reward_account,
            user,
            user_receipt_token_account,
            user_fund_account,
            user_reward_account,
            _current_slot: clock.slot,
            current_timestamp: clock.unix_timestamp,
        })
    }

    pub fn process_request_withdrawal(
        &mut self,
        receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
        supported_token_mint: Option<Pubkey>,
        pricing_sources: &'info [AccountInfo<'info>],
        receipt_token_amount: u64,
    ) -> Result<events::UserRequestedWithdrawalFromFund> {
        // validate user receipt token account balance
        require_gte!(self.user_receipt_token_account.amount, receipt_token_amount);
        require_gt!(receipt_token_amount, 0);

        // update fund value before processing request
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .new_pricing_service(pricing_sources, true)?;

        // create a user withdrawal request
        let withdrawal_request = self.fund_account.load_mut()?.create_withdrawal_request(
            supported_token_mint,
            receipt_token_amount,
            self.current_timestamp,
        )?;

        // requested receipt_token_amount can be reduced based on the status of the underlying asset.
        require_gte!(
            receipt_token_amount,
            withdrawal_request.receipt_token_amount
        );
        let receipt_token_amount = withdrawal_request.receipt_token_amount;
        let batch_id = withdrawal_request.batch_id;
        let request_id = withdrawal_request.request_id;

        self.user_fund_account
            .push_withdrawal_request(withdrawal_request)?;

        // lock requested user receipt token amount
        // first, burn user receipt token (use burn/mint instead of transfer to avoid circular CPI through transfer hook)
        token_2022::burn(
            CpiContext::new(
                self.receipt_token_program.to_account_info(),
                token_2022::Burn {
                    mint: self.receipt_token_mint.to_account_info(),
                    from: self.user_receipt_token_account.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            receipt_token_amount,
        )?;

        // then, mint receipt token to fund's lock account
        token_2022::mint_to(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                token_2022::MintTo {
                    mint: self.receipt_token_mint.to_account_info(),
                    to: receipt_token_lock_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[self.fund_account.load()?.get_seeds().as_ref()],
            ),
            receipt_token_amount,
        )?;

        self.fund_account
            .load_mut()?
            .reload_receipt_token_supply(self.receipt_token_mint)?;

        self.user_fund_account
            .reload_receipt_token_amount(self.user_receipt_token_account)?;

        let user_reward_account_option = self
            .user_reward_account
            .as_account_info()
            .parse_optional_account_loader::<UserRewardAccount>()?;

        // reduce user's reward accrual rate
        let updated_user_reward_accounts =
            RewardService::new(self.receipt_token_mint, self.reward_account)?
                .update_reward_pools_token_allocation(
                    user_reward_account_option.as_ref(),
                    None,
                    receipt_token_amount,
                    None,
                )?;

        // log withdrawal request event
        Ok(events::UserRequestedWithdrawalFromFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: self.fund_account.key(),
            supported_token_mint,
            updated_user_reward_accounts,

            user: self.user.key(),
            user_receipt_token_account: self.user_receipt_token_account.key(),
            user_fund_account: self.user_fund_account.key(),

            batch_id,
            request_id,
            requested_receipt_token_amount: receipt_token_amount,
        })
    }
}
