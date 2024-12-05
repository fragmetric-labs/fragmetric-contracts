use std::collections::{BTreeMap, BTreeSet};

use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::errors::ErrorCode;
use crate::modules::pricing::{PricingService, TokenPricingSource};
use crate::modules::reward;
use crate::utils::*;
use crate::{events, utils};

use super::command::{
    OperationCommandAccountMeta, OperationCommandContext, OperationCommandEntry, SelfExecutable,
};
use super::*;

pub struct FundService<'info: 'a, 'a> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    fund_account: &'a mut Account<'info, FundAccount>,
    current_timestamp: i64,
}

impl Drop for FundService<'_, '_> {
    fn drop(&mut self) {
        self.fund_account.exit(&crate::ID).unwrap();
    }
}

impl<'info: 'a, 'a> FundService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        fund_account: &'a mut Account<'info, FundAccount>,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            fund_account,
            current_timestamp: clock.unix_timestamp,
        })
    }

    // create a pricing service and register fund assets' value resolver
    pub(in crate::modules) fn new_pricing_service(
        &mut self,
        pricing_sources: impl IntoIterator<Item = &'info AccountInfo<'info>>,
    ) -> Result<PricingService<'info>> {
        let mut pricing_service = PricingService::new(pricing_sources)?
            .register_token_pricing_source_account(self.fund_account.as_account_info());

        // try to update current underlying assets' price
        self.update_asset_values(&mut pricing_service)?;

        Ok(pricing_service)
    }

    pub(in crate::modules) fn get_pricing_sources(&self) -> Result<Vec<Pubkey>> {
        self
            .fund_account
            .normalized_token
            .iter()
            .map(|normalized_token| &normalized_token.pricing_source)
            .chain(self.fund_account.restaking_vaults.iter().map(
                |restaking_vault| &restaking_vault.receipt_token_pricing_source,
            ))
            .chain(
                self.fund_account
                    .supported_tokens
                    .iter()
                    .map(|supported_token| &supported_token.pricing_source),
            )
            .map(|pricing_source| {
                Ok(match pricing_source {
                    TokenPricingSource::SPLStakePool { address }
                    | TokenPricingSource::MarinadeStakePool { address }
                    | TokenPricingSource::JitoRestakingVault { address }
                    | TokenPricingSource::FragmetricNormalizedTokenPool {
                        address,
                    } => *address,
                    _ => err!(ErrorCode::TokenPricingSourceAccountNotFoundError)?,
                })
            })
            .collect::<Result<Vec<_>>>()
    }

    pub(super) fn update_asset_values(
        &mut self,
        pricing_service: &mut PricingService,
    ) -> Result<()> {
        // ensure any update on fund account written before do pricing
        self.fund_account.exit(&crate::ID)?;

        // update fund asset values
        pricing_service.resolve_token_pricing_source(
            &self.fund_account.receipt_token_mint.key(),
            &TokenPricingSource::FragmetricRestakingFund {
                address: self.fund_account.key(),
            },
        )?;

        // the values being written below are informative, only for event emission.
        self.fund_account
            .supported_tokens
            .iter_mut()
            .try_for_each(|supported_token| {
                supported_token.one_token_as_sol = pricing_service.get_token_amount_as_sol(
                    &supported_token.mint,
                    10u64
                        .checked_pow(supported_token.decimals as u32)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
                )?;

                Ok::<(), Error>(())
            })?;

        if let Some(fund_account_normalized_token) = &mut self.fund_account.normalized_token {
            fund_account_normalized_token.one_token_as_sol = pricing_service
                .get_token_amount_as_sol(
                    &fund_account_normalized_token.mint,
                    10u64
                        .checked_pow(fund_account_normalized_token.decimals as u32)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
                )?;
        }

        let receipt_token_mint_key = &self.receipt_token_mint.key();
        self.fund_account.one_receipt_token_as_sol = pricing_service.get_token_amount_as_sol(
            receipt_token_mint_key,
            10u64
                .checked_pow(self.receipt_token_mint.decimals as u32)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
        )?;

        self.fund_account.receipt_token_value =
            pricing_service.get_token_total_value_as_atomic(receipt_token_mint_key)?;

        self.fund_account.receipt_token_value_updated_at = self.current_timestamp;

        Ok(())
    }

    pub fn process_transfer_hook(
        &self,
        reward_account: &mut AccountLoader<'info, reward::RewardAccount>,
        source_receipt_token_account: &mut InterfaceAccount<TokenAccount>,
        destination_receipt_token_account: &mut InterfaceAccount<TokenAccount>,
        extra_accounts: &'info [AccountInfo<'info>],
        transfer_amount: u64,
    ) -> Result<()> {
        let mut extra_accounts = extra_accounts.iter();
        // parse extra accounts
        let source_fund_account_option = extra_accounts
            .next()
            .ok_or(ProgramError::NotEnoughAccountKeys)?
            .parse_optional_account_boxed::<UserFundAccount>()?;
        let mut source_reward_account_option = extra_accounts
            .next()
            .ok_or(ProgramError::NotEnoughAccountKeys)?
            .parse_optional_account_loader::<reward::UserRewardAccount>()?;
        let destination_fund_account_option = extra_accounts
            .next()
            .ok_or(ProgramError::NotEnoughAccountKeys)?
            .parse_optional_account_boxed::<UserFundAccount>()?;
        let mut destination_reward_account_option = extra_accounts
            .next()
            .ok_or(ProgramError::NotEnoughAccountKeys)?
            .parse_optional_account_loader::<reward::UserRewardAccount>()?;

        // transfer source's reward accrual rate to destination
        reward::RewardService::new(self.receipt_token_mint, reward_account)?
            .update_reward_pools_token_allocation(
                source_reward_account_option.as_mut(),
                destination_reward_account_option.as_mut(),
                transfer_amount,
                None,
            )?;

        // sync user fund accounts
        if let Some(mut source_fund_account) = source_fund_account_option {
            source_fund_account.reload_receipt_token_amount(source_receipt_token_account)?;
            source_fund_account.exit(&crate::ID)?;
        }
        if let Some(mut destination_fund_account) = destination_fund_account_option {
            destination_fund_account
                .reload_receipt_token_amount(destination_receipt_token_account)?;
            destination_fund_account.exit(&crate::ID)?;
        }

        emit!(events::UserTransferredReceiptToken {
            receipt_token_mint: self.receipt_token_mint.key(),
            transferred_receipt_token_amount: transfer_amount,

            source_receipt_token_account: source_receipt_token_account.key(),
            source: source_receipt_token_account.owner,
            source_fund_account: UserFundAccount::placeholder(
                source_receipt_token_account.owner,
                self.receipt_token_mint.key(),
                source_receipt_token_account.amount,
            ),
            destination_receipt_token_account: destination_receipt_token_account.key(),
            destination: destination_receipt_token_account.owner,
            destination_fund_account: UserFundAccount::placeholder(
                destination_receipt_token_account.owner,
                self.receipt_token_mint.key(),
                destination_receipt_token_account.amount,
            ),
        });

        // TODO v0.4/transfer: token transfer is temporarily disabled
        err!(ErrorCode::TokenNotTransferableError)?;

        Ok(())
    }

    pub fn process_run(
        &mut self,
        operator: &Signer<'info>,
        system_program: &Program<'info, System>,
        remaining_accounts: &'info [AccountInfo<'info>],
        reset_command: Option<OperationCommandEntry>,
    ) -> Result<()> {
        let mut operation_state = std::mem::take(&mut self.fund_account.operation);
        operation_state.initialize_command_if_needed(self.current_timestamp, reset_command)?;

        let mut execution_count = 0;
        let mut executed_commands = Vec::new();

        let pricing_sources = self.get_pricing_sources()?;
        let remaining_accounts_map: BTreeMap<Pubkey, &AccountInfo> = remaining_accounts
            .iter()
            .map(|info| (*info.key, info))
            .collect();

        'command_loop: while let Some((command, required_accounts)) = operation_state.get_command()
        {
            // rearrange given accounts in required order
            let mut required_account_infos = Vec::new();
            let mut unused_account_keys = remaining_accounts_map
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>();

            for account_meta in required_accounts {
                // append required accounts in exact order
                match remaining_accounts_map.get(&account_meta.pubkey) {
                    Some(account) => {
                        required_account_infos.push(*account);
                        unused_account_keys.remove(&account_meta.pubkey);
                    }
                    None => {
                        if execution_count > 0 {
                            // maintain the current command and gracefully stop executing commands
                            msg!(
                                "COMMAND#{}: {:?} has not enough accounts after {} execution(s)",
                                operation_state.next_sequence,
                                command,
                                execution_count
                            );
                            break 'command_loop;
                        }

                        // error if it is the first command in this tx
                        msg!(
                            "COMMAND#{}: {:?} has not enough accounts at the first execution",
                            operation_state.next_sequence,
                            command
                        );
                        return err!(ErrorCode::OperationCommandAccountComputationException);
                    }
                }
            }

            // append all unused accounts
            for unused_account_key in &unused_account_keys {
                // SAFETY: `unused_account_key` is a subset of `remaining_accounts_map`.
                let remaining_account = remaining_accounts_map.get(unused_account_key).unwrap();
                required_account_infos.push(*remaining_account);
            }

            let mut ctx = OperationCommandContext {
                operator,
                receipt_token_mint: self.receipt_token_mint,
                fund_account: self.fund_account,
                system_program,
            };
            match command.execute(&mut ctx, required_account_infos.as_slice()) {
                Ok(mut next_command) => {
                    // msg!("COMMAND: {:?} with {:?} passed", command, required_accounts);
                    // msg!("COMMAND#{}: {:?} passed", operation_state.sequence, command);
                    executed_commands.push(command.clone());
                    execution_count += 1;

                    // append pricing sources to required_accounts
                    match next_command {
                        Some(ref mut next_command) => {
                            next_command.append_readonly_accounts(pricing_sources.iter().cloned());
                        }
                        _ => {}
                    }
                    operation_state.set_command(next_command, self.current_timestamp);
                }
                Err(error) => {
                    // msg!("COMMAND: {:?} with {:?} failed", command, required_accounts);
                    msg!(
                        "COMMAND#{}: {:?} failed",
                        operation_state.next_sequence,
                        command
                    );
                    return Err(error);
                }
            };
        }

        // write back operation state
        self.fund_account.operation = operation_state;

        emit!(events::OperatorRanFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: FundAccountInfo::from(self.fund_account),
            executed_commands,
        });

        Ok(())
    }

    /// returns (enqueued)
    pub(super) fn enqueue_withdrawal_batch(&mut self, forced: bool) -> bool {
        if !(forced
            || self
                .fund_account
                .withdrawal
                .is_batch_enqueuing_threshold_satisfied(self.current_timestamp))
        {
            // Threshold unmet, skip enqueue
            return false;
        }

        self.fund_account
            .withdrawal
            .enqueue_pending_batch(self.current_timestamp)
    }

    /// returns [receipt_token_program, receipt_token_lock_account, fund_reserve_account, fund_treasury_account, withdrawal_batch_accounts @ ..]
    pub(super) fn find_accounts_to_process_withdrawal_batch(&self) -> Result<Vec<(Pubkey, bool)>> {
        let mut accounts =
            Vec::with_capacity(4 + self.fund_account.withdrawal.queued_batches.len());
        accounts.extend([
            (self.fund_account.receipt_token_program, false),
            (
                self.fund_account
                    .find_receipt_token_lock_account_address()?,
                true,
            ),
            (self.fund_account.get_reserve_account_address()?, true),
            (self.fund_account.get_treasury_account_address()?, true),
        ]);
        accounts.extend(
            self.fund_account
                .withdrawal
                .queued_batches
                .iter()
                .map(|batch| {
                    (
                        FundWithdrawalBatchAccount::find_account_address(
                            &self.receipt_token_mint.key(),
                            batch.batch_id,
                        )
                        .0,
                        true,
                    )
                }),
        );
        Ok(accounts)
    }

    /// returns (receipt_token_amount_processing)
    pub(super) fn process_withdrawal_batch(
        &mut self,
        operator: &Signer<'info>,
        system_program: &Program<'info, System>,
        receipt_token_program: &AccountInfo<'info>,
        receipt_token_lock_account: &AccountInfo<'info>,
        fund_reserve_account: &AccountInfo<'info>,
        fund_treasury_account: &AccountInfo<'info>,
        uninitialized_withdrawal_batch_accounts: &[&'info AccountInfo<'info>],
        pricing_sources: &[&'info AccountInfo<'info>],
        forced: bool,
        receipt_token_amount_to_process: u64,
    ) -> Result<u64> {
        if !(forced
            || self
                .fund_account
                .withdrawal
                .is_batch_processing_threshold_satisfied(self.current_timestamp))
        {
            // threshold unmet, skip process
            return Ok(0);
        }

        let pricing_service = self.new_pricing_service(pricing_sources.iter().cloned())?;

        let mut sol_user_amount_processing = 0;
        let mut sol_fee_amount_processing = 0;
        let mut receipt_token_amount_processing = 0;
        let mut processing_batch_count = 0;

        // examine withdrawal batches to process with current fund status
        for batch in &self.fund_account.withdrawal.queued_batches {
            let next_receipt_token_amount_processing =
                receipt_token_amount_processing + batch.receipt_token_amount;
            if next_receipt_token_amount_processing > receipt_token_amount_to_process {
                break;
            }

            let sol_amount = pricing_service.get_token_amount_as_sol(
                &self.receipt_token_mint.key(),
                batch.receipt_token_amount,
            )?;
            let sol_fee_amount = self
                .fund_account
                .withdrawal
                .get_sol_fee_amount(sol_amount)?;
            let sol_user_amount = sol_amount - sol_fee_amount;

            let next_sol_user_amount_processing = sol_user_amount_processing + sol_user_amount;
            let next_sol_fee_amount_processing = sol_fee_amount_processing + sol_fee_amount;

            // [sol_user_amount_processing] should primarily be covered by cash.
            // condition 1: sol_operation_reserved_amount (cash) >= sol_user_amount_processing (cash/debt)
            // if condition 1 is met, the user's sol withdrawal will be fully processed using cash.

            // if condition 1 fails, the withdrawal can still proceed if:
            // condition 1-2: sol_operation_reserved_amount (cash) + sol_treasury_reserved_amount (cash/debt) >= sol_user_amount_processing (cash/debt)
            // in this case, the user's sol withdrawal will rely on the treasury's ability to provide additional liquidity, even by taking on debt.

            // additionally, the [sol_fee_amount_processing], which belongs to the treasury, can be covered using sol_operation_receivable_amount (bonds).
            // the treasury is willing to accept receivables from the fund to offset its debt obligations.
            // this leads to condition 2:
            // sol_operation_reserved_amount (cash) + sol_operation_receivable_amount (bond) >= sol_user_amount_processing (cash/debt) + sol_fee_amount_processing (debt)

            // to summarize:
            // - sol_operation_reserved_amount + sol_operation_receivable_amount + [optional debt from sol_treasury_reserved_amount] will offset sol_user_amount_processing + sol_fee_amount_processing.
            // - sol_operation_reserved_amount + [optional debt from sol_treasury_reserved_amount] will offset sol_user_amount_processing.
            // - sol_operation_receivable_amount will offset sol_fee_amount_processing + [optional debt from sol_treasury_reserved_amount].
            // - any remaining portion of sol_fee_amount_processing not covered by the receivables will be offset by the leftover sol_operation_reserved_amount, transferring the surplus to the treasury fund as revenue.

            if self.fund_account.sol_operation_reserved_amount
                + self.fund_account.sol_operation_receivable_amount
                < next_sol_user_amount_processing + next_sol_fee_amount_processing
                || self.fund_account.sol_operation_reserved_amount
                    + fund_treasury_account.lamports()
                    < next_sol_user_amount_processing
            {
                break;
            }

            receipt_token_amount_processing = next_receipt_token_amount_processing;
            sol_user_amount_processing = next_sol_user_amount_processing;
            sol_fee_amount_processing = next_sol_fee_amount_processing;
            processing_batch_count += 1;
        }

        // borrow sol cash from treasury if needed (condition 1-2)
        if sol_user_amount_processing > self.fund_account.sol_operation_reserved_amount {
            let sol_debt_amount_from_treasury =
                sol_user_amount_processing - self.fund_account.sol_operation_reserved_amount;
            anchor_lang::system_program::transfer(
                CpiContext::new_with_signer(
                    system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: fund_treasury_account.clone(),
                        to: fund_reserve_account.clone(),
                    },
                    &[&self.fund_account.get_treasury_account_seeds()],
                ),
                sol_debt_amount_from_treasury,
            )?;

            // increase debt, can use this variable as the entire sol_fee_amount_processing should be given to the treasury account in the first place
            sol_fee_amount_processing += sol_debt_amount_from_treasury;

            // reserve transferred cash
            self.fund_account.sol_operation_reserved_amount += sol_debt_amount_from_treasury;
        }

        // burn receipt tokens
        anchor_spl::token_2022::burn(
            CpiContext::new_with_signer(
                receipt_token_program.to_account_info(),
                anchor_spl::token_2022::Burn {
                    mint: self.receipt_token_mint.to_account_info(),
                    from: receipt_token_lock_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[self.fund_account.get_seeds().as_ref()],
            ),
            receipt_token_amount_processing,
        )?;
        self.fund_account
            .reload_receipt_token_supply(self.receipt_token_mint)?;
        let receipt_token_amount_processed = receipt_token_amount_processing;

        // reserve each sol_user_amount to batch accounts: sol_operation_reserved_amount -= sol_user_amount_processing;
        let processing_batches = self
            .fund_account
            .withdrawal
            .dequeue_batches(processing_batch_count, self.current_timestamp)?;

        require_gte!(
            uninitialized_withdrawal_batch_accounts.len(),
            processing_batches.len(),
        );

        for (uninitialized_batch_account, batch) in uninitialized_withdrawal_batch_accounts
            .iter()
            .cloned()
            .zip(processing_batches)
        {
            // create a batch account
            let (batch_account_address, bump) = FundWithdrawalBatchAccount::find_account_address(
                &self.receipt_token_mint.key(),
                batch.batch_id,
            );
            require_keys_eq!(uninitialized_batch_account.key(), batch_account_address);
            let mut batch_account = {
                system_program.create_account(
                    uninitialized_batch_account,
                    FundWithdrawalBatchAccount::get_seeds(
                        &self.receipt_token_mint.key(),
                        batch.batch_id,
                    )
                    .iter()
                    .map(Vec::as_slice)
                    .collect::<Vec<_>>()
                    .as_slice(),
                    operator,
                    &[],
                    8 + FundWithdrawalBatchAccount::INIT_SPACE,
                )?;
                Account::<FundWithdrawalBatchAccount>::try_from_unchecked(
                    uninitialized_batch_account,
                )?
            };
            batch_account.initialize(bump, self.receipt_token_mint.key(), batch.batch_id);

            // reserve user_sol_amount of the batch account
            let sol_amount = pricing_service.get_token_amount_as_sol(
                &self.receipt_token_mint.key(),
                batch.receipt_token_amount,
            )?;
            let sol_fee_amount = self
                .fund_account
                .withdrawal
                .get_sol_fee_amount(sol_amount)?;
            let sol_user_amount = sol_amount - sol_fee_amount;

            self.fund_account.sol_operation_reserved_amount -= sol_amount;
            self.fund_account.withdrawal.sol_user_reserved_amount += sol_user_amount;
            batch_account.set_claimable_amount(
                batch.num_requests,
                batch.receipt_token_amount,
                sol_user_amount,
                sol_fee_amount,
                self.current_timestamp,
            );
            batch_account.exit(&crate::ID)?;

            sol_user_amount_processing -= sol_user_amount;
            receipt_token_amount_processing -= batch.receipt_token_amount;
        }

        // during evaluation, up to [processing_batch_count] lamports can be deducted.
        // any remaining sol_user_amount_processing simply remains to the fund (no further action).
        require_gte!(processing_batch_count as u64, sol_user_amount_processing);

        // pay the treasury debt with receivables first (no further action).
        let receivable_amount_to_pay = self
            .fund_account
            .sol_operation_receivable_amount
            .min(sol_fee_amount_processing);
        self.fund_account.sol_operation_receivable_amount -= receivable_amount_to_pay;
        sol_fee_amount_processing -= receivable_amount_to_pay;

        // pay remaining debt with cash
        if sol_fee_amount_processing > 0 {
            anchor_lang::system_program::transfer(
                CpiContext::new_with_signer(
                    system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: fund_reserve_account.clone(),
                        to: fund_treasury_account.clone(),
                    },
                    &[&self.fund_account.get_reserve_account_seeds()],
                ),
                sol_fee_amount_processing,
            )?;
            sol_fee_amount_processing = 0;
        }
        self.fund_account.sol_operation_reserved_amount -= sol_fee_amount_processing;

        require_eq!(
            sol_user_amount_processing
                + sol_fee_amount_processing
                + receipt_token_amount_processing,
            0
        );
        Ok(receipt_token_amount_processed)
    }

    /// returns (sol_amount_transferred)
    pub(super) fn harvest_from_treasury_account(
        &mut self,
        system_program: &Program<'info, System>,
        fund_treasury_account: &AccountInfo<'info>,
        to_account: &'info AccountInfo<'info>,
    ) -> Result<u64> {
        require_keys_eq!(
            fund_treasury_account.key(),
            self.fund_account.get_treasury_account_address()?
        );

        let treasury_account_lamports = fund_treasury_account.lamports();
        if treasury_account_lamports == 0 {
            return Ok(0);
        }

        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: fund_treasury_account.clone(),
                    to: to_account.to_account_info(),
                },
                &[&self.fund_account.get_treasury_account_seeds()],
            ),
            treasury_account_lamports,
        )?;

        Ok(treasury_account_lamports)
    }

    /// receipt token amount in queued withdrawals.
    pub(super) fn get_receipt_token_withdrawal_obligated_amount(&self) -> u64 {
        self.fund_account
            .withdrawal
            .queued_batches
            .iter()
            .map(|b| b.receipt_token_amount)
            .sum()
    }

    /// estimated $SOL amount to process queued withdrawals.
    pub(super) fn get_sol_withdrawal_obligated_reserve_amount(
        &self,
        pricing_service: &PricingService,
    ) -> Result<u64> {
        pricing_service.get_token_amount_as_sol(
            &self.receipt_token_mint.key(),
            self.get_receipt_token_withdrawal_obligated_amount(),
        )
    }

    /// based on normal reserve configuration, the normal reserve amount relative to total value of the fund.
    fn get_sol_withdrawal_normal_reserve_amount(
        &self,
        pricing_service: &PricingService,
    ) -> Result<u64> {
        let (total_token_value_as_sol, _total_token_amount) =
            pricing_service.get_token_total_value_as_sol(&self.receipt_token_mint.key())?;
        Ok(get_proportional_amount(
            total_token_value_as_sol,
            self.fund_account.withdrawal.sol_normal_reserve_rate_bps as u64,
            10_000,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?
        .max(self.fund_account.withdrawal.sol_normal_reserve_max_amount))
    }

    /// total $SOL amount required for withdrawal in current state, including normal reserve if there is remaining sol_operation_reserved_amount after withdrawal obligation met.
    /// sol_withdrawal_obligated_reserve_amount + MIN(sol_withdrawal_normal_reserve_amount, MAX(0, sol_operation_reserved_amount - sol_withdrawal_obligated_reserve_amount))
    fn get_sol_withdrawal_reserve_amount(&self, pricing_service: &PricingService) -> Result<u64> {
        let sol_withdrawal_obligated_reserve_amount =
            self.get_sol_withdrawal_obligated_reserve_amount(pricing_service)?;

        Ok(sol_withdrawal_obligated_reserve_amount
            + self
                .get_sol_withdrawal_normal_reserve_amount(pricing_service)?
                .min(
                    self.fund_account
                        .sol_operation_reserved_amount
                        .saturating_sub(sol_withdrawal_obligated_reserve_amount),
                ))
    }

    /// surplus/shortage will be handled in staking stage.
    /// sol_operation_reserved_amount - sol_withdrawal_reserve_amount
    pub(super) fn get_sol_staking_reserved_amount(
        &self,
        pricing_service: &PricingService,
    ) -> Result<i128> {
        Ok(self.fund_account.sol_operation_reserved_amount as i128
            - self.get_sol_withdrawal_reserve_amount(pricing_service)? as i128)
    }
}
