use anchor_lang::prelude::*;
use anchor_spl::token::accessor::{amount, mint};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::{token_2022, token_interface};
use std::cell::RefMut;
use std::cmp::min;

use crate::errors::ErrorCode;
use crate::modules::pricing::{PricingService, TokenPricingSource};
use crate::modules::reward;
use crate::utils::*;
use crate::{events, utils};

use super::command::{OperationCommandAccountMeta, OperationCommandContext, OperationCommandEntry, SelfExecutable, OPERATION_COMMAND_MAX_ACCOUNT_SIZE};
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

    fn get_pricing_source_infos(
        &self,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<Vec<&'info AccountInfo<'info>>> {
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
            .map(|pricing_source| match pricing_source.try_deserialize()? {
                Some(TokenPricingSource::SPLStakePool { address })
                | Some(TokenPricingSource::MarinadeStakePool { address })
                | Some(TokenPricingSource::JitoRestakingVault { address })
                | Some(TokenPricingSource::FragmetricNormalizedTokenPool { address }) => {
                    for remaining_account in remaining_accounts {
                        if address == remaining_account.key() {
                            return Ok(remaining_account);
                        }
                    }
                    err!(ErrorCode::TokenPricingSourceAccountNotFoundError)
                }
                _ => err!(ErrorCode::TokenPricingSourceAccountNotFoundError),
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

            for supported_token in fund_account.get_supported_tokens_iter_mut() {
                supported_token.one_token_as_sol = pricing_service.get_token_amount_as_sol(
                    &supported_token.mint,
                    10u64
                        .checked_pow(supported_token.decimals as u32)
                        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
                )?;
            }

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
                .serialize_as_pod(&mut fund_account.receipt_token_value)?;

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
    ) -> Result<events::UserTransferredReceiptToken> {
        let [source_fund_account_option, source_reward_account_option, destination_fund_account_option, destination_reward_account_option, ..] =
            extra_accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys)?;
        };

        // parse extra accounts
        let mut source_fund_account_option =
            source_fund_account_option.parse_optional_account_boxed::<UserFundAccount>()?;
        let mut source_reward_account_option = source_reward_account_option
            .parse_optional_account_loader::<reward::UserRewardAccount>(
        )?;
        let mut destination_fund_account_option =
            destination_fund_account_option.parse_optional_account_boxed::<UserFundAccount>()?;
        let mut destination_reward_account_option = destination_reward_account_option
            .parse_optional_account_loader::<reward::UserRewardAccount>(
        )?;

        // transfer source's reward accrual rate to destination
        let event1 = reward::RewardService::new(self.receipt_token_mint, reward_account)?
            .update_reward_pools_token_allocation(
                source_reward_account_option.as_mut(),
                destination_reward_account_option.as_mut(),
                transfer_amount,
                None,
            )?;

        // sync user fund accounts
        if let Some(source_fund_account) = source_fund_account_option.as_deref_mut() {
            source_fund_account.reload_receipt_token_amount(source_receipt_token_account)?;
            source_fund_account.exit(&crate::ID)?;
        }
        if let Some(destination_fund_account) = destination_fund_account_option.as_deref_mut() {
            destination_fund_account
                .reload_receipt_token_amount(destination_receipt_token_account)?;
            destination_fund_account.exit(&crate::ID)?;
        }

        if self.fund_account.load()?.transfer_enabled != 1 {
            err!(ErrorCode::TokenNotTransferableError)?;
        }

        Ok(events::UserTransferredReceiptToken {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: self.fund_account.key(),
            updated_user_reward_accounts: event1.updated_user_reward_accounts,

            source: source_receipt_token_account.owner,
            source_receipt_token_account: source_receipt_token_account.key(),
            source_fund_account: source_fund_account_option.map(|account| account.key()),

            destination: destination_receipt_token_account.owner,
            destination_receipt_token_account: destination_receipt_token_account.key(),
            destination_fund_account: destination_fund_account_option.map(|account| account.key()),

            transferred_receipt_token_amount: transfer_amount,
        })
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
    ) -> Result<events::OperatorRanFund> {
        let pricing_source_infos = self.get_pricing_source_infos(remaining_accounts)?;

        let mut operation_state = self.clone_operation_state()?;
        operation_state.initialize_command_if_needed(
            reset_command,
            self.current_slot,
            self.current_timestamp,
        )?;

        let (command, required_accounts) = &operation_state
            .get_next_command()?
            .ok_or_else(|| error!(ErrorCode::FundOperationCommandExecutionFailedException))?;
        // rearrange given accounts in required order
        let mut required_account_infos = Vec::with_capacity(OPERATION_COMMAND_MAX_ACCOUNT_SIZE);
        let mut remaining_accounts_used: [bool; OPERATION_COMMAND_MAX_ACCOUNT_SIZE] = [false; OPERATION_COMMAND_MAX_ACCOUNT_SIZE];

        for required_account in required_accounts {
            // append required accounts in exact order
            let mut found = false;
            for (i, remaining_account) in remaining_accounts.iter().take(OPERATION_COMMAND_MAX_ACCOUNT_SIZE).enumerate() {
                if required_account.pubkey == *remaining_account.key {
                    required_account_infos.push(remaining_account);
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
                return err!(ErrorCode::FundOperationCommandAccountComputationException);
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
        for pricing_source in &pricing_source_infos {
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

        let next_operation_sequence = operation_state.next_sequence;

        // write back operation state
        std::mem::swap(
            &mut self.fund_account.load_mut()?.operation,
            operation_state.as_mut(),
        );

        Ok(events::OperatorRanFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: self.fund_account.key(),
            executed_command: command.clone(),
            next_operation_sequence,
        })
    }

    pub(super) fn enqueue_withdrawal_batches(&mut self, forced: bool) -> Result<bool> {
        let withdrawal_batch_threshold_interval_seconds = self
            .fund_account
            .load()?
            .withdrawal_batch_threshold_interval_seconds;

        let mut fund_account = self.fund_account.load_mut()?;
        let mut enqueued = fund_account.sol.enqueue_withdrawal_pending_batch(
            withdrawal_batch_threshold_interval_seconds,
            self.current_timestamp,
            forced,
        );

        for supported_token in fund_account.get_supported_tokens_iter_mut() {
            if supported_token.token.enqueue_withdrawal_pending_batch(
                withdrawal_batch_threshold_interval_seconds,
                self.current_timestamp,
                forced,
            ) {
                enqueued = true;
            }
        }

        Ok(enqueued)
    }

    /// returns [receipt_token_program, receipt_token_lock_account, fund_reserve_account, fund_treasury_account, supported_token_mint, supported_token_program, fund_supported_token_reserve_account, fund_supported_token_treasury_account, withdrawal_batch_accounts @ ..]
    pub(super) fn find_accounts_to_process_withdrawal_batches(
        &self,
        supported_token_mint: Option<Pubkey>,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let fund_account = self.fund_account.load()?;
        let supported_token = supported_token_mint
            .map(|mint| {
                Ok::<Option<&SupportedToken>, Error>(Some(fund_account.get_supported_token(&mint)?))
            })
            .unwrap_or_else(|| Ok(None))?;
        let asset = fund_account.get_asset_state(supported_token_mint)?;

        let mut accounts =
            Vec::with_capacity(8 + asset.get_withdrawal_queued_batches_iter().count());
        accounts.extend([
            (fund_account.receipt_token_program, false),
            (
                fund_account.find_receipt_token_lock_account_address()?,
                true,
            ),
            (fund_account.get_reserve_account_address()?, true),
            (fund_account.get_treasury_account_address()?, true),
            supported_token
                .map(|supported_token| (supported_token.mint, false))
                .unwrap_or_else(|| (Pubkey::default(), false)),
            supported_token
                .map(|supported_token| (supported_token.program, false))
                .unwrap_or_else(|| (Pubkey::default(), false)),
            supported_token
                .map(|supported_token| {
                    Ok::<(Pubkey, bool), Error>((
                        fund_account
                            .find_supported_token_reserve_account_address(&supported_token.mint)?,
                        true,
                    ))
                })
                .unwrap_or_else(|| Ok((Pubkey::default(), false)))?,
            supported_token
                .map(|supported_token| {
                    Ok::<(Pubkey, bool), Error>((
                        fund_account
                            .find_supported_token_treasury_account_address(&supported_token.mint)?,
                        true,
                    ))
                })
                .unwrap_or_else(|| Ok((Pubkey::default(), false)))?,
        ]);
        accounts.extend(asset.get_withdrawal_queued_batches_iter().map(|batch| {
            (
                FundWithdrawalBatchAccount::find_account_address(
                    &self.receipt_token_mint.key(),
                    supported_token_mint.as_ref(),
                    batch.batch_id,
                )
                .0,
                true,
            )
        }));
        Ok(accounts)
    }

    /// returns (receipt_token_amount_processed)
    pub(super) fn process_withdrawal_batches(
        &mut self,
        operator: &Signer<'info>,
        system_program: &Program<'info, System>,
        receipt_token_program: &'info AccountInfo<'info>,
        receipt_token_lock_account: &'info AccountInfo<'info>,

        // for SOL
        fund_reserve_account: &'info AccountInfo<'info>,
        fund_treasury_account: &'info AccountInfo<'info>,

        // for supported token
        supported_token_mint: Option<&'info AccountInfo<'info>>,
        supported_token_program: Option<&'info AccountInfo<'info>>,
        fund_supported_token_reserve_account: Option<&'info AccountInfo<'info>>,
        fund_supported_token_treasury_account: Option<&'info AccountInfo<'info>>,

        uninitialized_withdrawal_batch_accounts: &[&'info AccountInfo<'info>],
        pricing_sources: &[&'info AccountInfo<'info>],
        forced: bool,
        receipt_token_amount_to_process: u64,
    ) -> Result<u64> {
        let (
            supported_token_mint,
            supported_token_mint_key,
            supported_token_program,
            fund_supported_token_reserve_account,
            fund_supported_token_treasury_account,
        ) = match &supported_token_mint {
            Some(supported_token_mint) => (
                Some(supported_token_mint.parse_interface_account_boxed::<Mint>()?),
                Some(supported_token_mint.key()),
                Some(Interface::<TokenInterface>::try_from(
                    supported_token_program.unwrap(),
                )?),
                Some(
                    fund_supported_token_reserve_account
                        .unwrap()
                        .parse_interface_account_boxed::<TokenAccount>()?,
                ),
                Some(
                    fund_supported_token_treasury_account
                        .unwrap()
                        .parse_interface_account_boxed::<TokenAccount>()?,
                ),
            ),
            _ => (None, None, None, None, None),
        };
        let asset_treasury_reserved_amount = match &fund_supported_token_treasury_account {
            Some(fund_supported_token_treasury_account) => {
                fund_supported_token_treasury_account.amount
            }
            None => fund_treasury_account.lamports(),
        };

        let mut asset_user_amount_processing = 0;
        let mut asset_fee_amount_processing = 0;
        let mut receipt_token_amount_processing = 0;
        let mut processing_batch_count = 0;

        let pricing_service = self.new_pricing_service(pricing_sources.iter().cloned())?;
        {
            let fund_account = self.fund_account.load()?;

            // examine withdrawal batches to process with current fund status
            let asset = fund_account.get_asset_state(supported_token_mint_key)?;
            for batch in asset.get_withdrawal_queued_batches_iter_to_process(
                fund_account.withdrawal_batch_threshold_interval_seconds,
                self.current_timestamp,
                forced,
            ) {
                let next_receipt_token_amount_processing =
                    receipt_token_amount_processing + batch.receipt_token_amount;
                if next_receipt_token_amount_processing > receipt_token_amount_to_process {
                    break;
                }

                let sol_amount = pricing_service.get_token_amount_as_sol(
                    &self.receipt_token_mint.key(),
                    batch.receipt_token_amount,
                )?;
                let asset_amount = if let Some(supported_token_mint) = &supported_token_mint_key {
                    pricing_service.get_sol_amount_as_token(supported_token_mint, sol_amount)?
                } else {
                    sol_amount
                };
                let asset_fee_amount = fund_account.get_withdrawal_fee_amount(asset_amount)?;
                let asset_user_amount = asset_amount - asset_fee_amount;

                let next_asset_user_amount_processing =
                    asset_user_amount_processing + asset_user_amount;
                let next_asset_fee_amount_processing =
                    asset_fee_amount_processing + asset_fee_amount;

                // [asset_user_amount_processing] should primarily be covered by cash.
                // condition 1: asset_operation_reserved_amount (cash) >= asset_user_amount_processing (cash/debt)
                // if condition 1 is met, the user's sol withdrawal will be fully processed using cash.

                // if condition 1 fails, the withdrawal can still proceed if:
                // condition 1-2: asset_operation_reserved_amount (cash) + asset_treasury_reserved_amount (cash/debt) >= asset_user_amount_processing (cash/debt)
                // in this case, the user's sol withdrawal will rely on the treasury's ability to provide additional liquidity, even by taking on debt.

                // additionally, the [asset_fee_amount_processing], which belongs to the treasury, can be covered using asset_operation_receivable_amount (bonds).
                // the treasury is willing to accept receivables from the fund to offset its debt obligations.
                // this leads to condition 2:
                // asset_operation_reserved_amount (cash) + asset_operation_receivable_amount (bond) >= asset_user_amount_processing (cash/debt) + asset_fee_amount_processing (debt)

                // to summarize:
                // - asset_operation_reserved_amount + asset_operation_receivable_amount + [optional debt from asset_treasury_reserved_amount] will offset asset_user_amount_processing + asset_fee_amount_processing.
                // - asset_operation_reserved_amount + [optional debt from asset_treasury_reserved_amount] will offset asset_user_amount_processing.
                // - asset_operation_receivable_amount will offset asset_fee_amount_processing + [optional debt from asset_treasury_reserved_amount].
                // - any remaining portion of asset_fee_amount_processing not covered by the receivables will be offset by the leftover asset_operation_reserved_amount, transferring the surplus to the treasury fund as revenue.

                if asset.operation_reserved_amount + asset.operation_receivable_amount
                    < next_asset_user_amount_processing + next_asset_fee_amount_processing
                    || asset.operation_reserved_amount + asset_treasury_reserved_amount
                        < next_asset_user_amount_processing
                {
                    break;
                }

                receipt_token_amount_processing = next_receipt_token_amount_processing;
                asset_user_amount_processing = next_asset_user_amount_processing;
                asset_fee_amount_processing = next_asset_fee_amount_processing;
                processing_batch_count += 1;
            }

            // borrow asset (cash) from treasury if needed (condition 1-2)
            if asset_user_amount_processing > asset.operation_reserved_amount {
                let asset_debt_amount_from_treasury =
                    asset_user_amount_processing - asset.operation_reserved_amount;
                match &supported_token_mint {
                    Some(supported_token_mint) => {
                        token_interface::transfer_checked(
                            CpiContext::new_with_signer(
                                supported_token_program.as_ref().unwrap().to_account_info(),
                                token_interface::TransferChecked {
                                    from: fund_supported_token_treasury_account
                                        .as_ref()
                                        .unwrap()
                                        .to_account_info(),
                                    to: fund_supported_token_reserve_account
                                        .as_ref()
                                        .unwrap()
                                        .to_account_info(),
                                    mint: supported_token_mint.to_account_info(),
                                    authority: fund_treasury_account.to_account_info(),
                                },
                                &[&fund_account.get_treasury_account_seeds()],
                            ),
                            asset_debt_amount_from_treasury,
                            supported_token_mint.decimals,
                        )?;
                    }
                    None => {
                        anchor_lang::system_program::transfer(
                            CpiContext::new_with_signer(
                                system_program.to_account_info(),
                                anchor_lang::system_program::Transfer {
                                    from: fund_treasury_account.to_account_info(),
                                    to: fund_reserve_account.to_account_info(),
                                },
                                &[&fund_account.get_treasury_account_seeds()],
                            ),
                            asset_debt_amount_from_treasury,
                        )?;
                    }
                }

                // increase debt, can use this variable as the entire asset_fee_amount_processing should be given to the treasury account in the first place
                asset_fee_amount_processing += asset_debt_amount_from_treasury;

                // reserve transferred cash
                self.fund_account
                    .load_mut()?
                    .get_asset_state_mut(supported_token_mint_key)?
                    .operation_reserved_amount += asset_debt_amount_from_treasury;
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

            // reserve each asset_user_amount to batch accounts
            let processing_batches = fund_account
                .get_asset_state_mut(supported_token_mint_key)?
                .dequeue_withdrawal_batches(processing_batch_count, self.current_timestamp)?;
            drop(fund_account);

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
                        supported_token_mint_key.as_ref(),
                        batch.batch_id,
                    );
                require_keys_eq!(uninitialized_batch_account.key(), batch_account_address);
                let mut batch_account = {
                    system_program.create_account(
                        uninitialized_batch_account,
                        FundWithdrawalBatchAccount::get_seeds(
                            &self.receipt_token_mint.key(),
                            supported_token_mint_key.as_ref(),
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
                batch_account.initialize(
                    bump,
                    self.receipt_token_mint.key(),
                    supported_token_mint_key,
                    supported_token_program.as_ref().map(|program| program.key()),
                    batch.batch_id,
                );

                // reserve user_sol_amount of the batch account
                let sol_amount = pricing_service.get_token_amount_as_sol(
                    &self.receipt_token_mint.key(),
                    batch.receipt_token_amount,
                )?;
                let asset_amount = if let Some(supported_token_mint) = &supported_token_mint_key {
                    pricing_service.get_sol_amount_as_token(supported_token_mint, sol_amount)?
                } else {
                    sol_amount
                };
                let asset_fee_amount = self
                    .fund_account
                    .load()?
                    .get_withdrawal_fee_amount(asset_amount)?;
                let asset_user_amount = asset_amount - asset_fee_amount;

                // offset sol_user_amount and sol_fee_amount by sol_operation_reserved_amount
                let mut fund_account = self.fund_account.load_mut()?;
                let asset = fund_account.get_asset_state_mut(supported_token_mint_key)?;
                asset.operation_reserved_amount -= asset_amount;
                asset.withdrawal_user_reserved_amount += asset_user_amount;
                asset_user_amount_processing -= asset_user_amount;
                asset_fee_amount_processing -= asset_fee_amount;
                receipt_token_amount_processing -= batch.receipt_token_amount;

                batch_account.set_claimable_amount(
                    batch.num_requests,
                    batch.receipt_token_amount,
                    asset_user_amount,
                    asset_fee_amount,
                    self.current_timestamp,
                );
                batch_account.exit(&crate::ID)?;
            }

            // during evaluation, up to [processing_batch_count] amounts can be deducted.
            // any remaining asset_user_amount_processing simply remains to the fund (no further action).
            require_gte!(processing_batch_count as u64, asset_user_amount_processing);

            // pay the treasury debt with receivables first (no further action).
            let mut fund_account = self.fund_account.load_mut()?;
            let asset = fund_account.get_asset_state_mut(supported_token_mint_key)?;
            let receivable_amount_to_pay = asset
                .operation_receivable_amount
                .min(asset_fee_amount_processing);
            asset.operation_receivable_amount -= receivable_amount_to_pay;
            asset_fee_amount_processing -= receivable_amount_to_pay;

            // pay remaining debt with cash
            asset.operation_reserved_amount -= asset_fee_amount_processing;
            drop(fund_account);

            if asset_fee_amount_processing > 0 {
                let fund_account = self.fund_account.load()?;

                match supported_token_mint {
                    Some(supported_token_mint) => {
                        token_interface::transfer_checked(
                            CpiContext::new_with_signer(
                                supported_token_program.unwrap().to_account_info(),
                                token_interface::TransferChecked {
                                    from: fund_supported_token_reserve_account
                                        .unwrap()
                                        .to_account_info(),
                                    to: fund_supported_token_treasury_account
                                        .unwrap()
                                        .to_account_info(),
                                    mint: supported_token_mint.to_account_info(),
                                    authority: fund_reserve_account.to_account_info(),
                                },
                                &[&fund_account.get_reserve_account_seeds()],
                            ),
                            asset_fee_amount_processing,
                            supported_token_mint.decimals,
                        )?;
                    }
                    None => {
                        anchor_lang::system_program::transfer(
                            CpiContext::new_with_signer(
                                system_program.to_account_info(),
                                anchor_lang::system_program::Transfer {
                                    from: fund_reserve_account.to_account_info(),
                                    to: fund_treasury_account.to_account_info(),
                                },
                                &[&fund_account.get_reserve_account_seeds()],
                            ),
                            asset_fee_amount_processing,
                        )?;
                    }
                }
                asset_fee_amount_processing = 0;
            }
        }

        require_eq!(
            asset_user_amount_processing
                + asset_fee_amount_processing
                + receipt_token_amount_processing,
            0
        );
        Ok(receipt_token_amount_processed)
    }

    /// returns (sol_amount_transferred)
    pub(super) fn harvest_from_treasury_account(
        &mut self,
        system_program: &Program<'info, System>,

        // for SOL
        fund_treasury_account: &AccountInfo<'info>,
        program_revenue_account: &'info AccountInfo<'info>,

        // for supported token
        supported_token_mint: Option<&'info AccountInfo<'info>>,
        supported_token_program: Option<&'info AccountInfo<'info>>,
        fund_supported_token_treasury_account: Option<&'info AccountInfo<'info>>,
        program_supported_token_revenue_account: Option<&'info AccountInfo<'info>>,
    ) -> Result<u64> {
        let fund_account = self.fund_account.load()?;

        match supported_token_mint {
            Some(supported_token_mint) => {
                let supported_token_mint =
                    supported_token_mint.parse_interface_account_boxed::<Mint>()?;
                let fund_supported_token_treasury_account = fund_supported_token_treasury_account
                    .unwrap()
                    .parse_interface_account_boxed::<TokenAccount>()?;

                if fund_supported_token_treasury_account.amount == 0 {
                    Ok(0)
                } else {
                    // TODO: create program_supported_token_revenue_account if not exists

                    token_interface::transfer_checked(
                        CpiContext::new_with_signer(
                            supported_token_program.unwrap().to_account_info(),
                            token_interface::TransferChecked {
                                from: fund_supported_token_treasury_account.to_account_info(),
                                to: program_supported_token_revenue_account
                                    .unwrap()
                                    .to_account_info(),
                                mint: supported_token_mint.to_account_info(),
                                authority: fund_treasury_account.to_account_info(),
                            },
                            &[&fund_account.get_treasury_account_seeds()],
                        ),
                        fund_supported_token_treasury_account.amount,
                        supported_token_mint.decimals,
                    )?;

                    Ok(fund_supported_token_treasury_account.amount)
                }
            }
            None => {
                let treasury_account_lamports = fund_treasury_account.lamports();
                if treasury_account_lamports == 0 {
                    Ok(0)
                } else {
                    anchor_lang::system_program::transfer(
                        CpiContext::new_with_signer(
                            system_program.to_account_info(),
                            anchor_lang::system_program::Transfer {
                                from: fund_treasury_account.clone(),
                                to: program_revenue_account.to_account_info(),
                            },
                            &[&fund_account.get_treasury_account_seeds()],
                        ),
                        treasury_account_lamports,
                    )?;

                    Ok(treasury_account_lamports)
                }
            }
        }
    }

    /// receipt token amount in queued withdrawals.
    pub(super) fn get_receipt_token_withdrawal_obligated_amount(
        &self,
        supported_token_mint: Option<Pubkey>,
    ) -> Result<u64> {
        Ok(self
            .fund_account
            .load()?
            .get_asset_state(supported_token_mint)?
            .get_withdrawal_queued_batches_iter()
            .map(|b| b.receipt_token_amount)
            .sum())
    }

    /// estimated $SOL amount to process queued withdrawals.
    pub(super) fn get_asset_withdrawal_obligated_reserve_amount(
        &self,
        supported_token_mint: Option<Pubkey>,
        pricing_service: &PricingService,
    ) -> Result<u64> {
        pricing_service.get_token_amount_as_sol(
            &self.receipt_token_mint.key(),
            self.get_receipt_token_withdrawal_obligated_amount(supported_token_mint)?,
        )
    }

    /// based on asset normal reserve configuration, the normal reserve amount relative to total value of the fund.
    fn get_asset_withdrawal_normal_reserve_amount(
        &self,
        supported_token_mint: Option<Pubkey>,
        pricing_service: &PricingService,
    ) -> Result<u64> {
        let (total_token_value_as_sol, _total_token_amount) =
            pricing_service.get_token_total_value_as_sol(&self.receipt_token_mint.key())?;
        let fund_account = self.fund_account.load()?;
        let asset = fund_account.get_asset_state(supported_token_mint)?;

        Ok(get_proportional_amount(
            total_token_value_as_sol,
            asset.normal_reserve_rate_bps as u64,
            10_000,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?
        .max(asset.normal_reserve_max_amount))
    }

    /// total asset amount required for withdrawal in current state, including normal reserve if there is remaining asset_operation_reserved_amount after withdrawal obligation met.
    /// asset_withdrawal_obligated_reserve_amount + MIN(asset_withdrawal_normal_reserve_amount, MAX(0, asset_operation_reserved_amount - asset_withdrawal_obligated_reserve_amount))
    fn get_asset_withdrawal_reserve_amount(
        &self,
        supported_token_mint: Option<Pubkey>,
        pricing_service: &PricingService,
    ) -> Result<u64> {
        let asset_withdrawal_obligated_reserve_amount = self
            .get_asset_withdrawal_obligated_reserve_amount(supported_token_mint, pricing_service)?;

        Ok(asset_withdrawal_obligated_reserve_amount
            + self
                .get_asset_withdrawal_normal_reserve_amount(supported_token_mint, pricing_service)?
                .min(
                    self.fund_account
                        .load()?
                        .get_asset_state(supported_token_mint)?
                        .operation_reserved_amount
                        .saturating_sub(asset_withdrawal_obligated_reserve_amount),
                ))
    }

    /// which is going to be executed in a single withdrawal command. there can be remains after the execution.
    /// MIN(asset_withdrawal_obligated_reserve_amount, asset_operation_reserved_amount)
    pub(super) fn get_asset_withdrawal_execution_amount(
        &self,
        supported_token_mint: Option<Pubkey>,
        pricing_service: &PricingService,
    ) -> Result<u64> {
        Ok(self
            .fund_account
            .load()?
            .get_asset_state(supported_token_mint)?
            .operation_reserved_amount
            .min(self.get_asset_withdrawal_obligated_reserve_amount(
                supported_token_mint,
                pricing_service,
            )?))
    }

    /// surplus/shortage will be handled in staking stage.
    /// sol_operation_reserved_amount - sol_withdrawal_reserve_amount
    pub(super) fn get_sol_staking_reserved_amount(
        &self,
        pricing_service: &PricingService,
    ) -> Result<i128> {
        Ok(
            self.fund_account.load()?.sol.operation_reserved_amount as i128
                - self.get_asset_withdrawal_reserve_amount(None, pricing_service)? as i128,
        )
    }
}
