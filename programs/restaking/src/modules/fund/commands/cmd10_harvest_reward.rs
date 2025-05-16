use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::token_interface::TokenAccount;

use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::reward::{RewardAccount, RewardConfigurationService, RewardService};
use crate::modules::swap::{OrcaDEXLiquidityPoolService, TokenSwapSource};
use crate::utils::{AccountInfoExt, AsAccountInfo, PDASeeds};

use super::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct HarvestRewardCommand {
    state: HarvestRewardCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub enum HarvestRewardCommandState {
    /// Initializes a command based on the fund state and strategy.
    #[default]
    New,
    /// Initializes compounding reward harvest command based on the fund state.
    NewCompound,
    /// Prepares to harvest compounding rewards from the vault.
    PrepareCompound {
        vault: Pubkey,
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_COMPOUNDING_REWARD_TOKENS)]
        reward_token_mints: Vec<Pubkey>,
    },
    /// Prepares to harvest compounding rewards from the vault, with swap.
    PrepareSwap {
        vault: Pubkey,
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_COMPOUNDING_REWARD_TOKENS)]
        reward_token_mints: Vec<Pubkey>,
    },
    /// Harvest compounding rewards from the vault and transitions to the next command,
    /// either preparing the next item or preparing to harvest distributing rewards.
    ExecuteCompound {
        vault: Pubkey,
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_COMPOUNDING_REWARD_TOKENS)]
        reward_token_mints: Vec<Pubkey>,
    },
    /// Initializes distributing reward harvest command based on the fund state.
    NewDistribute,
    /// Prepares to harvest distributing rewards from the vault.
    PrepareDistribute {
        vault: Pubkey,
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_DISTRIBUTING_REWARD_TOKENS)]
        reward_token_mints: Vec<Pubkey>,
    },
    /// Harvest distributing rewards from the vault and transitions to the next command,
    /// either preparing the next item or performing an unstaking operation.
    ExecuteDistribute {
        vault: Pubkey,
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_DISTRIBUTING_REWARD_TOKENS)]
        reward_token_mints: Vec<Pubkey>,
    },
}

use HarvestRewardCommandState::*;

impl std::fmt::Debug for HarvestRewardCommandState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "New"),
            Self::NewCompound => write!(f, "NewCompound"),
            Self::PrepareCompound {
                vault,
                reward_token_mints,
            } => f
                .debug_struct("PrepareCompound")
                .field("vault", vault)
                .field_first_element("reward_token_mint", reward_token_mints)
                .finish(),
            Self::PrepareSwap {
                vault,
                reward_token_mints,
            } => f
                .debug_struct("PrepareSwap")
                .field("vault", vault)
                .field_first_element("reward_token_mint", reward_token_mints)
                .finish(),
            Self::ExecuteCompound {
                vault,
                reward_token_mints,
            } => f
                .debug_struct("ExecuteCompound")
                .field("vault", vault)
                .field_first_element("reward_token_mint", reward_token_mints)
                .finish(),
            Self::NewDistribute => write!(f, "NewDistribute"),
            Self::PrepareDistribute {
                vault,
                reward_token_mints,
            } => f
                .debug_struct("PrepareDistribute")
                .field("vault", vault)
                .field_first_element("reward_token_mints", reward_token_mints)
                .finish(),
            Self::ExecuteDistribute {
                vault,
                reward_token_mints,
            } => f
                .debug_struct("ExecuteDistribute")
                .field("vault", vault)
                .field_first_element("reward_token_mints", reward_token_mints)
                .finish(),
        }
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct HarvestRewardCommandResult {
    pub vault: Pubkey,
    pub reward_token_mint: Pubkey,
    pub reward_token_amount: u64,
    pub swapped_token_mint: Option<Pubkey>,
    pub compounded_token_amount: u64,
    pub distributed_token_amount: u64,
    pub updated_reward_account: Option<Pubkey>,
}

#[derive(Clone, Copy)]
enum HarvestType {
    Compound,
    Distribute,
}

impl SelfExecutable for HarvestRewardCommand {
    fn execute<'a, 'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> ExecutionResult {
        let (result, entry) = match &self.state {
            // 1. compounding_reward
            New | NewCompound => self.execute_new_compound_command(ctx, None, None)?,
            PrepareCompound {
                vault,
                reward_token_mints,
            } => self.execute_prepare_compound_command(ctx, accounts, vault, reward_token_mints)?,
            PrepareSwap {
                vault,
                reward_token_mints,
            } => self.execute_prepare_swap_command(ctx, accounts, vault, reward_token_mints)?,
            ExecuteCompound {
                vault,
                reward_token_mints,
            } => self.execute_execute_compound_command(ctx, accounts, vault, reward_token_mints)?,
            // 2. distributing_reward
            NewDistribute => self.execute_new_distribute_command(ctx, None, None)?,
            PrepareDistribute {
                vault,
                reward_token_mints,
            } => {
                self.execute_prepare_distribute_command(ctx, accounts, vault, reward_token_mints)?
            }
            ExecuteDistribute {
                vault,
                reward_token_mints,
            } => {
                self.execute_execute_distribute_command(ctx, accounts, vault, reward_token_mints)?
            }
        };

        Ok((
            result,
            entry.or_else(|| Some(StakeSOLCommand::default().without_required_accounts())),
        ))
    }
}

#[deny(clippy::wildcard_enum_match_arm)]
impl HarvestRewardCommand {
    fn execute_new_compound_command(
        &self,
        ctx: &OperationCommandContext,
        previous_vault: Option<&Pubkey>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> ExecutionResult {
        self.execute_new_command(
            ctx,
            HarvestType::Compound,
            previous_vault,
            previous_execution_result,
        )
    }

    fn execute_new_distribute_command(
        &self,
        ctx: &OperationCommandContext,
        previous_vault: Option<&Pubkey>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> ExecutionResult {
        self.execute_new_command(
            ctx,
            HarvestType::Distribute,
            previous_vault,
            previous_execution_result,
        )
    }

    #[inline(never)]
    fn execute_new_command(
        &self,
        ctx: &OperationCommandContext,
        harvest_type: HarvestType,
        previous_vault: Option<&Pubkey>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> ExecutionResult {
        let mut start_search = previous_vault.is_none();
        let Some((vault, reward_token_mints)) = ctx
            .fund_account
            .load()?
            .get_restaking_vaults_iter()
            .find_map(|restaking_vault| {
                if !start_search {
                    // before starting search, find previous vault
                    if previous_vault.is_some_and(|vault| restaking_vault.vault == *vault) {
                        // previous vault found, so start search from next vault
                        start_search = true;
                    }

                    return None;
                }

                let reward_token_mints = match harvest_type {
                    HarvestType::Compound => restaking_vault
                        .get_compounding_reward_tokens_iter()
                        .copied()
                        .collect(),
                    HarvestType::Distribute => restaking_vault
                        .get_distributing_reward_tokens_iter()
                        .copied()
                        .collect(),
                };

                Some((restaking_vault.vault, reward_token_mints))
            })
        else {
            return match harvest_type {
                // fallback: 2. distributing_reward
                HarvestType::Compound => {
                    self.execute_new_distribute_command(ctx, None, previous_execution_result)
                }
                // fallback: cmd11: stake_sol
                HarvestType::Distribute => Ok((previous_execution_result, None)),
            };
        };

        if let Some(entry) =
            self.create_prepare_command(ctx, harvest_type, vault, reward_token_mints)?
        {
            Ok((previous_execution_result, Some(entry)))
        } else {
            // fallback: next vault
            self.execute_new_command(ctx, harvest_type, Some(&vault), previous_execution_result)
        }
    }

    fn create_prepare_compound_command(
        &self,
        ctx: &OperationCommandContext,
        vault: Pubkey,
        reward_token_mints: Vec<Pubkey>,
    ) -> Result<Option<OperationCommandEntry>> {
        if reward_token_mints.is_empty() {
            return Ok(None);
        }

        let receipt_token_pricing_source = ctx
            .fund_account
            .load()?
            .get_restaking_vault(&vault)?
            .receipt_token_pricing_source
            .try_deserialize()?;

        let entry = match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { .. }) => {
                // We need to check vault's token account whether
                // the account is delegated to fund account or not.
                // Although we do not know whether the token
                // belongs to token program or token 2022 program,
                // we can try both ATAs.
                let required_accounts = [
                    (reward_token_mints[0], false),
                    (
                        associated_token::get_associated_token_address_with_program_id(
                            &vault,
                            &reward_token_mints[0],
                            &anchor_spl::token::ID,
                        ),
                        false,
                    ),
                    (
                        associated_token::get_associated_token_address_with_program_id(
                            &vault,
                            &reward_token_mints[0],
                            &anchor_spl::token_2022::ID,
                        ),
                        false,
                    ),
                ];

                let command = Self {
                    state: PrepareCompound {
                        vault,
                        reward_token_mints,
                    },
                };
                command.with_required_accounts(required_accounts)
            }
            // TODO/v0.7.0: deal with solv vault if needed
            Some(TokenPricingSource::SolvBTCVault { .. }) => return Ok(None),
            // otherwise fails
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        Ok(Some(entry))
    }

    fn create_prepare_distribute_command(
        &self,
        ctx: &OperationCommandContext,
        vault: Pubkey,
        reward_token_mints: Vec<Pubkey>,
    ) -> Result<Option<OperationCommandEntry>> {
        if reward_token_mints.is_empty() {
            return Ok(None);
        }

        let receipt_token_pricing_source = ctx
            .fund_account
            .load()?
            .get_restaking_vault(&vault)?
            .receipt_token_pricing_source
            .try_deserialize()?;

        let entry = match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. }) => {
                // We need to check vault's token account whether
                // the account is delegated to fund account or not.
                // Although we do not know whether the token
                // belongs to token program or token 2022 program,
                // we can try both ATAs.
                let required_accounts = [
                    (reward_token_mints[0], false),
                    (
                        associated_token::get_associated_token_address_with_program_id(
                            &vault,
                            &reward_token_mints[0],
                            &anchor_spl::token::ID,
                        ),
                        false,
                    ),
                    (
                        associated_token::get_associated_token_address_with_program_id(
                            &vault,
                            &reward_token_mints[0],
                            &anchor_spl::token_2022::ID,
                        ),
                        false,
                    ),
                    (
                        RewardAccount::find_account_address(&ctx.receipt_token_mint.key()),
                        false,
                    ),
                ];

                let command = Self {
                    state: PrepareDistribute {
                        vault,
                        reward_token_mints,
                    },
                };
                command.with_required_accounts(required_accounts)
            }
            // otherwise fails
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        Ok(Some(entry))
    }

    fn create_prepare_command(
        &self,
        ctx: &OperationCommandContext,
        harvest_type: HarvestType,
        vault: Pubkey,
        reward_token_mints: Vec<Pubkey>,
    ) -> Result<Option<OperationCommandEntry>> {
        if reward_token_mints.is_empty() {
            return Ok(None);
        }

        match harvest_type {
            HarvestType::Compound => {
                self.create_prepare_compound_command(ctx, vault, reward_token_mints)
            }
            HarvestType::Distribute => {
                self.create_prepare_distribute_command(ctx, vault, reward_token_mints)
            }
        }
    }

    fn execute_prepare_compound_command<'info>(
        &self,
        ctx: &OperationCommandContext,
        accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mints: &[Pubkey],
    ) -> ExecutionResult {
        self.execute_prepare_command(
            ctx,
            HarvestType::Compound,
            accounts,
            vault,
            reward_token_mints,
        )
    }

    fn execute_prepare_distribute_command<'info>(
        &self,
        ctx: &OperationCommandContext,
        accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mints: &[Pubkey],
    ) -> ExecutionResult {
        self.execute_prepare_command(
            ctx,
            HarvestType::Distribute,
            accounts,
            vault,
            reward_token_mints,
        )
    }

    #[inline(never)]
    fn execute_prepare_command<'info>(
        &self,
        ctx: &OperationCommandContext,
        harvest_type: HarvestType,
        accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mints: &[Pubkey],
    ) -> ExecutionResult {
        if reward_token_mints.is_empty() {
            // fallback: next vault
            return self.execute_new_command(ctx, harvest_type, Some(vault), None);
        }

        let fund_account = ctx.fund_account.load()?;
        let receipt_token_pricing_source = fund_account
            .get_restaking_vault(vault)?
            .receipt_token_pricing_source
            .try_deserialize()?;
        let Some(entry) = (|| {
            match harvest_type {
                HarvestType::Compound => match receipt_token_pricing_source {
                    Some(TokenPricingSource::JitoRestakingVault { .. }) => self
                        .create_execute_command_from_vault_ata(
                            ctx,
                            harvest_type,
                            accounts,
                            vault,
                            reward_token_mints,
                        ),
                    // TODO/v0.7.0: deal with solv vault if needed
                    Some(TokenPricingSource::SolvBTCVault { .. }) => return Ok(None),
                    // otherwise fails
                    Some(TokenPricingSource::SPLStakePool { .. })
                    | Some(TokenPricingSource::MarinadeStakePool { .. })
                    | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                    | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
                    | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                    | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                    | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                    | Some(TokenPricingSource::PeggedToken { .. })
                    | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
                    #[cfg(all(test, not(feature = "idl-build")))]
                    Some(TokenPricingSource::Mock { .. }) => {
                        err!(ErrorCode::FundOperationCommandExecutionFailedException)?
                    }
                },
                HarvestType::Distribute => match receipt_token_pricing_source {
                    Some(TokenPricingSource::JitoRestakingVault { .. })
                    | Some(TokenPricingSource::SolvBTCVault { .. }) => self
                        .create_execute_command_from_vault_ata(
                            ctx,
                            harvest_type,
                            accounts,
                            vault,
                            reward_token_mints,
                        ),
                    // otherwise fails
                    Some(TokenPricingSource::SPLStakePool { .. })
                    | Some(TokenPricingSource::MarinadeStakePool { .. })
                    | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                    | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
                    | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                    | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                    | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                    | Some(TokenPricingSource::PeggedToken { .. })
                    | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
                    #[cfg(all(test, not(feature = "idl-build")))]
                    Some(TokenPricingSource::Mock { .. }) => {
                        err!(ErrorCode::FundOperationCommandExecutionFailedException)?
                    }
                },
            }
        })()?
        else {
            // fallback: next reward
            let Some(entry) = self.create_prepare_command(
                ctx,
                harvest_type,
                *vault,
                reward_token_mints[1..].to_vec(),
            )?
            else {
                // fallback: next vault
                return self.execute_new_command(ctx, harvest_type, Some(vault), None);
            };

            return Ok((None, Some(entry)));
        };

        Ok((None, Some(entry)))
    }

    /// Creates an Execute command given that vault's ATA is the source reward token account.
    fn create_execute_command_from_vault_ata<'info>(
        &self,
        ctx: &OperationCommandContext,
        harvest_type: HarvestType,
        accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mints: &[Pubkey],
    ) -> Result<Option<OperationCommandEntry>> {
        let fund_account = ctx.fund_account.load()?;

        // Jito restaking or Solv BTC rewards are deposited in vault's ATA.
        // To harvest them, vault's ATA must be delegated to fund account.
        let [reward_token_mint, vault_reward_token_account, vault_reward_token_2022_account, remaining_accounts @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        require_keys_eq!(reward_token_mint.key(), reward_token_mints[0]);

        let reward_token_program = reward_token_mint.owner;
        let vault_reward_token_account = match reward_token_program {
            &anchor_spl::token::ID => vault_reward_token_account,
            &anchor_spl::token_2022::ID => vault_reward_token_2022_account,
            _ => err!(error::ErrorCode::InvalidProgramId)?,
        };

        // Token account does not exist, so move on to next item
        if !vault_reward_token_account.is_initialized() {
            return Ok(None);
        }

        require_keys_eq!(*vault_reward_token_account.owner, *reward_token_program);
        let vault_reward_token_account =
            InterfaceAccount::<TokenAccount>::try_from(vault_reward_token_account)?;
        require_keys_eq!(vault_reward_token_account.mint, reward_token_mint.key());
        require_keys_eq!(vault_reward_token_account.owner, *vault);

        // Token account is not delegated to fund account, so move on to next item
        if !vault_reward_token_account
            .delegate
            .contains(&ctx.fund_account.key())
        {
            return Ok(None);
        }

        let reward_token_amount = vault_reward_token_account
            .amount
            .min(vault_reward_token_account.delegated_amount);

        // No reward, so move on to next item
        if reward_token_amount == 0 {
            return Ok(None);
        }

        // Prepare based on harvest type
        let entry = match harvest_type {
            HarvestType::Compound
                if fund_account
                    .get_supported_token(reward_token_mint.key)
                    .is_err() =>
            {
                // Need to swap
                let swap_strategy = fund_account.get_token_swap_strategy(reward_token_mint.key)?;

                let swap_source = swap_strategy.swap_source.try_deserialize()?;
                match swap_source {
                    TokenSwapSource::OrcaDEXLiquidityPool { address } => {
                        let required_accounts = [
                            (address, false),
                            (reward_token_mint.key(), false),
                            (vault_reward_token_account.key(), false),
                            (swap_strategy.to_token_mint, false),
                        ];

                        let command = Self {
                            state: PrepareSwap {
                                vault: *vault,
                                reward_token_mints: reward_token_mints.to_vec(),
                            },
                        };
                        command.with_required_accounts(required_accounts)
                    }
                }
            }
            HarvestType::Compound => {
                // Just transfer
                let fund_supported_token_reserve_account = fund_account
                    .find_supported_token_reserve_account_address(reward_token_mint.key)?;
                let required_accounts = [
                    (reward_token_mint.key(), false),
                    (vault_reward_token_account.key(), true),
                    (fund_supported_token_reserve_account, true),
                    (*reward_token_program, false),
                ];

                let command = Self {
                    state: ExecuteCompound {
                        vault: *vault,
                        reward_token_mints: reward_token_mints.to_vec(),
                    },
                };
                command.with_required_accounts(required_accounts)
            }
            HarvestType::Distribute => {
                // Transfer + Settle
                let [reward_account, ..] = remaining_accounts else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(
                    reward_account.key(),
                    RewardAccount::find_account_address(&ctx.receipt_token_mint.key()),
                );

                let reward_account = AccountLoader::<RewardAccount>::try_from(reward_account)?;

                let Some(reward_token_reserve_account) = reward_account
                    .load()?
                    .find_reward_token_reserve_account_address(reward_token_mint.key)?
                else {
                    // Reward is not claimable, so move on to next reward
                    return Ok(None);
                };

                let required_accounts = [
                    (reward_token_mint.key(), false),
                    (vault_reward_token_account.key(), true),
                    (reward_token_reserve_account, true),
                    (*reward_token_program, false),
                    (reward_account.key(), true),
                ];

                let command = Self {
                    state: ExecuteDistribute {
                        vault: *vault,
                        reward_token_mints: reward_token_mints.to_vec(),
                    },
                };
                command.with_required_accounts(required_accounts)
            }
        };

        Ok(Some(entry))
    }

    #[inline(never)]
    fn execute_prepare_swap_command<'info>(
        &self,
        ctx: &OperationCommandContext,
        accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mints: &[Pubkey],
    ) -> ExecutionResult {
        if reward_token_mints.is_empty() {
            return self.execute_new_compound_command(ctx, Some(vault), None);
        }

        let fund_account = ctx.fund_account.load()?;
        let swap_strategy = fund_account.get_token_swap_strategy(&reward_token_mints[0])?;
        let swap_source = swap_strategy.swap_source.try_deserialize()?;
        let entry = match swap_source {
            TokenSwapSource::OrcaDEXLiquidityPool { address } => {
                let [pool_account, reward_token_mint, from_reward_token_account, supported_token_mint, ..] =
                    accounts
                else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(pool_account.key(), address);
                require_keys_eq!(reward_token_mint.key(), reward_token_mints[0]);
                require_keys_eq!(supported_token_mint.key(), swap_strategy.to_token_mint);

                let accounts_to_swap = OrcaDEXLiquidityPoolService::find_accounts_to_swap(
                    pool_account,
                    reward_token_mint,
                    supported_token_mint,
                )?;
                let fund_supported_token_reserve_account = fund_account
                    .find_supported_token_reserve_account_address(supported_token_mint.key)?;
                let required_accounts = accounts_to_swap.chain([
                    (from_reward_token_account.key(), true),
                    (fund_supported_token_reserve_account, true),
                ]);

                let command = Self {
                    state: ExecuteCompound {
                        vault: *vault,
                        reward_token_mints: reward_token_mints.to_vec(),
                    },
                };
                command.with_required_accounts(required_accounts)
            }
        };

        Ok((None, Some(entry)))
    }

    #[inline(never)]
    fn execute_execute_compound_command<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        mut accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mints: &[Pubkey],
    ) -> ExecutionResult {
        if reward_token_mints.is_empty() {
            return self.execute_new_compound_command(ctx, Some(vault), None);
        }

        let fund_account = ctx.fund_account.load()?;
        let result = if fund_account
            .get_supported_token(&reward_token_mints[0])
            .is_err()
        {
            // Swap
            self.execute_swap(ctx, &mut accounts, vault, &reward_token_mints[0])?
        } else {
            let receipt_token_pricing_source = fund_account
                .get_restaking_vault(vault)?
                .receipt_token_pricing_source
                .try_deserialize()?;
            match receipt_token_pricing_source {
                Some(TokenPricingSource::JitoRestakingVault { .. }) => {
                    // Transfer
                    self.execute_transfer(ctx, &mut accounts, vault, &reward_token_mints[0])?
                }
                Some(TokenPricingSource::SolvBTCVault { .. }) => {
                    // TODO/v0.7.0: deal with solv vault if needed
                    None
                }
                // otherwise fails
                Some(TokenPricingSource::SPLStakePool { .. })
                | Some(TokenPricingSource::MarinadeStakePool { .. })
                | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
                | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
                | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
                | Some(TokenPricingSource::FragmetricRestakingFund { .. })
                | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
                | Some(TokenPricingSource::PeggedToken { .. })
                | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
                #[cfg(all(test, not(feature = "idl-build")))]
                Some(TokenPricingSource::Mock { .. }) => {
                    err!(ErrorCode::FundOperationCommandExecutionFailedException)?
                }
            }
        };

        drop(fund_account);

        if let Some(result) = &result {
            let supported_token_mint = result
                .swapped_token_mint
                .as_ref()
                .unwrap_or(&result.reward_token_mint);
            ctx.fund_account
                .load_mut()?
                .get_supported_token_mut(supported_token_mint)?
                .token
                .operation_reserved_amount += result.compounded_token_amount;

            // Update pricing
            FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                .new_pricing_service(accounts.iter().copied(), true)?;
        }

        // move on to next item
        let result = result.map(Into::into);
        let Some(entry) =
            self.create_prepare_compound_command(ctx, *vault, reward_token_mints[1..].to_vec())?
        else {
            // fallback: next vault
            return self.execute_new_compound_command(ctx, Some(vault), result);
        };

        Ok((result, Some(entry)))
    }

    fn execute_swap<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        accounts: &mut &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mint: &Pubkey,
    ) -> Result<Option<HarvestRewardCommandResult>> {
        let fund_account = ctx.fund_account.load()?;
        let swap_strategy = fund_account.get_token_swap_strategy(reward_token_mint)?;
        let supported_token_mint = &swap_strategy.to_token_mint;

        let swap_source = swap_strategy.swap_source.try_deserialize()?;
        let result = match swap_source {
            TokenSwapSource::OrcaDEXLiquidityPool { address } => {
                let [pool_program, pool_account, token_mint_a, token_vault_a, token_program_a, token_mint_b, token_vault_b, token_program_b, memo_program, oracle, tick_array0, tick_array1, tick_array2, from_reward_token_account, fund_supported_token_reserve_account, pricing_sources @ ..] =
                    accounts
                else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                *accounts = pricing_sources;

                require_keys_eq!(pool_account.key(), address);
                require_keys_eq!(
                    fund_supported_token_reserve_account.key(),
                    fund_account
                        .find_supported_token_reserve_account_address(supported_token_mint)?
                );

                let from_reward_token_account =
                    InterfaceAccount::<TokenAccount>::try_from(from_reward_token_account)?;
                require_keys_eq!(from_reward_token_account.mint, *reward_token_mint);

                let dex_service = OrcaDEXLiquidityPoolService::new(
                    pool_program,
                    pool_account,
                    token_mint_a,
                    token_vault_a,
                    token_program_a,
                    token_mint_b,
                    token_vault_b,
                    token_program_b,
                )?;

                let (from_token_swapped_amount, to_token_swapped_amount) = dex_service.swap(
                    memo_program,
                    oracle,
                    tick_array0,
                    tick_array1,
                    tick_array2,
                    from_reward_token_account.as_account_info(),
                    fund_supported_token_reserve_account,
                    ctx.fund_account.as_ref(),
                    &[&fund_account.get_seeds()],
                    from_reward_token_account
                        .amount
                        .min(from_reward_token_account.delegated_amount),
                )?;

                if to_token_swapped_amount == 0 {
                    return Ok(None);
                }

                HarvestRewardCommandResult {
                    vault: *vault,
                    reward_token_mint: *reward_token_mint,
                    reward_token_amount: from_token_swapped_amount,
                    swapped_token_mint: Some(*supported_token_mint),
                    compounded_token_amount: to_token_swapped_amount,
                    distributed_token_amount: 0,
                    updated_reward_account: None,
                }
            }
        };

        Ok(Some(result))
    }

    fn execute_transfer<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        accounts: &mut &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        supported_token_mint: &Pubkey,
    ) -> Result<Option<HarvestRewardCommandResult>> {
        let fund_account = ctx.fund_account.load()?;

        let [reward_token_mint, from_reward_token_account, fund_supported_token_reserve_account, reward_token_program, pricing_sources @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        *accounts = pricing_sources;

        let reward_token_mint = InterfaceAccount::<Mint>::try_from(reward_token_mint)?;
        require_keys_eq!(reward_token_mint.key(), *supported_token_mint);

        let from_reward_token_account =
            InterfaceAccount::<TokenAccount>::try_from(from_reward_token_account)?;
        require_keys_eq!(from_reward_token_account.mint, *supported_token_mint);

        require_keys_eq!(
            fund_supported_token_reserve_account.key(),
            fund_account.find_supported_token_reserve_account_address(supported_token_mint)?
        );

        let reward_token_amount = from_reward_token_account
            .amount
            .min(from_reward_token_account.delegated_amount);

        if reward_token_amount == 0 {
            return Ok(None);
        }

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                reward_token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: from_reward_token_account.to_account_info(),
                    mint: reward_token_mint.to_account_info(),
                    to: fund_supported_token_reserve_account.to_account_info(),
                    authority: ctx.fund_account.to_account_info(),
                },
                &[&fund_account.get_seeds()],
            ),
            reward_token_amount,
            reward_token_mint.decimals,
        )?;

        let result = HarvestRewardCommandResult {
            vault: *vault,
            reward_token_mint: *supported_token_mint,
            reward_token_amount,
            swapped_token_mint: None,
            compounded_token_amount: reward_token_amount,
            distributed_token_amount: 0,
            updated_reward_account: None,
        };

        Ok(Some(result))
    }

    #[inline(never)]
    fn execute_execute_distribute_command<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mints: &[Pubkey],
    ) -> ExecutionResult {
        if reward_token_mints.is_empty() {
            return self.execute_new_distribute_command(ctx, Some(vault), None);
        }

        let receipt_token_pricing_source = ctx
            .fund_account
            .load()?
            .get_restaking_vault(vault)?
            .receipt_token_pricing_source
            .try_deserialize()?;
        let result = (|| match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. }) => {
                // Transfer & Settle
                let [reward_token_mint, from_reward_token_account, reward_token_reserve_account, reward_token_program, reward_account, ..] =
                    accounts
                else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };

                let reward_account = AccountLoader::<RewardAccount>::try_from(reward_account)?;
                RewardService::validate_reward_account(&ctx.receipt_token_mint, &reward_account)?;

                let reward_token_mint = InterfaceAccount::<Mint>::try_from(reward_token_mint)?;
                require_keys_eq!(reward_token_mint.key(), reward_token_mints[0]);

                let mut from_reward_token_account =
                    InterfaceAccount::<TokenAccount>::try_from(from_reward_token_account)?;
                require_keys_eq!(from_reward_token_account.mint, reward_token_mints[0]);

                let Some(reward_token_reserve_account_address) = reward_account
                    .load()?
                    .find_reward_token_reserve_account_address(&reward_token_mint.key())?
                else {
                    // reward isn't claimable, so move on to next item.
                    return Ok(None);
                };
                let mut reward_token_reserve_account =
                    InterfaceAccount::<TokenAccount>::try_from(reward_token_reserve_account)?;
                require_keys_eq!(
                    reward_token_reserve_account.key(),
                    reward_token_reserve_account_address,
                );

                let reward_token_program = Interface::try_from(*reward_token_program)?;

                let reward_token_amount = from_reward_token_account
                    .amount
                    .min(from_reward_token_account.delegated_amount);

                // No reward, so move on to next item
                if reward_token_amount == 0 {
                    return Ok(None);
                }

                anchor_spl::token_interface::transfer_checked(
                    CpiContext::new_with_signer(
                        reward_token_program.to_account_info(),
                        anchor_spl::token_interface::TransferChecked {
                            from: from_reward_token_account.to_account_info(),
                            mint: reward_token_mint.to_account_info(),
                            to: reward_token_reserve_account.to_account_info(),
                            authority: ctx.fund_account.to_account_info(),
                        },
                        &[&ctx.fund_account.load()?.get_seeds()],
                    ),
                    reward_token_amount,
                    reward_token_mint.decimals,
                )?;
                from_reward_token_account.reload()?;
                reward_token_reserve_account.reload()?;

                RewardConfigurationService::new(ctx.receipt_token_mint, &reward_account)?
                    .settle_reward(
                        Some(&reward_token_mint),
                        Some(&reward_token_program),
                        Some(&reward_token_reserve_account),
                        reward_token_mint.key(),
                        false,
                        reward_token_amount,
                    )?;

                let result = HarvestRewardCommandResult {
                    vault: *vault,
                    reward_token_mint: reward_token_mint.key(),
                    reward_token_amount,
                    swapped_token_mint: None,
                    compounded_token_amount: 0,
                    distributed_token_amount: reward_token_amount,
                    updated_reward_account: Some(reward_account.key()),
                }
                .into();

                Ok(Some(result))
            }
            // otherwise fails
            Some(TokenPricingSource::SPLStakePool { .. })
            | Some(TokenPricingSource::MarinadeStakePool { .. })
            | Some(TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. })
            | Some(TokenPricingSource::FragmetricNormalizedTokenPool { .. })
            | Some(TokenPricingSource::FragmetricRestakingFund { .. })
            | Some(TokenPricingSource::OrcaDEXLiquidityPool { .. })
            | Some(TokenPricingSource::PeggedToken { .. })
            | None => err!(ErrorCode::FundOperationCommandExecutionFailedException)?,
            #[cfg(all(test, not(feature = "idl-build")))]
            Some(TokenPricingSource::Mock { .. }) => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        })()?;

        // move on to next item
        let Some(entry) =
            self.create_prepare_distribute_command(ctx, *vault, reward_token_mints[1..].to_vec())?
        else {
            // fallback: next vault
            return self.execute_new_distribute_command(ctx, Some(vault), result);
        };

        Ok((result, Some(entry)))
    }
}
