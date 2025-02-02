use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking::*;
use crate::utils::{AccountInfoExt, AsAccountInfo, PDASeeds};

use super::{
    FundAccount, FundService, OperationCommandContext, OperationCommandEntry,
    OperationCommandResult, SelfExecutable, UnstakeLSTCommand, FUND_ACCOUNT_MAX_SUPPORTED_TOKENS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct ClaimUnstakedSOLCommand {
    state: ClaimUnstakedSOLCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub enum ClaimUnstakedSOLCommandState {
    /// Initializes a command with items based on the fund state and strategy.
    #[default]
    New,
    /// Prepares to execute claim for the first token mint in the list.
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        pool_token_mints: Vec<Pubkey>,
    },
    /// Executes claim for the first item and
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        pool_token_mints: Vec<Pubkey>,
    },
}

impl std::fmt::Debug for ClaimUnstakedSOLCommandState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => f.write_str("New"),
            Self::Prepare { pool_token_mints } => {
                if pool_token_mints.is_empty() {
                    f.write_str("Prepare")
                } else {
                    f.debug_struct("Prepare")
                        .field("pool_token_mint", &pool_token_mints[0])
                        .finish()
                }
            }
            Self::Execute { pool_token_mints } => {
                if pool_token_mints.is_empty() {
                    f.write_str("Execute")
                } else {
                    f.debug_struct("Execute")
                        .field("pool_token_mint", &pool_token_mints[0])
                        .finish()
                }
            }
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct ClaimUnstakedSOLCommandResult {
    pub token_mint: Pubkey,
    pub claimed_sol_amount: u64,
    pub total_unstaking_sol_amount: u64,
    pub transferred_sol_revenue_amount: u64,
    pub offsetted_sol_receivable_amount: u64,
    #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
    pub offsetted_asset_receivables: Vec<ClaimUnstakedSOLCommandResultAssetReceivable>,
    pub operation_reserved_sol_amount: u64,
    pub operation_receivable_sol_amount: u64,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct ClaimUnstakedSOLCommandResultAssetReceivable {
    asset_token_mint: Option<Pubkey>,
    asset_amount: u64,
}

impl SelfExecutable for ClaimUnstakedSOLCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            ClaimUnstakedSOLCommandState::New => self.execute_new(ctx, accounts)?,
            ClaimUnstakedSOLCommandState::Prepare {
                pool_token_mints: token_mints,
            } => self.execute_prepare(ctx, accounts, token_mints.clone(), None)?,
            ClaimUnstakedSOLCommandState::Execute {
                pool_token_mints: token_mints,
            } => self.execute_execute(ctx, accounts, token_mints)?,
        };

        Ok((
            result,
            entry.or_else(|| Some(UnstakeLSTCommand::default().without_required_accounts())),
        ))
    }
}

// These are implementations of each command state.
#[deny(clippy::wildcard_enum_match_arm)]
impl ClaimUnstakedSOLCommand {
    /// An initial state of `ClaimUnstakedSOL` command.
    /// In this state, operator iterates the fund and
    /// finds token to claim.
    #[inline(never)]
    fn execute_new<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let items = ctx
            .fund_account
            .load()?
            .get_supported_tokens_iter()
            .filter_map(|supported_token| {
                if supported_token.pending_unstaking_amount_as_sol > 0 {
                    Some(supported_token.mint)
                } else {
                    None
                }
            })
            .collect();

        // prepare state does not require additional accounts,
        // so we can execute directly.
        self.execute_prepare(ctx, accounts, items, None)
    }

    /// A pre-execution state of `ClaimUnstakedSOL` command.
    /// In this state, operator iterates unstaking ticket or stake accounts and
    /// find claimable SOL.
    #[inline(never)]
    fn execute_prepare<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        pool_token_mints: Vec<Pubkey>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if pool_token_mints.is_empty() {
            return Ok((previous_execution_result, None));
        }
        let pool_token_mint = &pool_token_mints[0];

        let fund_account = ctx.fund_account.load()?;
        let pricing_source = fund_account
            .get_supported_token(pool_token_mint)?
            .pricing_source
            .try_deserialize()?;
        let pool_account = match pricing_source {
            Some(TokenPricingSource::SPLStakePool { address })
            | Some(TokenPricingSource::MarinadeStakePool { address })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => *accounts
                .iter()
                .find(|account| account.key() == address)
                .ok_or_else(|| error!(ErrorCode::FundOperationCommandExecutionFailedException))?,
            // otherwise fails
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        let fund_reserve_account = fund_account.get_reserve_account_address()?;
        let fund_treasury_account = fund_account.get_treasury_account_address()?;

        let command = Self {
            state: ClaimUnstakedSOLCommandState::Execute { pool_token_mints },
        };

        drop(fund_account);
        let entry = match pricing_source {
            Some(TokenPricingSource::SPLStakePool { .. }) => self
                .spl_stake_pool_prepare_claim_sol::<SPLStakePool>(
                    ctx,
                    pool_account,
                    fund_reserve_account,
                    fund_treasury_account,
                    command,
                )?,
            Some(TokenPricingSource::MarinadeStakePool { .. }) => {
                let accounts_to_claim_sol =
                    MarinadeStakePoolService::find_accounts_to_claim_sol(pool_account)?;
                let withdrawal_ticket_accounts = (0..5).map(|index| {
                    let address = *FundAccount::find_unstaking_ticket_account_address(
                        &ctx.fund_account.key(),
                        pool_account.key,
                        index,
                    );
                    (address, true)
                });

                let required_accounts =
                    [(fund_reserve_account, true), (fund_treasury_account, true)]
                        .into_iter()
                        .chain(accounts_to_claim_sol)
                        .chain(withdrawal_ticket_accounts);

                command.with_required_accounts(required_accounts)
            }
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. }) => {
                self.spl_stake_pool_prepare_claim_sol::<SanctumSingleValidatorSPLStakePool>(
                    ctx,
                    pool_account,
                    fund_reserve_account,
                    fund_treasury_account,
                    command,
                )?
            }
            // otherwise fails
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        Ok((previous_execution_result, Some(entry)))
    }

    fn spl_stake_pool_prepare_claim_sol<'info, T: SPLStakePoolInterface>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        pool_account: &'info AccountInfo<'info>,
        fund_reserve_account: Pubkey,
        fund_treasury_account: Pubkey,
        next_command: Self,
    ) -> Result<OperationCommandEntry> {
        let accounts_to_claim_sol = SPLStakePoolService::<T>::find_accounts_to_claim_sol()?;
        let fund_stake_accounts = (0..5).map(|index| {
            let address = *FundAccount::find_stake_account_address(
                &ctx.fund_account.key(),
                pool_account.key,
                index,
            );
            (address, true)
        });

        let required_accounts = [(fund_reserve_account, true), (fund_treasury_account, true)]
            .into_iter()
            .chain(accounts_to_claim_sol)
            .chain(fund_stake_accounts);

        Ok(next_command.with_required_accounts(required_accounts))
    }

    #[inline(never)]
    fn execute_execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        pool_token_mints: &[Pubkey],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if pool_token_mints.is_empty() {
            return Ok((None, None));
        }
        let pool_token_mint = &pool_token_mints[0];

        let token_pricing_source = ctx
            .fund_account
            .load()?
            .get_supported_token(pool_token_mint)?
            .pricing_source
            .try_deserialize()?;

        let (to_sol_account_amount, claimed_sol_amount, should_resume) = match token_pricing_source
        {
            Some(TokenPricingSource::SPLStakePool { address }) => self
                .spl_stake_pool_claim_sol::<SPLStakePool>(
                    ctx,
                    accounts,
                    pool_token_mint,
                    address,
                )?,
            Some(TokenPricingSource::MarinadeStakePool { address }) => {
                self.marinade_stake_pool_claim_sol(ctx, accounts, pool_token_mint, address)?
            }
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => {
                self.spl_stake_pool_claim_sol::<SanctumSingleValidatorSPLStakePool>(
                    ctx,
                    accounts,
                    pool_token_mint,
                    address,
                )?
            }
            // otherwise fails
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        let result = if claimed_sol_amount == 0 {
            // claim did not happen
            None
        } else {
            // now update fund assets, the value of the fund and receipt token should remain as is.
            let [fund_reserve_account, fund_treasury_account, remaining_accounts @ ..] = accounts
            else {
                err!(error::ErrorCode::AccountNotEnoughKeys)?
            };

            // while paying treasury debt, offsets available receivables and sends remaining to the treasury account.
            let mut fund_service = FundService::new(ctx.receipt_token_mint, ctx.fund_account)?;
            let pricing_service =
                fund_service.new_pricing_service(remaining_accounts.into_iter().copied())?;

            let (
                transferred_sol_revenue_amount,
                offsetted_sol_receivable_amount,
                offsetted_asset_receivables,
            ) = fund_service.offset_receivables(
                ctx.system_program,
                fund_reserve_account,
                fund_treasury_account,
                None,
                None,
                None,
                None,
                claimed_sol_amount,
                &pricing_service,
            )?;
            drop(fund_service);

            let mut fund_account = ctx.fund_account.load_mut()?;
            require_gte!(
                to_sol_account_amount,
                fund_account.sol.get_total_reserved_amount(),
            );
            let supported_token = fund_account.get_supported_token_mut(pool_token_mint)?;
            supported_token.pending_unstaking_amount_as_sol -= claimed_sol_amount;

            Some(
                ClaimUnstakedSOLCommandResult {
                    token_mint: *pool_token_mint,
                    claimed_sol_amount,
                    total_unstaking_sol_amount: supported_token.pending_unstaking_amount_as_sol,
                    transferred_sol_revenue_amount,
                    offsetted_sol_receivable_amount,
                    offsetted_asset_receivables: offsetted_asset_receivables
                        .into_iter()
                        .map(|(asset_token_mint, asset_amount)| {
                            ClaimUnstakedSOLCommandResultAssetReceivable {
                                asset_token_mint,
                                asset_amount,
                            }
                        })
                        .collect::<Vec<_>>(),
                    operation_reserved_sol_amount: fund_account.sol.operation_reserved_amount,
                    operation_receivable_sol_amount: fund_account.sol.operation_receivable_amount,
                }
                .into(),
            )
        };

        // prepare state does not require additional accounts,
        // so we can execute directly.
        self.execute_prepare(
            ctx,
            accounts,
            if should_resume {
                pool_token_mints.to_vec()
            } else {
                pool_token_mints[1..].to_vec()
            },
            result,
        )
    }

    /// return [to_sol_account_amount, claimed_sol_amount, should_resume]
    fn spl_stake_pool_claim_sol<'info, T: SPLStakePoolInterface>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        pool_token_mint_address: &Pubkey, // just informative
        pool_account_address: Pubkey,
    ) -> Result<(u64, u64, bool)> {
        let [fund_reserve_account, fund_treasury_account, clock, stake_history, stake_program, remaining_accounts @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };

        if remaining_accounts.len() < 5 {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        }

        let (fund_stake_accounts, _pricing_sources) = remaining_accounts.split_at(5);

        let mut total_claimed_sol_amount = 0;

        let fund_account = ctx.fund_account.load()?;

        let mut processed = false;
        let mut should_resume = false;
        for (index, fund_stake_account) in fund_stake_accounts.iter().enumerate() {
            let fund_stake_account_address = *FundAccount::find_stake_account_address(
                &ctx.fund_account.key(),
                &pool_account_address,
                index as u8,
            );

            require_keys_eq!(fund_stake_account_address, fund_stake_account.key());

            // Skip uninitialized stake account
            if !fund_stake_account.is_initialized() {
                continue;
            }
            if processed {
                should_resume = true;
                break;
            }

            let claimed_sol_amount = SPLStakePoolService::<T>::claim_sol(
                pool_token_mint_address,
                ctx.system_program,
                clock,
                stake_history,
                stake_program,
                fund_reserve_account,
                &[&fund_account.get_reserve_account_seeds()],
                fund_stake_account,
                fund_treasury_account,
                ctx.fund_account.as_account_info(),
                &[&fund_account.get_seeds()],
            )?;

            total_claimed_sol_amount += claimed_sol_amount;
            processed = true;
        }

        let to_sol_account_amount = fund_reserve_account.lamports();

        Ok((
            to_sol_account_amount,
            total_claimed_sol_amount,
            should_resume,
        ))
    }

    /// return [to_sol_account_amount, claimed_sol_amount, should_resume]
    fn marinade_stake_pool_claim_sol<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        pool_token_mint_address: &Pubkey,
        pool_account_address: Pubkey,
    ) -> Result<(u64, u64, bool)> {
        let [fund_reserve_account, _fund_treasury_account, pool_program, pool_account, pool_token_mint, pool_token_program, pool_reserve_account, clock, remaining_accounts @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };

        if remaining_accounts.len() < 5 {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        }

        let (withdrawal_ticket_accounts, _pricing_sources) = remaining_accounts.split_at(5);

        require_keys_eq!(pool_account_address, pool_account.key());
        require_keys_eq!(*pool_token_mint_address, pool_token_mint.key());

        let marinade_stake_pool_service = MarinadeStakePoolService::new(
            pool_program,
            pool_account,
            pool_token_mint,
            pool_token_program,
        )?;

        let mut total_claimed_sol_amount = 0;

        let fund_account = ctx.fund_account.load()?;

        let mut processed = false;
        let mut should_resume = false;
        for (index, withdrawal_ticket_account) in withdrawal_ticket_accounts.iter().enumerate() {
            let withdrawal_ticket_account_address =
                *FundAccount::find_unstaking_ticket_account_address(
                    &ctx.fund_account.key(),
                    pool_account.key,
                    index as u8,
                );

            require_keys_eq!(
                withdrawal_ticket_account_address,
                withdrawal_ticket_account.key()
            );

            // Skip uninitialized stake account
            if !withdrawal_ticket_account.is_initialized() {
                continue;
            }
            if processed {
                should_resume = true;
                break;
            }

            let claimed_sol_amount = marinade_stake_pool_service.claim_sol(
                ctx.system_program,
                pool_reserve_account,
                clock,
                withdrawal_ticket_account,
                fund_reserve_account,
                &[&fund_account.get_reserve_account_seeds()],
            )?;

            total_claimed_sol_amount += claimed_sol_amount;
            processed = true;
        }

        let to_sol_account_amount = fund_reserve_account.lamports();

        Ok((
            to_sol_account_amount,
            total_claimed_sol_amount,
            should_resume,
        ))
    }
}
