use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::{token_2022, token_interface};

use crate::modules::fund::{DepositMetadata, FundAccount, FundService, UserFundAccount};
use crate::modules::reward::{RewardAccount, RewardService, UserRewardAccount};
use crate::utils::{AccountInfoExt, AsAccountInfo, PDASeeds};
use crate::{errors, events};

pub struct UserFundDepositService<'a, 'info> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    receipt_token_program: &'a Program<'info, Token2022>,
    fund_account: &'a mut AccountLoader<'info, FundAccount>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,

    user: &'a Signer<'info>,
    user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
    user_fund_account: &'a mut UncheckedAccount<'info>,
    user_reward_account: &'a mut UncheckedAccount<'info>,

    _current_slot: u64,
    current_timestamp: i64,
}

impl Drop for UserFundDepositService<'_, '_> {
    fn drop(&mut self) {
        self.fund_account.exit(&crate::ID).unwrap();
    }
}

impl<'a, 'info> UserFundDepositService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        receipt_token_program: &'a Program<'info, Token2022>,
        fund_account: &'a mut AccountLoader<'info, FundAccount>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
        user: &'a Signer<'info>,
        user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
        user_fund_account: &'a mut UncheckedAccount<'info>,
        user_reward_account: &'a mut UncheckedAccount<'info>,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            receipt_token_program,
            fund_account,
            reward_account,
            user,
            user_receipt_token_account,
            user_fund_account,
            user_reward_account,
            _current_slot: clock.slot,
            current_timestamp: clock.unix_timestamp,
        })
    }

    fn process_deposit_asset(
        &mut self,
        // for SOL
        system_program: Option<&Program<'info, System>>,
        fund_reserve_account: Option<&SystemAccount<'info>>,

        // for supported tokens
        supported_token_program: Option<&Interface<'info, TokenInterface>>,
        supported_token_mint: Option<&InterfaceAccount<'info, Mint>>,
        fund_supported_token_reserve_account: Option<&InterfaceAccount<'info, TokenAccount>>,
        user_supported_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,

        instructions_sysvar: &AccountInfo,
        pricing_sources: &'info [AccountInfo<'info>],

        asset_amount: u64,
        metadata: Option<DepositMetadata>,
        metadata_signer_key: &Pubkey,
    ) -> Result<events::UserDepositedToFund> {
        let supported_token_mint_key = supported_token_mint.map(|mint| mint.key());

        // validate user asset balance
        match supported_token_mint_key {
            Some(..) => {
                require_gte!(user_supported_token_account.unwrap().amount, asset_amount);
            }
            None => {
                require_gte!(self.user.lamports(), asset_amount);
            }
        }

        // validate deposit metadata
        let (wallet_provider, contribution_accrual_rate) = metadata
            .map(|metadata| {
                metadata.verify(
                    instructions_sysvar,
                    metadata_signer_key,
                    self.user.key,
                    self.current_timestamp,
                )
            })
            .transpose()?
            .unzip();

        // mint receipt token
        let mut pricing_service = FundService::new(self.receipt_token_mint, self.fund_account)?
            .new_pricing_service(pricing_sources, false)?;

        let mut deposit_residual_micro_receipt_token_amount = self
            .fund_account
            .load()?
            .deposit_residual_micro_receipt_token_amount;
        let receipt_token_mint_amount = if self.receipt_token_mint.supply == 0 {
            // receipt_token_mint_amount will be equal to asset_amount at the initial minting, so like either 1SOL = 1RECEIPT-TOKEN or 1SUPPORTED-TOKEN = 1RECEIPT-TOKEN.
            asset_amount
        } else {
            pricing_service.convert_asset_amount(
                supported_token_mint_key.as_ref(),
                asset_amount,
                Some(&self.receipt_token_mint.key()),
                &mut deposit_residual_micro_receipt_token_amount,
            )?
        };

        token_2022::mint_to(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                token_2022::MintTo {
                    mint: self.receipt_token_mint.to_account_info(),
                    to: self.user_receipt_token_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[self.fund_account.load()?.get_seeds().as_ref()],
            ),
            receipt_token_mint_amount,
        )?;

        if self.user_fund_account.is_initialized() {
            let mut user_fund_account = self
                .user_fund_account
                .as_account_info()
                .parse_account_boxed::<UserFundAccount>()?;

            user_fund_account.reload_receipt_token_amount(self.user_receipt_token_account)?;
            user_fund_account.exit(&crate::ID)?;
        }

        // increase user's reward accrual rate
        let user_reward_account_option = self
            .user_reward_account
            .as_account_info()
            .parse_optional_account_loader::<UserRewardAccount>()?;
        let updated_user_reward_accounts =
            RewardService::new(self.receipt_token_mint, self.reward_account)?
                .update_reward_pools_token_allocation(
                    None,
                    user_reward_account_option.as_ref(),
                    receipt_token_mint_amount,
                    contribution_accrual_rate,
                )?;

        // update fund state
        let deposited_amount = {
            let mut fund_account = self.fund_account.load_mut()?;
            fund_account.reload_receipt_token_supply(self.receipt_token_mint)?;
            fund_account.deposit_residual_micro_receipt_token_amount =
                deposit_residual_micro_receipt_token_amount;
            fund_account.deposit_asset(supported_token_mint_key, asset_amount)?
        };
        assert_eq!(asset_amount, deposited_amount);

        // transfer user asset to the fund
        match supported_token_mint {
            Some(supported_token_mint) => {
                token_interface::transfer_checked(
                    CpiContext::new(
                        supported_token_program.unwrap().to_account_info(),
                        token_interface::TransferChecked {
                            from: user_supported_token_account.unwrap().to_account_info(),
                            to: fund_supported_token_reserve_account
                                .unwrap()
                                .to_account_info(),
                            mint: supported_token_mint.to_account_info(),
                            authority: self.user.to_account_info(),
                        },
                    ),
                    deposited_amount,
                    supported_token_mint.decimals,
                )?;
            }
            None => {
                anchor_lang::system_program::transfer(
                    CpiContext::new(
                        system_program.unwrap().to_account_info(),
                        anchor_lang::system_program::Transfer {
                            from: self.user.to_account_info(),
                            to: fund_reserve_account.unwrap().to_account_info(),
                        },
                    ),
                    deposited_amount,
                )?;
            }
        }

        // update asset value again
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .update_asset_values(&mut pricing_service, true)?;

        Ok(events::UserDepositedToFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: self.fund_account.key(),
            supported_token_mint: supported_token_mint_key,
            updated_user_reward_accounts,

            user: self.user.key(),
            user_receipt_token_account: self.user_receipt_token_account.key(),
            // TODO: should think about if this field is needed when it's not initialized though
            user_fund_account: self.user_fund_account.key(),
            user_supported_token_account: user_supported_token_account
                .map(|token_account| token_account.key()),

            wallet_provider,
            contribution_accrual_rate,
            deposited_amount,
            minted_receipt_token_amount: receipt_token_mint_amount,
        })
    }

    pub fn process_deposit_sol(
        &mut self,
        system_program: &Program<'info, System>,
        fund_reserve_account: &SystemAccount<'info>,
        instructions_sysvar: &AccountInfo,
        pricing_sources: &'info [AccountInfo<'info>],
        sol_amount: u64,
        metadata: Option<DepositMetadata>,
        metadata_signer_key: &Pubkey,
    ) -> Result<events::UserDepositedToFund> {
        self.process_deposit_asset(
            Some(system_program),
            Some(fund_reserve_account),
            None,
            None,
            None,
            None,
            instructions_sysvar,
            pricing_sources,
            sol_amount,
            metadata,
            metadata_signer_key,
        )
    }

    pub fn process_deposit_supported_token(
        &mut self,
        supported_token_program: &Interface<'info, TokenInterface>,
        supported_token_mint: &InterfaceAccount<'info, Mint>,
        fund_supported_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,
        user_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        instructions_sysvar: &AccountInfo,
        pricing_sources: &'info [AccountInfo<'info>],
        supported_token_amount: u64,
        metadata: Option<DepositMetadata>,
        metadata_signer_key: &Pubkey,
    ) -> Result<events::UserDepositedToFund> {
        self.process_deposit_asset(
            None,
            None,
            Some(supported_token_program),
            Some(supported_token_mint),
            Some(fund_supported_token_reserve_account),
            Some(user_supported_token_account),
            instructions_sysvar,
            pricing_sources,
            supported_token_amount,
            metadata,
            metadata_signer_key,
        )
    }

    pub fn process_deposit_vault_receipt_token(
        &mut self,
        vault_receipt_token_program: &Program<'info, Token>,
        vault_receipt_token_mint: &InterfaceAccount<'info, Mint>,
        fund_vault_receipt_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,
        user_vault_receipt_token_account: &InterfaceAccount<'info, TokenAccount>,
        instructions_sysvar: &AccountInfo,
        pricing_sources: &'info [AccountInfo<'info>],
        metadata: Option<DepositMetadata>,
        metadata_signer_key: &Pubkey,
    ) -> Result<events::UserDepositedToVault> {
        // validate deposit metadata
        let (wallet_provider, contribution_accrual_rate) = metadata
            .map(|metadata| {
                metadata.verify(
                    instructions_sysvar,
                    metadata_signer_key,
                    self.user.key,
                    self.current_timestamp,
                )
            })
            .transpose()?
            .unzip();

        // mint receipt token
        let mut pricing_service = FundService::new(self.receipt_token_mint, self.fund_account)?
            .new_pricing_service(pricing_sources, false)?;

        let mut deposit_residual_micro_receipt_token_amount = self
            .fund_account
            .load()?
            .deposit_residual_micro_receipt_token_amount;
        let receipt_token_mint_amount = if self.receipt_token_mint.supply == 0 {
            user_vault_receipt_token_account.amount
        } else {
            pricing_service.convert_asset_amount(
                Some(&vault_receipt_token_mint.key()),
                user_vault_receipt_token_account.amount,
                Some(&self.receipt_token_mint.key()),
                &mut deposit_residual_micro_receipt_token_amount,
            )?
        };

        token_2022::mint_to(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                token_2022::MintTo {
                    mint: self.receipt_token_mint.to_account_info(),
                    to: self.user_receipt_token_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[self.fund_account.load()?.get_seeds().as_ref()],
            ),
            receipt_token_mint_amount,
        )?;

        if self.user_fund_account.is_initialized() {
            let mut user_fund_account = self
                .user_fund_account
                .as_account_info()
                .parse_account_boxed::<UserFundAccount>()?;

            user_fund_account.reload_receipt_token_amount(self.user_receipt_token_account)?;
            user_fund_account.exit(&crate::ID)?;
        }

        // increase user's reward accrual rate
        let user_reward_account_option = self
            .user_reward_account
            .as_account_info()
            .parse_optional_account_loader::<UserRewardAccount>()?;
        let updated_user_reward_accounts =
            RewardService::new(self.receipt_token_mint, self.reward_account)?
                .update_reward_pools_token_allocation(
                    None,
                    user_reward_account_option.as_ref(),
                    receipt_token_mint_amount,
                    contribution_accrual_rate,
                )?;

        // update fund state
        let deposited_amount = {
            let mut fund_account = self.fund_account.load_mut()?;
            fund_account.reload_receipt_token_supply(self.receipt_token_mint)?;
            fund_account.deposit_residual_micro_receipt_token_amount =
                deposit_residual_micro_receipt_token_amount;
            fund_account.deposit_vault_receipt_token(
                &vault_receipt_token_mint.key(),
                user_vault_receipt_token_account.amount,
            )?
        };
        assert_eq!(user_vault_receipt_token_account.amount, deposited_amount);

        // transfer user asset to the fund
        token_interface::transfer_checked(
            CpiContext::new(
                vault_receipt_token_program.to_account_info(),
                token_interface::TransferChecked {
                    from: user_vault_receipt_token_account.to_account_info(),
                    to: fund_vault_receipt_token_reserve_account.to_account_info(),
                    mint: vault_receipt_token_mint.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            deposited_amount,
            vault_receipt_token_mint.decimals,
        )?;

        // update asset value again
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .update_asset_values(&mut pricing_service, true)?;

        let vault_key = self
            .fund_account
            .load()?
            .get_restaking_vaults_iter()
            .find(|restaking_vault| {
                restaking_vault.receipt_token_mint == vault_receipt_token_mint.key()
            })
            .ok_or_else(|| error!(errors::ErrorCode::FundRestakingVaultNotFoundError))?
            .vault;

        Ok(events::UserDepositedToVault {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: self.fund_account.key(),
            vault_account: vault_key,
            vault_receipt_token_mint: vault_receipt_token_mint.key(),
            updated_user_reward_accounts,
            user: self.user.key(),
            user_receipt_token_account: self.user_receipt_token_account.key(),
            user_fund_account: self.user_fund_account.key(),
            user_vault_receipt_token_account: user_vault_receipt_token_account.key(),
            wallet_provider,
            contribution_accrual_rate,
            deposited_amount,
            minted_receipt_token_amount: receipt_token_mint_amount,
        })
    }
}
