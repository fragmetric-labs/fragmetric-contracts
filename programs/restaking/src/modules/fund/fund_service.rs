use std::ops::Neg;

use anchor_lang::prelude::*;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::pricing::{Asset, PricingService, TokenPricingSource, TokenValue};
use crate::modules::reward;
use crate::utils::*;

use super::commands::{OperationCommandContext, OperationCommandEntry, SelfExecutable};
use super::*;

pub struct FundService<'a, 'info> {
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

impl<'a, 'info> FundService<'a, 'info> {
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

    // create a pricing service and update current underlying assets' price
    pub(in crate::modules) fn new_pricing_service<I>(
        &mut self,
        pricing_sources: I,
        refresh_token_price: bool,
    ) -> Result<PricingService<'info>>
    where
        I: IntoIterator<Item = &'info AccountInfo<'info>> + Clone,
        I::IntoIter: ExactSizeIterator,
    {
        let mut pricing_service = if pricing_sources
            .clone()
            .into_iter()
            .find(|source| source.key() == self.fund_account.key())
            .is_some()
        {
            PricingService::new(pricing_sources)
        } else {
            PricingService::new(
                pricing_sources
                    .into_iter()
                    .chain([self.fund_account.as_account_info()]),
            )
        };

        // try to update current underlying assets' price
        self.update_asset_values(&mut pricing_service, refresh_token_price)?;

        Ok(pricing_service)
    }

    pub fn process_update_prices(
        &mut self,
        pricing_sources: &'info [AccountInfo<'info>],
    ) -> Result<events::OperatorUpdatedFundPrices> {
        let mut fund_account = self.fund_account.load_mut()?;
        fund_account.update_pricing_source_addresses()?;
        drop(fund_account);

        self.new_pricing_service(pricing_sources, true)?;
        Ok(events::OperatorUpdatedFundPrices {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: self.fund_account.key(),
        })
    }

    fn get_pricing_source_infos(
        &self,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<Vec<&'info AccountInfo<'info>>> {
        let fund_account = self.fund_account.load()?;

        let pricing_sources_max_len =
            FUND_ACCOUNT_MAX_SUPPORTED_TOKENS + 1 + FUND_ACCOUNT_MAX_RESTAKING_VAULTS;
        let mut pricing_sources = Vec::with_capacity(pricing_sources_max_len);

        fund_account
            .get_supported_tokens_iter()
            .map(|supported_token| &supported_token.pricing_source)
            .chain(
                fund_account
                    .get_restaking_vaults_iter()
                    .map(|restaking_vault| &restaking_vault.receipt_token_pricing_source),
            )
            .chain(
                fund_account
                    .get_normalized_token()
                    .into_iter()
                    .map(|normalized_token| &normalized_token.pricing_source),
            )
            .try_for_each(|pricing_source| match pricing_source.try_deserialize()? {
                Some(TokenPricingSource::SPLStakePool { address })
                | Some(TokenPricingSource::MarinadeStakePool { address })
                | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address })
                | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { address })
                | Some(TokenPricingSource::OrcaDEXLiquidityPool { address })
                | Some(TokenPricingSource::JitoRestakingVault { address })
                | Some(TokenPricingSource::FragmetricNormalizedTokenPool { address })
                | Some(TokenPricingSource::SolvBTCVault { address })
                | Some(TokenPricingSource::VirtualRestakingVault { address }) => {
                    for remaining_account in remaining_accounts {
                        if address == remaining_account.key() {
                            pricing_sources.push(remaining_account);
                            return Ok(());
                        }
                    }
                    err!(ErrorCode::TokenPricingSourceAccountNotFoundError)
                }
                Some(TokenPricingSource::FragmetricRestakingFund { .. }) | None => {
                    err!(ErrorCode::TokenPricingSourceAccountNotFoundError)
                }
                Some(TokenPricingSource::PeggedToken { .. }) => Ok(()),
                #[cfg(all(test, not(feature = "idl-build")))]
                Some(TokenPricingSource::Mock { .. }) => {
                    err!(ErrorCode::TokenPricingSourceAccountNotFoundError)
                }
            })?;

        Ok(pricing_sources)
    }

    pub(super) fn update_asset_values(
        &mut self,
        pricing_service: &mut PricingService,
        refresh_token_price: bool,
    ) -> Result<()> {
        // ensure any update on fund account written before do pricing
        self.fund_account.exit(&crate::ID)?;

        // update fund asset values
        let receipt_token_mint_key = self.fund_account.load()?.receipt_token_mint.key();
        pricing_service.resolve_token_pricing_source(
            &receipt_token_mint_key,
            &TokenPricingSource::FragmetricRestakingFund {
                address: self.fund_account.key(),
            },
        )?;

        {
            // the values being written below are informative, only for event emission.
            let mut fund_account = self.fund_account.load_mut()?;
            let mut receipt_token_value = TokenValue::default();

            pricing_service
                .flatten_token_value(&self.receipt_token_mint.key(), &mut receipt_token_value)?;
            receipt_token_value.serialize_as_pod(&mut fund_account.receipt_token_value)?;
            fund_account.receipt_token_value_updated_slot = self.current_slot;

            if refresh_token_price {
                for supported_token in fund_account.get_supported_tokens_iter_mut() {
                    supported_token.one_token_as_sol = pricing_service
                        .get_one_token_amount_as_sol(
                            &supported_token.mint,
                            supported_token.decimals,
                        )?
                        .unwrap_or_default();
                    supported_token.one_token_as_receipt_token = pricing_service
                        .get_one_token_amount_as_token(
                            &supported_token.mint,
                            supported_token.decimals,
                            &self.receipt_token_mint.key(),
                        )?
                        .unwrap_or_default()
                }

                if let Some(normalized_token) = fund_account.get_normalized_token_mut() {
                    normalized_token.one_token_as_sol = pricing_service
                        .get_one_token_amount_as_sol(
                            &normalized_token.mint,
                            normalized_token.decimals,
                        )?
                        .unwrap_or_default();
                }

                for restaking_vault in fund_account.get_restaking_vaults_iter_mut() {
                    restaking_vault.one_receipt_token_as_sol = pricing_service
                        .get_one_token_amount_as_sol(
                            &restaking_vault.receipt_token_mint,
                            restaking_vault.receipt_token_decimals,
                        )?
                        .unwrap_or_default();
                }

                fund_account.one_receipt_token_as_sol = pricing_service
                    .get_one_token_amount_as_sol(
                        &self.receipt_token_mint.key(),
                        self.receipt_token_mint.decimals,
                    )?
                    .unwrap_or_default();

                // now estimate withdrawal-request acceptable amount for each assets.
                let mut total_withdrawal_requested_receipt_token_amount = 0;

                // here, atomic assets of receipt_token_value should be either SOL or one of supported tokens.
                for asset_value in &receipt_token_value.numerator {
                    match asset_value {
                        Asset::SOL(..) => {
                            // just count the already processing withdrawal amount
                            total_withdrawal_requested_receipt_token_amount += fund_account
                                .sol
                                .get_receipt_token_withdrawal_requested_amount();
                        }
                        Asset::Token(token_mint, pricing_source, token_amount) => {
                            match pricing_source {
                                None => err!(ErrorCode::TokenPricingSourceAccountNotFoundError)?,
                                Some(pricing_source) => {
                                    #[deny(clippy::wildcard_enum_match_arm)]
                                    match pricing_source {
                                        TokenPricingSource::SPLStakePool { .. }
                                        | TokenPricingSource::MarinadeStakePool { .. }
                                        | TokenPricingSource::SanctumSingleValidatorSPLStakePool {
                                            ..
                                        }
                                        | TokenPricingSource::SanctumMultiValidatorSPLStakePool {
                                            ..
                                        }
                                        | TokenPricingSource::OrcaDEXLiquidityPool { .. }
                                        | TokenPricingSource::PeggedToken { .. } => {
                                            let asset =
                                                fund_account.get_asset_state_mut(Some(*token_mint))?;
                                            let asset_value_as_receipt_token_amount = pricing_service
                                                .get_token_amount_as_token(
                                                    token_mint,
                                                    *token_amount,
                                                    &self.receipt_token_mint.key(),
                                                )?;
                                            let withdrawal_requested_receipt_token_amount =
                                                asset.get_receipt_token_withdrawal_requested_amount();
                                            asset.withdrawable_value_as_receipt_token_amount =
                                                asset_value_as_receipt_token_amount.saturating_sub(
                                                    withdrawal_requested_receipt_token_amount,
                                                );

                                            // sum the already processing withdrawal amount
                                            total_withdrawal_requested_receipt_token_amount +=
                                                withdrawal_requested_receipt_token_amount;
                                        }
                                        TokenPricingSource::JitoRestakingVault { .. }
                                        | TokenPricingSource::SolvBTCVault { .. }
                                        | TokenPricingSource::VirtualRestakingVault { .. }
                                        | TokenPricingSource::FragmetricNormalizedTokenPool {
                                            ..
                                        }
                                        | TokenPricingSource::FragmetricRestakingFund { .. } => {}
                                        #[cfg(all(test, not(feature = "idl-build")))]
                                        TokenPricingSource::Mock { .. } => {}
                                    }
                                }
                            }
                        }
                    }
                }

                // when SOL is withdrawable, it assumes that any kind of underlying assets can be either unstaked, or swapped to be withdrawn as SOL.
                fund_account.sol.withdrawable_value_as_receipt_token_amount = fund_account
                    .receipt_token_supply_amount
                    - total_withdrawal_requested_receipt_token_amount;
            }
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
        let source_reward_account_option = source_reward_account_option
            .parse_optional_account_loader::<reward::UserRewardAccount>()?;
        let mut destination_fund_account_option =
            destination_fund_account_option.parse_optional_account_boxed::<UserFundAccount>()?;
        let destination_reward_account_option = destination_reward_account_option
            .parse_optional_account_loader::<reward::UserRewardAccount>(
        )?;

        // transfer source's reward accrual rate to destination
        let updated_user_reward_accounts =
            reward::RewardService::new(self.receipt_token_mint, reward_account)?
                .update_reward_pools_token_allocation(
                    source_reward_account_option.as_ref(),
                    destination_reward_account_option.as_ref(),
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
            updated_user_reward_accounts,

            source: source_receipt_token_account.owner,
            source_receipt_token_account: source_receipt_token_account.key(),
            source_fund_account: source_fund_account_option.map(|account| account.key()),

            destination: destination_receipt_token_account.owner,
            destination_receipt_token_account: destination_receipt_token_account.key(),
            destination_fund_account: destination_fund_account_option.map(|account| account.key()),

            transferred_receipt_token_amount: transfer_amount,
        })
    }

    pub fn process_run_command(
        &mut self,
        operator: &Signer<'info>,
        system_program: &Program<'info, System>,
        remaining_accounts: &'info [AccountInfo<'info>],
        reset_command: Option<OperationCommandEntry>,
    ) -> Result<events::OperatorRanFundCommand> {
        let pricing_source_accounts = self.get_pricing_source_infos(remaining_accounts)?;
        let mut fund_account = self.fund_account.load_mut()?;
        let operation_sequence = fund_account.operation.next_sequence;

        // First reset command
        fund_account.operation.initialize_command_if_needed(
            reset_command,
            self.current_slot,
            self.current_timestamp,
        )?;

        let OperationCommandEntry {
            command,
            required_accounts: required_account_metas,
        } = fund_account
            .operation
            .get_next_command()?
            .ok_or_else(|| error!(ErrorCode::FundOperationCommandExecutionFailedException))?;
        // rearrange given accounts in required order
        // accounts = [...required_accounts, ...pricing_sources]
        let mut operation_command_accounts = Vec::with_capacity(
            OperationCommandEntry::MAX_ACCOUNT_SIZE + pricing_source_accounts.len(),
        );
        for required_account_meta in required_account_metas {
            if let Some(remaining_account) = remaining_accounts
                .iter()
                .find(|remaining_account| required_account_meta.pubkey == *remaining_account.key)
            {
                operation_command_accounts.push(remaining_account);
            } else {
                msg!(
                    "COMMAND#{}: {:?} failed due to missing required account {}",
                    operation_sequence,
                    command,
                    required_account_meta.pubkey,
                );
                return err!(ErrorCode::FundOperationCommandAccountComputationException);
            }
        }
        operation_command_accounts.extend(pricing_source_accounts);

        // execute the command
        drop(fund_account);
        let mut ctx = OperationCommandContext {
            operator,
            receipt_token_mint: self.receipt_token_mint,
            fund_account: self.fund_account,
            system_program,
        };
        let (result, next_command) = match command.execute(&mut ctx, &operation_command_accounts) {
            Ok((result, next_command)) => {
                msg!("COMMAND#{}: {:?} passed", operation_sequence, command);
                (result, next_command)
            }
            Err(err) => {
                msg!("COMMAND#{}: {:?} failed", operation_sequence, command);
                Err(err)?
            }
        };

        let mut fund_account = ctx.fund_account.load_mut()?;
        fund_account.operation.set_command(
            next_command,
            self.current_slot,
            self.current_timestamp,
        )?;
        let next_sequence = fund_account.operation.next_sequence;
        let num_operated = fund_account.operation.num_operated;

        Ok(events::OperatorRanFundCommand {
            receipt_token_mint: ctx.receipt_token_mint.key(),
            fund_account: ctx.fund_account.key(),
            next_sequence,
            num_operated,
            command,
            result,
        })
    }

    /// returns [enqueued_receipt_token_amount]
    pub(super) fn enqueue_withdrawal_batches(&mut self, forced: bool) -> Result<u64> {
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
            enqueued += supported_token.token.enqueue_withdrawal_pending_batch(
                withdrawal_batch_threshold_interval_seconds,
                self.current_timestamp,
                forced,
            );
        }

        Ok(enqueued)
    }

    /// returns (num_processing_batches, [receipt_token_program, receipt_token_lock_account, fund_reserve_account, fund_treasury_account, supported_token_mint, supported_token_program, fund_supported_token_reserve_account, fund_supported_token_treasury_account, withdrawal_batch_accounts @ ..])
    pub(super) fn find_accounts_to_process_withdrawal_batches(
        &self,
        supported_token_mint: Option<Pubkey>,
        forced: bool,
    ) -> Result<(u8, Vec<(Pubkey, bool)>)> {
        let fund_account = self.fund_account.load()?;
        let supported_token = supported_token_mint
            .map(|mint| {
                Ok::<Option<&SupportedToken>, Error>(Some(fund_account.get_supported_token(&mint)?))
            })
            .unwrap_or_else(|| Ok(None))?;
        let asset = fund_account.get_asset_state(supported_token_mint)?;
        let withdrawal_batches = asset
            .get_queued_withdrawal_batches_to_process_iter(
                fund_account.withdrawal_batch_threshold_interval_seconds,
                self.current_timestamp,
                forced,
            )
            .collect::<Vec<_>>();

        let mut accounts = Vec::with_capacity(8 + withdrawal_batches.len());
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
        let num_withdrawal_batches = withdrawal_batches.len() as u8;
        accounts.extend(withdrawal_batches.into_iter().map(|batch| {
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
        Ok((num_withdrawal_batches, accounts))
    }

    /// returns [processed_receipt_token_amount, required_asset_amount, reserved_asset_user_amount, deducted_asset_fee_amount, offsetted_asset_receivables]
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
        forced: bool,
        receipt_token_amount_to_process: u64,
        _receipt_token_amount_to_return: u64,

        pricing_service: &PricingService,
    ) -> Result<(u64, u64, u64, u64, Vec<(Option<Pubkey>, u64)>)> {
        let (
            supported_token_mint,
            supported_token_mint_key,
            supported_token_program,
            fund_supported_token_reserve_account,
            fund_supported_token_treasury_account,
        ) = match &supported_token_mint {
            Some(supported_token_mint) => (
                Some(InterfaceAccount::<Mint>::try_from(supported_token_mint)?),
                Some(supported_token_mint.key()),
                Some(Interface::<TokenInterface>::try_from(
                    supported_token_program.unwrap(),
                )?),
                Some(InterfaceAccount::<TokenAccount>::try_from(
                    fund_supported_token_reserve_account.unwrap(),
                )?),
                Some(InterfaceAccount::<TokenAccount>::try_from(
                    fund_supported_token_treasury_account.unwrap(),
                )?),
            ),
            _ => (None, None, None, None, None),
        };
        let rent = Rent::get()?;
        let asset_treasury_reserved_amount = fund_supported_token_treasury_account
            .as_ref()
            .map(|account| account.amount)
            .unwrap_or_else(|| {
                fund_treasury_account
                    .lamports()
                    .saturating_sub(rent.minimum_balance(0))
            });

        let mut asset_user_amount_processing = 0;
        let mut asset_fee_amount_processing = 0;
        let mut receipt_token_amount_processing = 0;
        let mut processing_batch_count = 0;

        let fund_account = self.fund_account.load()?;
        let asset = fund_account.get_asset_state(supported_token_mint_key)?;
        let total_operation_receivable_amount_as_asset = fund_account
            .get_total_operation_receivable_amount_as_asset(
                supported_token_mint_key,
                pricing_service,
            )?;

        // examine withdrawal batches to process with current fund status
        for batch in asset.get_queued_withdrawal_batches_to_process_iter(
            fund_account.withdrawal_batch_threshold_interval_seconds,
            self.current_timestamp,
            forced,
        ) {
            let next_receipt_token_amount_processing =
                receipt_token_amount_processing + batch.receipt_token_amount;
            if next_receipt_token_amount_processing > receipt_token_amount_to_process {
                break;
            }

            let asset_amount = pricing_service.get_token_amount_as_asset(
                &self.receipt_token_mint.key(),
                batch.receipt_token_amount,
                supported_token_mint_key.as_ref(),
            )?;
            let asset_fee_amount = fund_account.get_withdrawal_fee_amount(asset_amount)?;
            let asset_user_amount = asset_amount - asset_fee_amount;

            let next_asset_user_amount_processing =
                asset_user_amount_processing + asset_user_amount;
            let next_asset_fee_amount_processing = asset_fee_amount_processing + asset_fee_amount;

            // [asset_user_amount_processing] should primarily be covered by cash.
            // condition 1: asset_operation_reserved_amount (cash) >= asset_user_amount_processing (cash/debt)
            // if condition 1 is met, the user's sol withdrawal will be fully processed using cash.

            // if condition 1 fails, the withdrawal can still proceed if:
            // condition 1-2: asset_operation_reserved_amount (cash) + asset_treasury_reserved_amount (cash/debt) >= asset_user_amount_processing (cash/debt)
            // in this case, the user's sol withdrawal will rely on the treasury's ability to provide additional liquidity, even by taking on debt.

            // additionally, the [asset_fee_amount_processing], which belongs to the treasury, can be paid by total_operation_receivable_amount_as_asset (bonds).
            // the treasury is willing to accept receivables from the fund to offset its debt obligations.
            // this leads to condition 2:
            // asset_operation_reserved_amount (cash) + total_operation_receivable_amount_as_asset (bond) >= asset_user_amount_processing (cash/debt) + asset_fee_amount_processing (debt)

            // to summarize:
            // - asset_operation_reserved_amount + total_operation_receivable_amount_as_asset + [optional debt from asset_treasury_reserved_amount] will offset asset_user_amount_processing + asset_fee_amount_processing.
            // - asset_operation_reserved_amount + [optional debt from asset_treasury_reserved_amount] will offset asset_user_amount_processing.
            // - total_operation_receivable_amount_as_asset will offset asset_fee_amount_processing + [optional debt from asset_treasury_reserved_amount].
            // - any remaining portion of asset_fee_amount_processing which cannot be paid by the receivables will be offset by the leftover asset_operation_reserved_amount, transferring the surplus to the treasury fund as revenue.

            // check cash is enough to pay user's share
            if next_asset_user_amount_processing
                > asset.operation_reserved_amount + asset_treasury_reserved_amount
            {
                break;
            }

            // check cash + receivable is enough to pay shares of each treasury and user.
            // here, the lack of pennies during calculation which originally belongs to the treasury is tolerable up to FUND_ACCOUNT_MAX_SUPPORTED_TOKENS (16).
            let lack_of_asset_amount = (next_asset_user_amount_processing
                + next_asset_fee_amount_processing)
                .saturating_sub(
                    asset.operation_reserved_amount + total_operation_receivable_amount_as_asset,
                );
            if lack_of_asset_amount > FUND_ACCOUNT_MAX_SUPPORTED_TOKENS as u64 {
                break;
            }

            receipt_token_amount_processing = next_receipt_token_amount_processing;
            asset_user_amount_processing = next_asset_user_amount_processing;
            asset_fee_amount_processing = next_asset_fee_amount_processing - lack_of_asset_amount;
            processing_batch_count += 1;
        }

        // borrow asset (cash) from treasury if needed (condition 1-2)
        if asset_user_amount_processing > asset.operation_reserved_amount {
            let asset_debt_amount_from_treasury =
                asset_user_amount_processing - asset.operation_reserved_amount;
            match &supported_token_mint {
                Some(supported_token_mint) => {
                    anchor_spl::token_interface::transfer_checked(
                        CpiContext::new_with_signer(
                            supported_token_program.as_ref().unwrap().to_account_info(),
                            anchor_spl::token_interface::TransferChecked {
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

            // increase debt, can use this variable as the entire asset_fee_amount_processing will be paid back to the treasury account.
            asset_fee_amount_processing += asset_debt_amount_from_treasury;

            // reserve borrowed cash
            self.fund_account
                .load_mut()?
                .get_asset_state_mut(supported_token_mint_key)?
                .operation_reserved_amount += asset_debt_amount_from_treasury;
        }
        drop(fund_account);

        // burn receipt tokens
        if receipt_token_amount_processing > 0 {
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
        let asset_user_amount_reserved = asset_user_amount_processing;
        let asset_fee_amount_deducted = asset_fee_amount_processing;
        let mut offsetted_asset_receivables = Vec::<(Option<Pubkey>, u64)>::new();

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
                    system_program.initialize_account(
                        uninitialized_batch_account,
                        operator,
                        &[FundWithdrawalBatchAccount::get_seeds(
                            &self.receipt_token_mint.key(),
                            supported_token_mint_key.as_ref(),
                            batch.batch_id,
                        )
                        .iter()
                        .map(Vec::as_slice)
                        .collect::<Vec<_>>()
                        .as_slice()],
                        8 + FundWithdrawalBatchAccount::INIT_SPACE,
                        None,
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
                    supported_token_program
                        .as_ref()
                        .map(|program| program.key()),
                    batch.batch_id,
                );

                // to reserve user_asset_amount of the batch account
                let mut withdrawal_residual_micro_asset_amount = self
                    .fund_account
                    .load()?
                    .get_asset_state(supported_token_mint_key)?
                    .withdrawal_residual_micro_asset_amount;
                let asset_amount = pricing_service.convert_asset_amount(
                    Some(&self.receipt_token_mint.key()),
                    batch.receipt_token_amount,
                    supported_token_mint_key.as_ref(),
                    &mut withdrawal_residual_micro_asset_amount,
                )?;
                let asset_fee_amount = self
                    .fund_account
                    .load()?
                    .get_withdrawal_fee_amount(asset_amount)?;
                let asset_user_amount = asset_amount - asset_fee_amount;

                // offset asset_user_amount by asset_operation_reserved_amount
                let mut fund_account = self.fund_account.load_mut()?;
                let asset = fund_account.get_asset_state_mut(supported_token_mint_key)?;
                asset.operation_reserved_amount -= asset_user_amount;
                asset.withdrawal_user_reserved_amount += asset_user_amount;
                asset_user_amount_processing =
                    asset_user_amount_processing.saturating_sub(asset_user_amount);
                receipt_token_amount_processing -= batch.receipt_token_amount;

                // update residual
                asset.withdrawal_residual_micro_asset_amount =
                    withdrawal_residual_micro_asset_amount;

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
            // any remaining asset_user_amount_processing simply remains to the fund.
            require_gte!(processing_batch_count as u64, asset_user_amount_processing);

            // pay the treasury debt now
            let (transferred_asset_amount, offsetted_asset_amount, offsetted_asset_receivables2) =
                self.pay_treasury_debt_with_receivables(
                    system_program,
                    // for SOL
                    fund_reserve_account,
                    fund_treasury_account,
                    // for supported token
                    &supported_token_mint,
                    &supported_token_program,
                    &fund_supported_token_reserve_account,
                    &fund_supported_token_treasury_account,
                    asset_fee_amount_processing,
                    total_operation_receivable_amount_as_asset,
                    pricing_service,
                )?;

            require_eq!(
                transferred_asset_amount + offsetted_asset_amount,
                asset_fee_amount_processing
            );
            offsetted_asset_receivables.extend(offsetted_asset_receivables2.into_iter());
            asset_fee_amount_processing = 0;
        }

        require_eq!(
            asset_user_amount_processing
                + asset_fee_amount_processing
                + receipt_token_amount_processing,
            0
        );

        let fund_account = self.fund_account.load()?;
        let asset_amount_required = u64::try_from(
            fund_account
                .get_asset_net_operation_reserved_amount(
                    supported_token_mint_key,
                    false,
                    pricing_service,
                )?
                .min(0)
                .neg(),
        )?;

        Ok((
            receipt_token_amount_processed,
            asset_amount_required,
            asset_user_amount_reserved,
            asset_fee_amount_deducted,
            offsetted_asset_receivables,
        ))
    }

    /// at first, it adds given [asset_amount] to the operation reserved,
    /// then offset receivables as much as possible, then transfer remaining assets to the program revenue account
    /// returns [transferred_asset_revenue_amount, offsetted_asset_amount, offsetted_asset_receivables]
    pub(super) fn offset_receivables(
        &mut self,
        system_program: &Program<'info, System>,

        // for SOL
        fund_reserve_account: &'info AccountInfo<'info>,
        fund_treasury_account: &'info AccountInfo<'info>,

        // for supported token
        supported_token_mint: Option<&'info AccountInfo<'info>>,
        supported_token_program: Option<&'info AccountInfo<'info>>,
        fund_supported_token_reserve_account: Option<&'info AccountInfo<'info>>,
        fund_supported_token_treasury_account: Option<&'info AccountInfo<'info>>,

        asset_amount: u64,
        pricing_service: &PricingService,
    ) -> Result<(u64, u64, Vec<(Option<Pubkey>, u64)>)> {
        let (
            supported_token_mint,
            supported_token_mint_key,
            supported_token_program,
            fund_supported_token_reserve_account,
            fund_supported_token_treasury_account,
        ) = match &supported_token_mint {
            Some(supported_token_mint) => (
                Some(InterfaceAccount::<Mint>::try_from(supported_token_mint)?),
                Some(supported_token_mint.key()),
                Some(Interface::<TokenInterface>::try_from(
                    supported_token_program.unwrap(),
                )?),
                Some(InterfaceAccount::<TokenAccount>::try_from(
                    fund_supported_token_reserve_account.unwrap(),
                )?),
                Some(InterfaceAccount::<TokenAccount>::try_from(
                    fund_supported_token_treasury_account.unwrap(),
                )?),
            ),
            _ => (None, None, None, None, None),
        };

        let total_operation_receivable_amount_as_asset = self
            .fund_account
            .load()?
            .get_total_operation_receivable_amount_as_asset(
                supported_token_mint_key,
                pricing_service,
            )?;

        let mut fund_account = self.fund_account.load_mut()?;
        let asset = fund_account.get_asset_state_mut(supported_token_mint_key)?;
        asset.operation_reserved_amount += asset_amount;
        drop(fund_account);

        let (transferred_asset_revenue_amount, offsetted_asset_amount, offsetted_asset_receivables) =
            self.pay_treasury_debt_with_receivables(
                system_program,
                // for SOL
                fund_reserve_account,
                fund_treasury_account,
                // for supported token
                &supported_token_mint,
                &supported_token_program,
                &fund_supported_token_reserve_account,
                &fund_supported_token_treasury_account,
                asset_amount,
                total_operation_receivable_amount_as_asset,
                pricing_service,
            )?;

        Ok((
            transferred_asset_revenue_amount,
            offsetted_asset_amount,
            offsetted_asset_receivables,
        ))
    }

    /// returns [transferred_asset_revenue_amount, offsetted_asset_amount, offsetted_asset_receivables]
    fn pay_treasury_debt_with_receivables(
        &mut self,
        system_program: &Program<'info, System>,

        // for SOL
        fund_reserve_account: &'info AccountInfo<'info>,
        fund_treasury_account: &'info AccountInfo<'info>,

        // for supported token
        supported_token_mint: &Option<InterfaceAccount<'info, Mint>>,
        supported_token_program: &Option<Interface<'info, TokenInterface>>,
        fund_supported_token_reserve_account: &Option<InterfaceAccount<'info, TokenAccount>>,
        fund_supported_token_treasury_account: &Option<InterfaceAccount<'info, TokenAccount>>,

        asset_debt_amount: u64,
        receivable_amount_to_redeem_as_asset: u64,
        pricing_service: &PricingService,
    ) -> Result<(u64, u64, Vec<(Option<Pubkey>, u64)>)> {
        let mut asset_debt_amount_processing = asset_debt_amount;
        let mut asset_receivable_amount_processing =
            receivable_amount_to_redeem_as_asset.min(asset_debt_amount);

        // pay the treasury debt with receivables first.
        let mut fund_account = self.fund_account.load_mut()?;
        let supported_token_mint_key = supported_token_mint.as_ref().map(|mint| mint.key());
        let asset = fund_account.get_asset_state_mut(supported_token_mint_key)?;

        // pay with receivable of current asset first.
        let mut offsetted_asset_receivables = Vec::<(Option<Pubkey>, u64)>::new();
        let receivable_amount_processing_for_current_asset = asset
            .operation_receivable_amount
            .min(asset_receivable_amount_processing);
        asset.operation_receivable_amount -= receivable_amount_processing_for_current_asset;
        asset_debt_amount_processing -= receivable_amount_processing_for_current_asset;
        asset_receivable_amount_processing -= receivable_amount_processing_for_current_asset;
        if receivable_amount_processing_for_current_asset > 0 {
            offsetted_asset_receivables.push((
                asset.get_token_mint_and_program().unzip().0,
                receivable_amount_processing_for_current_asset,
            ));
        }

        let mut receivable_amount_processing_as_sol = if asset_receivable_amount_processing > 0 {
            match asset.get_token_mint_and_program() {
                Some((token_mint, _)) => pricing_service
                    .get_token_amount_as_sol(&token_mint, asset_receivable_amount_processing)?,
                None => asset_receivable_amount_processing,
            }
        } else {
            0
        };
        drop(fund_account);

        // pay with receivable of other assets if possible.
        if receivable_amount_processing_as_sol > 0 {
            let mut fund_account = self.fund_account.load_mut()?;
            let asset_token_mint_and_program = fund_account
                .get_asset_state(supported_token_mint_key)?
                .get_token_mint_and_program();

            for other_asset in fund_account.get_asset_states_iter_mut() {
                if other_asset.operation_receivable_amount > 0 {
                    let other_asset_operation_receivable_amount_as_sol =
                        match other_asset.get_token_mint_and_program() {
                            Some((token_mint, _)) => pricing_service.get_token_amount_as_sol(
                                &token_mint,
                                other_asset.operation_receivable_amount,
                            )?,
                            None => other_asset.operation_receivable_amount,
                        };
                    let receivable_amount_processing_as_sol_for_other_asset =
                        other_asset_operation_receivable_amount_as_sol
                            .min(receivable_amount_processing_as_sol);
                    let receivable_amount_processing_as_other_asset_for_other_asset =
                        match other_asset.get_token_mint_and_program() {
                            Some((token_mint, _)) => pricing_service.get_sol_amount_as_token(
                                &token_mint,
                                receivable_amount_processing_as_sol_for_other_asset,
                            )?,
                            None => receivable_amount_processing_as_sol_for_other_asset,
                        };
                    other_asset.operation_receivable_amount -=
                        receivable_amount_processing_as_other_asset_for_other_asset;
                    receivable_amount_processing_as_sol -=
                        receivable_amount_processing_as_sol_for_other_asset;
                    offsetted_asset_receivables.push((
                        other_asset.get_token_mint_and_program().unzip().0,
                        receivable_amount_processing_as_other_asset_for_other_asset,
                    ));

                    if receivable_amount_processing_as_sol == 0 {
                        break;
                    }
                }
            }
            let receivable_amount_processed = asset_receivable_amount_processing
                - match asset_token_mint_and_program {
                    Some((token_mint, _)) => pricing_service.get_sol_amount_as_token(
                        &token_mint,
                        receivable_amount_processing_as_sol,
                    )?,
                    None => receivable_amount_processing_as_sol,
                };
            asset_debt_amount_processing -= receivable_amount_processed;
            // asset_receivable_amount_processing -= receivable_amount_processed;
        }

        // pay remaining debt with cash with current asset
        let mut fund_account = self.fund_account.load_mut()?;
        let asset = fund_account.get_asset_state_mut(supported_token_mint_key)?;
        asset.operation_reserved_amount -= asset_debt_amount_processing;
        drop(fund_account);

        let mut transferred_asset_amount = 0;
        if asset_debt_amount_processing > 0 {
            transferred_asset_amount = asset_debt_amount_processing;
            let fund_account = self.fund_account.load()?;

            match supported_token_mint {
                Some(supported_token_mint) => {
                    anchor_spl::token_interface::transfer_checked(
                        CpiContext::new_with_signer(
                            supported_token_program.as_ref().unwrap().to_account_info(),
                            anchor_spl::token_interface::TransferChecked {
                                from: fund_supported_token_reserve_account
                                    .as_ref()
                                    .unwrap()
                                    .to_account_info(),
                                to: fund_supported_token_treasury_account
                                    .as_ref()
                                    .unwrap()
                                    .to_account_info(),
                                mint: supported_token_mint.to_account_info(),
                                authority: fund_reserve_account.to_account_info(),
                            },
                            &[&fund_account.get_reserve_account_seeds()],
                        ),
                        asset_debt_amount_processing,
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
                        asset_debt_amount_processing,
                    )?;
                }
            }
            asset_debt_amount_processing = 0;
        }

        require_eq!(asset_debt_amount_processing, 0);

        Ok((
            transferred_asset_amount,
            asset_debt_amount - transferred_asset_amount,
            offsetted_asset_receivables,
        ))
    }

    /// returns (transferred_asset_revenue_amount)
    #[inline(never)]
    pub(super) fn harvest_from_treasury_account(
        &mut self,
        payer: &Signer<'info>,
        system_program: &Program<'info, System>,

        // for SOL and supported token
        fund_treasury_account: &AccountInfo<'info>,
        program_revenue_account: &'info AccountInfo<'info>,

        // for supported token
        associated_token_account_program: Option<&'info AccountInfo<'info>>,
        supported_token_mint: Option<&'info AccountInfo<'info>>,
        supported_token_program: Option<&'info AccountInfo<'info>>,
        fund_supported_token_treasury_account: Option<&'info AccountInfo<'info>>,
        program_supported_token_revenue_account: Option<&'info AccountInfo<'info>>,
    ) -> Result<u64> {
        let fund_account = self.fund_account.load()?;

        match supported_token_mint {
            Some(supported_token_mint) => {
                let supported_token_mint =
                    InterfaceAccount::<Mint>::try_from(supported_token_mint)?;
                let fund_supported_token_treasury_account =
                    InterfaceAccount::<TokenAccount>::try_from(
                        fund_supported_token_treasury_account.unwrap(),
                    )?;

                if fund_supported_token_treasury_account.amount == 0 {
                    Ok(0)
                } else {
                    // create program_supported_token_revenue_account if not exists
                    let program_supported_token_revenue_account =
                        program_supported_token_revenue_account.unwrap();
                    let supported_token_program = supported_token_program.unwrap();
                    if !program_supported_token_revenue_account.is_initialized() {
                        anchor_lang::solana_program::program::invoke(
                            &spl_associated_token_account::instruction::create_associated_token_account(
                                &payer.key(),
                                &program_revenue_account.key(),
                                &supported_token_mint.key(),
                                &supported_token_program.key(),
                            ),
                            &[
                                payer.to_account_info(),
                                program_supported_token_revenue_account.to_account_info(),
                                program_revenue_account.to_account_info(),
                                supported_token_mint.to_account_info(),
                                system_program.to_account_info(),
                                supported_token_program.to_account_info(),
                                associated_token_account_program.unwrap().to_account_info(),
                            ],
                        )?;
                    }

                    anchor_spl::token_interface::transfer_checked(
                        CpiContext::new_with_signer(
                            supported_token_program.to_account_info(),
                            anchor_spl::token_interface::TransferChecked {
                                from: fund_supported_token_treasury_account.to_account_info(),
                                to: program_supported_token_revenue_account.to_account_info(),
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
                let rent = Rent::get()?;
                let min_lamports_for_system_account = rent.minimum_balance(0);
                let treasury_account_lamports = fund_treasury_account
                    .lamports()
                    .saturating_sub(min_lamports_for_system_account);
                if treasury_account_lamports < min_lamports_for_system_account {
                    Ok(0)
                } else {
                    anchor_lang::system_program::transfer(
                        CpiContext::new_with_signer(
                            system_program.to_account_info(),
                            anchor_lang::system_program::Transfer {
                                from: fund_treasury_account.to_account_info(),
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

    pub(super) fn update_wrapped_token_holder(
        &self,
        // fixed
        reward_account: &AccountLoader<'info, reward::RewardAccount>,
        fund_wrap_account: &SystemAccount,
        fund_wrap_account_reward_account: &AccountLoader<reward::UserRewardAccount>,

        // variant
        wrapped_token_holder: &InterfaceAccount<TokenAccount>,
        wrapped_token_holder_reward_account: &AccountLoader<reward::UserRewardAccount>,
    ) -> Result<()> {
        let mut fund_account = self.fund_account.load_mut()?;
        require_keys_eq!(
            fund_wrap_account.key(),
            fund_account.get_wrap_account_address()?,
        );

        reward::UserRewardService::validate_user_reward_account(
            self.receipt_token_mint,
            fund_wrap_account,
            reward_account,
            fund_wrap_account_reward_account,
        )?;

        reward::UserRewardService::validate_user_reward_account(
            self.receipt_token_mint,
            wrapped_token_holder.as_ref(),
            reward_account,
            wrapped_token_holder_reward_account,
        )?;

        let wrapped_token = fund_account
            .get_wrapped_token_mut()
            .ok_or_else(|| error!(ErrorCode::FundWrappedTokenNotSetError))?;

        let (old_wrapped_token_holder_amount, old_wrapped_token_retained_amount) = wrapped_token
            .update_holder(&wrapped_token_holder.key(), wrapped_token_holder.amount)?;

        // update reward
        let reward_service = reward::RewardService::new(self.receipt_token_mint, reward_account)?;

        // holder gained or lost ∆wrapped_token_holder_amount
        let wrapped_token_holder_amount_delta = wrapped_token_holder
            .amount
            .abs_diff(old_wrapped_token_holder_amount);
        let wrapped_token_holder_amount_increased =
            wrapped_token_holder.amount > old_wrapped_token_holder_amount;
        if wrapped_token_holder_amount_increased {
            reward_service.update_reward_pools_token_allocation(
                None,
                Some(wrapped_token_holder_reward_account),
                wrapped_token_holder_amount_delta,
                None,
            )?;
        } else {
            reward_service.update_reward_pools_token_allocation(
                Some(wrapped_token_holder_reward_account),
                None,
                wrapped_token_holder_amount_delta,
                None,
            )?;
        }

        // fund_wrap_account gained or lost ∆wrapped_token_retained_amount
        let wrapped_token_retained_amount_delta = wrapped_token
            .retained_amount
            .abs_diff(old_wrapped_token_retained_amount);
        let wrapped_token_retained_amount_increased =
            wrapped_token.retained_amount > old_wrapped_token_retained_amount;
        if wrapped_token_retained_amount_increased {
            reward_service.update_reward_pools_token_allocation(
                None,
                Some(fund_wrap_account_reward_account),
                wrapped_token_retained_amount_delta,
                None,
            )?;
        } else {
            reward_service.update_reward_pools_token_allocation(
                Some(fund_wrap_account_reward_account),
                None,
                wrapped_token_retained_amount_delta,
                None,
            )?;
        }

        Ok(())
    }

    pub fn process_donate_sol(
        &mut self,
        operator: &Signer<'info>,

        system_program: &Program<'info, System>,
        fund_reserve_account: &SystemAccount<'info>,

        pricing_sources: &'info [AccountInfo<'info>],

        asset_amount: u64,
        offset_receivable: bool,
    ) -> Result<events::OperatorDonatedToFund> {
        self.process_donate(
            operator,
            Some(system_program),
            Some(fund_reserve_account),
            None,
            None,
            None,
            None,
            pricing_sources,
            asset_amount,
            offset_receivable,
        )
    }

    pub fn process_donate_supported_token(
        &mut self,
        operator: &Signer<'info>,

        supported_token_program: &Interface<'info, TokenInterface>,
        supported_token_mint: &InterfaceAccount<'info, Mint>,
        fund_supported_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,
        operator_supported_token_account: &InterfaceAccount<'info, TokenAccount>,

        pricing_sources: &'info [AccountInfo<'info>],

        asset_amount: u64,
        offset_receivable: bool,
    ) -> Result<events::OperatorDonatedToFund> {
        self.process_donate(
            operator,
            None,
            None,
            Some(supported_token_program),
            Some(supported_token_mint),
            Some(fund_supported_token_reserve_account),
            Some(operator_supported_token_account),
            pricing_sources,
            asset_amount,
            offset_receivable,
        )
    }

    /// for testing and operation purposes
    fn process_donate(
        &mut self,
        operator: &Signer<'info>,

        // for SOL
        system_program: Option<&Program<'info, System>>,
        fund_reserve_account: Option<&SystemAccount<'info>>,

        // for supported tokens
        supported_token_program: Option<&Interface<'info, TokenInterface>>,
        supported_token_mint: Option<&InterfaceAccount<'info, Mint>>,
        fund_supported_token_reserve_account: Option<&InterfaceAccount<'info, TokenAccount>>,
        operator_supported_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,

        pricing_sources: &'info [AccountInfo<'info>],

        asset_amount: u64,
        offset_receivable: bool,
    ) -> Result<events::OperatorDonatedToFund> {
        let supported_token_mint_key = supported_token_mint.map(|mint| mint.key());

        // validate operator asset balance
        match supported_token_mint_key {
            Some(..) => {
                require_gte!(
                    operator_supported_token_account.unwrap().amount,
                    asset_amount
                );
            }
            None => {
                require_gte!(operator.lamports(), asset_amount);
            }
        }

        // transfer operator asset to the fund
        let (deposited_amount, offsetted_receivable_amount) = self
            .fund_account
            .load_mut()?
            .donate(supported_token_mint_key, asset_amount, offset_receivable)?;
        let donated_amount = deposited_amount + offsetted_receivable_amount;
        assert_eq!(asset_amount, donated_amount);

        match supported_token_mint {
            Some(supported_token_mint) => {
                anchor_spl::token_interface::transfer_checked(
                    CpiContext::new(
                        supported_token_program.unwrap().to_account_info(),
                        anchor_spl::token_interface::TransferChecked {
                            from: operator_supported_token_account.unwrap().to_account_info(),
                            to: fund_supported_token_reserve_account
                                .unwrap()
                                .to_account_info(),
                            mint: supported_token_mint.to_account_info(),
                            authority: operator.to_account_info(),
                        },
                    ),
                    donated_amount,
                    supported_token_mint.decimals,
                )?;
            }
            None => {
                anchor_lang::system_program::transfer(
                    CpiContext::new(
                        system_program.unwrap().to_account_info(),
                        anchor_lang::system_program::Transfer {
                            from: operator.to_account_info(),
                            to: fund_reserve_account.unwrap().to_account_info(),
                        },
                    ),
                    donated_amount,
                )?;
            }
        }

        // update asset value
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .new_pricing_service(pricing_sources, true)?;

        Ok(events::OperatorDonatedToFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: self.fund_account.key(),
            supported_token_mint: supported_token_mint_key,
            donated_amount,
            deposited_amount,
            offsetted_receivable_amount,
        })
    }
}
