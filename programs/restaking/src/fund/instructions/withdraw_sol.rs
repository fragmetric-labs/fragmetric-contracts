use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use fragmetric_util::{request, Upgradable};

use crate::{constants::*, fund::*};

#[derive(Accounts)]
pub struct FundWithdrawSOL<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [USER_ACCOUNT_SEED, receipt_token_mint.key().as_ref()],
        bump,
        realloc = 8 + UserAccount::INIT_SPACE,
        realloc::payer = user,
        realloc::zero = false,
    )]
    pub user_account: Account<'info, UserAccount>,

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

    #[account(address = FRAGSOL_MINT_ADDRESS)]
    pub receipt_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub system_program: Program<'info, System>,
}

impl<'info> FundWithdrawSOL<'info> {
    pub fn withdraw_sol(ctx: Context<Self>, request: FundWithdrawSOLRequest) -> Result<()> {
        let FundWithdrawSOLArgs { request_id } = request.into();
        let Self {
            user,
            user_account,
            fund,
            ..
        } = ctx.accounts;

        let WithdrawalRequest {
            batch_id,
            receipt_token_amount,
            ..
        } = user_account
            .to_latest_version()
            .pop_withdrawal_request(request_id)?;

        let lamports = {
            let fund = fund.to_latest_version();
            fund.check_if_withdrawal_completed(batch_id)?;
            fund.reserved_fund.withdraw_sol(receipt_token_amount)?
        };

        fund.sub_lamports(lamports)?;
        user.add_lamports(lamports)?;

        Ok(())
    }
}

pub struct FundWithdrawSOLArgs {
    pub request_id: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
#[request(FundWithdrawSOLArgs)]
pub enum FundWithdrawSOLRequest {
    V1(FundWithdrawSOLRequestV1),
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundWithdrawSOLRequestV1 {
    pub request_id: u64,
}

impl From<FundWithdrawSOLRequest> for FundWithdrawSOLArgs {
    fn from(value: FundWithdrawSOLRequest) -> Self {
        match value {
            FundWithdrawSOLRequest::V1(value) => Self {
                request_id: value.request_id,
            },
        }
    }
}
