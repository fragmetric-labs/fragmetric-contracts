use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::errors::VaultError;
use crate::states::VaultAccount;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum VaultAdminRole {
    VaultManager = 0,
    RewardManager = 1,
    FundManager = 2,
    SolvManager = 3,
}

#[event_cpi]
#[derive(Accounts)]
pub struct VaultAdminRoleUpdateContext<'info> {
    pub old_vault_admin: Signer<'info>,
    /// CHECK: ..
    pub new_vault_admin: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [VaultAccount::SEED, vault_receipt_token_mint.key().as_ref()],
        bump = vault_account.load()?.get_bump(),
        constraint = vault_account.load()?.is_latest_version() @ VaultError::InvalidAccountDataVersionError,
    )]
    pub vault_account: AccountLoader<'info, VaultAccount>,

    pub vault_receipt_token_mint: Account<'info, Mint>,
}

pub fn process_update_vault_admin_role(
    ctx: Context<VaultAdminRoleUpdateContext>,
    role: VaultAdminRole,
) -> Result<()> {
    let VaultAdminRoleUpdateContext {
        old_vault_admin,
        new_vault_admin,
        vault_account,
        ..
    } = ctx.accounts;

    let mut vault = vault_account.load_mut()?;

    match role {
        VaultAdminRole::VaultManager => {
            require_keys_eq!(
                old_vault_admin.key(),
                vault.vault_manager,
                VaultError::VaultAdminMismatchError,
            );

            vault.set_vault_manager(new_vault_admin.key())?;
        }
        VaultAdminRole::RewardManager => {
            require_keys_eq!(
                old_vault_admin.key(),
                vault.reward_manager,
                VaultError::VaultAdminMismatchError,
            );

            vault.set_reward_manager(new_vault_admin.key())?
        }
        VaultAdminRole::FundManager => {
            require_keys_eq!(
                old_vault_admin.key(),
                vault.fund_manager,
                VaultError::VaultAdminMismatchError,
            );

            vault.set_fund_manager(new_vault_admin.key())?
        }
        VaultAdminRole::SolvManager => {
            require_keys_eq!(
                old_vault_admin.key(),
                vault.solv_manager,
                VaultError::VaultAdminMismatchError,
            );

            vault.set_solv_manager(new_vault_admin.key())?
        }
    }

    Ok(())
}
