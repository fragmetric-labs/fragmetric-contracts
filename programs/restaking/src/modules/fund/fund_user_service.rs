use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{token_2022, token_interface};
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::{events, modules};
use crate::errors::ErrorCode;
use crate::modules::{ed25519, reward};
use crate::modules::fund::{DepositMetadata, FundAccount, FundAccountInfo, FundService, UserFundAccount};
use crate::modules::reward::{RewardAccount, UserRewardAccount};
use crate::utils::PDASeeds;

pub struct FundUserService<'info, 'a> where 'info : 'a {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    receipt_token_program: &'a Program<'info, Token2022>,
    user: &'a Signer<'info>,
    user_fund_account: &'a mut Account<'info, UserFundAccount>,
    fund_account: &'a mut Account<'info, FundAccount>,
    user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,

    // TODO: use user_reward_service
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,
    user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,

    current_slot: u64,
    current_timestamp: i64,
}

impl<'info, 'a> FundUserService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        receipt_token_program: &'a Program<'info, Token2022>,
        user: &'a Signer<'info>,
        user_fund_account: &'a mut Account<'info, UserFundAccount>,
        fund_account: &'a mut Account<'info, FundAccount>,
        user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
        user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            receipt_token_program,
            user,
            user_fund_account,
            fund_account,
            user_receipt_token_account,
            reward_account,
            user_reward_account,
            current_slot: clock.slot,
            current_timestamp: clock.unix_timestamp,
        })
    }

    pub fn process_deposit_sol(
        &mut self,
        fund_reserve_account: &'a SystemAccount<'info>,
        system_program: &'a Program<'info, System>,
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
            .map(|metadata| metadata.verify(instructions_sysvar, metadata_signer_key, self.current_timestamp))
            .transpose()?
            .unzip();

        // calculate receipt token minting amount
        // TODO: use pricing module for checking update, calculating amount
        FundService::new(
            self.receipt_token_mint,
            self.fund_account,
            pricing_sources,
        )?.update_asset_prices()?;
        let receipt_token_mint_amount = crate::utils::get_proportional_amount(
            sol_amount,
            self.receipt_token_mint.supply,
            self.fund_account.get_assets_total_amount_as_sol()?,
        )
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        // mint receipt token to user & update user reward accrual status
        self.mint_receipt_token_to_user(
            receipt_token_mint_amount,
            *contribution_accrual_rate,
        )?;

        // transfer user $SOL to fund
        self.fund_account.deposit_sol(sol_amount)?;
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

        // log deposit event
        emit!(events::UserDepositedSOLToFund {
            user: self.user.key(),
            user_receipt_token_account: self.user_receipt_token_account.key(),
            user_fund_account: Clone::clone(self.user_fund_account),
            deposited_sol_amount: sol_amount,
            receipt_token_mint: self.receipt_token_mint.key(),
            minted_receipt_token_amount: receipt_token_mint_amount,
            wallet_provider: wallet_provider.clone(),
            contribution_accrual_rate: *contribution_accrual_rate,
            fund_account: FundAccountInfo::from(
                self.fund_account,
                self.receipt_token_mint,
            ),
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
            .map(|metadata| metadata.verify(instructions_sysvar, metadata_signer_key, self.current_timestamp))
            .transpose()?
            .unzip();

        // calculate receipt token minting amount
        // TODO: use pricing module for checking update, calculating amount
        FundService::new(
            self.receipt_token_mint,
            self.fund_account,
            pricing_sources,
        )?.update_asset_prices()?;
        let receipt_token_mint_amount = crate::utils::get_proportional_amount(
            self.fund_account
                .get_supported_token(supported_token_mint.key())?
                .get_token_amount_as_sol(supported_token_amount)?,
            self.receipt_token_mint.supply,
            self.fund_account.get_assets_total_amount_as_sol()?,
        )
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        // mint receipt token to user & update user reward accrual status
        self.mint_receipt_token_to_user(
            receipt_token_mint_amount,
            *contribution_accrual_rate,
        )?;

        // transfer user supported token to fund
        self.fund_account
            .get_supported_token_mut(supported_token_mint.key())?
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
        )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

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
            fund_account: FundAccountInfo::from(
                self.fund_account,
                self.receipt_token_mint,
            ),
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
                &[self.fund_account.get_signer_seeds().as_ref()],
            ),
            receipt_token_mint_amount,
        )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;
        self.receipt_token_mint.reload()?;
        self.user_fund_account.sync_receipt_token_amount(self.user_receipt_token_account)?;

        // increase user's reward accrual rate
        // TODO: use user reward service
        reward::update_reward_pools_token_allocation(
            &mut *self.reward_account.load_mut()?,
            None,
            Some(&mut *self.user_reward_account.load_mut()?),
            vec![self.user_reward_account.key()],
            self.receipt_token_mint.key(),
            receipt_token_mint_amount,
            contribution_accrual_rate,
            self.current_slot,
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

        // validate configuration
        self.fund_account.withdrawal.assert_withdrawal_enabled()?;

        // create a user withdrawal request
        let (batch_id, request_id) = self.user_fund_account.create_withdrawal_request(
            &mut self.fund_account.withdrawal,
            receipt_token_amount,
            self.current_timestamp,
        )?;

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
        )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

        // then, mint receipt token to fund's lock account
        token_2022::mint_to(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                token_2022::MintTo {
                    mint: self.receipt_token_mint.to_account_info(),
                    to: receipt_token_lock_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[self.fund_account.get_signer_seeds().as_ref()],
            ),
            receipt_token_amount,
        )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

        self.receipt_token_mint.reload()?;
        receipt_token_lock_account.reload()?;
        self.user_fund_account.sync_receipt_token_amount(self.user_receipt_token_account)?;

        // reduce user's reward accrual rate
        // TODO: use user reward service
        reward::update_reward_pools_token_allocation(
            &mut *self.reward_account.load_mut()?,
            Some(&mut *self.user_reward_account.load_mut()?),
            None,
            vec![self.user_reward_account.key()],
            self.receipt_token_mint.key(),
            receipt_token_amount,
            None,
            self.current_slot,
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
        // clear pending amount from both user fund account and global fund account
        let receipt_token_amount =
            self.user_fund_account.cancel_withdrawal_request(&mut self.fund_account.withdrawal, request_id)?;

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
                &[self.fund_account.get_signer_seeds().as_ref()],
            ),
            receipt_token_amount,
        )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

        // then, mint receipt token to user's receipt token account
        token_2022::mint_to(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                token_2022::MintTo {
                    mint: self.receipt_token_mint.to_account_info(),
                    to: self.user_receipt_token_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[self.fund_account.get_signer_seeds().as_ref()],
            ),
            receipt_token_amount,
        )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

        self.receipt_token_mint.reload()?;
        receipt_token_lock_account.reload()?;
        self.user_fund_account.sync_receipt_token_amount(self.user_receipt_token_account)?;

        // increase user's reward accrual rate
        // TODO: use user reward service
        reward::update_reward_pools_token_allocation(
            &mut *self.reward_account.load_mut()?,
            None,
            Some(&mut *self.user_reward_account.load_mut()?),
            vec![self.user_reward_account.key()],
            self.receipt_token_mint.key(),
            receipt_token_amount,
            None,
            self.current_slot,
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
        fund_reserve_account: &SystemAccount<'info>,
        fund_reserve_account_bump: u8,
        fund_treasury_account: &SystemAccount<'info>,
        system_program: &'a Program<'info, System>,
        request_id: u64,
    ) -> Result<()> {
        // calculate $SOL amounts and mark withdrawal request as claimed
        let (sol_withdraw_amount, sol_fee_amount, receipt_token_burn_amount) =
            self.user_fund_account.claim_withdrawal_request(&mut self.fund_account.withdrawal, request_id)?;

        // transfer sol_withdraw_amount to user wallet
        let receipt_token_mint_key = self.receipt_token_mint.key();
        let fund_reserve_account_signer_seeds: &[&[&[u8]]] = &[&[
            FundAccount::RESERVE_SEED,
            receipt_token_mint_key.as_ref(),
            &[fund_reserve_account_bump],
        ]];

        system_program::transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: fund_reserve_account.to_account_info(),
                    to: self.user.to_account_info(),
                },
                fund_reserve_account_signer_seeds,
            ),
            sol_withdraw_amount,
        )?;

        // transfer sol_fee_amount to fund treasury account
        system_program::transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: fund_reserve_account.to_account_info(),
                    to: fund_treasury_account.to_account_info(),
                },
                fund_reserve_account_signer_seeds,
            ),
            sol_fee_amount,
        )?;

        // log withdraw event
        emit!(events::UserWithdrewSOLFromFund {
            receipt_token_mint: self.fund_account.receipt_token_mint,
            fund_account: FundAccountInfo::from(self.fund_account, self.receipt_token_mint),
            request_id,
            user_fund_account: Clone::clone(self.user_fund_account),
            user: self.user.key(),
            burnt_receipt_token_amount: receipt_token_burn_amount,
            withdrawn_sol_amount: sol_withdraw_amount,
            deducted_sol_fee_amount: sol_fee_amount,
        });

        Ok(())
    }
}
