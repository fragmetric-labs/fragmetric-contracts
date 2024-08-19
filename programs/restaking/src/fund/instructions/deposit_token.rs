use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{common::*, constants::*, error::ErrorCode, fund::*, token::*};

#[derive(Accounts)]
pub struct FundDepositToken<'info> {
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
        mut,
        seeds = [FundTokenAuthority::SEED, receipt_token_mint.key().as_ref()],
        bump,
        // bump = fund_token_authority.bump,
        // has_one = receipt_token_mint,
    )]
    // pub fund_token_authority: Account<'info, FundTokenAuthority>,
    /// CHECK: due to stack size limit this is not deserialize yet
    pub fund_token_authority: UncheckedAccount<'info>,

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

    pub deposit_token_program: Interface<'info, TokenInterface>,
    pub receipt_token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> FundDepositToken<'info> {
    fn deserialize_fund_token_authority_manually(
        info: &UncheckedAccount<'info>,
        bump: u8,
        receipt_token_mint: Pubkey,
    ) -> Result<FundTokenAuthority> {
        if info.owner == &anchor_lang::solana_program::system_program::ID && info.lamports() == 0 {
            return Err(anchor_lang::error::ErrorCode::AccountNotInitialized.into());
        }
        if info.owner != &FundTokenAuthority::owner() {
            return Err(
                Error::from(anchor_lang::error::ErrorCode::AccountOwnedByWrongProgram)
                    .with_pubkeys((*info.owner, FundTokenAuthority::owner())),
            );
        }

        let mut data: &[u8] = &info.try_borrow_data()?;
        let fund_token_authority = FundTokenAuthority::try_deserialize(&mut data)?;

        if bump != fund_token_authority.bump {
            return Err(Error::from(anchor_lang::error::ErrorCode::ConstraintSeeds)
                .with_account_name("fund_token_authority"));
        }

        let my_key = fund_token_authority.receipt_token_mint;
        let target_key = receipt_token_mint;
        if my_key != target_key {
            return Err(Error::from(anchor_lang::error::ErrorCode::ConstraintHasOne)
                .with_account_name("fund_token_authority")
                .with_pubkeys((my_key, target_key)));
        }

        Ok(FundTokenAuthority::try_deserialize(&mut data)?)
    }

    pub fn deposit_token(mut ctx: Context<Self>, amount: u64) -> Result<()> {
        ctx.accounts.user_receipt.initialize_if_needed(
            ctx.bumps.user_receipt,
            ctx.accounts.user.key(),
            ctx.accounts.receipt_token_mint.key(),
        );

        Self::transfer_token_cpi(&ctx, amount)?;

        let token_mint = ctx.accounts.token_mint.key();
        let token_info = ctx
            .accounts
            .fund
            .whitelisted_token_mut(token_mint)
            .ok_or_else(|| error!(ErrorCode::FundNotExistingToken))?;
        token_info.deposit_token(amount)?;
        let token_amount_in_fund = token_info.token_amount_in;

        let mint_amount = Self::get_receipt_token_by_token_exchange_rate(&ctx, amount)?;
        Self::mint_receipt_token(&mut ctx, mint_amount)?;

        emit!(FundTokenDeposited {
            user: ctx.accounts.user.key(),
            user_lrt_account: ctx.accounts.receipt_token_account.key(),
            user_receipt: Clone::clone(&ctx.accounts.user_receipt),
            deposited_token_mint: ctx.accounts.token_mint.key(),
            deposited_token_user_account: ctx.accounts.user_token_account.key(),
            token_deposit_amount: amount,
            token_amount_in_fund,
            minted_lrt_mint: ctx.accounts.fund.receipt_token_mint.key(),
            minted_lrt_amount: mint_amount,
            lrt_amount_in_user_lrt_account: ctx.accounts.receipt_token_account.amount,
            wallet_provider: None,
            fpoint_accrual_rate_multiplier: None,
            fund_info: FundInfo::new_from_fund(ctx.accounts.fund.as_ref()),
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
        let fund_token_authority = Self::deserialize_fund_token_authority_manually(
            &ctx.accounts.fund_token_authority,
            ctx.bumps.fund_token_authority,
            ctx.accounts.receipt_token_mint.key(),
        )?;

        ctx.accounts.receipt_token_program.mint_token_cpi(
            &ctx.accounts.receipt_token_mint,
            &mut ctx.accounts.receipt_token_account,
            ctx.accounts.fund_token_authority.to_account_info(),
            Some(&[fund_token_authority.signer_seeds().as_ref()]),
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
