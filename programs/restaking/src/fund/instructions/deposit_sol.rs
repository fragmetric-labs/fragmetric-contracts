use anchor_lang::{
    prelude::*, solana_program::sysvar::instructions as instructions_sysvar, system_program,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount},
};

use crate::{common::*, constants::*, error::ErrorCode, fund::*, token::*};

#[derive(Accounts)]
pub struct FundDepositSOL<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [UserReceipt::SEED, user.key().as_ref(), receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + UserReceipt::INIT_SPACE,
        constraint = user_receipt.data_version == 0 || user_receipt.user == user.key(),
        constraint = user_receipt.data_version == 0 || user_receipt.receipt_token_mint == receipt_token_mint.key(),
    )]
    pub user_receipt: Account<'info, UserReceipt>,

    #[account(
        mut,
        seeds = [Fund::SEED, receipt_token_mint.key().as_ref()],
        bump = fund.bump,
        has_one = receipt_token_mint,
    )]
    pub fund: Box<Account<'info, Fund>>,

    #[account(
        seeds = [ReceiptTokenMintAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump = receipt_token_mint_authority.bump,
        has_one = receipt_token_mint,
    )]
    pub receipt_token_mint_authority: Account<'info, ReceiptTokenMintAuthority>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // user's fragSOL token account

    // TODO: use address lookup table!
    #[account(address = BSOL_STAKE_POOL_ADDRESS)]
    /// CHECK: will be checked and deserialized when needed
    pub token_pricing_source_0: UncheckedAccount<'info>,

    // TODO: use address lookup table!
    #[account(address = MSOL_STAKE_POOL_ADDRESS)]
    /// CHECK: will be checked and deserialized when needed
    pub token_pricing_source_1: UncheckedAccount<'info>,

    /// CHECK: This is safe that checks it's ID
    #[account(address = instructions_sysvar::ID)]
    pub instruction_sysvar: Option<UncheckedAccount<'info>>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundDepositSOL<'info> {
    pub fn deposit_sol(
        mut ctx: Context<Self>,
        amount: u64,
        metadata: Option<Metadata>,
    ) -> Result<()> {
        if let Some(metadata) = &metadata {
            match ctx.accounts.instruction_sysvar.as_ref() {
                Some(sysvar) => {
                    let ed25519_ix =
                        Ed25519Instruction::new_from_instruction_sysvar(sysvar.as_ref())?;
                    let payload = metadata.try_to_vec()?;
                    ed25519_ix.verify(&ADMIN_PUBKEY.to_bytes(), payload.as_slice())?;
                }
                None => {
                    msg!("Error: Instruction sysvar not provided");
                    err!(ErrorCode::SigVerificationFailed)?;
                }
            }
        }

        let (wallet_provider, fpoint_accrual_rate_multiplier) = metadata
            .map(|metadata| {
                (
                    metadata.wallet_provider,
                    metadata.fpoint_accrual_rate_multiplier,
                )
            })
            .unzip();

        // Initialize
        ctx.accounts.user_receipt.initialize_if_needed(
            ctx.bumps.user_receipt,
            ctx.accounts.user.key(),
            ctx.accounts.receipt_token_mint.key(),
        );

        // Verify
        require_gte!(ctx.accounts.user.lamports(), amount);

        // Step 1: Calculate mint amount
        let fund = &mut ctx.accounts.fund;
        let sources = [
            ctx.accounts.token_pricing_source_0.as_ref(),
            ctx.accounts.token_pricing_source_1.as_ref(),
        ];
        fund.update_token_prices(&sources)?;
        let receipt_token_total_supply = ctx.accounts.receipt_token_mint.supply;
        let receipt_token_mint_amount =
            fund.calculate_receipt_tokens_from_sol(amount, receipt_token_total_supply)?;
        let receipt_token_price = fund.receipt_token_price(
            ctx.accounts.receipt_token_mint.decimals,
            receipt_token_total_supply,
        )?;

        // Step 2: Deposit SOL
        Self::transfer_sol_cpi(&ctx, amount)?;
        ctx.accounts.fund.deposit_sol(amount)?;

        // Step 3: Mint receipt token
        Self::call_mint_token_cpi(&mut ctx, receipt_token_mint_amount)?;
        Self::call_transfer_hook(&ctx, receipt_token_mint_amount)?;

        emit!(FundSOLDeposited {
            user: ctx.accounts.user.key(),
            user_receipt_token_account: ctx.accounts.receipt_token_account.key(),
            user_receipt: Clone::clone(&ctx.accounts.user_receipt),
            sol_deposit_amount: amount,
            sol_amount_in_fund: ctx.accounts.fund.sol_amount_in,
            minted_receipt_token_mint: ctx.accounts.receipt_token_mint.key(),
            minted_receipt_token_amount: receipt_token_mint_amount,
            receipt_token_price,
            receipt_token_amount_in_user_receipt_token_account: ctx
                .accounts
                .receipt_token_account
                .amount,
            wallet_provider,
            fpoint_accrual_rate_multiplier,
            fund_info: FundInfo::new_from_fund(ctx.accounts.fund.as_ref()),
        });

        Ok(())
    }

    fn transfer_sol_cpi(ctx: &Context<Self>, amount: u64) -> Result<()> {
        let sol_transfer_cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.fund.to_account_info(),
            },
        );

        system_program::transfer(sol_transfer_cpi_ctx, amount)
            .map_err(|_| error!(ErrorCode::FundSOLTransferFailed))
    }

    fn call_mint_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts
            .token_program
            .mint_token_cpi(
                &mut ctx.accounts.receipt_token_mint,
                &mut ctx.accounts.receipt_token_account,
                ctx.accounts.receipt_token_mint_authority.to_account_info(),
                Some(&[ctx
                    .accounts
                    .receipt_token_mint_authority
                    .signer_seeds()
                    .as_ref()]),
                amount,
            )
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailed))
    }

    fn call_transfer_hook(ctx: &Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.receipt_token_mint.transfer_hook(
            None,
            Some(&ctx.accounts.receipt_token_account),
            &ctx.accounts.fund,
            amount,
        )
    }
}
