use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount};

#[event_cpi]
#[derive(Accounts)]
pub struct VaultAccountInitialContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,

    /// Mint authority must be authority or vault account,
    /// otherwise `set_authority` CPI will fail.
    /// Therefore, no extra constraint is needed.
    #[account(mut)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub supported_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub token_program: Program<'info, Token>,

    #[account(
        init,
        payer = payer,
        seeds = [VaultAccount::SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = std::cmp::min(
            solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE,
            8 + std::mem::size_of::<VaultAccount>(),
        ),
    )]
    pub vault_account: AccountLoader<'info, VaultAccount>,

    #[account(
        associated_token::mint = receipt_token_mint,
        associated_token::authority = vault_account,
        associated_token::token_program = token_program,
    )]
    pub vault_receipt_token_lock_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [VaultAccount::RESERVE_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub vault_reserve_account: SystemAccount<'info>,
}

#[account(zero_copy)]
#[repr(C)]
pub struct VaultAccount {
    _todo: [u8; 10240],
}

impl VaultAccount {
    const SEED: &'static [u8] = b"vault";
    const RESERVE_SEED: &'static [u8] = b"vault_reserve";
}
