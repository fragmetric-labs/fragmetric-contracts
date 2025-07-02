use anchor_lang::prelude::*;

use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::staking::*;
use crate::utils::{AccountInfoExt, AsAccountInfo, PDASeeds};

use super::*;

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
    /// Before execute claim, find claimable stake accounts.
    GetClaimableStakeAccounts {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        pool_token_mints: Vec<Pubkey>,
    },
    /// Executes claim for the first item and
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_SUPPORTED_TOKENS)]
        pool_token_mints: Vec<Pubkey>,
        #[max_len(5)]
        claimable_stake_account_indices: Vec<u8>,
    },
}
use ClaimUnstakedSOLCommandState::*;

impl std::fmt::Debug for ClaimUnstakedSOLCommandState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            New => f.write_str("New"),
            Prepare { pool_token_mints } => {
                if pool_token_mints.is_empty() {
                    f.write_str("Prepare")
                } else {
                    f.debug_struct("Prepare")
                        .field("pool_token_mint", &pool_token_mints[0])
                        .finish()
                }
            }
            GetClaimableStakeAccounts { pool_token_mints } => {
                if pool_token_mints.is_empty() {
                    f.write_str("GetClaimableStakeAccounts")
                } else {
                    f.debug_struct("GetClaimableStakeAccounts")
                        .field("pool_token_mint", &pool_token_mints[0])
                        .finish()
                }
            }
            Execute {
                pool_token_mints, ..
            } => {
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
    fn execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            New => self.execute_new(ctx, accounts)?,
            Prepare { pool_token_mints } => {
                self.execute_prepare(ctx, accounts, pool_token_mints.clone(), None)?
            }
            GetClaimableStakeAccounts { pool_token_mints } => {
                self.execute_get_claimable_stake_accounts(ctx, accounts, pool_token_mints)?
            }
            Execute {
                pool_token_mints,
                claimable_stake_account_indices,
            } => self.execute_execute(
                ctx,
                accounts,
                pool_token_mints,
                claimable_stake_account_indices,
            )?,
        };

        Ok((
            result,
            entry.or_else(|| {
                Some(ProcessWithdrawalBatchCommand::default().without_required_accounts())
            }),
        ))
    }
}

// These are implementations of each command state.
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

        let pricing_source = ctx
            .fund_account
            .load()?
            .get_supported_token(pool_token_mint)?
            .pricing_source
            .try_deserialize()?;
        let pool_account = match pricing_source {
            Some(TokenPricingSource::SPLStakePool { address })
            | Some(TokenPricingSource::MarinadeStakePool { address })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address })
            | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { address }) => *accounts
                .iter()
                .find(|account| account.key() == address)
                .ok_or_else(|| error!(ErrorCode::FundOperationCommandExecutionFailedException))?,
            // otherwise fails
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. })
            | Some(TokenPricingSource::VirtualVault { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        let entry = match pricing_source {
            Some(TokenPricingSource::SPLStakePool { .. }) => self
                .spl_stake_pool_prepare_get_claimable_stake_accounts::<SPLStakePool>(
                    ctx,
                    pool_account,
                    pool_token_mints,
                )?,
            Some(TokenPricingSource::MarinadeStakePool { .. }) => {
                let fund_account = ctx.fund_account.load()?;
                let fund_reserve_account = fund_account.get_reserve_account_address()?;
                let fund_treasury_account = fund_account.get_treasury_account_address()?;
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

                Self {
                    state: Execute {
                        pool_token_mints,
                        claimable_stake_account_indices: vec![],
                    },
                }
                .with_required_accounts(required_accounts)
            }
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. }) => {
                self.spl_stake_pool_prepare_get_claimable_stake_accounts::<SanctumSingleValidatorSPLStakePool>(
                    ctx,
                    pool_account,
                    pool_token_mints,
                )?
            }
            Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. }) => {
                self.spl_stake_pool_prepare_get_claimable_stake_accounts::<SanctumMultiValidatorSPLStakePool>(
                    ctx,
                    pool_account,
                    pool_token_mints,
                )?
            }
            // otherwise fails
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. })
            | Some(TokenPricingSource::VirtualVault { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        Ok((previous_execution_result, Some(entry)))
    }

    fn spl_stake_pool_prepare_get_claimable_stake_accounts<'info, T: SPLStakePoolInterface>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        pool_account: &'info AccountInfo<'info>,
        pool_token_mints: Vec<Pubkey>,
    ) -> Result<OperationCommandEntry> {
        let accounts_to_get_claimable_stake_accounts =
            SPLStakePoolService::<T>::find_accounts_to_get_claimable_stake_accounts()?;
        let fund_stake_accounts = (0..5).map(|index| {
            let address = *FundAccount::find_stake_account_address(
                &ctx.fund_account.key(),
                pool_account.key,
                index,
            );
            (address, false)
        });

        let required_accounts = accounts_to_get_claimable_stake_accounts.chain(fund_stake_accounts);

        Ok(Self {
            state: GetClaimableStakeAccounts { pool_token_mints },
        }
        .with_required_accounts(required_accounts))
    }

    #[inline(never)]
    fn execute_get_claimable_stake_accounts<'info>(
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

        let entry = match token_pricing_source {
            Some(TokenPricingSource::SPLStakePool { address }) => self
                .spl_stake_pool_get_claimable_stake_accounts::<SPLStakePool>(
                    ctx, accounts, pool_token_mints, address,
                ),
            Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { address }) => {
                self.spl_stake_pool_get_claimable_stake_accounts::<SanctumSingleValidatorSPLStakePool>(
                    ctx, accounts, pool_token_mints, address,
                )
            }
            Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { address }) => {
                self.spl_stake_pool_get_claimable_stake_accounts::<SanctumMultiValidatorSPLStakePool>(
                    ctx, accounts, pool_token_mints, address,
                )
            }
            // otherwise fails
            Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. })
            | Some(TokenPricingSource::VirtualVault { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        }?;

        Ok((None, Some(entry)))
    }

    fn spl_stake_pool_get_claimable_stake_accounts<'info, T: SPLStakePoolInterface>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        pool_token_mints: &[Pubkey],
        pool_account_address: Pubkey,
    ) -> Result<OperationCommandEntry> {
        let [clock, stake_history, remaining_accounts @ ..] = accounts else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        let fund_stake_accounts = {
            if remaining_accounts.len() < 5 {
                err!(error::ErrorCode::AccountNotEnoughKeys)?
            }
            &remaining_accounts[..5]
        };

        for (index, fund_stake_account) in fund_stake_accounts.iter().enumerate() {
            let fund_stake_account_address = *FundAccount::find_stake_account_address(
                &ctx.fund_account.key(),
                &pool_account_address,
                index as u8,
            );

            require_keys_eq!(fund_stake_account_address, fund_stake_account.key());
        }

        let initialized_fund_stake_accounts = fund_stake_accounts
            .iter()
            .cloned()
            .filter(|fund_stake_account| fund_stake_account.is_initialized());
        let claimable_fund_stake_accounts = SPLStakePoolService::<T>::get_claimable_stake_accounts(
            clock,
            stake_history,
            initialized_fund_stake_accounts,
        )?;

        let claimable_stake_account_indices = claimable_fund_stake_accounts
            .iter()
            .flat_map(|&address| {
                fund_stake_accounts
                    .iter()
                    .enumerate()
                    .find_map(|(index, fund_stake_account)| {
                        (fund_stake_account.key == address).then_some(index as u8)
                    })
            })
            .collect();

        let fund_account = ctx.fund_account.load()?;
        let fund_reserve_account = fund_account.get_reserve_account_address()?;
        let fund_treasury_account = fund_account.get_treasury_account_address()?;
        let accounts_to_claim_sol = SPLStakePoolService::<T>::find_accounts_to_claim_sol()?;
        let claimable_fund_stake_accounts = claimable_fund_stake_accounts
            .into_iter()
            .map(|&address| (address, true));

        let required_accounts = [(fund_reserve_account, true), (fund_treasury_account, true)]
            .into_iter()
            .chain(accounts_to_claim_sol)
            .chain(claimable_fund_stake_accounts);

        Ok(Self {
            state: Execute {
                pool_token_mints: pool_token_mints.to_vec(),
                claimable_stake_account_indices,
            },
        }
        .with_required_accounts(required_accounts))
    }

    #[inline(never)]
    fn execute_execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        pool_token_mints: &[Pubkey],
        claimable_stake_account_indices: &[u8],
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
                    claimable_stake_account_indices,
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
                    claimable_stake_account_indices,
                    pool_token_mint,
                    address,
                )?
            }
            Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { address }) => {
                self.spl_stake_pool_claim_sol::<SanctumMultiValidatorSPLStakePool>(
                    ctx,
                    accounts,
                    claimable_stake_account_indices,
                    pool_token_mint,
                    address,
                )?
            }
            // otherwise fails
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. })
            | Some(TokenPricingSource::VirtualVault { .. })
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
            let mut pricing_service =
                fund_service.new_pricing_service(remaining_accounts.iter().copied(), false)?;

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
            fund_service.update_asset_values(&mut pricing_service, true)?;
            drop(fund_service);

            let mut fund_account = ctx.fund_account.load_mut()?;
            require_gte!(
                to_sol_account_amount,
                fund_account.sol.get_total_reserved_amount(),
            );
            let supported_token = fund_account.get_supported_token_mut(pool_token_mint)?;

            // Deactivating stake account is treated as active stake, so it receives epoch reward!
            supported_token.pending_unstaking_amount_as_sol = supported_token
                .pending_unstaking_amount_as_sol
                .saturating_sub(claimed_sol_amount);

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
        claimable_stake_account_indices: &[u8],
        pool_token_mint_address: &Pubkey, // just informative
        pool_account_address: Pubkey,
    ) -> Result<(u64, u64, bool)> {
        let [fund_reserve_account, fund_treasury_account, clock, stake_history, stake_program, remaining_accounts @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        let fund_stake_accounts = {
            let num_stake_accounts = claimable_stake_account_indices.len();
            if remaining_accounts.len() < num_stake_accounts {
                err!(error::ErrorCode::AccountNotEnoughKeys)?
            }
            &remaining_accounts[..num_stake_accounts]
        };

        let mut total_claimed_sol_amount = 0;

        let fund_account = ctx.fund_account.load()?;

        for (&index, fund_stake_account) in claimable_stake_account_indices
            .iter()
            .zip(fund_stake_accounts)
        {
            let fund_stake_account_address = *FundAccount::find_stake_account_address(
                &ctx.fund_account.key(),
                &pool_account_address,
                index,
            );

            require_keys_eq!(fund_stake_account_address, fund_stake_account.key());

            // Skip uninitialized stake account
            if !fund_stake_account.is_initialized() {
                continue;
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
        }

        let to_sol_account_amount = fund_reserve_account.lamports();

        Ok((to_sol_account_amount, total_claimed_sol_amount, false))
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
        let withdrawal_ticket_accounts = {
            if remaining_accounts.len() < 5 {
                err!(error::ErrorCode::AccountNotEnoughKeys)?
            }
            &remaining_accounts[..5]
        };

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
