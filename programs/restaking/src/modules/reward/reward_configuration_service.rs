use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::errors;
use crate::events;
use crate::utils::{AccountLoaderExt, SystemProgramExt};

use super::*;

pub struct RewardConfigurationService<'a, 'info> {
    receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,

    current_slot: u64,
}

impl Drop for RewardConfigurationService<'_, '_> {
    fn drop(&mut self) {
        self.reward_account.exit(&crate::ID).unwrap();
    }
}

impl<'a, 'info> RewardConfigurationService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a InterfaceAccount<'info, Mint>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            reward_account,
            current_slot: clock.slot,
        })
    }

    pub fn process_initialize_reward_account(&mut self, reward_account_bump: u8) -> Result<()> {
        if self.reward_account.as_ref().data_len() < 8 + std::mem::size_of::<RewardAccount>() {
            self.reward_account
                .initialize_zero_copy_header(reward_account_bump)
        } else {
            self.reward_account
                .load_init()?
                .initialize(reward_account_bump, self.receipt_token_mint.key())?;
            self.reward_account.exit(&crate::ID)
        }
    }

    pub fn process_update_reward_account_if_needed(
        &self,
        payer: &Signer<'info>,
        system_program: &Program<'info, System>,
        desired_account_size: Option<u32>,
    ) -> Result<()> {
        let min_account_size = 8 + std::mem::size_of::<RewardAccount>();
        let target_account_size = desired_account_size
            .map(|size| std::cmp::max(size as usize, min_account_size))
            .unwrap_or(min_account_size);

        let new_account_size = system_program.expand_account_size_if_needed(
            self.reward_account.as_ref(),
            payer,
            &[],
            target_account_size,
            None,
        )?;

        if new_account_size >= min_account_size {
            self.reward_account
                .load_mut()?
                .update_if_needed(self.receipt_token_mint.key())?;
        }

        Ok(())
    }

    pub fn process_add_reward_pool(
        &self,
        name: String,
        custom_contribution_accrual_rate_enabled: bool,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        self.reward_account.load_mut()?.add_reward_pool(
            name,
            custom_contribution_accrual_rate_enabled,
            self.current_slot,
        )?;

        self.create_fund_manager_updated_reward_pool_event()
    }

    pub fn process_add_reward(
        &self,
        reward_token_mint: Option<&InterfaceAccount<'info, Mint>>,
        reward_token_program: Option<&Interface<'info, TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<'info, TokenAccount>>,

        name: String,
        description: String,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        claimable: bool,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        if claimable {
            // Constraint check
            let reward_token_mint =
                reward_token_mint.ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
            let reward_token_program =
                reward_token_program.ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
            let reward_token_reserve_account = reward_token_reserve_account
                .ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;

            require_keys_eq!(reward_token_reserve_account.mint, mint);
            require_keys_eq!(reward_token_program.key(), program);
            require_eq!(reward_token_mint.decimals, decimals);
        }

        self.reward_account.load_mut()?.add_reward(
            name,
            description,
            mint,
            program,
            decimals,
            claimable,
        )?;

        self.create_fund_manager_updated_reward_pool_event()
    }

    pub fn process_update_reward(
        &self,
        reward_token_mint: Option<&InterfaceAccount<'info, Mint>>,
        reward_token_program: Option<&Interface<'info, TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<'info, TokenAccount>>,
        from_reward_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,
        from_reward_token_account_signer: Option<&Signer<'info>>,

        reward_id: u16,
        mint: Option<Pubkey>,
        program: Option<Pubkey>,
        decimals: Option<u8>,
        claimable: bool,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        let mut reward_account = self.reward_account.load_mut()?;
        let total_unclaimed_amount = reward_account.get_total_reward_unclaimed_amount(reward_id);
        let reward = reward_account.get_reward_mut(reward_id)?;

        require_eq!(
            reward.claimable,
            0,
            errors::ErrorCode::RewardAlreadyClaimableError
        );

        let mint = mint.unwrap_or(reward.mint);
        let program = program.unwrap_or(reward.program);
        let decimals = decimals.unwrap_or(reward.decimals);

        reward
            .set_claimable(claimable)
            .set_reward_token(mint, program, decimals);

        drop(reward_account);

        if claimable {
            let reward_token_mint =
                reward_token_mint.ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
            let reward_token_program =
                reward_token_program.ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
            let reward_token_reserve_account = reward_token_reserve_account
                .ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;

            // Constraint check
            require_keys_eq!(reward_token_reserve_account.mint, mint);
            require_keys_eq!(reward_token_program.key(), program);
            require_eq!(reward_token_mint.decimals, decimals);

            if total_unclaimed_amount > 0 {
                let from_reward_token_account = from_reward_token_account
                    .ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
                let from_reward_token_account_signer = from_reward_token_account_signer
                    .ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;

                self.transfer_reward(
                    reward_token_mint,
                    reward_token_program,
                    reward_token_reserve_account,
                    from_reward_token_account,
                    from_reward_token_account_signer,
                    &[],
                    reward_id,
                    total_unclaimed_amount,
                )?;
            }
        }

        self.create_fund_manager_updated_reward_pool_event()
    }

    pub fn process_settle_reward(
        &mut self,
        reward_token_mint: Option<&InterfaceAccount<'info, Mint>>,
        reward_token_program: Option<&Interface<'info, TokenInterface>>,
        reward_token_reserve_account: Option<&InterfaceAccount<'info, TokenAccount>>,
        from_reward_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,
        from_reward_token_account_signer: Option<&Signer<'info>>,

        reward_pool_id: u8,
        reward_id: u16,
        amount: u64,
        transfer: bool,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        let (mint, program, decimals, claimable) = {
            let reward_account = self.reward_account.load_mut()?;
            let reward = reward_account.get_reward(reward_id)?;
            (
                reward.mint,
                reward.program,
                reward.decimals,
                reward.claimable,
            )
        };

        if !transfer {
            require_eq!(claimable, 0, errors::ErrorCode::RewardAlreadyClaimableError);

            self.settle_reward(reward_pool_id, reward_id, amount)?;
        } else {
            let reward_token_mint =
                reward_token_mint.ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
            let reward_token_program =
                reward_token_program.ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
            let reward_token_reserve_account = reward_token_reserve_account
                .ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
            let from_reward_token_account = from_reward_token_account
                .ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;
            let from_reward_token_account_signer = from_reward_token_account_signer
                .ok_or_else(|| error!(ErrorCode::ConstraintAccountIsNone))?;

            // Constraint check
            require_keys_eq!(reward_token_reserve_account.mint, mint);
            require_keys_eq!(reward_token_program.key(), program);
            require_eq!(reward_token_mint.decimals, decimals);

            self.settle_and_transfer_reward(
                reward_token_mint,
                reward_token_program,
                reward_token_reserve_account,
                from_reward_token_account,
                from_reward_token_account_signer,
                &[],
                reward_pool_id,
                reward_id,
                amount,
            )?;
        }

        self.create_fund_manager_updated_reward_pool_event()
    }

    pub(in crate::modules) fn settle_and_transfer_reward(
        &self,
        reward_token_mint: &InterfaceAccount<'info, Mint>,
        reward_token_program: &Interface<'info, TokenInterface>,
        reward_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,
        from_reward_token_account: &InterfaceAccount<'info, TokenAccount>,
        from_reward_token_account_signer: &AccountInfo<'info>,
        from_reward_token_account_signer_seeds: &[&[&[u8]]],
        reward_pool_id: u8,
        reward_id: u16,
        amount: u64,
    ) -> Result<()> {
        self.settle_reward(reward_pool_id, reward_id, amount)?;
        self.transfer_reward(
            reward_token_mint,
            reward_token_program,
            reward_token_reserve_account,
            from_reward_token_account,
            from_reward_token_account_signer,
            from_reward_token_account_signer_seeds,
            reward_id,
            amount,
        )?;

        Ok(())
    }

    /// Settle reward.
    fn settle_reward(&self, reward_pool_id: u8, reward_id: u16, amount: u64) -> Result<()> {
        self.reward_account
            .load_mut()?
            .get_reward_pool_mut(reward_pool_id)?
            .settle_reward(reward_id, amount, self.current_slot)
    }

    /// Transfer reward.
    fn transfer_reward(
        &self,
        reward_token_mint: &InterfaceAccount<'info, Mint>,
        reward_token_program: &Interface<'info, TokenInterface>,
        reward_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,
        from_reward_token_account: &InterfaceAccount<'info, TokenAccount>,
        from_reward_token_account_signer: &AccountInfo<'info>,
        from_reward_token_account_signer_seeds: &[&[&[u8]]],
        reward_id: u16,
        amount: u64,
    ) -> Result<()> {
        require_keys_eq!(
            reward_token_reserve_account.key(),
            self.reward_account
                .load()?
                .find_reward_token_reserve_account_address(reward_id)?,
        );

        require_gte!(from_reward_token_account.amount, amount);

        anchor_spl::token_interface::transfer_checked(
            CpiContext::new_with_signer(
                reward_token_program.to_account_info(),
                anchor_spl::token_interface::TransferChecked {
                    from: from_reward_token_account.to_account_info(),
                    mint: reward_token_mint.to_account_info(),
                    to: reward_token_reserve_account.to_account_info(),
                    authority: from_reward_token_account_signer.to_account_info(),
                },
                from_reward_token_account_signer_seeds,
            ),
            amount,
            reward_token_mint.decimals,
        )?;

        Ok(())
    }

    fn create_fund_manager_updated_reward_pool_event(
        &self,
    ) -> Result<events::FundManagerUpdatedRewardPool> {
        Ok(events::FundManagerUpdatedRewardPool {
            receipt_token_mint: self.receipt_token_mint.key(),
            reward_account: self.reward_account.key(),
        })
    }
}
