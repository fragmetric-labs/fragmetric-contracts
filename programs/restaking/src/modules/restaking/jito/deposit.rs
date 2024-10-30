use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::{invoke};
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount};
use jito_vault_sdk::sdk::{mint_to, update_vault_balance, initialize_vault_update_state_tracker, close_vault_update_state_tracker};
use super::*;

#[derive(Debug, Clone)]
pub struct Jito;

impl Id for Jito {
    fn id() -> Pubkey {
        JitoRestakingProtocol::JITO_VAULT_PROGRAM_ID
    }
}


#[derive(Accounts)]
pub struct RestakingDepositContext<'info> {
    #[account(address = JitoRestakingProtocol::JITO_VAULT_PROGRAM_ID)]
    pub program: Program<'info, Jito>,

    /// CHECK: blabla
    #[account(mut, address = JitoRestakingProtocol::JITO_VAULT_CONFIG_ADDRESS)]
    pub config: UncheckedAccount<'info>,

    /// CHECK: blabla
    #[account(mut, address = JitoRestakingProtocol::JITO_VAULT_ADDRESS)]
    pub vault: UncheckedAccount<'info>,

    /// CHECK: blabla
    #[account(mut, address = JitoRestakingProtocol::JITO_VAULT_RECEIPT_TOKEN)]
    pub vault_receipt_token: UncheckedAccount<'info>,

    #[account(mut, address = JitoRestakingProtocol::JITO_VAULT_SUPPORTED_TOKEN)]
    pub vault_supported_token: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = vault_supported_token,
        associated_token::token_program = token_program,
        associated_token::authority = vault,
    )]
    pub vault_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: blabla
    #[account(mut)]
    pub vault_update_state_tracker: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = vault_supported_token,
        associated_token::token_program = token_program,
        associated_token::authority = user,
    )]
    pub user_supported_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = vault_receipt_token,
        associated_token::token_program = token_program,
        associated_token::authority = user,
    )]
    pub user_receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>,


    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> RestakingDepositContext<'info> {
    pub fn initialize_vault_update_state_tracker(ctx: &Context<Self>) -> Result<()> {
        let initialize_vault_update_state_tracker_ix = initialize_vault_update_state_tracker(
            ctx.accounts.program.key,
            ctx.accounts.config.key,
            ctx.accounts.vault.key,
            ctx.accounts.vault_update_state_tracker.key,
            ctx.accounts.user.key,
            TryFrom::try_from(0u8).unwrap(),
        );

        invoke(
            &initialize_vault_update_state_tracker_ix,
            &[ctx.accounts.program.to_account_info(),
                ctx.accounts.config.to_account_info(),
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.vault_update_state_tracker.to_account_info(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.system_program.to_account_info()
            ],
        )?;

        Ok(())
    }
    pub fn close_vault_update_state_tracker(ctx: &Context<Self>) -> Result<()> {
        let close_vault_update_state_tracker_ix = close_vault_update_state_tracker(
            ctx.accounts.program.key,
            ctx.accounts.config.key,
            ctx.accounts.vault.key,
            ctx.accounts.vault_update_state_tracker.key,
            ctx.accounts.user.key,
            Clock::get()?.slot.checked_div(432000).unwrap(),
        );

        invoke(
            &close_vault_update_state_tracker_ix,
            &[ctx.accounts.program.to_account_info(),
                ctx.accounts.config.to_account_info(),
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.vault_update_state_tracker.to_account_info(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.system_program.to_account_info()
            ],
        )?;

        Ok(())
    }
    pub fn update_vault_balance(ctx: &Context<Self>) -> Result<()> {
        let update_vault_balance_ix = update_vault_balance(
            ctx.accounts.program.key,
            ctx.accounts.config.key,
            ctx.accounts.vault.key,
            &ctx.accounts.vault_supported_token_account.key(),
            ctx.accounts.vault_receipt_token.key,
            &ctx.accounts.user_receipt_token_account.key(),
            ctx.accounts.token_program.key,
        );

        invoke(
            &update_vault_balance_ix,
            &[
                ctx.accounts.program.to_account_info(),
                ctx.accounts.config.to_account_info(),
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.vault_supported_token_account.to_account_info(),
                ctx.accounts.vault_receipt_token.to_account_info(),
                ctx.accounts.user_receipt_token_account.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
        )?;

        Ok(())
    }
    pub fn mint_to(ctx: &Context<Self>, amount_in: u64, min_amount_out: u64) -> Result<()> {
        let mint_to_ix = mint_to(
            ctx.accounts.program.key,
            ctx.accounts.config.key,
            ctx.accounts.vault.key,
            ctx.accounts.vault_receipt_token.key,
            ctx.accounts.user.key,
            &ctx.accounts.user_supported_token_account.key(),
            &ctx.accounts.vault_supported_token_account.key(),
            &ctx.accounts.user_receipt_token_account.key(),
            &ctx.accounts.user_receipt_token_account.key(),
            Some(ctx.accounts.user.key),
            amount_in,
            min_amount_out,
        );

        invoke(
            &mint_to_ix,
            &[
                ctx.accounts.program.to_account_info(),
                ctx.accounts.config.to_account_info(),
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.vault_receipt_token.to_account_info(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.user_supported_token_account.to_account_info(),
                ctx.accounts.vault_supported_token_account.to_account_info(),
                ctx.accounts.user_receipt_token_account.to_account_info(),
                ctx.accounts.user_supported_token_account.to_account_info(),
            ],
        )?;

        Ok(())
    }
    pub fn deposit(ctx: Context<Self>, amount_in: u64, min_amount_out: u64) -> Result<()> {
        RestakingDepositContext::mint_to(&ctx, amount_in, min_amount_out)?;
        Ok(())
    }
}
