use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::token_interface::TokenAccount;

use crate::constants::PROGRAM_REVENUE_ADDRESS;
use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;
use crate::modules::restaking::VirtualVaultService;
use crate::modules::reward::{RewardAccount, RewardService};
use crate::modules::swap::{OrcaDEXLiquidityPoolService, TokenSwapSource};
use crate::utils::{AccountInfoExt, PDASeeds};

use super::*;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct HarvestRestakingYieldCommand {
    state: HarvestRestakingYieldState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub enum HarvestRestakingYieldState {
    /// Initializes a command based on the fund state and strategy.
    #[default]
    New,
    /// Initializes compounding reward harvest command based on the fund state.
    NewCompoundReward,
    /// Prepares to harvest compounding rewards from the vault.
    PrepareCompoundReward {
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
    ExecuteCompoundReward {
        vault: Pubkey,
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_COMPOUNDING_REWARD_TOKENS)]
        reward_token_mints: Vec<Pubkey>,
    },
    /// Harvest compounding rewards from the vault by swap and transitions to the next command,
    /// either preparing the next item or preparing to harvest distributing rewards.
    ExecuteSwap {
        vault: Pubkey,
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_COMPOUNDING_REWARD_TOKENS)]
        reward_token_mints: Vec<Pubkey>,
    },
    /// Initializes distributing reward harvest command based on the fund state.
    NewDistributeReward,
    /// Prepares to harvest distributing rewards from the vault.
    PrepareDistributeReward {
        vault: Pubkey,
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_DISTRIBUTING_REWARD_TOKENS)]
        reward_token_mints: Vec<Pubkey>,
    },
    /// Harvest distributing rewards from the vault and transitions to the next command,
    /// either preparing the next item or preparing to harvest compounding vault supported token.
    ExecuteDistributeReward {
        vault: Pubkey,
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULT_DISTRIBUTING_REWARD_TOKENS)]
        reward_token_mints: Vec<Pubkey>,
    },
    /// Initializes compounding vault supported token command based on the fund state.
    NewCompoundVaultSupportedToken,
    /// Prepares to harvest compounding vault supported tokens from the vault.
    PrepareCompoundVaultSupportedToken { vault: Pubkey },
    /// Harvest compounding vault supported token by calculating vst changed amount
    /// and transitions to the next command either preparing the next item or performing stake operation.
    ExecuteCompoundVaultSupportedToken { vault: Pubkey },
}

use HarvestRestakingYieldState::*;

impl std::fmt::Debug for HarvestRestakingYieldState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "New"),
            Self::NewCompoundReward => write!(f, "NewCompoundReward"),
            Self::PrepareCompoundReward {
                vault,
                reward_token_mints,
            } => f
                .debug_struct("PrepareCompoundReward")
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
            Self::ExecuteCompoundReward {
                vault,
                reward_token_mints,
            } => f
                .debug_struct("ExecuteCompoundReward")
                .field("vault", vault)
                .field_first_element("reward_token_mint", reward_token_mints)
                .finish(),
            Self::ExecuteSwap {
                vault,
                reward_token_mints,
            } => f
                .debug_struct("ExecuteSwap")
                .field("vault", vault)
                .field_first_element("reward_token_mint", reward_token_mints)
                .finish(),
            Self::NewDistributeReward => write!(f, "NewDistributeReward"),
            Self::PrepareDistributeReward {
                vault,
                reward_token_mints,
            } => f
                .debug_struct("PrepareDistributeReward")
                .field("vault", vault)
                .field_first_element("reward_token_mints", reward_token_mints)
                .finish(),
            Self::ExecuteDistributeReward {
                vault,
                reward_token_mints,
            } => f
                .debug_struct("ExecuteDistributeReward")
                .field("vault", vault)
                .field_first_element("reward_token_mints", reward_token_mints)
                .finish(),
            Self::NewCompoundVaultSupportedToken => write!(f, "NewCompoundVaultSupportedToken"),
            Self::PrepareCompoundVaultSupportedToken { vault } => f
                .debug_struct("PrepareCompoundVaultSupportedToken")
                .field("vault", vault)
                .finish(),
            Self::ExecuteCompoundVaultSupportedToken { vault } => f
                .debug_struct("ExecuteCompoundVaultSupportedToken")
                .field("vault", vault)
                .finish(),
        }
    }
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct HarvestRestakingYieldCommandResult {
    pub vault: Pubkey,
    pub yield_token_mint: Pubkey,
    pub yield_token_amount: u64,
    pub swapped_token_mint: Option<Pubkey>,
    pub fund_supported_token_compounded_token_amount: u64,
    pub reward_token_distributed_token_amount: u64,
    pub updated_reward_account: Option<Pubkey>,
    pub vault_supported_token_compounded_amount: i128,
    // TODO: add new field for deducted amount by commission & reorder fields
}

#[derive(Clone, Copy)]
enum HarvestType {
    CompoundReward,
    DistributeReward,
    CompoundVaultSupportedToken,
}

impl SelfExecutable for HarvestRestakingYieldCommand {
    fn execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> ExecutionResult {
        let (result, entry) = match &self.state {
            // 1. compounding reward
            New | NewCompoundReward => self.execute_new_compound_reward_command(ctx, None, None)?,
            PrepareCompoundReward {
                vault,
                reward_token_mints,
            } => self.execute_prepare_compound_reward_command(
                ctx,
                accounts,
                vault,
                reward_token_mints,
            )?,
            PrepareSwap {
                vault,
                reward_token_mints,
            } => self.execute_prepare_swap_command(ctx, accounts, vault, reward_token_mints)?,
            ExecuteCompoundReward {
                vault,
                reward_token_mints,
            } => self.execute_execute_compound_reward_command(
                ctx,
                accounts,
                vault,
                reward_token_mints,
            )?,
            ExecuteSwap {
                vault,
                reward_token_mints,
            } => self.execute_execute_swap_command(ctx, accounts, vault, reward_token_mints)?,
            // 2. distributing reward
            NewDistributeReward => self.execute_new_distribute_reward_command(ctx, None, None)?,
            PrepareDistributeReward {
                vault,
                reward_token_mints,
            } => self.execute_prepare_distribute_reward_command(
                ctx,
                accounts,
                vault,
                reward_token_mints,
            )?,
            ExecuteDistributeReward {
                vault,
                reward_token_mints,
            } => self.execute_execute_distribute_reward_command(
                ctx,
                accounts,
                vault,
                reward_token_mints,
            )?,
            // 3. compounding vault supported token
            NewCompoundVaultSupportedToken => {
                self.execute_new_compound_vault_supported_token_command(ctx, None, None)?
            }
            PrepareCompoundVaultSupportedToken { vault } => {
                self.execute_prepare_compound_vault_supported_token_command(ctx, vault)?
            }
            ExecuteCompoundVaultSupportedToken { vault } => {
                self.execute_execute_compound_vault_supported_token_command(ctx, vault)?
            }
        };

        Ok((
            result,
            entry.or_else(|| Some(StakeSOLCommand::default().without_required_accounts())),
        ))
    }
}

impl HarvestRestakingYieldCommand {
    fn execute_new_compound_reward_command(
        &self,
        ctx: &OperationCommandContext,
        previous_vault: Option<&Pubkey>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> ExecutionResult {
        self.execute_new_command(
            ctx,
            HarvestType::CompoundReward,
            previous_vault,
            previous_execution_result,
        )
    }

    fn execute_new_distribute_reward_command(
        &self,
        ctx: &OperationCommandContext,
        previous_vault: Option<&Pubkey>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> ExecutionResult {
        self.execute_new_command(
            ctx,
            HarvestType::DistributeReward,
            previous_vault,
            previous_execution_result,
        )
    }

    fn execute_new_compound_vault_supported_token_command(
        &self,
        ctx: &OperationCommandContext,
        previous_vault: Option<&Pubkey>,
        previous_execution_result: Option<OperationCommandResult>,
    ) -> ExecutionResult {
        self.execute_new_command(
            ctx,
            HarvestType::CompoundVaultSupportedToken,
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
        let fund_account = ctx.fund_account.load()?;
        let Some((vault, reward_token_mints)) = (|| {
            let mut restaking_vaults_iter = fund_account.get_restaking_vaults_iter();
            let restaking_vault = if let Some(previous_vault) = previous_vault {
                restaking_vaults_iter
                    .skip_while(|restaking_vault| restaking_vault.vault != *previous_vault)
                    .nth(1)?
            } else {
                restaking_vaults_iter.next()?
            };

            let reward_token_mints = match harvest_type {
                HarvestType::CompoundReward => restaking_vault
                    .get_compounding_reward_tokens_iter()
                    .map(|reward_token| reward_token.mint)
                    .collect(),
                HarvestType::DistributeReward => restaking_vault
                    .get_distributing_reward_tokens_iter()
                    .map(|reward_token| reward_token.mint)
                    .collect(),
                HarvestType::CompoundVaultSupportedToken => Vec::new(),
            };

            Some((restaking_vault.vault, reward_token_mints))
        })() else {
            return match harvest_type {
                // fallback: 2. distributing_reward
                HarvestType::CompoundReward => {
                    self.execute_new_distribute_reward_command(ctx, None, previous_execution_result)
                }
                // fallback: 3. compounding_vault_supported_token
                HarvestType::DistributeReward => self
                    .execute_new_compound_vault_supported_token_command(
                        ctx,
                        None,
                        previous_execution_result,
                    ),
                // fallback: cmd11: stake_sol
                HarvestType::CompoundVaultSupportedToken => Ok((previous_execution_result, None)),
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

    fn create_prepare_command(
        &self,
        ctx: &OperationCommandContext,
        harvest_type: HarvestType,
        vault: Pubkey,
        reward_token_mints: Vec<Pubkey>,
    ) -> Result<Option<OperationCommandEntry>> {
        match harvest_type {
            HarvestType::CompoundReward => {
                self.create_prepare_compound_reward_command(ctx, vault, reward_token_mints)
            }
            HarvestType::DistributeReward => {
                self.create_prepare_distribute_reward_command(ctx, vault, reward_token_mints)
            }
            HarvestType::CompoundVaultSupportedToken => {
                self.create_prepare_compound_vault_supported_token_command(ctx, vault)
            }
        }
    }

    fn create_prepare_compound_reward_command(
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
            | Some(TokenPricingSource::SolvBTCVault { .. })
            | Some(TokenPricingSource::VirtualVault { .. }) => {
                let required_accounts = VaultRewardTokenAccountCandidates::find_accounts(
                    &vault,
                    &reward_token_mints[0],
                );

                let command = Self {
                    state: PrepareCompoundReward {
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

    fn create_prepare_distribute_reward_command(
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
            | Some(TokenPricingSource::SolvBTCVault { .. })
            | Some(TokenPricingSource::VirtualVault { .. }) => {
                let required_accounts = VaultRewardTokenAccountCandidates::find_accounts(
                    &vault,
                    &reward_token_mints[0],
                )
                .chain([(
                    RewardAccount::find_account_address(&ctx.receipt_token_mint.key()),
                    false,
                )]);

                let command = Self {
                    state: PrepareDistributeReward {
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

    fn create_prepare_compound_vault_supported_token_command(
        &self,
        ctx: &OperationCommandContext,
        vault: Pubkey,
    ) -> Result<Option<OperationCommandEntry>> {
        let receipt_token_pricing_source = ctx
            .fund_account
            .load()?
            .get_restaking_vault(&vault)?
            .receipt_token_pricing_source
            .try_deserialize()?;

        let entry = match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. }) => {
                let command = Self {
                    state: PrepareCompoundVaultSupportedToken { vault },
                };
                command.without_required_accounts()
            }
            Some(TokenPricingSource::VirtualVault { .. }) => return Ok(None),
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

    #[inline(never)]
    fn execute_prepare_compound_reward_command<'info>(
        &self,
        ctx: &OperationCommandContext,
        accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mints: &[Pubkey],
    ) -> ExecutionResult {
        if reward_token_mints.is_empty() {
            // fallback: next vault
            return self.execute_new_compound_reward_command(ctx, Some(vault), None);
        }

        let fund_account = ctx.fund_account.load()?;
        let receipt_token_pricing_source = fund_account
            .get_restaking_vault(vault)?
            .receipt_token_pricing_source
            .try_deserialize()?;

        let Some(entry) = (match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. }) => self
                .create_execute_compound_reward_command_from_vault_ata(
                    ctx,
                    accounts,
                    vault,
                    reward_token_mints,
                    true,
                )?,
            Some(TokenPricingSource::VirtualVault { .. }) => self
                .create_execute_compound_reward_command_from_vault_ata(
                    ctx,
                    accounts,
                    vault,
                    reward_token_mints,
                    false,
                )?,
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
        }) else {
            // fallback: next reward
            let Some(entry) = self.create_prepare_compound_reward_command(
                ctx,
                *vault,
                reward_token_mints[1..].to_vec(),
            )?
            else {
                // fallback: next vault
                return self.execute_new_compound_reward_command(ctx, Some(vault), None);
            };

            return Ok((None, Some(entry)));
        };

        Ok((None, Some(entry)))
    }

    /// Create an ExecuteCompoundReward or PrepareSwap command given that vault's ATA is the source reward token account.
    fn create_execute_compound_reward_command_from_vault_ata<'info>(
        &self,
        ctx: &OperationCommandContext,
        mut accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mints: &[Pubkey],
        is_delegate: bool,
    ) -> Result<Option<OperationCommandEntry>> {
        let VaultRewardTokenAccountCandidates {
            reward_token_mint,
            vault_reward_token_account,
        } = VaultRewardTokenAccountCandidates::pop_from(&mut accounts, &reward_token_mints[0])?;

        // Token account does not exist, so move on to next reward
        if !vault_reward_token_account.is_initialized() {
            return Ok(None);
        }

        let vault_reward_token_account_signer = if is_delegate {
            ctx.fund_account.key()
        } else {
            *vault
        };

        let reward_token_amount = self.get_reward_token_amount(
            vault_reward_token_account,
            &vault_reward_token_account_signer,
            &reward_token_mints[0],
        )?;

        let available_reward_token_amount_to_harvest = self.apply_reward_harvest_threshold(
            ctx,
            vault,
            &reward_token_mints[0],
            HarvestType::CompoundReward,
            reward_token_amount,
        )?;

        // No reward to harvest (or threshold unmet), so move on to next item
        if available_reward_token_amount_to_harvest == 0 {
            return Ok(None);
        }

        // Determine whether to swap or transfer
        let fund_account = ctx.fund_account.load()?;
        let entry = if fund_account
            .get_supported_token(reward_token_mint.key)
            .is_err()
        {
            // Need to swap reward token to one of fund's supported token
            let swap_strategy = fund_account.get_token_swap_strategy(reward_token_mint.key)?;

            let swap_source = swap_strategy.swap_source.try_deserialize()?;
            match swap_source {
                TokenSwapSource::OrcaDEXLiquidityPool { address } => {
                    let required_accounts = [
                        (address, false),                           // pool_account
                        (reward_token_mint.key(), false),           // reward_token_mint
                        (vault_reward_token_account.key(), false),  // from_reward_token_account
                        (swap_strategy.to_token_mint, false),       // supported_token_mint
                        (vault_reward_token_account_signer, false), // from_reward_token_account_signer
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
        } else {
            // Just transfer to fund supported token reserve account
            let fund_supported_token_reserve_account =
                fund_account.find_supported_token_reserve_account_address(reward_token_mint.key)?;

            let required_accounts = CommonAccounts::find_accounts(
                reward_token_mint,
                vault_reward_token_account,
                &vault_reward_token_account_signer,
            )
            .chain(CommissionAccounts::find_accounts(reward_token_mint))
            .chain([(fund_supported_token_reserve_account, true)]);

            let command = Self {
                state: ExecuteCompoundReward {
                    vault: *vault,
                    reward_token_mints: reward_token_mints.to_vec(),
                },
            };
            command.with_required_accounts(required_accounts)
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
            return self.execute_new_compound_reward_command(ctx, Some(vault), None);
        }

        let fund_account = ctx.fund_account.load()?;
        let swap_strategy = fund_account.get_token_swap_strategy(&reward_token_mints[0])?;
        let swap_source = swap_strategy.swap_source.try_deserialize()?;
        let entry = match swap_source {
            TokenSwapSource::OrcaDEXLiquidityPool { address } => {
                let [pool_account, reward_token_mint, from_reward_token_account, supported_token_mint, from_reward_token_account_signer, ..] =
                    accounts
                else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(pool_account.key(), address);
                require_keys_eq!(reward_token_mint.key(), reward_token_mints[0]);
                require_keys_eq!(supported_token_mint.key(), swap_strategy.to_token_mint);

                let fund_supported_token_reserve_account = fund_account
                    .find_supported_token_reserve_account_address(supported_token_mint.key)?;

                let required_accounts = CommonAccounts::find_accounts(
                    reward_token_mint,
                    from_reward_token_account,
                    from_reward_token_account_signer.key,
                )
                .chain(CommissionAccounts::find_accounts(reward_token_mint))
                .chain([(fund_supported_token_reserve_account, true)])
                .chain(OrcaDEXLiquidityPoolService::find_accounts_to_swap(
                    pool_account,
                    reward_token_mint,
                    supported_token_mint,
                )?);

                let command = Self {
                    state: ExecuteSwap {
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
    fn execute_prepare_distribute_reward_command<'info>(
        &self,
        ctx: &OperationCommandContext,
        accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mints: &[Pubkey],
    ) -> ExecutionResult {
        if reward_token_mints.is_empty() {
            // fallback: next vault
            return self.execute_new_distribute_reward_command(ctx, Some(vault), None);
        }

        let fund_account = ctx.fund_account.load()?;
        let receipt_token_pricing_source = fund_account
            .get_restaking_vault(vault)?
            .receipt_token_pricing_source
            .try_deserialize()?;

        let Some(entry) = (match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. }) => self
                .create_execute_distribute_reward_command_from_vault_ata(
                    ctx,
                    accounts,
                    vault,
                    reward_token_mints,
                    true,
                )?,
            Some(TokenPricingSource::VirtualVault { .. }) => self
                .create_execute_distribute_reward_command_from_vault_ata(
                    ctx,
                    accounts,
                    vault,
                    reward_token_mints,
                    false,
                )?,
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
        }) else {
            // fallback: next reward
            let Some(entry) = self.create_prepare_distribute_reward_command(
                ctx,
                *vault,
                reward_token_mints[1..].to_vec(),
            )?
            else {
                // fallback: next vault
                return self.execute_new_distribute_reward_command(ctx, Some(vault), None);
            };

            return Ok((None, Some(entry)));
        };

        Ok((None, Some(entry)))
    }

    /// Create an ExecuteDistributeReward command given that vault's ATA is the source reward token account.
    fn create_execute_distribute_reward_command_from_vault_ata<'info>(
        &self,
        ctx: &OperationCommandContext,
        mut accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mints: &[Pubkey],
        is_delegate: bool,
    ) -> Result<Option<OperationCommandEntry>> {
        let VaultRewardTokenAccountCandidates {
            reward_token_mint,
            vault_reward_token_account,
        } = VaultRewardTokenAccountCandidates::pop_from(&mut accounts, &reward_token_mints[0])?;

        let reward_account = accounts
            .get(0)
            .ok_or_else(|| error!(error::ErrorCode::AccountNotEnoughKeys))?;
        require_keys_eq!(
            reward_account.key(),
            RewardAccount::find_account_address(&ctx.receipt_token_mint.key()),
        );

        let reward_account = AccountLoader::<RewardAccount>::try_from(reward_account)?;
        let reward_account_data = reward_account.load()?;
        let reward_reserve_account = reward_account_data.get_reserve_account_address()?;
        let Some(reward_token_reserve_account) =
            reward_account_data.get_reward_token_reserve_account_address(reward_token_mint.key)?
        else {
            // Reward is not claimable, so move on to next reward
            return Ok(None);
        };

        // Token account does not exist, so move on to next reward
        if !vault_reward_token_account.is_initialized() {
            return Ok(None);
        }

        let vault_reward_token_account_signer = if is_delegate {
            ctx.fund_account.key()
        } else {
            *vault
        };

        let reward_token_amount = self.get_reward_token_amount(
            vault_reward_token_account,
            &vault_reward_token_account_signer,
            &reward_token_mints[0],
        )?;

        let available_reward_token_amount_to_harvest = self.apply_reward_harvest_threshold(
            ctx,
            vault,
            &reward_token_mints[0],
            HarvestType::DistributeReward,
            reward_token_amount,
        )?;

        // No reward to harvest (or threshold unmet), so move on to next item
        if available_reward_token_amount_to_harvest == 0 {
            return Ok(None);
        }

        let program_reward_token_revenue_account =
            anchor_spl::associated_token::get_associated_token_address_with_program_id(
                &PROGRAM_REVENUE_ADDRESS,
                reward_token_mint.key,
                reward_token_mint.owner,
            );

        let required_accounts = CommonAccounts::find_accounts(
            reward_token_mint,
            vault_reward_token_account,
            &vault_reward_token_account_signer,
        )
        .chain(CommissionAccounts::find_accounts(reward_token_mint))
        .chain([
            (reward_token_reserve_account.key(), true),
            (reward_account.key(), true),
            (reward_reserve_account, false),
            (program_reward_token_revenue_account, true),
        ]);

        let command = Self {
            state: ExecuteDistributeReward {
                vault: *vault,
                reward_token_mints: reward_token_mints.to_vec(),
            },
        };
        let entry = command.with_required_accounts(required_accounts);

        Ok(Some(entry))
    }

    #[inline(never)]
    fn execute_prepare_compound_vault_supported_token_command<'info>(
        &self,
        ctx: &OperationCommandContext,
        vault: &Pubkey,
    ) -> ExecutionResult {
        let fund_account = ctx.fund_account.load()?;
        let receipt_token_pricing_source = fund_account
            .get_restaking_vault(vault)?
            .receipt_token_pricing_source
            .try_deserialize()?;

        let Some(entry) = (match receipt_token_pricing_source {
            Some(TokenPricingSource::JitoRestakingVault { .. })
            | Some(TokenPricingSource::SolvBTCVault { .. }) => {
                let command = Self {
                    state: ExecuteCompoundVaultSupportedToken { vault: *vault },
                };
                Some(command.without_required_accounts())
            }
            Some(TokenPricingSource::VirtualVault { .. }) => None,
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
        }) else {
            // fallback: next vault
            return self.execute_new_compound_vault_supported_token_command(ctx, Some(vault), None);
        };

        Ok((None, Some(entry)))
    }

    #[inline(never)]
    fn execute_execute_compound_reward_command<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        mut accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mints: &[Pubkey],
    ) -> ExecutionResult {
        if reward_token_mints.is_empty() {
            // fallback: next vault
            return self.execute_new_compound_reward_command(ctx, Some(vault), None);
        }
        let common_accounts = CommonAccounts::pop_from(&mut accounts, &reward_token_mints[0])?;

        // check harvest threshold
        let available_reward_token_amount_to_harvest = self
            .get_available_reward_token_amount_to_harvest(
                ctx,
                &common_accounts,
                vault,
                &reward_token_mints[0],
                HarvestType::CompoundReward,
            )?;

        let result = if available_reward_token_amount_to_harvest > 0 {
            let fund_account = ctx.fund_account.load()?;
            let fund_supported_token_account_address = fund_account
                .find_supported_token_reserve_account_address(&reward_token_mints[0])?;
            let restaking_vault = fund_account.get_restaking_vault(vault)?;
            let receipt_token_pricing_source = restaking_vault
                .receipt_token_pricing_source
                .try_deserialize()?;

            let compounded_token_amount = match receipt_token_pricing_source {
                Some(TokenPricingSource::JitoRestakingVault { .. })
                | Some(TokenPricingSource::SolvBTCVault { .. }) => {
                    let deducted_amount = self.apply_commission(
                        ctx,
                        &mut accounts,
                        &common_accounts,
                        vault,
                        &fund_account.get_seeds(),
                        available_reward_token_amount_to_harvest,
                    )?;

                    self.transfer_reward(
                        &mut accounts,
                        &common_accounts,
                        &fund_account.get_seeds(),
                        &fund_supported_token_account_address,
                        available_reward_token_amount_to_harvest - deducted_amount,
                    )?
                }
                Some(TokenPricingSource::VirtualVault { .. }) => {
                    let deducted_amount = self.apply_commission(
                        ctx,
                        &mut accounts,
                        &common_accounts,
                        vault,
                        &VirtualVaultService::find_vault_address(
                            &reward_token_mints[0],
                            &ctx.fund_account.key(),
                        )
                        .get_seeds(),
                        available_reward_token_amount_to_harvest,
                    )?;

                    self.transfer_reward(
                        &mut accounts,
                        &common_accounts,
                        &VirtualVaultService::find_vault_address(
                            &restaking_vault.receipt_token_mint,
                            &ctx.fund_account.key(),
                        )
                        .get_seeds(),
                        &fund_supported_token_account_address,
                        available_reward_token_amount_to_harvest - deducted_amount,
                    )?
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

            drop(fund_account);

            if compounded_token_amount > 0 {
                let mut fund_account = ctx.fund_account.load_mut()?;

                fund_account
                    .get_supported_token_mut(&reward_token_mints[0])?
                    .token
                    .operation_reserved_amount += compounded_token_amount;
                fund_account
                    .get_restaking_vault_mut(vault)?
                    .get_compounding_reward_token_mut(&reward_token_mints[0])?
                    .last_harvested_at = Clock::get()?.unix_timestamp;

                drop(fund_account);

                // Update pricing
                FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                    .new_pricing_service(accounts.iter().copied(), true)?;

                Some(
                    HarvestRestakingYieldCommandResult {
                        vault: *vault,
                        yield_token_mint: reward_token_mints[0],
                        yield_token_amount: compounded_token_amount,
                        swapped_token_mint: None,
                        fund_supported_token_compounded_token_amount: compounded_token_amount,
                        reward_token_distributed_token_amount: 0,
                        updated_reward_account: None,
                        vault_supported_token_compounded_amount: 0,
                    }
                    .into(),
                )
            } else {
                None
            }
        } else {
            None
        };

        // move on to next item
        let Some(entry) = self.create_prepare_compound_reward_command(
            ctx,
            *vault,
            reward_token_mints[1..].to_vec(),
        )?
        else {
            // fallback: next vault
            return self.execute_new_compound_reward_command(ctx, Some(vault), result);
        };

        Ok((result, Some(entry)))
    }

    #[inline(never)]
    fn execute_execute_swap_command<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        mut accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mints: &[Pubkey],
    ) -> ExecutionResult {
        if reward_token_mints.is_empty() {
            // fallback: next vault
            return self.execute_new_compound_reward_command(ctx, Some(vault), None);
        }
        let common_accounts = CommonAccounts::pop_from(&mut accounts, &reward_token_mints[0])?;

        // check harvest threshold
        let available_reward_token_amount_to_harvest = self
            .get_available_reward_token_amount_to_harvest(
                ctx,
                &common_accounts,
                vault,
                &reward_token_mints[0],
                HarvestType::CompoundReward,
            )?;

        let result = if available_reward_token_amount_to_harvest > 0 {
            let fund_account = ctx.fund_account.load()?;
            let supported_token_mint = fund_account
                .get_token_swap_strategy(&reward_token_mints[0])?
                .to_token_mint;
            let fund_supported_token_reserve_account_address =
                fund_account.find_supported_token_reserve_account_address(&supported_token_mint)?;

            let restaking_vault = fund_account.get_restaking_vault(vault)?;
            let receipt_token_pricing_source = restaking_vault
                .receipt_token_pricing_source
                .try_deserialize()?;

            let (reward_token_amount, compounded_token_amount) = match receipt_token_pricing_source
            {
                Some(TokenPricingSource::JitoRestakingVault { .. })
                | Some(TokenPricingSource::SolvBTCVault { .. }) => {
                    let deducted_amount = self.apply_commission(
                        ctx,
                        &mut accounts,
                        &common_accounts,
                        vault,
                        &fund_account.get_seeds(),
                        available_reward_token_amount_to_harvest,
                    )?;

                    self.swap_reward(
                        ctx,
                        &mut accounts,
                        &common_accounts,
                        &reward_token_mints[0],
                        &fund_account.get_seeds(),
                        &fund_supported_token_reserve_account_address,
                        available_reward_token_amount_to_harvest - deducted_amount,
                    )?
                }
                Some(TokenPricingSource::VirtualVault { .. }) => {
                    let deducted_amount = self.apply_commission(
                        ctx,
                        &mut accounts,
                        &common_accounts,
                        vault,
                        &VirtualVaultService::find_vault_address(
                            &reward_token_mints[0],
                            &ctx.fund_account.key(),
                        )
                        .get_seeds(),
                        available_reward_token_amount_to_harvest,
                    )?;

                    self.swap_reward(
                        ctx,
                        &mut accounts,
                        &common_accounts,
                        &reward_token_mints[0],
                        &VirtualVaultService::find_vault_address(
                            &restaking_vault.receipt_token_mint,
                            &ctx.fund_account.key(),
                        )
                        .get_seeds(),
                        &fund_supported_token_reserve_account_address,
                        available_reward_token_amount_to_harvest - deducted_amount,
                    )?
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

            drop(fund_account);

            if compounded_token_amount > 0 {
                let mut fund_account = ctx.fund_account.load_mut()?;

                fund_account
                    .get_supported_token_mut(&supported_token_mint)?
                    .token
                    .operation_reserved_amount += compounded_token_amount;
                fund_account
                    .get_restaking_vault_mut(vault)?
                    .get_compounding_reward_token_mut(&reward_token_mints[0])?
                    .last_harvested_at = Clock::get()?.unix_timestamp;

                drop(fund_account);

                // Update pricing
                FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                    .new_pricing_service(accounts.iter().copied(), true)?;

                Some(
                    HarvestRestakingYieldCommandResult {
                        vault: *vault,
                        yield_token_mint: reward_token_mints[0],
                        yield_token_amount: reward_token_amount,
                        swapped_token_mint: Some(supported_token_mint),
                        fund_supported_token_compounded_token_amount: compounded_token_amount,
                        reward_token_distributed_token_amount: 0,
                        updated_reward_account: None,
                        vault_supported_token_compounded_amount: 0,
                    }
                    .into(),
                )
            } else {
                None
            }
        } else {
            None
        };

        // move on to next item
        let Some(entry) = self.create_prepare_compound_reward_command(
            ctx,
            *vault,
            reward_token_mints[1..].to_vec(),
        )?
        else {
            // fallback: next vault
            return self.execute_new_compound_reward_command(ctx, Some(vault), result);
        };

        Ok((result, Some(entry)))
    }

    #[inline(never)]
    fn execute_execute_distribute_reward_command<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        mut accounts: &[&'info AccountInfo<'info>],
        vault: &Pubkey,
        reward_token_mints: &[Pubkey],
    ) -> ExecutionResult {
        if reward_token_mints.is_empty() {
            // fallback: next vault
            return self.execute_new_distribute_reward_command(ctx, Some(vault), None);
        }
        let common_accounts = CommonAccounts::pop_from(&mut accounts, &reward_token_mints[0])?;

        // check harvest threshold
        let available_reward_token_amount_to_harvest = self
            .get_available_reward_token_amount_to_harvest(
                ctx,
                &common_accounts,
                vault,
                &reward_token_mints[0],
                HarvestType::DistributeReward,
            )?;

        let result = (|| {
            if available_reward_token_amount_to_harvest > 0 {
                // get reward related accounts in advance for validation
                let [_, _, _, _, reward_token_reserve_account, reward_account, reward_reserve_account, ..] =
                    accounts
                else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };

                let reward_account = AccountLoader::<RewardAccount>::try_from(reward_account)?;
                let reward_account_data = reward_account.load()?;

                require_keys_eq!(
                    reward_account.key(),
                    RewardAccount::find_account_address(&ctx.receipt_token_mint.key()),
                );

                require_keys_eq!(
                    reward_reserve_account.key(),
                    reward_account_data.get_reserve_account_address()?
                );

                let Some(to_reward_token_account_address) = reward_account_data
                    .get_reward_token_reserve_account_address(&reward_token_mints[0])?
                else {
                    // reward isn't claimable
                    return Ok(None);
                };

                drop(reward_account_data);

                let fund_account = ctx.fund_account.load()?;
                let restaking_vault = fund_account.get_restaking_vault(vault)?;
                let receipt_token_pricing_source = restaking_vault
                    .receipt_token_pricing_source
                    .try_deserialize()?;

                let distributed_token_amount = match receipt_token_pricing_source {
                    Some(TokenPricingSource::JitoRestakingVault { .. })
                    | Some(TokenPricingSource::SolvBTCVault { .. }) => {
                        let deducted_amount = self.apply_commission(
                            ctx,
                            &mut accounts,
                            &common_accounts,
                            vault,
                            &fund_account.get_seeds(),
                            available_reward_token_amount_to_harvest,
                        )?;

                        self.transfer_reward(
                            &mut accounts,
                            &common_accounts,
                            &fund_account.get_seeds(),
                            &to_reward_token_account_address,
                            available_reward_token_amount_to_harvest - deducted_amount,
                        )?
                    }
                    Some(TokenPricingSource::VirtualVault { .. }) => {
                        let deducted_amount = self.apply_commission(
                            ctx,
                            &mut accounts,
                            &common_accounts,
                            vault,
                            &VirtualVaultService::find_vault_address(
                                &reward_token_mints[0],
                                &ctx.fund_account.key(),
                            )
                            .get_seeds(),
                            available_reward_token_amount_to_harvest,
                        )?;

                        self.transfer_reward(
                            &mut accounts,
                            &common_accounts,
                            &VirtualVaultService::find_vault_address(
                                &restaking_vault.receipt_token_mint,
                                &ctx.fund_account.key(),
                            )
                            .get_seeds(),
                            &to_reward_token_account_address,
                            available_reward_token_amount_to_harvest - deducted_amount,
                        )?
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

                drop(fund_account);

                Ok(if distributed_token_amount > 0 {
                    let [_, _, program_reward_token_revenue_account, ..] = accounts else {
                        return err!(error::ErrorCode::AccountNotEnoughKeys)?;
                    };

                    require_keys_eq!(
                        program_reward_token_revenue_account.key(),
                        anchor_spl::associated_token::get_associated_token_address_with_program_id(
                            &PROGRAM_REVENUE_ADDRESS,
                            common_accounts.reward_token_mint.key,
                            common_accounts.reward_token_program.key,
                        )
                    );

                    let reward_reserve_account = SystemAccount::try_from(reward_reserve_account)?;
                    let program_reward_token_revenue_account =
                        InterfaceAccount::try_from(program_reward_token_revenue_account)?;
                    let reward_token_mint =
                        InterfaceAccount::try_from(common_accounts.reward_token_mint)?;
                    let reward_token_reserve_account =
                        InterfaceAccount::try_from(reward_token_reserve_account)?;
                    let reward_token_program =
                        Interface::try_from(common_accounts.reward_token_program)?;

                    let reward_service =
                        RewardService::new(ctx.receipt_token_mint, &reward_account)?;
                    reward_service.settle_reward(
                        Some(&reward_token_mint),
                        Some(&reward_token_program),
                        Some(&reward_token_reserve_account),
                        reward_token_mint.key(),
                        false,
                        distributed_token_amount,
                    )?;

                    reward_service.claim_remaining_reward(
                        &reward_token_mint,
                        &reward_token_program,
                        &reward_reserve_account,
                        &reward_token_reserve_account,
                        &program_reward_token_revenue_account,
                    )?;

                    ctx.fund_account
                        .load_mut()?
                        .get_restaking_vault_mut(vault)?
                        .get_distributing_reward_token_mut(&reward_token_mint.key())?
                        .last_harvested_at = Clock::get()?.unix_timestamp;

                    Some(
                        HarvestRestakingYieldCommandResult {
                            vault: *vault,
                            yield_token_mint: reward_token_mint.key(),
                            yield_token_amount: distributed_token_amount,
                            swapped_token_mint: None,
                            fund_supported_token_compounded_token_amount: 0,
                            reward_token_distributed_token_amount: distributed_token_amount,
                            updated_reward_account: Some(reward_account.key()),
                            vault_supported_token_compounded_amount: 0,
                        }
                        .into(),
                    )
                } else {
                    None
                })
            } else {
                Ok(None)
            }
        })()?;

        // move on to next item
        let Some(entry) = self.create_prepare_distribute_reward_command(
            ctx,
            *vault,
            reward_token_mints[1..].to_vec(),
        )?
        else {
            // fallback: next vault
            return self.execute_new_distribute_reward_command(ctx, Some(vault), result);
        };

        Ok((result, Some(entry)))
    }

    #[inline(never)]
    fn execute_execute_compound_vault_supported_token_command<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        vault: &Pubkey,
    ) -> ExecutionResult {
        let mut fund_account = ctx.fund_account.load_mut()?;
        let restaking_vault = fund_account.get_restaking_vault_mut(vault)?;
        let receipt_token_pricing_source = restaking_vault
            .receipt_token_pricing_source
            .try_deserialize()?;

        let supported_token_mint = restaking_vault.supported_token_mint;
        let (yield_token_amount, vault_supported_token_compounded_amount) =
            match receipt_token_pricing_source {
                Some(TokenPricingSource::JitoRestakingVault { .. })
                | Some(TokenPricingSource::SolvBTCVault { .. }) => {
                    let yield_token_amount = restaking_vault
                        .supported_token_to_receipt_token_exchange_ratio
                        .numerator;
                    let vault_supported_token_compounded_amount =
                        restaking_vault.supported_token_compounded_amount;

                    restaking_vault.supported_token_compounded_amount = 0;

                    (yield_token_amount, vault_supported_token_compounded_amount)
                }
                Some(TokenPricingSource::VirtualVault { .. }) => (0, 0),
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

        drop(fund_account);

        let result = if vault_supported_token_compounded_amount != 0 {
            Some(
                HarvestRestakingYieldCommandResult {
                    vault: *vault,
                    yield_token_mint: supported_token_mint,
                    yield_token_amount, // TODO: change this value to be equal to compounded amount
                    swapped_token_mint: None,
                    fund_supported_token_compounded_token_amount: 0,
                    reward_token_distributed_token_amount: 0,
                    updated_reward_account: None,
                    vault_supported_token_compounded_amount,
                }
                .into(),
            )
        } else {
            None
        };

        // next vault
        self.execute_new_compound_vault_supported_token_command(ctx, Some(vault), result)
    }

    fn get_available_reward_token_amount_to_harvest(
        &self,
        ctx: &OperationCommandContext,
        common_accounts: &CommonAccounts,
        vault: &Pubkey,
        mint: &Pubkey,
        harvest_type: HarvestType,
    ) -> Result<u64> {
        let reward_token_amount = self.get_reward_token_amount(
            common_accounts.from_reward_token_account,
            common_accounts.from_reward_token_account_signer.key,
            common_accounts.reward_token_mint.key,
        )?;

        self.apply_reward_harvest_threshold(ctx, vault, mint, harvest_type, reward_token_amount)
    }

    fn get_reward_token_amount<'info>(
        &self,
        reward_token_account: &'info AccountInfo<'info>,
        reward_token_account_signer: &Pubkey,
        reward_token_mint: &Pubkey,
    ) -> Result<u64> {
        // Validate vault reward token account
        let reward_token_account =
            InterfaceAccount::<TokenAccount>::try_from(reward_token_account)?;
        require_keys_eq!(reward_token_account.mint, reward_token_mint.key());

        let mut reward_token_amount = reward_token_account.amount;
        if reward_token_account.owner != *reward_token_account_signer {
            reward_token_amount = if reward_token_account
                .delegate
                .contains(reward_token_account_signer)
            {
                reward_token_amount.min(reward_token_account.delegated_amount)
            } else {
                // Signer is neither owner nor delegate, so move on to next reward
                0
            };
        }

        Ok(reward_token_amount)
    }

    fn apply_reward_harvest_threshold(
        &self,
        ctx: &OperationCommandContext,
        vault: &Pubkey,
        mint: &Pubkey,
        harvest_type: HarvestType,
        reward_token_amount: u64,
    ) -> Result<u64> {
        // check harvest threshold
        let current_timestamp = Clock::get()?.unix_timestamp;

        let fund_account = ctx.fund_account.load()?;
        let restaking_vault = fund_account.get_restaking_vault(vault)?;
        let reward_token = match harvest_type {
            HarvestType::CompoundReward => restaking_vault.get_compounding_reward_token(mint)?,
            HarvestType::DistributeReward => restaking_vault.get_distributing_reward_token(mint)?,
            HarvestType::CompoundVaultSupportedToken => {
                err!(ErrorCode::FundOperationCommandExecutionFailedException)?
            }
        };

        Ok(reward_token.get_available_amount_to_harvest(reward_token_amount, current_timestamp))
    }

    /// returns deducted_amount
    fn apply_commission<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        accounts: &mut &[&'info AccountInfo<'info>],
        common_accounts: &CommonAccounts<'info>,
        vault: &Pubkey,
        from_reward_token_account_signer_seeds: &[&[u8]],
        reward_token_amount: u64,
    ) -> Result<u64> {
        let commission_accounts =
            CommissionAccounts::pop_from(accounts, common_accounts.reward_token_mint)?;

        if !commission_accounts
            .program_reward_token_revenue_account
            .is_initialized()
        {
            anchor_spl::associated_token::create(CpiContext::new(
                commission_accounts
                    .associated_token_program
                    .to_account_info(),
                anchor_spl::associated_token::Create {
                    payer: ctx.operator.to_account_info(),
                    associated_token: commission_accounts
                        .program_reward_token_revenue_account
                        .to_account_info(),
                    authority: commission_accounts
                        .program_revenue_account
                        .to_account_info(),
                    mint: common_accounts.reward_token_mint.to_account_info(),
                    system_program: commission_accounts.system_program.to_account_info(),
                    token_program: common_accounts.reward_token_program.to_account_info(),
                },
            ))?;
        }

        let fund_account = ctx.fund_account.load()?;
        let restaking_vault = fund_account.get_restaking_vault(vault)?;

        if restaking_vault.reward_commission_rate_bps > 0 {
            let amount_to_deduct = crate::utils::get_proportional_amount(
                reward_token_amount,
                restaking_vault.reward_commission_rate_bps as u64,
                10_000,
            )?;

            let reward_token_mint =
                InterfaceAccount::<Mint>::try_from(common_accounts.reward_token_mint)?;

            anchor_spl::token_interface::transfer_checked(
                CpiContext::new_with_signer(
                    common_accounts.reward_token_program.to_account_info(),
                    anchor_spl::token_interface::TransferChecked {
                        from: common_accounts.from_reward_token_account.to_account_info(),
                        mint: reward_token_mint.to_account_info(),
                        to: commission_accounts
                            .program_reward_token_revenue_account
                            .to_account_info(),
                        authority: commission_accounts
                            .program_revenue_account
                            .to_account_info(),
                    },
                    &[from_reward_token_account_signer_seeds],
                ),
                amount_to_deduct,
                reward_token_mint.decimals,
            )?;

            return Ok(amount_to_deduct);
        }

        Ok(0)
    }

    /// returns transferred_token_amount
    fn transfer_reward<'info>(
        &self,
        accounts: &mut &[&'info AccountInfo<'info>],
        common_accounts: &CommonAccounts<'info>,
        from_reward_token_account_signer_seeds: &[&[u8]],
        to_reward_token_account_address: &Pubkey,

        amount_to_transfer: u64,
    ) -> Result<u64> {
        let [to_reward_token_account, remaining_accounts @ ..] = accounts else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        *accounts = remaining_accounts;

        require_keys_eq!(
            to_reward_token_account.key(),
            *to_reward_token_account_address,
        );

        let reward_token_mint =
            InterfaceAccount::<Mint>::try_from(common_accounts.reward_token_mint)?;

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                common_accounts.reward_token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: common_accounts.from_reward_token_account.to_account_info(),
                    mint: reward_token_mint.to_account_info(),
                    to: to_reward_token_account.to_account_info(),
                    authority: common_accounts
                        .from_reward_token_account_signer
                        .to_account_info(),
                },
                &[from_reward_token_account_signer_seeds],
            ),
            amount_to_transfer,
            reward_token_mint.decimals,
        )?;

        Ok(amount_to_transfer)
    }

    /// returns [reward_token_amount, swapped_token_amount]
    fn swap_reward<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        accounts: &mut &[&'info AccountInfo<'info>],
        common_accounts: &CommonAccounts<'info>,
        mint: &Pubkey,
        from_reward_token_account_signer_seeds: &[&[u8]],
        to_supported_token_account_address: &Pubkey,

        amount_to_swap: u64,
    ) -> Result<(u64, u64)> {
        let fund_account = ctx.fund_account.load()?;
        let swap_strategy = fund_account.get_token_swap_strategy(mint)?;
        let swap_source = swap_strategy.swap_source.try_deserialize()?;

        match swap_source {
            TokenSwapSource::OrcaDEXLiquidityPool { address } => {
                let [to_supported_token_account, pool_program, pool_account, token_mint_a, token_vault_a, token_program_a, token_mint_b, token_vault_b, token_program_b, memo_program, oracle, tick_array0, tick_array1, tick_array2, remaining_accounts @ ..] =
                    accounts
                else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                *accounts = remaining_accounts;

                require_keys_eq!(pool_account.key(), address);
                require_keys_eq!(
                    to_supported_token_account.key(),
                    *to_supported_token_account_address,
                );

                let (from_reward_token_swapped_amount, to_supported_token_swapped_amount) =
                    OrcaDEXLiquidityPoolService::new(
                        pool_program,
                        pool_account,
                        token_mint_a,
                        token_vault_a,
                        token_program_a,
                        token_mint_b,
                        token_vault_b,
                        token_program_b,
                    )?
                    .swap(
                        memo_program,
                        oracle,
                        tick_array0,
                        tick_array1,
                        tick_array2,
                        common_accounts.from_reward_token_account,
                        to_supported_token_account,
                        common_accounts.from_reward_token_account_signer,
                        &[from_reward_token_account_signer_seeds],
                        amount_to_swap,
                    )?;

                Ok((
                    from_reward_token_swapped_amount,
                    to_supported_token_swapped_amount,
                ))
            }
        }
    }
}

struct VaultRewardTokenAccountCandidates<'info> {
    reward_token_mint: &'info AccountInfo<'info>,
    vault_reward_token_account: &'info AccountInfo<'info>,
}

impl<'info> VaultRewardTokenAccountCandidates<'info> {
    /// Although we do not know whether the token belongs to
    /// token program or token 2022 program, we can try both ATAs.
    /// * (0) reward token mint
    /// * (1) vault reward token account (Token)
    /// * (2) vault reward token account (Token2022)
    fn find_accounts(vault: &Pubkey, mint: &Pubkey) -> impl Iterator<Item = (Pubkey, bool)> {
        [
            (*mint, false),
            (
                associated_token::get_associated_token_address_with_program_id(
                    vault,
                    mint,
                    &anchor_spl::token::ID,
                ),
                false,
            ),
            (
                associated_token::get_associated_token_address_with_program_id(
                    vault,
                    mint,
                    &anchor_spl::token_2022::ID,
                ),
                false,
            ),
        ]
        .into_iter()
    }

    fn pop_from(accounts: &mut &[&'info AccountInfo<'info>], mint: &Pubkey) -> Result<Self> {
        let [reward_token_mint, vault_reward_token_account, vault_reward_token_2022_account, remaining_accounts @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        *accounts = remaining_accounts;

        require_keys_eq!(reward_token_mint.key(), *mint);

        let vault_reward_token_account = match *reward_token_mint.owner {
            anchor_spl::token::ID => vault_reward_token_account,
            anchor_spl::token_2022::ID => vault_reward_token_2022_account,
            _ => err!(error::ErrorCode::InvalidProgramId)?,
        };

        Ok(Self {
            reward_token_mint,
            vault_reward_token_account,
        })
    }
}

/// manages commonly needed accounts in execution steps (apply commission, compound reward, swap reward, distribute reward)
struct CommonAccounts<'info> {
    reward_token_program: &'info AccountInfo<'info>,
    reward_token_mint: &'info AccountInfo<'info>,
    from_reward_token_account: &'info AccountInfo<'info>,
    from_reward_token_account_signer: &'info AccountInfo<'info>,
}

impl<'info> CommonAccounts<'info> {
    /// * (0) token program
    /// * (1) reward token mint
    /// * (2) vault reward token account
    /// * (3) vault reward token account signer
    fn find_accounts(
        reward_token_mint: &AccountInfo,
        vault_reward_token_account: &AccountInfo,
        vault_reward_token_account_signer: &Pubkey,
    ) -> impl Iterator<Item = (Pubkey, bool)> {
        let required_accounts = [
            (*reward_token_mint.owner, false),
            (reward_token_mint.key(), false),
            (vault_reward_token_account.key(), true),
            (*vault_reward_token_account_signer, false),
        ]
        .into_iter();

        required_accounts
    }

    fn pop_from(accounts: &mut &[&'info AccountInfo<'info>], mint: &Pubkey) -> Result<Self> {
        let [reward_token_program, reward_token_mint, from_reward_token_account, from_reward_token_account_signer, remaining_accounts @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        *accounts = remaining_accounts;

        require_keys_eq!(*mint, reward_token_mint.key());

        Ok(Self {
            reward_token_program,
            reward_token_mint,
            from_reward_token_account,
            from_reward_token_account_signer,
        })
    }
}

/// manages additionally needed accounts for applying commission fee
struct CommissionAccounts<'info> {
    program_revenue_account: &'info AccountInfo<'info>,
    program_reward_token_revenue_account: &'info AccountInfo<'info>,
    system_program: &'info AccountInfo<'info>,
    associated_token_program: &'info AccountInfo<'info>,
}

impl<'info> CommissionAccounts<'info> {
    /// * (0) program revenue account
    /// * (1) program revenue reward token account
    /// * (2) system program
    /// * (3) associated token program
    fn find_accounts(reward_token_mint: &AccountInfo) -> impl Iterator<Item = (Pubkey, bool)> {
        let required_accounts = [
            (PROGRAM_REVENUE_ADDRESS, false),
            (
                associated_token::get_associated_token_address_with_program_id(
                    &PROGRAM_REVENUE_ADDRESS,
                    reward_token_mint.key,
                    reward_token_mint.owner,
                ),
                true,
            ),
            (System::id(), false),
            (anchor_spl::associated_token::ID, false),
        ]
        .into_iter();

        required_accounts
    }

    fn pop_from(accounts: &mut &[&'info AccountInfo<'info>], mint: &AccountInfo) -> Result<Self> {
        let [program_revenue_account, program_reward_token_revenue_account, system_program, associated_token_program, remaining_accounts @ ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        *accounts = remaining_accounts;

        require_keys_eq!(program_revenue_account.key(), PROGRAM_REVENUE_ADDRESS);
        require_keys_eq!(
            program_reward_token_revenue_account.key(),
            anchor_spl::associated_token::get_associated_token_address_with_program_id(
                &PROGRAM_REVENUE_ADDRESS,
                mint.key,
                mint.owner,
            )
        );

        Ok(Self {
            program_revenue_account,
            program_reward_token_revenue_account,
            system_program,
            associated_token_program,
        })
    }
}
