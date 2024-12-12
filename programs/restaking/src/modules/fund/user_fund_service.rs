use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::{token_2022, token_interface};
use std::cell::RefMut;

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::fund::{
    DepositMetadata, FundAccount, FundAccountInfo, FundService, UserFundAccount,
};
use crate::modules::reward::{RewardAccount, RewardService, UserRewardAccount};
use crate::utils::PDASeeds;

use super::FundWithdrawalBatchAccount;

pub struct UserFundService<'info: 'a, 'a> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    receipt_token_program: &'a Program<'info, Token2022>,
    fund_account: &'a mut AccountLoader<'info, FundAccount>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,

    user: &'a Signer<'info>,
    user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
    user_fund_account: &'a mut Account<'info, UserFundAccount>,
    user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,

    _current_slot: u64,
    current_timestamp: i64,
}

impl Drop for UserFundService<'_, '_> {
    fn drop(&mut self) {
        self.fund_account.exit(&crate::ID).unwrap();
        self.user_fund_account.exit(&crate::ID).unwrap();
    }
}

impl<'info, 'a> UserFundService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        receipt_token_program: &'a Program<'info, Token2022>,
        fund_account: &'a mut AccountLoader<'info, FundAccount>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
        user: &'a Signer<'info>,
        user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
        user_fund_account: &'a mut Account<'info, UserFundAccount>,
        user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
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

    pub fn process_deposit_sol(
        &mut self,
        fund_reserve_account: &SystemAccount<'info>,
        system_program: &Program<'info, System>,
        instructions_sysvar: &AccountInfo,
        pricing_sources: &'info [AccountInfo<'info>],
        sol_amount: u64,
        metadata: Option<DepositMetadata>,
        metadata_signer_key: &Pubkey,
    ) -> Result<()> {
        // validate user wallet balance
        require_gte!(self.user.lamports(), sol_amount);

        // validate deposit metadata
        let (wallet_provider, contribution_accrual_rate) = &metadata
            .map(|metadata| {
                metadata.verify(
                    instructions_sysvar,
                    metadata_signer_key,
                    self.user.key,
                    self.current_timestamp,
                )
            })
            .transpose()?
            .unzip();

        // mint receipt token to user & update user reward accrual status
        let mut pricing_service = FundService::new(self.receipt_token_mint, self.fund_account)?
            .new_pricing_service(pricing_sources)?;
        let receipt_token_mint_amount =
            pricing_service.get_token_amount_as_sol(&self.receipt_token_mint.key(), sol_amount)?;
        self.mint_receipt_token_to_user(receipt_token_mint_amount, *contribution_accrual_rate)?;

        // transfer user $SOL to fund
        self.fund_account.load_mut()?.deposit_sol(sol_amount)?;

        system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: self.user.to_account_info(),
                    to: fund_reserve_account.to_account_info(),
                },
            ),
            sol_amount,
        )?;

        // update asset value again
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .update_asset_values(&mut pricing_service)?;

        emit!(events::UserDepositedSOLToFund {
            user: self.user.key(),
            user_receipt_token_account: self.user_receipt_token_account.key(),
            user_fund_account: Clone::clone(self.user_fund_account),
            deposited_sol_amount: sol_amount,
            receipt_token_mint: self.receipt_token_mint.key(),
            minted_receipt_token_amount: receipt_token_mint_amount,
            wallet_provider: wallet_provider.clone(),
            contribution_accrual_rate: *contribution_accrual_rate,
            fund_account: FundAccountInfo::from(self.fund_account.load()?)?,
        });

        Ok(())
    }

    pub fn process_deposit_supported_token(
        &mut self,
        supported_token_mint: &InterfaceAccount<'info, Mint>,
        fund_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        user_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        supported_token_program: &Interface<'info, TokenInterface>,
        instructions_sysvar: &AccountInfo,
        pricing_sources: &'info [AccountInfo<'info>],
        supported_token_amount: u64,
        metadata: Option<DepositMetadata>,
        metadata_signer_key: &Pubkey,
    ) -> Result<()> {
        // validate user token account balance
        require_gte!(user_supported_token_account.amount, supported_token_amount);

        // validate deposit metadata
        let (wallet_provider, contribution_accrual_rate) = &metadata
            .map(|metadata| {
                metadata.verify(
                    instructions_sysvar,
                    metadata_signer_key,
                    self.user.key,
                    self.current_timestamp,
                )
            })
            .transpose()?
            .unzip();

        // calculate receipt token minting amount
        let mut pricing_service = FundService::new(self.receipt_token_mint, self.fund_account)?
            .new_pricing_service(pricing_sources)?;
        let receipt_token_mint_amount = pricing_service.get_sol_amount_as_token(
            &self.receipt_token_mint.key(),
            pricing_service
                .get_token_amount_as_sol(&supported_token_mint.key(), supported_token_amount)?,
        )?;

        // mint receipt token to user & update user reward accrual status
        self.mint_receipt_token_to_user(receipt_token_mint_amount, *contribution_accrual_rate)?;

        // transfer user supported token to fund
        self.fund_account
            .load_mut()?
            .get_supported_token_mut(&supported_token_mint.key())?
            .deposit_token(supported_token_amount)?;
        token_interface::transfer_checked(
            CpiContext::new(
                supported_token_program.to_account_info(),
                token_interface::TransferChecked {
                    from: user_supported_token_account.to_account_info(),
                    to: fund_supported_token_account.to_account_info(),
                    mint: supported_token_mint.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            supported_token_amount,
            supported_token_mint.decimals,
        )?;

        // update fund asset value
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .update_asset_values(&mut pricing_service)?;

        // log deposit event
        emit!(events::UserDepositedSupportedTokenToFund {
            user: self.user.key(),
            user_receipt_token_account: self.user_receipt_token_account.key(),
            user_fund_account: Clone::clone(self.user_fund_account),
            supported_token_mint: supported_token_mint.key(),
            supported_token_user_account: user_supported_token_account.key(),
            deposited_supported_token_amount: supported_token_amount,
            receipt_token_mint: self.receipt_token_mint.key(),
            minted_receipt_token_amount: receipt_token_mint_amount,
            wallet_provider: wallet_provider.clone(),
            contribution_accrual_rate: *contribution_accrual_rate,
            fund_account: FundAccountInfo::from(self.fund_account.load()?)?,
        });

        Ok(())
    }

    fn mint_receipt_token_to_user(
        &mut self,
        receipt_token_mint_amount: u64,
        contribution_accrual_rate: Option<u8>,
    ) -> Result<()> {
        // mint receipt token
        token_2022::mint_to(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                token_2022::MintTo {
                    mint: self.receipt_token_mint.to_account_info(),
                    to: self.user_receipt_token_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[self.fund_account.load()?.get_seeds().as_ref()],
            ),
            receipt_token_mint_amount,
        )?;

        self.fund_account
            .load_mut()?
            .reload_receipt_token_supply(self.receipt_token_mint)?;

        self.user_fund_account
            .reload_receipt_token_amount(self.user_receipt_token_account)?;

        // increase user's reward accrual rate
        RewardService::new(self.receipt_token_mint, self.reward_account)?
            .update_reward_pools_token_allocation(
                None,
                Some(self.user_reward_account),
                receipt_token_mint_amount,
                contribution_accrual_rate,
            )
    }

    pub fn process_request_withdrawal(
        &mut self,
        receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
        receipt_token_amount: u64,
    ) -> Result<()> {
        // validate user receipt token account balance
        require_gte!(self.user_receipt_token_account.amount, receipt_token_amount);
        require_gt!(receipt_token_amount, 0);

        let (batch_id, request_id) = {
            // validate configuration
            let mut fund_account = self.fund_account.load_mut()?;

            // create a user withdrawal request
            self.user_fund_account.create_withdrawal_request(
                &mut fund_account.withdrawal,
                receipt_token_amount,
                self.current_timestamp,
            )?
        };

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

        receipt_token_lock_account.reload()?;

        self.fund_account
            .load_mut()?
            .reload_receipt_token_supply(self.receipt_token_mint)?;

        self.user_fund_account
            .reload_receipt_token_amount(self.user_receipt_token_account)?;

        // reduce user's reward accrual rate
        RewardService::new(self.receipt_token_mint, self.reward_account)?
            .update_reward_pools_token_allocation(
                Some(self.user_reward_account),
                None,
                receipt_token_amount,
                None,
            )?;

        // log withdrawal request event
        emit!(events::UserRequestedWithdrawalFromFund {
            user: self.user.key(),
            user_receipt_token_account: self.user_receipt_token_account.key(),
            user_fund_account: Clone::clone(self.user_fund_account),
            batch_id,
            request_id,
            receipt_token_mint: self.receipt_token_mint.key(),
            requested_receipt_token_amount: receipt_token_amount,
        });

        Ok(())
    }

    pub fn process_cancel_withdrawal_request(
        &mut self,
        receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
        request_id: u64,
    ) -> Result<()> {
        let receipt_token_amount = {
            let mut fund_account = self.fund_account.load_mut()?;

            // clear pending amount from both user fund account and global fund account
            self.user_fund_account
                .cancel_withdrawal_request(&mut fund_account.withdrawal, request_id)?
        };

        // unlock requested user receipt token amount
        // first, burn locked receipt token (use burn/mint instead of transfer to avoid circular CPI through transfer hook)
        token_2022::burn(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                token_2022::Burn {
                    mint: self.receipt_token_mint.to_account_info(),
                    from: receipt_token_lock_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[self.fund_account.load()?.get_seeds().as_ref()],
            ),
            receipt_token_amount,
        )?;

        // then, mint receipt token to user's receipt token account
        token_2022::mint_to(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                token_2022::MintTo {
                    mint: self.receipt_token_mint.to_account_info(),
                    to: self.user_receipt_token_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[self.fund_account.load()?.get_seeds().as_ref()],
            ),
            receipt_token_amount,
        )?;

        receipt_token_lock_account.reload()?;

        self.fund_account
            .load_mut()?
            .reload_receipt_token_supply(self.receipt_token_mint)?;

        self.user_fund_account
            .reload_receipt_token_amount(self.user_receipt_token_account)?;

        // increase user's reward accrual rate
        RewardService::new(self.receipt_token_mint, self.reward_account)?
            .update_reward_pools_token_allocation(
                None,
                Some(self.user_reward_account),
                receipt_token_amount,
                None,
            )?;

        // log withdrawal request canceled event
        emit!(events::UserCanceledWithdrawalRequestFromFund {
            user: self.user.key(),
            user_receipt_token_account: self.user_receipt_token_account.key(),
            user_fund_account: Clone::clone(self.user_fund_account),
            request_id,
            receipt_token_mint: self.receipt_token_mint.key(),
            requested_receipt_token_amount: receipt_token_amount,
        });

        Ok(())
    }

    pub fn process_withdraw(
        &mut self,
        fund_withdrawal_batch_account: &mut Account<'info, FundWithdrawalBatchAccount>,
        fund_reserve_account: &SystemAccount<'info>,
        fund_treasury_account: &SystemAccount<'info>,
        request_id: u64,
    ) -> Result<()> {
        let (sol_user_amount, sol_fee_amount, receipt_token_burn_amount) = {
            let mut fund_account = self.fund_account.load_mut()?;

            // calculate $SOL amounts and mark withdrawal request as claimed
            // withdrawal fee is already paid.
            let (sol_user_amount, sol_fee_amount, receipt_token_burn_amount) =
                self.user_fund_account.settle_withdrawal_request(
                    &mut fund_account.withdrawal,
                    fund_withdrawal_batch_account,
                    request_id,
                )?;

            // transfer sol_user_amount to user wallet
            fund_reserve_account.sub_lamports(sol_user_amount)?;
            self.user.add_lamports(sol_user_amount)?;

            // close ticket and collect rent if stale
            if fund_withdrawal_batch_account.is_settled() {
                fund_withdrawal_batch_account.close(fund_treasury_account.to_account_info())?;
            }

            (sol_user_amount, sol_fee_amount, receipt_token_burn_amount)
        };

        {
            let fund_account = self.fund_account.load()?;
            emit!(events::UserWithdrewSOLFromFund {
                receipt_token_mint: fund_account.receipt_token_mint,
                fund_account: FundAccountInfo::from(fund_account)?,
                request_id,
                user_fund_account: Clone::clone(self.user_fund_account),
                user: self.user.key(),
                burnt_receipt_token_amount: receipt_token_burn_amount,
                withdrawn_sol_amount: sol_user_amount,
                deducted_sol_fee_amount: sol_fee_amount,
            });
        }

        Ok(())
    }
}
