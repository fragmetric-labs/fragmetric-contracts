use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use anchor_spl::token_interface::{Mint, TokenAccount};
use std::cell::RefMut;
use std::cmp::min;

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
    fund_account: &'a mut AccountLoader<'info, FundAccount>,
    current_timestamp: i64,
    current_slot: u64,
}

impl Drop for FundService<'_, '_> {
    fn drop(&mut self) {
        self.fund_account.exit(&crate::ID).unwrap();
    }
}

impl<'info: 'a, 'a> FundService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        fund_account: &'a mut AccountLoader<'info, FundAccount>,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            fund_account,
            current_timestamp: clock.unix_timestamp,
            current_slot: clock.slot,
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
        let fund_account = self.fund_account.load()?;

        fund_account
            .get_normalized_token()
            .iter()
            .map(|normalized_token| &normalized_token.pricing_source)
            .chain(
                fund_account
                    .get_restaking_vaults_iter()
                    .map(|restaking_vault| &restaking_vault.receipt_token_pricing_source),
            )
            .chain(
                fund_account
                    .get_supported_tokens_iter()
                    .map(|supported_token| &supported_token.pricing_source),
            )
            .map(|pricing_source| {
                Ok(match pricing_source.try_deserialize()? {
                    Some(TokenPricingSource::SPLStakePool { address })
                    | Some(TokenPricingSource::MarinadeStakePool { address })
                    | Some(TokenPricingSource::JitoRestakingVault { address })
                    | Some(TokenPricingSource::FragmetricNormalizedTokenPool { address }) => {
                        address
                    }
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

        {
            // update fund asset values
            let receipt_token_mint_key = self.fund_account.load()?.receipt_token_mint.key();
            pricing_service.resolve_token_pricing_source(
                &receipt_token_mint_key,
                &TokenPricingSource::FragmetricRestakingFund {
                    address: self.fund_account.key(),
                },
            )?;
        }

        {
            // the values being written below are informative, only for event emission.
            let mut fund_account = self.fund_account.load_mut()?;

            fund_account
                .get_supported_tokens_iter_mut()
                .try_for_each(|supported_token| {
                    supported_token.one_token_as_sol = pricing_service.get_token_amount_as_sol(
                        &supported_token.mint,
                        10u64
                            .checked_pow(supported_token.decimals as u32)
                            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
                    )?;

                    Ok::<(), Error>(())
                })?;

            if let Some(normalized_token) = fund_account.get_normalized_token_mut() {
                normalized_token.one_token_as_sol = pricing_service.get_token_amount_as_sol(
                    &normalized_token.mint,
                    10u64
                        .checked_pow(normalized_token.decimals as u32)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
                )?;
            }

            let receipt_token_mint_key = &self.receipt_token_mint.key();
            fund_account.one_receipt_token_as_sol = pricing_service.get_token_amount_as_sol(
                receipt_token_mint_key,
                10u64
                    .checked_pow(self.receipt_token_mint.decimals as u32)
                    .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
            )?;

            pricing_service
                .get_token_total_value_as_atomic(receipt_token_mint_key)?
                .serialize_as_pod(&mut fund_account.receipt_token_value);

            fund_account.receipt_token_value_updated_slot = self.current_slot;
        }

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

    #[inline(never)]
    fn clone_operation_state(&self) -> Result<Box<OperationState>> {
        Ok(Box::new(self.fund_account.load()?.operation.clone()))
    }

    pub fn process_run(
        &mut self,
        operator: &Signer<'info>,
        system_program: &Program<'info, System>,
        remaining_accounts: &'info [AccountInfo<'info>],
        reset_command: Option<OperationCommandEntry>,
    ) -> Result<()> {
        let mut operation_state = self.clone_operation_state()?;
        operation_state.initialize_command_if_needed(
            reset_command,
            self.current_slot,
            self.current_timestamp,
        )?;

        let pricing_sources_info = self
            .get_pricing_sources()?
            .into_iter()
            .map(|source| {
                for remaining_account in remaining_accounts {
                    if source == *remaining_account.key {
                        return Ok(remaining_account);
                    }
                }
                Err(error!(ErrorCode::TokenPricingSourceAccountNotFoundError))
            })
            .collect::<Result<Vec<_>>>()?;

        let (command, required_accounts) = &operation_state
            .get_next_command()?
            .ok_or_else(|| error!(ErrorCode::OperationCommandExecutionFailedException))?;
        // rearrange given accounts in required order
        let mut required_account_infos = Vec::with_capacity(32);
        let mut remaining_accounts_used: [bool; 32] = [false; 32];

        for required_account in required_accounts {
            // append required accounts in exact order
            let mut found = false;
            for (i, remaining_account) in remaining_accounts.iter().enumerate() {
                if required_account.pubkey == *remaining_account.key {
                    required_account_infos.push(remaining_account);
                    // SAFETY: the length of remaining_accounts is less or equal to 27
                    remaining_accounts_used[i] = true;
                    found = true;
                    break;
                }
            }

            if !found {
                // error if it is the first command in this tx
                msg!(
                    "COMMAND#{}: {:?} has not given all the required accounts",
                    operation_state.next_sequence,
                    command
                );
                return err!(ErrorCode::OperationCommandAccountComputationException);
            }
        }

        // append all unused accounts & pricing sources
        for (i, used) in remaining_accounts_used
            .into_iter()
            .take(remaining_accounts.len())
            .enumerate()
        {
            if !used {
                required_account_infos.push(&remaining_accounts[i]);
            }
        }
        for pricing_source in &pricing_sources_info {
            if required_account_infos.len() == 32 {
                break;
            }
            required_account_infos.push(*pricing_source);
        }

        // execute the command
        let mut ctx = OperationCommandContext {
            operator,
            receipt_token_mint: self.receipt_token_mint,
            fund_account: self.fund_account,
            system_program,
        };
        match command.execute(&mut ctx, required_account_infos.as_slice()) {
            Ok(next_command) => {
                msg!(
                    "COMMAND#{}: {:?} passed",
                    operation_state.next_sequence,
                    command
                );
                operation_state.set_command(
                    next_command,
                    self.current_slot,
                    self.current_timestamp,
                )?;
            }
            Err(error) => {
                msg!(
                    "COMMAND#{}: {:?} failed",
                    operation_state.next_sequence,
                    command
                );
                return Err(error);
            }
        };

        // write back operation state
        std::mem::swap(
            &mut self.fund_account.load_mut()?.operation,
            operation_state.as_mut(),
        );

        emit!(events::OperatorRanFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: FundAccountInfo::from(self.fund_account.load()?)?,
            executed_command: command.clone(),
        });

        Ok(())
    }

    pub(super) fn enqueue_withdrawal_batch(&mut self, forced: bool) -> Result<bool> {
        Ok(self
            .fund_account
            .load_mut()?
            .withdrawal
            .enqueue_pending_batch(self.current_timestamp, forced))
    }

    /// returns [receipt_token_program, receipt_token_lock_account, fund_reserve_account, fund_treasury_account, withdrawal_batch_accounts @ ..]
    pub(super) fn find_accounts_to_process_withdrawal_batch(&self) -> Result<Vec<(Pubkey, bool)>> {
        let fund_account = self.fund_account.load()?;

        let mut accounts =
            Vec::with_capacity(4 + fund_account.withdrawal.get_queued_batches_iter().count());
        accounts.extend([
            (fund_account.receipt_token_program, false),
            (
                fund_account.find_receipt_token_lock_account_address()?,
                true,
            ),
            (fund_account.get_reserve_account_address()?, true),
            (fund_account.get_treasury_account_address()?, true),
        ]);
        accounts.extend(
            fund_account
                .withdrawal
                .get_queued_batches_iter()
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

    /// returns (receipt_token_amount_processed)
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
        let mut sol_user_amount_processing = 0;
        let mut sol_fee_amount_processing = 0;
        let mut receipt_token_amount_processing = 0;
        let mut processing_batch_count = 0;

        let pricing_service = self.new_pricing_service(pricing_sources.iter().cloned())?;
        {
            let fund_account = self.fund_account.load()?;

            // examine withdrawal batches to process with current fund status
            for batch in fund_account
                .withdrawal
                .get_queued_batches_iter_to_process(self.current_timestamp, forced)
            {
                let next_receipt_token_amount_processing =
                    receipt_token_amount_processing + batch.receipt_token_amount;
                if next_receipt_token_amount_processing > receipt_token_amount_to_process {
                    break;
                }

                let sol_amount = pricing_service.get_token_amount_as_sol(
                    &self.receipt_token_mint.key(),
                    batch.receipt_token_amount,
                )?;
                let sol_fee_amount = fund_account.withdrawal.get_sol_fee_amount(sol_amount)?;
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

                if fund_account.sol_operation_reserved_amount
                    + fund_account.sol_operation_receivable_amount
                    < next_sol_user_amount_processing + next_sol_fee_amount_processing
                    || fund_account.sol_operation_reserved_amount + fund_treasury_account.lamports()
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
            if sol_user_amount_processing > fund_account.sol_operation_reserved_amount {
                let sol_debt_amount_from_treasury =
                    sol_user_amount_processing - fund_account.sol_operation_reserved_amount;
                anchor_lang::system_program::transfer(
                    CpiContext::new_with_signer(
                        system_program.to_account_info(),
                        anchor_lang::system_program::Transfer {
                            from: fund_treasury_account.clone(),
                            to: fund_reserve_account.clone(),
                        },
                        &[&fund_account.get_treasury_account_seeds()],
                    ),
                    sol_debt_amount_from_treasury,
                )?;

                // increase debt, can use this variable as the entire sol_fee_amount_processing should be given to the treasury account in the first place
                sol_fee_amount_processing += sol_debt_amount_from_treasury;

                // reserve transferred cash
                self.fund_account.load_mut()?.sol_operation_reserved_amount +=
                    sol_debt_amount_from_treasury;
            }
        }

        if receipt_token_amount_processing > 0 {
            // burn receipt tokens
            let fund_account = self.fund_account.load()?;
            anchor_spl::token_2022::burn(
                CpiContext::new_with_signer(
                    receipt_token_program.to_account_info(),
                    anchor_spl::token_2022::Burn {
                        mint: self.receipt_token_mint.to_account_info(),
                        from: receipt_token_lock_account.to_account_info(),
                        authority: self.fund_account.to_account_info(),
                    },
                    &[fund_account.get_seeds().as_ref()],
                ),
                receipt_token_amount_processing,
            )?;
        }

        let receipt_token_amount_processed = receipt_token_amount_processing;

        if processing_batch_count > 0 {
            let mut fund_account = self.fund_account.load_mut()?;
            fund_account.reload_receipt_token_supply(self.receipt_token_mint)?;

            // reserve each sol_user_amount to batch accounts: sol_operation_reserved_amount -= sol_user_amount_processing;
            let processing_batches = fund_account
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
                let (batch_account_address, bump) =
                    FundWithdrawalBatchAccount::find_account_address(
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
                        &crate::ID,
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
                let sol_fee_amount = fund_account.withdrawal.get_sol_fee_amount(sol_amount)?;
                let sol_user_amount = sol_amount - sol_fee_amount;

                fund_account.sol_operation_reserved_amount -= sol_amount;
                fund_account.withdrawal.sol_user_reserved_amount += sol_user_amount;
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
            let receivable_amount_to_pay = fund_account
                .sol_operation_receivable_amount
                .min(sol_fee_amount_processing);
            fund_account.sol_operation_receivable_amount -= receivable_amount_to_pay;
            sol_fee_amount_processing -= receivable_amount_to_pay;

            // pay remaining debt with cash
            fund_account.sol_operation_reserved_amount -= sol_fee_amount_processing;
        }

        if sol_fee_amount_processing > 0 {
            let fund_account = self.fund_account.load()?;
            anchor_lang::system_program::transfer(
                CpiContext::new_with_signer(
                    system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: fund_reserve_account.clone(),
                        to: fund_treasury_account.clone(),
                    },
                    &[&fund_account.get_reserve_account_seeds()],
                ),
                sol_fee_amount_processing,
            )?;
            sol_fee_amount_processing = 0;
        }

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
        let fund_account = self.fund_account.load()?;
        require_keys_eq!(
            fund_treasury_account.key(),
            fund_account.get_treasury_account_address()?
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
                &[&fund_account.get_treasury_account_seeds()],
            ),
            treasury_account_lamports,
        )?;

        Ok(treasury_account_lamports)
    }

    /// receipt token amount in queued withdrawals.
    pub(super) fn get_receipt_token_withdrawal_obligated_amount(&self) -> Result<u64> {
        Ok(self
            .fund_account
            .load()?
            .withdrawal
            .get_queued_batches_iter()
            .map(|b| b.receipt_token_amount)
            .sum())
    }

    /// estimated $SOL amount to process queued withdrawals.
    pub(super) fn get_sol_withdrawal_obligated_reserve_amount(
        &self,
        pricing_service: &PricingService,
    ) -> Result<u64> {
        pricing_service.get_token_amount_as_sol(
            &self.receipt_token_mint.key(),
            self.get_receipt_token_withdrawal_obligated_amount()?,
        )
    }

    /// based on normal reserve configuration, the normal reserve amount relative to total value of the fund.
    fn get_sol_withdrawal_normal_reserve_amount(
        &self,
        pricing_service: &PricingService,
    ) -> Result<u64> {
        let (total_token_value_as_sol, _total_token_amount) =
            pricing_service.get_token_total_value_as_sol(&self.receipt_token_mint.key())?;
        let fund_account = self.fund_account.load()?;

        Ok(get_proportional_amount(
            total_token_value_as_sol,
            fund_account.withdrawal.sol_normal_reserve_rate_bps as u64,
            10_000,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?
        .max(fund_account.withdrawal.sol_normal_reserve_max_amount))
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
                        .load()?
                        .sol_operation_reserved_amount
                        .saturating_sub(sol_withdrawal_obligated_reserve_amount),
                ))
    }

    /// which is going to be executed in this stage. there can be remains after the execution.
    /// MIN(sol_withdrawal_obligated_reserve_amount, sol_operation_reserved_amount)
    pub(super) fn get_sol_withdrawal_execution_amount(
        &self,
        pricing_service: &PricingService,
    ) -> Result<u64> {
        Ok(self
            .fund_account
            .load()?
            .sol_operation_reserved_amount
            .min(self.get_sol_withdrawal_obligated_reserve_amount(pricing_service)?))
    }

    /// surplus/shortage will be handled in staking stage.
    /// sol_operation_reserved_amount - sol_withdrawal_reserve_amount
    pub(super) fn get_sol_staking_reserved_amount(
        &self,
        pricing_service: &PricingService,
    ) -> Result<i128> {
        Ok(
            self.fund_account.load()?.sol_operation_reserved_amount as i128
                - self.get_sol_withdrawal_reserve_amount(pricing_service)? as i128,
        )
    }
}
