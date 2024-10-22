use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::events;
use crate::utils::AccountLoaderExt;

use super::*;

pub fn process_update_reward_account_if_needed<'info>(
    payer: &Signer<'info>,
    receipt_token_mint: &InterfaceAccount<Mint>,
    reward_account: &AccountLoader<'info, RewardAccount>,
    system_program: &Program<'info, System>,
    desired_account_size: Option<u32>,
    initialize: bool,
) -> Result<()> {
    reward_account.expand_account_size_if_needed(
        payer,
        system_program,
        desired_account_size,
        initialize,
    )?;

    if initialize {
        reward_account
            .load_mut()?
            .update_if_needed(receipt_token_mint.key());
    }

    Ok(())
}

pub fn process_update_user_reward_account_if_needed<'info>(
    user: &Signer<'info>,
    receipt_token_mint: &InterfaceAccount<Mint>,
    user_reward_account: &AccountLoader<'info, UserRewardAccount>,
    system_program: &Program<'info, System>,
    desired_account_size: Option<u32>,
    initialize: bool,
) -> Result<()> {
    user_reward_account.expand_account_size_if_needed(
        user,
        system_program,
        desired_account_size,
        initialize,
    )?;

    if initialize {
        user_reward_account
            .load_mut()?
            .update_if_needed(receipt_token_mint.key(), user.key());

        emit!(events::UserUpdatedRewardPool {
            receipt_token_mint: receipt_token_mint.key(),
            updated_user_reward_account_addresses: vec![user_reward_account.key()],
        });
    }

    Ok(())
}

pub fn process_add_reward_pool_holder(
    receipt_token_mint: &InterfaceAccount<Mint>,
    reward_account: &mut AccountLoader<RewardAccount>,
    name: String,
    description: String,
    pubkeys: Vec<Pubkey>,
) -> Result<()> {
    reward_account
        .load_mut()?
        .add_holder(name, description, pubkeys)?;

    emit!(events::FundManagerUpdatedRewardPool {
        receipt_token_mint: receipt_token_mint.key(),
        reward_account_address: reward_account.key(),
    });

    Ok(())
}

pub fn process_add_reward_pool(
    receipt_token_mint: &InterfaceAccount<Mint>,
    reward_account: &mut AccountLoader<RewardAccount>,
    name: String,
    holder_id: Option<u8>,
    custom_contribution_accrual_rate_enabled: bool,
    current_slot: u64,
) -> Result<()> {
    reward_account.load_mut()?.add_reward_pool(
        name,
        holder_id,
        custom_contribution_accrual_rate_enabled,
        current_slot,
    )?;

    emit!(events::FundManagerUpdatedRewardPool {
        receipt_token_mint: receipt_token_mint.key(),
        reward_account_address: reward_account.key(),
    });

    Ok(())
}

pub fn process_close_reward_pool(
    receipt_token_mint: &InterfaceAccount<Mint>,
    reward_account: &mut AccountLoader<RewardAccount>,
    reward_pool_id: u8,
    current_slot: u64,
) -> Result<()> {
    reward_account
        .load_mut()?
        .close_reward_pool(reward_pool_id, current_slot)?;

    emit!(events::FundManagerUpdatedRewardPool {
        receipt_token_mint: receipt_token_mint.key(),
        reward_account_address: reward_account.key(),
    });

    Ok(())
}

pub fn process_add_reward(
    receipt_token_mint: &InterfaceAccount<Mint>,
    reward_account: &mut AccountLoader<RewardAccount>,
    name: String,
    description: String,
    reward_type: RewardType,
) -> Result<()> {
    reward_account
        .load_mut()?
        .add_reward(name, description, reward_type)?;

    emit!(events::FundManagerUpdatedRewardPool {
        receipt_token_mint: receipt_token_mint.key(),
        reward_account_address: reward_account.key(),
    });

    Ok(())
}

pub fn process_update_reward_pools(
    receipt_token_mint: &InterfaceAccount<Mint>,
    reward_account: &mut AccountLoader<RewardAccount>,
    current_slot: u64,
) -> Result<()> {
    reward_account
        .load_mut()?
        .update_reward_pools(current_slot)?;

    emit!(events::OperatorUpdatedRewardPools {
        receipt_token_mint: receipt_token_mint.key(),
        reward_account_address: reward_account.key(),
    });

    Ok(())
}

pub fn process_update_user_reward_pools(
    reward_account: &mut AccountLoader<RewardAccount>,
    user_reward_account: &mut AccountLoader<UserRewardAccount>,
    current_slot: u64,
) -> Result<()> {
    reward_account
        .load_mut()?
        .update_user_reward_pools(&mut *user_reward_account.load_mut()?, current_slot)

    // no events required practically...
    // emit!(UserUpdatedRewardPool::new(
    //     receipt_token_mint.key(),
    //     vec![update],
    // ));
}

pub(in crate::modules) fn update_reward_pools_token_allocation(
    reward_account: &mut RewardAccount,
    from: Option<&mut UserRewardAccount>,
    to: Option<&mut UserRewardAccount>,
    updated_user_reward_account_addresses: Vec<Pubkey>,
    receipt_token_mint: Pubkey,
    amount: u64,
    contribution_accrual_rate: Option<u8>,
    current_slot: u64,
) -> Result<()> {
    reward_account.update_reward_pools_token_allocation(
        receipt_token_mint,
        amount,
        contribution_accrual_rate,
        from,
        to,
        current_slot,
    )?;

    emit!(events::UserUpdatedRewardPool {
        receipt_token_mint,
        updated_user_reward_account_addresses,
    });

    Ok(())
}
