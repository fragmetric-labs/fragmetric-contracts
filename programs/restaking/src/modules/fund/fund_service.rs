use std::collections::{BTreeMap, BTreeSet};

use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::pricing::{PricingService, TokenPricingSource};
use crate::modules::reward;
use crate::utils::*;

use super::command::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
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
        let remaining_accounts_map: BTreeMap<Pubkey, &AccountInfo> = remaining_accounts
            .iter()
            .map(|info| (*info.key, info))
            .collect();
        let mut executed_commands = Vec::new();

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
            // TODO v0.3/operation: write pricing sources in command?
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
                Ok(next_command) => {
                    // msg!("COMMAND: {:?} with {:?} passed", command, required_accounts);
                    // msg!("COMMAND#{}: {:?} passed", operation_state.sequence, command);
                    executed_commands.push(command.clone());
                    operation_state.set_command(next_command, self.current_timestamp);
                    execution_count += 1;
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

    pub(super) fn enqueue_withdrawal_batch(&mut self, forced: bool) -> Result<()> {
        if !(forced
            || self
                .fund_account
                .withdrawal
                .is_batch_enqueuing_threshold_satisfied(self.current_timestamp))
        {
            // Threshold unmet, skip enqueue
            return Ok(());
        }

        self.fund_account
            .withdrawal
            .enqueue_pending_batch(self.current_timestamp);

        Ok(())
    }

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
                        FundBatchWithdrawalTicketAccount::find_account_address(
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

    pub(super) fn process_withdrawal_batch(
        &mut self,
        operator: &Signer<'info>,
        system_program: &Program<'info, System>,
        receipt_token_program: &AccountInfo<'info>,
        receipt_token_lock_account: &AccountInfo<'info>,
        fund_reserve_account: &AccountInfo<'info>,
        treasury_account: &AccountInfo<'info>,
        uninitialized_batch_withdrawal_tickets: &[&'info AccountInfo<'info>],
        pricing_sources: &[&'info AccountInfo<'info>],
        forced: bool,
    ) -> Result<()> {
        if !(forced
            || self
                .fund_account
                .withdrawal
                .is_batch_processing_threshold_satisfied(self.current_timestamp))
        {
            // Threshold unmet, skip process
            return Ok(());
        }

        let pricing_service = self.new_pricing_service(pricing_sources.iter().cloned())?;
        // TODO v0.3/operation: later use get_sol_withdrawal_obligated_reserve_amount
        let available_receipt_token_amount_to_process = pricing_service.get_sol_amount_as_token(
            &self.receipt_token_mint.key(),
            self.fund_account.sol_operation_reserved_amount,
        )?;
        let processible_batches = self.fund_account.withdrawal.dequeue_processible_batches(
            available_receipt_token_amount_to_process,
            self.current_timestamp,
        );

        #[cfg(debug_assertions)]
        require_gte!(
            uninitialized_batch_withdrawal_tickets.len(),
            processible_batches.len(),
        );

        let receipt_token_amount_to_burn: u64 = processible_batches
            .iter()
            .map(|batch| batch.receipt_token_amount)
            .sum();

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
            receipt_token_amount_to_burn,
        )?;

        self.fund_account
            .reload_receipt_token_supply(self.receipt_token_mint)?;

        let mut sol_withdrawal_fee = 0;
        for (ticket, batch) in uninitialized_batch_withdrawal_tickets
            .iter()
            .cloned()
            .zip(processible_batches)
        {
            let (ticket_address, bump) = FundBatchWithdrawalTicketAccount::find_account_address(
                &self.receipt_token_mint.key(),
                batch.batch_id,
            );

            require_keys_eq!(ticket.key(), ticket_address);

            // create account
            let mut ticket = {
                system_program.create_account(
                    ticket,
                    FundBatchWithdrawalTicketAccount::get_seeds(
                        &self.receipt_token_mint.key(),
                        batch.batch_id,
                    )
                    .iter()
                    .map(Vec::as_slice)
                    .collect::<Vec<_>>()
                    .as_slice(),
                    operator,
                    &[],
                    8 + FundBatchWithdrawalTicketAccount::INIT_SPACE,
                )?;
                Account::<FundBatchWithdrawalTicketAccount>::try_from_unchecked(ticket)?
            };
            ticket.initialize(bump, self.receipt_token_mint.key(), batch.batch_id);

            let sol_amount = pricing_service.get_token_amount_as_sol(
                &self.receipt_token_mint.key(),
                batch.receipt_token_amount,
            )?;
            self.fund_account.set_batch_withdrawal_ticket(
                &mut ticket,
                batch,
                sol_amount,
                self.current_timestamp,
            )?;
            sol_withdrawal_fee += ticket.sol_fee_amount;

            ticket.exit(&crate::ID)?;
        }

        // transfer fee to treasury
        // TODO v0.3/operation: use sol_operation_receivable_expense in fund account
        let mut sol_operation_receivable_expense = 0;
        let sol_to_transfer = sol_withdrawal_fee.saturating_sub(sol_operation_receivable_expense);
        sol_operation_receivable_expense -= sol_withdrawal_fee.saturating_sub(sol_to_transfer);

        if sol_to_transfer > 0 {
            anchor_lang::system_program::transfer(
                CpiContext::new_with_signer(
                    system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: fund_reserve_account.clone(),
                        to: treasury_account.clone(),
                    },
                    &[&self.fund_account.get_reserve_account_seeds()],
                ),
                sol_to_transfer,
            )?;
        }

        Ok(())
    }
}
