use anchor_lang::prelude::*;
use anchor_spl::associated_token;
use anchor_spl::token_interface::TokenAccount;

use crate::errors::ErrorCode;
use crate::modules::swap::{OrcaDEXLiquidityPoolService, TokenSwapSource};
use crate::utils::{AccountInfoExt, PDASeeds};

use super::{
    FundService, OperationCommandContext, OperationCommandEntry, OperationCommandResult,
    SelfExecutable, UnstakeLSTCommand, FUND_ACCOUNT_MAX_RESTAKING_VAULTS,
    FUND_ACCOUNT_RESTAKING_VAULT_MAX_COMPOUNDING_REWARD_TOKENS,
};

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct HarvestRewardCommand {
    state: HarvestRewardCommandState,
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub enum HarvestRewardCommandState {
    /// Initializes a command with items based on the fund state and strategy.
    #[default]
    New,
    /// Before harvest, find reward tokens to compound or distribute.
    Prepare {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        vaults: Vec<Pubkey>,
        // TODO distribute
        // + FUND_ACCOUNT_RESTAKING_VAULT_MAX_DISTRIBUTING_REWARD_TOKENS
        #[max_len(FUND_ACCOUNT_RESTAKING_VAULT_MAX_COMPOUNDING_REWARD_TOKENS)]
        items: Vec<HarvestRewardCommandItem>,
    },
    /// Before swap, find required accounts. Swap needs a number of accounts.
    PrepareSwap {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        vaults: Vec<Pubkey>,
        // TODO distribute
        // + FUND_ACCOUNT_RESTAKING_VAULT_MAX_DISTRIBUTING_REWARD_TOKENS
        #[max_len(FUND_ACCOUNT_RESTAKING_VAULT_MAX_COMPOUNDING_REWARD_TOKENS)]
        items: Vec<HarvestRewardCommandItem>,
    },
    /// Executes swap, transfer, or settle. Destination token account of swapped
    /// token is fund supported token reserve account, so transfer is not needed.
    Execute {
        #[max_len(FUND_ACCOUNT_MAX_RESTAKING_VAULTS)]
        vaults: Vec<Pubkey>,
        // TODO distribute
        // + FUND_ACCOUNT_RESTAKING_VAULT_MAX_DISTRIBUTING_REWARD_TOKENS
        #[max_len(FUND_ACCOUNT_RESTAKING_VAULT_MAX_COMPOUNDING_REWARD_TOKENS)]
        items: Vec<HarvestRewardCommandItem>,
    },
}

impl std::fmt::Debug for HarvestRewardCommandState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn debug_vault_and_item(
            f: &mut std::fmt::Formatter,
            variant: &'static str,
            vaults: &[Pubkey],
            items: &[HarvestRewardCommandItem],
        ) -> std::fmt::Result {
            if vaults.is_empty() {
                return f.write_str(variant);
            }
            let mut f = f.debug_struct(variant);
            f.field("vault", &vaults[0]);
            if !items.is_empty() {
                f.field("item", &items[0]);
            }
            f.finish()
        }

        match self {
            Self::New => f.write_str("New"),
            Self::Prepare { vaults, items } => debug_vault_and_item(f, "Prepare", vaults, items),
            Self::PrepareSwap { vaults, items } => {
                debug_vault_and_item(f, "PrepareSwap", vaults, items)
            }
            Self::Execute { vaults, items } => debug_vault_and_item(f, "Execute", vaults, items),
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub struct HarvestRewardCommandItem {
    reward_token_mint: Pubkey,
    harvest_type: HarvestType,
}

impl std::fmt::Debug for HarvestRewardCommandItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({:?})", self.reward_token_mint, self.harvest_type)
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, PartialEq, Copy)]
pub enum HarvestType {
    Swap(Pubkey),
    Transfer,
    // TODO distribute
    // Settle,
}

impl std::fmt::Debug for HarvestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Swap(_) => f.write_str("Swap"),
            Self::Transfer => f.write_str("Transfer"),
            // TODO distribute
            // Self::Settle => f.write_str("Settle"),
        }
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct HarvestRewardCommandResult {
    pub reward_token_mint: Pubkey,
    pub reward_token_amount: u64,
    pub swapped_token_mint: Option<Pubkey>,
    pub distributed_token_amount: u64,
    pub compounded_token_amount: u64,
}

impl SelfExecutable for HarvestRewardCommand {
    fn execute<'a, 'info: 'a>(
        &self,
        ctx: &mut OperationCommandContext<'info, 'a>,
        accounts: &[&'info AccountInfo<'info>],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let (result, entry) = match &self.state {
            HarvestRewardCommandState::New => self.execute_new(ctx)?,
            HarvestRewardCommandState::Prepare { vaults, items } => {
                self.execute_prepare(ctx, accounts, vaults, items)?
            }
            HarvestRewardCommandState::PrepareSwap { vaults, items } => {
                self.execute_prepare_swap(ctx, accounts, vaults, items)?
            }
            HarvestRewardCommandState::Execute { vaults, items } => {
                self.execute_execute(ctx, accounts, vaults, items)?
            }
        };

        Ok((
            result,
            entry.or_else(|| Some(UnstakeLSTCommand::default().without_required_accounts())),
        ))
    }
}

#[deny(clippy::wildcard_enum_match_arm)]
impl HarvestRewardCommand {
    #[inline(never)]
    fn execute_new<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        let fund_account = ctx.fund_account.load()?;
        let vaults = fund_account
            .get_restaking_vaults_iter()
            .map(|vault| vault.vault);

        Ok((None, self.create_prepare_command(ctx, vaults)?))
    }

    fn create_prepare_command<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        vaults: impl Iterator<Item = Pubkey>,
    ) -> Result<Option<OperationCommandEntry>> {
        let (vaults, items) = self.find_command_items(ctx, vaults)?;
        if vaults.is_empty() || items.is_empty() {
            return Ok(None);
        }

        return Ok(self.create_prepare_command_with_items(vaults, items));
    }

    fn find_command_items<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        vaults: impl Iterator<Item = Pubkey>,
    ) -> Result<(Vec<Pubkey>, Vec<HarvestRewardCommandItem>)> {
        let fund_account = ctx.fund_account.load()?;
        let mut vaults = vaults.peekable();
        while let Some(vault) = vaults.peek() {
            let restaking_vault = fund_account.get_restaking_vault(vault)?;
            let items = {
                // TODO distribute
                // + restaking_vault.get_distributing_reward_tokens_iter().count();
                let num_reward_tokens =
                    restaking_vault.get_compounding_reward_tokens_iter().count();
                let mut items = Vec::with_capacity(num_reward_tokens);
                restaking_vault
                    .get_compounding_reward_tokens_iter()
                    .try_for_each(|reward_token_mint| {
                        let harvest_type =
                            if fund_account.get_supported_token(reward_token_mint).is_ok() {
                                HarvestType::Transfer
                            } else {
                                let swap_strategy =
                                    fund_account.get_token_swap_strategy(reward_token_mint)?;
                                HarvestType::Swap(swap_strategy.to_token_mint)
                            };
                        items.push(HarvestRewardCommandItem {
                            reward_token_mint: *reward_token_mint,
                            harvest_type,
                        });
                        Ok::<_, Error>(())
                    })?;
                // TODO distribute
                // items.extend(restaking_vault.get_distributing_reward_tokens_iter().map(
                //     |reward_token_mint| HarvestRewardCommandItem {
                //         reward_token_mint: *reward_token_mint,
                //         harvest_type: HarvestType::Settle,
                //     },
                // ));
                items
            };

            if !items.is_empty() {
                return Ok((vaults.collect(), items));
            }

            // this vault does not have rewards, so proceed to next vault
            vaults.next();
        }

        // none of the vaults has rewards
        Ok((vec![], vec![]))
    }

    fn create_prepare_command_with_items<'info>(
        &self,
        vaults: Vec<Pubkey>,
        items: Vec<HarvestRewardCommandItem>,
    ) -> Option<OperationCommandEntry> {
        if vaults.is_empty() || items.is_empty() {
            return None;
        }
        let vault = &vaults[0];
        let item = &items[0];

        // We need to check vault's token account whether
        // the account is delegated to fund account or not.
        // Although we do not know whether the token
        // belongs to token program or token 2022 program,
        // we can try both ATAs.
        let required_accounts = [
            (item.reward_token_mint, false),
            (
                associated_token::get_associated_token_address_with_program_id(
                    vault,
                    &item.reward_token_mint,
                    &anchor_spl::token::ID,
                ),
                false,
            ),
            (
                associated_token::get_associated_token_address_with_program_id(
                    vault,
                    &item.reward_token_mint,
                    &anchor_spl::token_2022::ID,
                ),
                false,
            ),
        ];
        let command = Self {
            state: HarvestRewardCommandState::Prepare { vaults, items },
        };

        return Some(command.with_required_accounts(required_accounts));
    }

    #[inline(never)]
    fn execute_prepare<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        vaults: &[Pubkey],
        items: &[HarvestRewardCommandItem],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if vaults.is_empty() {
            return Ok((None, None));
        }
        if items.is_empty() {
            let vaults = vaults[1..].iter().copied();
            return Ok((None, self.create_prepare_command(ctx, vaults)?));
        }
        let vault = &vaults[0];
        let item = &items[0];

        let [reward_token_mint, vault_reward_token_account, vault_reward_token_2022_account, ..] =
            accounts
        else {
            err!(error::ErrorCode::AccountNotEnoughKeys)?
        };
        let reward_token_program = reward_token_mint.owner;

        let Some(vault_reward_token_account) = (|| {
            let vault_reward_token_account = match reward_token_program {
                &anchor_spl::token::ID => vault_reward_token_account,
                &anchor_spl::token_2022::ID => vault_reward_token_2022_account,
                _ => err!(error::ErrorCode::InvalidProgramId)?,
            };

            // Token account does not exist, so skip
            if !vault_reward_token_account.is_initialized() {
                return Ok(None);
            }
            require_keys_eq!(*vault_reward_token_account.owner, *reward_token_program);

            let vault_reward_token_account =
                { InterfaceAccount::<TokenAccount>::try_from(vault_reward_token_account)? };
            require_keys_eq!(vault_reward_token_account.mint, reward_token_mint.key());
            require_keys_eq!(vault_reward_token_account.owner, *vault);

            let reward_token_amount = vault_reward_token_account
                .amount
                .min(vault_reward_token_account.delegated_amount);

            // No reward, so skip
            if reward_token_amount == 0 {
                return Ok(None);
            }

            // Token account must be delegated to fund account, otherwise skip and harvest manually
            Ok(vault_reward_token_account
                .delegate
                .contains(&ctx.fund_account.key())
                .then_some(vault_reward_token_account))
        })()?
        else {
            let items = &items[1..];
            let entry = if items.is_empty() {
                self.create_prepare_command(ctx, vaults[1..].iter().copied())?
            } else {
                self.create_prepare_command_with_items(vaults.to_vec(), items.to_vec())
            };
            return Ok((None, entry));
        };

        // Prepare based on harvest type
        let entry = match &item.harvest_type {
            HarvestType::Swap(supported_token_mint) => self
                .create_prepare_swap_command_with_items(
                    ctx,
                    &item.reward_token_mint,
                    supported_token_mint,
                    vaults.to_vec(),
                    items.to_vec(),
                )?,
            HarvestType::Transfer => self.create_execute_transfer_command_with_items(
                ctx,
                &item.reward_token_mint,
                vault_reward_token_account.as_ref(),
                vaults.to_vec(),
                items.to_vec(),
            )?,
            // TODO distribute
            // HarvestType::Settle => unimplemented!("distributing reward is not implemented yet"),
        };

        Ok((None, Some(entry)))
    }

    fn create_prepare_swap_command_with_items<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        reward_token_mint: &Pubkey,
        supported_token_mint: &Pubkey,
        // Ingredients for creating command
        vaults: Vec<Pubkey>,
        items: Vec<HarvestRewardCommandItem>,
    ) -> Result<OperationCommandEntry> {
        let fund_account = ctx.fund_account.load()?;
        let swap_strategy = fund_account.get_token_swap_strategy(reward_token_mint)?;

        let command = Self {
            state: HarvestRewardCommandState::PrepareSwap { vaults, items },
        };
        let entry = match swap_strategy.swap_source.try_deserialize()? {
            TokenSwapSource::OrcaDEXLiquidityPool { address } => command.with_required_accounts([
                (address, false),
                (*reward_token_mint, false),
                (*supported_token_mint, false),
            ]),
        };

        Ok(entry)
    }

    fn create_execute_transfer_command_with_items<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        reward_token_mint: &Pubkey,
        vault_reward_token_account: &AccountInfo,
        // Ingredients for creating command
        vaults: Vec<Pubkey>,
        items: Vec<HarvestRewardCommandItem>,
    ) -> Result<OperationCommandEntry> {
        let fund_supported_token_reserve_account = ctx
            .fund_account
            .load()?
            .find_supported_token_reserve_account_address(reward_token_mint)?;
        let required_accounts = [
            (*reward_token_mint, false),
            (vault_reward_token_account.key(), true),
            (fund_supported_token_reserve_account, true),
            (*vault_reward_token_account.owner, false), // token program
        ];
        let entry = Self {
            state: HarvestRewardCommandState::Execute { vaults, items },
        }
        .with_required_accounts(required_accounts);

        Ok(entry)
    }

    #[inline(never)]
    fn execute_prepare_swap<'info>(
        &self,
        ctx: &OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        vaults: &[Pubkey],
        items: &[HarvestRewardCommandItem],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if vaults.is_empty() {
            return Ok((None, None));
        }
        if items.is_empty() {
            let vaults = vaults[1..].iter().copied();
            return Ok((None, self.create_prepare_command(ctx, vaults)?));
        }
        let vault = &vaults[0];
        let item = &items[0];

        let fund_account = ctx.fund_account.load()?;
        let swap_strategy = fund_account.get_token_swap_strategy(&item.reward_token_mint)?;
        match swap_strategy.swap_source.try_deserialize()? {
            TokenSwapSource::OrcaDEXLiquidityPool { address } => {
                let [pool_account, reward_token_mint, supported_token_mint, ..] = accounts else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(pool_account.key(), address);
                require_keys_eq!(reward_token_mint.key(), item.reward_token_mint);
                require_keys_eq!(supported_token_mint.key(), {
                    let HarvestType::Swap(supported_token_mint) = &item.harvest_type else {
                        err!(ErrorCode::FundOperationCommandExecutionFailedException)?
                    };
                    *supported_token_mint
                });

                let accounts_to_swap = OrcaDEXLiquidityPoolService::find_accounts_to_swap(
                    pool_account,
                    reward_token_mint,
                    supported_token_mint,
                )?;
                let vault_reward_token_account =
                    associated_token::get_associated_token_address_with_program_id(
                        vault,
                        reward_token_mint.key,
                        reward_token_mint.owner,
                    );
                let fund_supported_token_reserve_account = fund_account
                    .find_supported_token_reserve_account_address(supported_token_mint.key)?;

                let required_accounts = accounts_to_swap.chain([
                    (vault_reward_token_account, true),
                    (fund_supported_token_reserve_account, true),
                ]);
                let entry = Self {
                    state: HarvestRewardCommandState::Execute {
                        vaults: vaults.to_vec(),
                        items: items.to_vec(),
                    },
                }
                .with_required_accounts(required_accounts);

                Ok((None, Some(entry)))
            }
        }
    }

    #[inline(never)]
    fn execute_execute<'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &[&'info AccountInfo<'info>],
        vaults: &[Pubkey],
        items: &[HarvestRewardCommandItem],
    ) -> Result<(
        Option<OperationCommandResult>,
        Option<OperationCommandEntry>,
    )> {
        if vaults.is_empty() {
            return Ok((None, None));
        }
        if items.is_empty() {
            let vaults = vaults[1..].iter().copied().peekable();
            return Ok((None, self.create_prepare_command(ctx, vaults)?));
        }
        let vault = &vaults[0];
        let item = &items[0];

        let (result, pricing_sources) = match &item.harvest_type {
            HarvestType::Swap(supported_token_mint) => {
                self.execute_swap(ctx, accounts, vault, item, supported_token_mint)?
            }
            HarvestType::Transfer => self.execute_transfer(ctx, accounts, vault, item)?,
            // TODO distribute
            // HarvestType::Settle => unimplemented!("Distributing reward is not implemented yet"),
        }
        .unzip();

        // Update pricing
        if let Some(pricing_sources) = pricing_sources {
            FundService::new(ctx.receipt_token_mint, ctx.fund_account)?
                .new_pricing_service(pricing_sources.iter().copied())?;
        }

        let items = &items[1..];
        let entry = if items.is_empty() {
            self.create_prepare_command(ctx, vaults[1..].iter().copied())?
        } else {
            self.create_prepare_command_with_items(vaults.to_vec(), items.to_vec())
        };

        Ok((result, entry))
    }

    fn execute_swap<'a, 'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &'a [&'info AccountInfo<'info>],
        vault: &Pubkey,
        item: &HarvestRewardCommandItem,
        supported_token_mint: &Pubkey,
    ) -> Result<Option<(OperationCommandResult, &'a [&'info AccountInfo<'info>])>> {
        let fund_account = ctx.fund_account.load()?;
        let swap_strategy = fund_account.get_token_swap_strategy(&item.reward_token_mint)?;

        let (
            harvested_reward_token_amount,
            swapped_supported_token_amount,
            fund_supported_token_reserve_account_amount,
            pricing_sources,
        ) = match swap_strategy.swap_source.try_deserialize()? {
            TokenSwapSource::OrcaDEXLiquidityPool { address } => {
                let [pool_program, pool_account, token_mint_a, token_vault_a, token_program_a, token_mint_b, token_vault_b, token_program_b, memo_program, oracle, tick_array0, tick_array1, tick_array2, vault_reward_token_account, fund_supported_token_reserve_account, pricing_sources @ ..] =
                    accounts
                else {
                    err!(error::ErrorCode::AccountNotEnoughKeys)?
                };
                require_keys_eq!(pool_account.key(), address);
                require_keys_eq!(
                    vault_reward_token_account.key(),
                    associated_token::get_associated_token_address_with_program_id(
                        vault,
                        &item.reward_token_mint,
                        vault_reward_token_account.owner,
                    )
                );
                require_keys_eq!(
                    fund_supported_token_reserve_account.key(),
                    fund_account
                        .find_supported_token_reserve_account_address(supported_token_mint)?,
                );

                let reward_token_amount = {
                    let vault_reward_token_account =
                        InterfaceAccount::<TokenAccount>::try_from(vault_reward_token_account)?;

                    require_keys_eq!(vault_reward_token_account.mint, item.reward_token_mint);
                    require_keys_eq!(vault_reward_token_account.owner, *vault);

                    vault_reward_token_account
                        .amount
                        .min(vault_reward_token_account.delegated_amount)
                };

                let orca_dex_liquidity_pool_service = OrcaDEXLiquidityPoolService::new(
                    pool_program,
                    pool_account,
                    token_mint_a,
                    token_vault_a,
                    token_program_a,
                    token_mint_b,
                    token_vault_b,
                    token_program_b,
                )?;

                let (to_token_account_amount, from_token_swapped_amount, to_token_swapped_amount) =
                    orca_dex_liquidity_pool_service.swap(
                        memo_program,
                        oracle,
                        tick_array0,
                        tick_array1,
                        tick_array2,
                        vault_reward_token_account,
                        fund_supported_token_reserve_account,
                        ctx.fund_account.as_ref(),
                        &[&fund_account.get_seeds()],
                        reward_token_amount,
                    )?;

                (
                    from_token_swapped_amount,
                    to_token_swapped_amount,
                    to_token_account_amount,
                    pricing_sources,
                )
            }
        };

        if harvested_reward_token_amount == 0 {
            return Ok(None);
        }

        // Update fund account
        drop(fund_account);
        let mut fund_account = ctx.fund_account.load_mut()?;
        let supported_token = fund_account.get_supported_token_mut(supported_token_mint)?;
        supported_token.token.operation_reserved_amount += swapped_supported_token_amount;

        require_gte!(
            fund_supported_token_reserve_account_amount,
            supported_token.token.get_total_reserved_amount(),
        );

        Ok(Some((
            HarvestRewardCommandResult {
                reward_token_mint: item.reward_token_mint,
                reward_token_amount: harvested_reward_token_amount,
                swapped_token_mint: Some(*supported_token_mint),
                distributed_token_amount: 0,
                compounded_token_amount: swapped_supported_token_amount,
            }
            .into(),
            pricing_sources,
        )))
    }

    fn execute_transfer<'a, 'info>(
        &self,
        ctx: &mut OperationCommandContext<'info, '_>,
        accounts: &'a [&'info AccountInfo<'info>],
        vault: &Pubkey,
        item: &HarvestRewardCommandItem,
    ) -> Result<Option<(OperationCommandResult, &'a [&'info AccountInfo<'info>])>> {
        let fund_account = ctx.fund_account.load()?;
        let supported_token = fund_account.get_supported_token(&item.reward_token_mint)?;

        let [reward_token_mint, vault_reward_token_account, fund_supported_token_reserve_account, reward_token_program, pricing_sources @ ..] =
            accounts
        else {
            err!(ErrorCode::FundOperationCommandExecutionFailedException)?
        };
        require_keys_eq!(reward_token_mint.key(), item.reward_token_mint);
        require_keys_eq!(reward_token_mint.key(), supported_token.mint);
        require_keys_eq!(
            vault_reward_token_account.key(),
            associated_token::get_associated_token_address_with_program_id(
                vault,
                &item.reward_token_mint,
                &supported_token.program,
            )
        );
        require_keys_eq!(
            fund_supported_token_reserve_account.key(),
            fund_account.find_supported_token_reserve_account_address(&item.reward_token_mint)?,
        );
        require_keys_eq!(reward_token_program.key(), supported_token.program);

        let reward_token_amount = {
            let vault_reward_token_account =
                InterfaceAccount::<TokenAccount>::try_from(vault_reward_token_account)?;

            require_keys_eq!(vault_reward_token_account.mint, item.reward_token_mint);
            require_keys_eq!(vault_reward_token_account.owner, *vault);

            vault_reward_token_account
                .amount
                .min(vault_reward_token_account.delegated_amount)
        };

        if reward_token_amount == 0 {
            return Ok(None);
        }

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                reward_token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: vault_reward_token_account.to_account_info(),
                    mint: reward_token_mint.to_account_info(),
                    to: fund_supported_token_reserve_account.to_account_info(),
                    authority: ctx.fund_account.to_account_info(),
                },
                &[&fund_account.get_seeds()],
            ),
            reward_token_amount,
            supported_token.decimals,
        )?;

        // Update fund account
        drop(fund_account);
        let mut fund_account = ctx.fund_account.load_mut()?;
        let supported_token = fund_account.get_supported_token_mut(&item.reward_token_mint)?;
        supported_token.token.operation_reserved_amount += reward_token_amount;

        require_gte!(
            InterfaceAccount::<TokenAccount>::try_from(fund_supported_token_reserve_account)?
                .amount,
            supported_token.token.get_total_reserved_amount(),
        );

        Ok(Some((
            HarvestRewardCommandResult {
                reward_token_mint: item.reward_token_mint,
                reward_token_amount,
                swapped_token_mint: None,
                distributed_token_amount: 0,
                compounded_token_amount: reward_token_amount,
            }
            .into(),
            pricing_sources,
        )))
    }
}
