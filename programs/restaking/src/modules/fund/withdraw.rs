use anchor_lang::prelude::*;
use anchor_spl::token_2022::{self, Token2022};
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::reward::{self, RewardAccount, UserRewardAccount};
use crate::utils::PDASeeds;

use super::*;

pub fn request_withdrawal<'info>(
    user: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    fund_account: &mut FundAccount,
    reward_account: &mut RewardAccount,
    user_fund_account: &mut UserFundAccount,
    user_reward_account: &mut UserRewardAccount,
    user_reward_account_address: Pubkey,
    receipt_token_program: &Program<'info, Token2022>,
    receipt_token_amount: u64,
    current_slot: u64,
) -> Result<()> {
    let withdrawal_request = user_fund_account
        .create_withdrawal_request(&mut fund_account.withdrawal_status, receipt_token_amount)?;
    let batch_id = withdrawal_request.batch_id;
    let request_id = withdrawal_request.request_id;

    lock_receipt_token(
        receipt_token_program,
        receipt_token_mint,
        receipt_token_mint_authority,
        receipt_token_lock_account,
        user,
        user_receipt_token_account,
        reward_account,
        user_fund_account,
        user_reward_account,
        user_reward_account_address,
        receipt_token_amount,
        current_slot,
    )?;

    emit!(events::UserRequestedWithdrawalFromFund {
        user: user.key(),
        user_receipt_token_account: user_receipt_token_account.key(),
        user_fund_account: user_fund_account.clone(),
        batch_id,
        request_id,
        receipt_token_mint: receipt_token_mint.key(),
        requested_receipt_token_amount: receipt_token_amount,
    });

    Ok(())
}

pub fn cancel_withdrawal_request<'info>(
    user: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
    receipt_token_lock_authority: &Account<'info, ReceiptTokenLockAuthority>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    fund_account: &mut FundAccount,
    reward_account: &mut RewardAccount,
    user_fund_account: &mut UserFundAccount,
    user_reward_account: &mut UserRewardAccount,
    user_reward_account_address: Pubkey,
    receipt_token_program: &Program<'info, Token2022>,
    request_id: u64,
    current_slot: u64,
) -> Result<()> {
    let request = user_fund_account
        .cancel_withdrawal_request(&mut fund_account.withdrawal_status, request_id)?;

    unlock_receipt_token(
        receipt_token_program,
        receipt_token_mint,
        receipt_token_mint_authority,
        receipt_token_lock_account,
        receipt_token_lock_authority,
        user_receipt_token_account,
        reward_account,
        user_fund_account,
        user_reward_account,
        user_reward_account_address,
        request.receipt_token_amount,
        current_slot,
    )?;

    emit!(events::UserCanceledWithdrawalRequestFromFund {
        user: user.key(),
        user_receipt_token_account: user_receipt_token_account.key(),
        user_fund_account: user_fund_account.clone(),
        request_id,
        receipt_token_mint: receipt_token_mint.key(),
        requested_receipt_token_amount: request.receipt_token_amount,
    });

    Ok(())
}

pub fn withdraw(
    user: &Signer,
    receipt_token_mint: &Mint,
    fund_account: &mut Account<FundAccount>,
    user_fund_account: &mut UserFundAccount,
    request_id: u64,
) -> Result<()> {
    let request = user_fund_account
        .pop_completed_withdrawal_request(&mut fund_account.withdrawal_status, request_id)?;

    let (sol_withdraw_amount, sol_fee_amount) =
        transfer_sol_from_fund_to_user(user, fund_account, request.receipt_token_amount)?;

    let receipt_token_price = fund_account.receipt_token_sol_value_per_token(
        receipt_token_mint.decimals,
        receipt_token_mint.supply,
    )?;

    emit!(events::UserWithdrewSOLFromFund {
        receipt_token_mint: fund_account.receipt_token_mint,
        fund_account: FundAccountInfo::new(
            fund_account.as_ref(),
            receipt_token_price,
            receipt_token_mint.supply
        ),
        request_id,
        user_fund_account: user_fund_account.clone(),
        user: user.key(),
        burnt_receipt_token_amount: request.receipt_token_amount,
        withdrawn_sol_amount: sol_withdraw_amount,
        deducted_sol_fee_amount: sol_fee_amount,
    });

    Ok(())
}

fn lock_receipt_token<'info>(
    receipt_token_program: &Program<'info, Token2022>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
    user: &Signer<'info>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    reward_account: &mut RewardAccount,
    user_fund_account: &mut UserFundAccount,
    user_reward_account: &mut UserRewardAccount,
    user_reward_account_address: Pubkey,
    receipt_token_amount: u64,
    current_slot: u64,
) -> Result<()> {
    token_2022::burn(
        CpiContext::new(
            receipt_token_program.to_account_info(),
            token_2022::Burn {
                mint: receipt_token_mint.to_account_info(),
                from: user_receipt_token_account.to_account_info(),
                authority: user.to_account_info(),
            },
        ),
        receipt_token_amount,
    )
    .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

    token_2022::mint_to(
        CpiContext::new_with_signer(
            receipt_token_program.to_account_info(),
            token_2022::MintTo {
                mint: receipt_token_mint.to_account_info(),
                to: receipt_token_lock_account.to_account_info(),
                authority: receipt_token_mint_authority.to_account_info(),
            },
            &[receipt_token_mint_authority.signer_seeds().as_ref()],
        ),
        receipt_token_amount,
    )
    .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

    receipt_token_mint.reload()?;
    receipt_token_lock_account.reload()?;
    user_receipt_token_account.reload()?;
    user_fund_account.set_receipt_token_amount(user_receipt_token_account.amount);

    reward::update_reward_pools_token_allocation(
        reward_account,
        Some(user_reward_account),
        None,
        vec![user_reward_account_address],
        receipt_token_mint.key(),
        receipt_token_amount,
        None,
        current_slot,
    )
}

fn unlock_receipt_token<'info>(
    receipt_token_program: &Program<'info, Token2022>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
    receipt_token_lock_authority: &Account<'info, ReceiptTokenLockAuthority>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    reward_account: &mut RewardAccount,
    user_fund_account: &mut UserFundAccount,
    user_reward_account: &mut UserRewardAccount,
    user_reward_account_address: Pubkey,
    receipt_token_amount: u64,
    current_slot: u64,
) -> Result<()> {
    token_2022::burn(
        CpiContext::new_with_signer(
            receipt_token_program.to_account_info(),
            token_2022::Burn {
                mint: receipt_token_mint.to_account_info(),
                from: receipt_token_lock_account.to_account_info(),
                authority: receipt_token_lock_authority.to_account_info(),
            },
            &[receipt_token_lock_authority.signer_seeds().as_ref()],
        ),
        receipt_token_amount,
    )
    .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

    token_2022::mint_to(
        CpiContext::new_with_signer(
            receipt_token_program.to_account_info(),
            token_2022::MintTo {
                mint: receipt_token_mint.to_account_info(),
                to: user_receipt_token_account.to_account_info(),
                authority: receipt_token_mint_authority.to_account_info(),
            },
            &[receipt_token_mint_authority.signer_seeds().as_ref()],
        ),
        receipt_token_amount,
    )
    .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;

    receipt_token_mint.reload()?;
    receipt_token_lock_account.reload()?;
    user_receipt_token_account.reload()?;
    user_fund_account.set_receipt_token_amount(user_receipt_token_account.amount);

    reward::update_reward_pools_token_allocation(
        reward_account,
        None,
        Some(user_reward_account),
        vec![user_reward_account_address],
        receipt_token_mint.key(),
        receipt_token_amount,
        None,
        current_slot,
    )
}

fn transfer_sol_from_fund_to_user(
    user: &Signer,
    fund_account: &mut Account<FundAccount>,
    receipt_token_withdraw_amount: u64,
) -> Result<(u64, u64)> {
    let sol_amount = fund_account
        .withdrawal_status
        .reserved_fund
        .calculate_sol_amount_for_receipt_token_amount(receipt_token_withdraw_amount)?;
    let sol_fee_amount = fund_account
        .withdrawal_status
        .calculate_sol_withdrawal_fee(sol_amount)?;
    let sol_withdraw_amount = sol_amount
        .checked_sub(sol_fee_amount)
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

    fund_account.withdrawal_status.withdraw(
        sol_amount,
        sol_fee_amount,
        receipt_token_withdraw_amount,
    )?;
    fund_account.sub_lamports(sol_withdraw_amount)?;
    user.add_lamports(sol_withdraw_amount)?;

    Ok((sol_withdraw_amount, sol_fee_amount))
}

impl BatchWithdrawal {
    fn add_receipt_token_to_process(&mut self, amount: u64) -> Result<()> {
        self.num_withdrawal_requests += 1;
        self.receipt_token_to_process = self
            .receipt_token_to_process
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    fn remove_receipt_token_to_process(&mut self, amount: u64) -> Result<()> {
        self.num_withdrawal_requests -= 1;
        self.receipt_token_to_process = self
            .receipt_token_to_process
            .checked_sub(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    fn start_batch_processing(&mut self) -> Result<()> {
        self.processing_started_at = Some(crate::utils::timestamp_now()?);
        Ok(())
    }

    fn is_completed(&self) -> bool {
        self.processing_started_at.is_some()
            && self.receipt_token_to_process == 0
            && self.receipt_token_being_processed == 0
    }

    // Called by operator
    pub fn record_unstaking_start(&mut self, receipt_token_amount: u64) -> Result<()> {
        self.receipt_token_to_process = self
            .receipt_token_to_process
            .checked_sub(receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.receipt_token_being_processed = self
            .receipt_token_being_processed
            .checked_add(receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    // Called by operator
    pub fn record_unstaking_end(
        &mut self,
        receipt_token_amount: u64,
        sol_amount: u64,
    ) -> Result<()> {
        self.receipt_token_being_processed = self
            .receipt_token_being_processed
            .checked_sub(receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.receipt_token_processed = self
            .receipt_token_processed
            .checked_add(receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.sol_reserved = self
            .sol_reserved
            .checked_add(sol_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}

impl ReservedFund {
    fn record_completed_batch_withdrawal(&mut self, batch: BatchWithdrawal) -> Result<()> {
        self.receipt_token_processed_amount = self
            .receipt_token_processed_amount
            .checked_add(batch.receipt_token_processed)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.sol_withdrawal_reserved_amount = self
            .sol_withdrawal_reserved_amount
            .checked_add(batch.sol_reserved)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }

    pub fn calculate_sol_amount_for_receipt_token_amount(
        &self,
        receipt_token_withdraw_amount: u64,
    ) -> Result<u64> {
        crate::utils::proportional_amount(
            receipt_token_withdraw_amount,
            self.sol_withdrawal_reserved_amount,
            self.receipt_token_processed_amount,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    fn withdraw(
        &mut self,
        sol_amount: u64,
        sol_fee_amount: u64,
        receipt_token_amount: u64,
    ) -> Result<()> {
        self.receipt_token_processed_amount = self
            .receipt_token_processed_amount
            .checked_sub(receipt_token_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        self.sol_withdrawal_reserved_amount = self
            .sol_withdrawal_reserved_amount
            .checked_sub(sol_amount)
            .ok_or_else(|| error!(ErrorCode::FundWithdrawalReservedSOLExhaustedException))?;

        // send fee to fee income
        self.sol_fee_income_reserved_amount = self
            .sol_fee_income_reserved_amount
            .checked_add(sol_fee_amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}

impl WithdrawalStatus {
    fn issue_new_request_id(&mut self) -> u64 {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        request_id
    }

    pub fn calculate_sol_withdrawal_fee(&self, amount: u64) -> Result<u64> {
        crate::utils::proportional_amount(
            amount,
            self.sol_withdrawal_fee_rate as u64,
            Self::WITHDRAWAL_FEE_RATE_DIVISOR,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    pub fn withdraw(
        &mut self,
        sol_amount: u64,
        sol_fee_amount: u64,
        receipt_token_amount: u64,
    ) -> Result<()> {
        self.reserved_fund
            .withdraw(sol_amount, sol_fee_amount, receipt_token_amount)
    }

    fn check_withdrawal_enabled(&self) -> Result<()> {
        if !self.withdrawal_enabled_flag {
            err!(ErrorCode::FundWithdrawalDisabledError)?
        }

        Ok(())
    }

    fn check_batch_processing_not_started(&self, batch_id: u64) -> Result<()> {
        if batch_id < self.pending_batch_withdrawal.batch_id {
            err!(ErrorCode::FundProcessingWithdrawalRequestError)?
        }

        Ok(())
    }

    fn check_batch_processing_completed(&self, batch_id: u64) -> Result<()> {
        if batch_id > self.last_completed_batch_id {
            err!(ErrorCode::FundPendingWithdrawalRequestError)?
        }

        Ok(())
    }

    // Called by operator
    pub fn start_processing_pending_batch_withdrawal(&mut self) -> Result<()> {
        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;
        let new = BatchWithdrawal::new(batch_id);

        let mut old = std::mem::replace(&mut self.pending_batch_withdrawal, new);
        old.start_batch_processing()?;

        self.num_withdrawal_requests_in_progress += old.num_withdrawal_requests;
        self.last_batch_processing_started_at = old.processing_started_at;
        self.batch_withdrawals_in_progress.push(old);

        Ok(())
    }

    // Called by operator
    pub fn end_processing_completed_batch_withdrawals(&mut self) -> Result<()> {
        let completed_batch_withdrawals = self.pop_completed_batch_withdrawals();
        if let Some(batch) = completed_batch_withdrawals.last() {
            self.last_completed_batch_id = batch.batch_id;
            self.last_batch_processing_completed_at = Some(crate::utils::timestamp_now()?);
        }
        for batch in completed_batch_withdrawals {
            self.reserved_fund
                .record_completed_batch_withdrawal(batch)?;
        }

        Ok(())
    }

    fn pop_completed_batch_withdrawals(&mut self) -> Vec<BatchWithdrawal> {
        let (completed, remaining) = std::mem::take(&mut self.batch_withdrawals_in_progress)
            .into_iter()
            .partition(|batch| {
                if batch.is_completed() {
                    self.num_withdrawal_requests_in_progress -= batch.num_withdrawal_requests;
                    true
                } else {
                    false
                }
            });
        self.batch_withdrawals_in_progress = remaining;
        completed
    }
}

impl UserFundAccount {
    fn push_withdrawal_request(&mut self, request: WithdrawalRequest) -> Result<()> {
        if self.withdrawal_requests.len() == Self::MAX_WITHDRAWAL_REQUESTS_SIZE {
            err!(ErrorCode::FundExceededMaxWithdrawalRequestError)?;
        }

        self.withdrawal_requests.push(request);

        Ok(())
    }

    fn pop_withdrawal_request(&mut self, request_id: u64) -> Result<WithdrawalRequest> {
        let index = self
            .withdrawal_requests
            .binary_search_by_key(&request_id, |req| req.request_id)
            .map_err(|_| error!(ErrorCode::FundWithdrawalRequestNotFoundError))?;
        Ok(self.withdrawal_requests.remove(index))
    }

    pub fn create_withdrawal_request(
        &mut self,
        withdrawal_status: &mut WithdrawalStatus,
        receipt_token_amount: u64,
    ) -> Result<&WithdrawalRequest> {
        withdrawal_status.check_withdrawal_enabled()?;

        self.push_withdrawal_request(WithdrawalRequest::new(
            withdrawal_status.pending_batch_withdrawal.batch_id,
            withdrawal_status.issue_new_request_id(),
            receipt_token_amount,
        )?)?;
        withdrawal_status
            .pending_batch_withdrawal
            .add_receipt_token_to_process(receipt_token_amount)?;

        Ok(self.withdrawal_requests.last().unwrap())
    }

    pub fn cancel_withdrawal_request(
        &mut self,
        withdrawal_status: &mut WithdrawalStatus,
        request_id: u64,
    ) -> Result<WithdrawalRequest> {
        if request_id >= withdrawal_status.next_request_id {
            err!(ErrorCode::FundWithdrawalRequestNotFoundError)?;
        }

        let request = self.pop_withdrawal_request(request_id)?;
        withdrawal_status.check_batch_processing_not_started(request.batch_id)?;
        withdrawal_status
            .pending_batch_withdrawal
            .remove_receipt_token_to_process(request.receipt_token_amount)?;

        Ok(request)
    }

    pub fn pop_completed_withdrawal_request(
        &mut self,
        withdrawal_status: &mut WithdrawalStatus,
        request_id: u64,
    ) -> Result<WithdrawalRequest> {
        if request_id >= withdrawal_status.next_request_id {
            err!(ErrorCode::FundWithdrawalRequestNotFoundError)?;
        }

        withdrawal_status.check_withdrawal_enabled()?;
        let request = self.pop_withdrawal_request(request_id)?;
        withdrawal_status.check_batch_processing_completed(request.batch_id)?;

        Ok(request)
    }
}
