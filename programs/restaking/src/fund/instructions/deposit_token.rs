use anchor_lang::{
    prelude::*,
    solana_program::sysvar::{
        instructions as instructions_sysvar_module, instructions::load_instruction_at_checked,
    },
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use fragmetric_util::{request, Upgradable};

use crate::{common::*, constants::*, error::ErrorCode, fund::*, token::*, Empty};

#[derive(Accounts)]
pub struct FundDepositToken<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [USER_RECEIPT_SEED, receipt_token_mint.key().as_ref()],
        bump,
        space = 8 + UserReceipt::INIT_SPACE,
    )]
    pub user_receipt: Account<'info, UserReceipt>,

    #[account(
        mut,
        seeds = [FUND_SEED, receipt_token_mint.key().as_ref()],
        bump,
        realloc = 8 + Fund::INIT_SPACE,
        // TODO must paid by fund
        realloc::payer = user,
        realloc::zero = false,
    )]
    pub fund: Account<'info, Fund>,

    #[account(
        mut,
        seeds = [FUND_TOKEN_AUTHORITY_SEED, receipt_token_mint.key().as_ref()],
        bump,
    )]
    pub fund_token_authority: Account<'info, Empty>,

    #[account(mut, address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = receipt_token_mint,
        associated_token::authority = user,
        associated_token::token_program = receipt_token_program,
    )]
    pub receipt_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // user's fragSOL token account

    #[account(mut)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>, // lst token mint account
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = user.key()
    )]
    pub user_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // depositor's lst token account
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = fund_token_authority,
        associated_token::token_program = deposit_token_program,
    )]
    pub fund_token_account: Box<InterfaceAccount<'info, TokenAccount>>, // fund's lst token account

    /// CHECK: This is safe that checks it's ID
    #[account(address = instructions_sysvar_module::ID)]
    pub instruction_sysvar: Option<UncheckedAccount<'info>>,

    pub deposit_token_program: Interface<'info, TokenInterface>,
    pub receipt_token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundDepositToken<'info> {
    pub fn deposit_token(
        mut ctx: Context<Self>,
        request: FundDepositTokenRequest,
        metadata: Option<Metadata>,
    ) -> Result<()> {
        let wallet_provider: Option<String>;
        let fpoint_accrual_rate_multiplier: Option<f32>;
        match metadata {
            None => {
                wallet_provider = None;
                fpoint_accrual_rate_multiplier = None;

                msg!("metadata is null");
            }
            Some(_) => {
                let metadata_unwrap = metadata.clone().unwrap();
                wallet_provider = Some(metadata_unwrap.wallet_provider);
                fpoint_accrual_rate_multiplier =
                    Some(metadata_unwrap.fpoint_accrual_rate_multiplier);

                // need signature verification
                msg!("metadata is not null");
                // Get what should be the Ed25519Program instruction
                let instruction_sysvar = ctx.accounts.instruction_sysvar.as_ref().unwrap();
                let ed25519_ix =
                    load_instruction_at_checked(EXPTECED_IX_SYSVAR_INDEX, instruction_sysvar)?;

                // Check that ix is what we expect to have been sent
                let metadata_unwrap = metadata.clone().unwrap(); // re-clone to use it
                let payload_vec = metadata_unwrap.try_to_vec()?;
                let payload = payload_vec.as_slice();
                verify_ed25519_ix(&ed25519_ix, &ADMIN_PUBKEY.to_bytes(), payload)?;
                msg!("Signature verification succeed");
            }
        }

        let FundDepositTokenArgs { amount } = request.into();
        Self::transfer_token_cpi(&ctx, amount)?;

        let Self {
            fund, token_mint, ..
        } = ctx.accounts;
        let token_amount_in_fund = fund
            .to_latest_version()
            .deposit_token(token_mint.key(), amount)?;

        let mint_amount = Self::get_receipt_token_by_token_exchange_rate(&ctx, amount)?;
        Self::mint_receipt_token(&mut ctx, mint_amount)?;

        let admin = ctx.accounts.fund.admin;
        let receipt_token_mint = ctx.accounts.fund.receipt_token_mint;
        let fund = ctx.accounts.fund.to_latest_version();
        emit!(FundTokenDeposited {
            user: ctx.accounts.user.key(),
            user_lrt_account: ctx.accounts.receipt_token_account.key(),
            user_receipt: Clone::clone(&ctx.accounts.user_receipt),
            deposited_token_mint: ctx.accounts.token_mint.key(),
            deposited_token_user_account: ctx.accounts.user_token_account.key(),
            token_deposit_amount: amount,
            token_amount_in_fund,
            minted_lrt_mint: receipt_token_mint.key(),
            minted_lrt_amount: mint_amount,
            lrt_amount_in_user_lrt_account: ctx.accounts.receipt_token_account.amount,
            wallet_provider: wallet_provider,
            fpoint_accrual_rate_multiplier: fpoint_accrual_rate_multiplier,
            fund_info: fund.to_info(admin, receipt_token_mint),
        });

        Ok(())
    }

    fn transfer_token_cpi(ctx: &Context<Self>, amount: u64) -> Result<()> {
        let Self {
            user: authority,
            user_token_account,
            fund_token_account,
            token_mint,
            deposit_token_program: token_interface,
            ..
        } = &*ctx.accounts;

        let token_transfer_cpi_ctx = CpiContext::new(
            token_interface.to_account_info(),
            TransferChecked {
                from: user_token_account.to_account_info(),
                to: fund_token_account.to_account_info(),
                mint: token_mint.to_account_info(),
                authority: authority.to_account_info(),
            },
        );

        transfer_checked(token_transfer_cpi_ctx, amount, token_mint.decimals)
            .map_err(|_| error!(ErrorCode::FundTokenTransferFailed))?;

        Ok(())
    }

    #[allow(unused_variables)]
    fn get_receipt_token_by_token_exchange_rate(ctx: &Context<Self>, amount: u64) -> Result<u64> {
        Ok(amount)
    }

    fn mint_receipt_token(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        let receipt_token_account_key = ctx.accounts.receipt_token_account.key();
        msg!(
            "user's receipt token account key: {:?}",
            receipt_token_account_key
        );

        Self::call_mint_token_cpi(ctx, amount)?;
        msg!(
            "Minted {} to user token account {:?}",
            amount,
            receipt_token_account_key
        );

        Self::call_transfer_hook(ctx, amount)?;

        Ok(())
    }

    fn call_mint_token_cpi(ctx: &mut Context<Self>, amount: u64) -> Result<()> {
        let bump = ctx.bumps.fund_token_authority;
        let key = ctx.accounts.receipt_token_mint.key();
        let signer_seeds = [FUND_TOKEN_AUTHORITY_SEED, key.as_ref(), &[bump]];

        ctx.accounts.receipt_token_program.mint_token_cpi(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_account,
            ctx.accounts.fund_token_authority.to_account_info(),
            Some(&[signer_seeds.as_ref()]),
            amount,
        )
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

pub struct FundDepositTokenArgs {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
#[request(FundDepositTokenArgs)]
pub enum FundDepositTokenRequest {
    V1(FundDepositTokenRequestV1),
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundDepositTokenRequestV1 {
    pub amount: u64,
}

impl From<FundDepositTokenRequest> for FundDepositTokenArgs {
    fn from(value: FundDepositTokenRequest) -> Self {
        match value {
            FundDepositTokenRequest::V1(value) => Self {
                amount: value.amount,
            },
        }
    }
}
