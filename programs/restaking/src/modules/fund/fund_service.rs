use crate::constants::ADMIN_PUBKEY;
use crate::errors::ErrorCode;
use crate::events;
use crate::modules::fund::command::{
    OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable,
    StakeSOLCommand,
};
use crate::modules::fund::{
    FundAccount, FundAccountInfo, UserFundAccount, UserFundConfigurationService,
};
use crate::modules::pricing::{PricingService, TokenPricingSource};
use crate::modules::reward::{RewardAccount, RewardService, UserRewardAccount};
use crate::modules::{fund, pricing};
use crate::utils;
use anchor_lang::prelude::*;
use anchor_spl::token::accessor::amount;
use anchor_spl::token_interface::{Mint, TokenAccount};
use spl_transfer_hook_interface::instruction::execute;
use std::collections::{BTreeMap, BTreeSet};
use anchor_spl::token_2022;
use crate::utils::PDASeeds;

pub struct FundService<'info: 'a, 'a> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    fund_account: &'a mut Account<'info, FundAccount>,
    _current_slot: u64,
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
            _current_slot: clock.slot,
            current_timestamp: clock.unix_timestamp,
        })
    }

    // create a pricing service and register fund assets' value resolvers
    pub(in crate::modules) fn new_pricing_service(
        &mut self,
        pricing_sources: &[AccountInfo<'info>],
    ) -> Result<PricingService<'info>> {
        // ensure any update on fund account written before do pricing
        self.fund_account.exit(&crate::ID)?;

        let mut pricing_service = PricingService::new(pricing_sources)?;
        pricing_service
            .register_token_pricing_source_account(self.fund_account.as_ref())
            .register_token_pricing_source_account(self.receipt_token_mint.as_ref())
            .resolve_token_pricing_source(
                &self.fund_account.receipt_token_mint.key(),
                &TokenPricingSource::FundReceiptToken {
                    mint_address: self.fund_account.receipt_token_mint.key(),
                    fund_address: self.fund_account.key(),
                },
            )?;

        // try to update current underlying assets' price
        self.update_asset_prices(&pricing_service)?;

        Ok(pricing_service)
    }

    // values being updated below are informative, only for event emission.
    fn update_asset_prices(&mut self, pricing_service: &PricingService) -> Result<()> {
        self.fund_account
            .supported_tokens
            .iter_mut()
            .try_for_each(|supported_token| {
                supported_token.one_token_as_sol = pricing_service.get_token_amount_as_sol(
                    &supported_token.get_mint(),
                    10u64
                        .checked_pow(supported_token.get_decimals() as u32)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
                )?;

                Ok::<(), Error>(())
            })?;

        self.fund_account.one_receipt_token_as_sol = pricing_service.get_token_amount_as_sol(
            &self.receipt_token_mint.key(),
            10u64
                .checked_pow(self.receipt_token_mint.decimals as u32)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
        )?;

        Ok(())
    }

    pub fn process_update_prices(
        &mut self,
        token_pricing_source_accounts: &'a [AccountInfo<'info>],
    ) -> Result<()> {
        self.new_pricing_service(token_pricing_source_accounts)?;

        emit!(events::OperatorUpdatedFundPrice {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: FundAccountInfo::from(self.fund_account, self.receipt_token_mint),
        });

        Ok(())
    }

    pub fn process_transfer_hook(
        &self,
        reward_account: &mut AccountLoader<'info, RewardAccount>,
        source_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
        destination_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
        extra_accounts: &'info [AccountInfo<'info>],
        transfer_amount: u64,
    ) -> Result<()> {
        // parse extra accounts
        let source_fund_account_info = extra_accounts
            .get(0)
            .ok_or(ProgramError::NotEnoughAccountKeys)?;
        let source_fund_account_option =
            utils::parse_optional_account_boxed::<UserFundAccount>(source_fund_account_info)?;
        let source_reward_account_info = extra_accounts
            .get(1)
            .ok_or(ProgramError::NotEnoughAccountKeys)?;
        let mut source_reward_account_option = utils::parse_optional_account_loader_boxed::<
            UserRewardAccount,
        >(source_reward_account_info)?;
        let destination_fund_account_info = extra_accounts
            .get(2)
            .ok_or(ProgramError::NotEnoughAccountKeys)?;
        let destination_fund_account_option =
            utils::parse_optional_account_boxed::<UserFundAccount>(destination_fund_account_info)?;
        let destination_reward_account_info = extra_accounts
            .get(3)
            .ok_or(ProgramError::NotEnoughAccountKeys)?;
        let mut destination_reward_account_option = utils::parse_optional_account_loader_boxed::<
            UserRewardAccount,
        >(destination_reward_account_info)?;

        // transfer source's reward accrual rate to destination
        RewardService::new(self.receipt_token_mint, reward_account)?
            .update_reward_pools_token_allocation(
                source_reward_account_option
                    .as_mut()
                    .map(|account_loader| &mut **account_loader),
                destination_reward_account_option
                    .as_mut()
                    .map(|account_loader| &mut **account_loader),
                transfer_amount,
                None,
            )?;

        // sync user fund accounts
        if let Some(mut source_fund_account) = source_fund_account_option {
            source_fund_account.sync_receipt_token_amount(source_receipt_token_account)?;
            source_fund_account.exit(&crate::ID)?;
        }
        if let Some(mut destination_fund_account) = destination_fund_account_option {
            destination_fund_account
                .sync_receipt_token_amount(destination_receipt_token_account)?;
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
        remaining_accounts: &[AccountInfo<'info>],
        reset_command: Option<OperationCommandEntry>,
    ) -> Result<()> {
        let mut operation_state = std::mem::take(&mut self.fund_account.operation);

        operation_state.initialize_command_if_needed(self.current_timestamp, reset_command)?;

        let mut execution_count = 0;
        let remaining_accounts_map: BTreeMap<Pubkey, &AccountInfo> = remaining_accounts
            .iter()
            .map(|info| (info.key.clone(), info))
            .collect();

        'command_loop: while let Some((command, required_accounts)) = operation_state.get_command()
        {
            let sequence = operation_state.get_sequence();

            // rearrange given accounts in required order
            let mut required_account_infos = Vec::new();
            let mut unused_account_keys = BTreeSet::new();
            remaining_accounts_map.keys().for_each(|key| {
                unused_account_keys.insert(*key);
            });

            for account_key in required_accounts.iter() {
                // append required accounts in exact order
                match remaining_accounts_map.get(&account_key) {
                    Some(account) => {
                        required_account_infos.push((*account).clone());
                        unused_account_keys.remove(&account_key);
                    }
                    None => {
                        if execution_count > 0 {
                            // maintain the current command and gracefully stop executing commands
                            msg!(
                                "COMMAND#{}: {:?} has not enough accounts after {} execution(s)",
                                sequence,
                                command,
                                execution_count
                            );
                            break 'command_loop;
                        }

                        // error if it is the first command in this tx
                        msg!(
                            "COMMAND#{}: {:?} has not enough accounts at the first execution",
                            sequence,
                            command
                        );
                        return err!(ErrorCode::OperationCommandAccountComputationException);
                    }
                }
            }

            // append all unused accounts
            for unused_account_key in unused_account_keys.iter() {
                let remaining_account = remaining_accounts_map.get(unused_account_key).unwrap();
                required_account_infos.push((*remaining_account).clone().clone());
            }

            let mut ctx = OperationCommandContext {
                receipt_token_mint: self.receipt_token_mint,
                fund_account: self.fund_account,
            };
            match command.execute(&mut ctx, required_account_infos.as_slice()) {
                Ok(next_command) => {
                    // msg!("COMMAND: {:?} with {:?} passed", command, required_accounts);
                    msg!("COMMAND#{}: {:?} passed", sequence, command);
                    operation_state.set_command(next_command, self.current_timestamp);
                    execution_count += 1;
                }
                Err(error) => {
                    // msg!("COMMAND: {:?} with {:?} failed", command, required_accounts);
                    msg!("COMMAND#{}: {:?} failed", sequence, command);
                    return Err(error);
                }
            };
        }

        // write back operation state
        self.fund_account.operation = operation_state;

        emit!(events::OperatorProcessedJob {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: FundAccountInfo::from(self.fund_account, self.receipt_token_mint),
        });

        Ok(())
    }

    pub(super) fn enqueue_withdrawal_batch(
        &mut self,
        receipt_token_program: AccountInfo<'info>,
        receipt_token_lock_account: AccountInfo<'info>,
        pricing_sources: &[AccountInfo<'info>],
    ) -> Result<()> {
        let mut withdrawal_state = std::mem::take(&mut self.fund_account.withdrawal);

        withdrawal_state.assert_withdrawal_threshold_satisfied(self.current_timestamp)?;
        withdrawal_state.start_processing_pending_batch_withdrawal(self.current_timestamp)?;

        let pricing_service = self.new_pricing_service(pricing_sources)?;

        let mut receipt_token_amount_to_burn: u64 = 0;
        for batch in &mut withdrawal_state.batch_withdrawals_in_progress {
            let amount = batch.receipt_token_to_process;
            batch.record_unstaking_start(amount)?;
            receipt_token_amount_to_burn = receipt_token_amount_to_burn
                .checked_add(amount)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        }

        let mut receipt_token_amount_not_burned = receipt_token_amount_to_burn;
        let mut total_sol_reserved_amount: u64 = 0;
        for batch in &mut withdrawal_state.batch_withdrawals_in_progress {
            if receipt_token_amount_not_burned == 0 {
                break;
            }

            let receipt_token_amount = std::cmp::min(
                receipt_token_amount_not_burned,
                batch.receipt_token_being_processed,
            );
            receipt_token_amount_not_burned -= receipt_token_amount; // guaranteed to be safe

            let sol_reserved_amount = pricing_service
                .get_token_amount_as_sol(&self.receipt_token_mint.key(), receipt_token_amount)?;
            total_sol_reserved_amount = total_sol_reserved_amount
                .checked_add(sol_reserved_amount)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
            batch.record_unstaking_end(receipt_token_amount, sol_reserved_amount)?;
        }
        self.fund_account.sol_operation_reserved_amount = self
            .fund_account
            .sol_operation_reserved_amount
            .checked_sub(total_sol_reserved_amount)
            .ok_or_else(|| error!(ErrorCode::FundOperationReservedSOLExhaustedException))?;

        token_2022::burn(
            CpiContext::new_with_signer(
                receipt_token_program.to_account_info(),
                token_2022::Burn {
                    mint: self.receipt_token_mint.to_account_info(),
                    from: receipt_token_lock_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[self.fund_account.get_signer_seeds().as_ref()],
            ),
            receipt_token_amount_to_burn,
        )?;
        self.receipt_token_mint.reload()?;
        // TODO: receipt_token_lock_account.reload()?;

        withdrawal_state.end_processing_completed_batch_withdrawals(self.current_timestamp)?;

        // write back operation state
        self.fund_account.withdrawal = withdrawal_state;

        Ok(())
    }
}
