use anchor_lang::{prelude::*, system_program};
use anchor_spl::token_2022::{self, Token2022};
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::ed25519;
use crate::modules::reward::{self, RewardAccount, UserRewardAccount};
use crate::utils::PDASeeds;

use super::*;

pub fn deposit_sol<'info>(
    user: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    fund_account: &mut Account<'info, FundAccount>,
    reward_account: &mut RewardAccount,
    user_fund_account: &mut UserFundAccount,
    user_reward_account: &mut UserRewardAccount,
    user_reward_account_address: Pubkey,
    system_program: &Program<'info, System>,
    receipt_token_program: &Program<'info, Token2022>,
    instructions_sysvar: &AccountInfo,
    pricing_sources: &[AccountInfo],
    sol_amount: u64,
    metadata: Option<DepositMetadata>,
    current_slot: u64,
) -> Result<()> {
    if let Some(metadata) = &metadata {
        ed25519::verify_preceding_ed25519_instruction(
            instructions_sysvar,
            metadata.try_to_vec()?.as_slice(),
        )?;
        metadata.verify_expiration()?;
    }
    let (wallet_provider, contribution_accrual_rate) = metadata
        .map(|metadata| (metadata.wallet_provider, metadata.contribution_accrual_rate))
        .unzip();

    fund_account.update_token_prices(pricing_sources)?;
    let receipt_token_mint_amount =
        fund_account.receipt_token_mint_amount_for(sol_amount, receipt_token_mint.supply)?;

    mint_receipt_token_to_user(
        receipt_token_mint,
        receipt_token_mint_authority,
        user_receipt_token_account,
        reward_account,
        user_fund_account,
        user_reward_account,
        user_reward_account_address,
        receipt_token_program,
        receipt_token_mint_amount,
        contribution_accrual_rate,
        current_slot,
    )?;

    transfer_sol_from_user_to_fund(user, fund_account, system_program, sol_amount)?;

    let receipt_token_price = fund_account.receipt_token_sol_value_per_token(
        receipt_token_mint.decimals,
        receipt_token_mint.supply,
    )?;

    emit!(events::UserDepositedSOLToFund {
        user: user.key(),
        user_receipt_token_account: user_receipt_token_account.key(),
        user_fund_account: user_fund_account.clone(),
        deposited_sol_amount: sol_amount,
        receipt_token_mint: receipt_token_mint.key(),
        minted_receipt_token_amount: receipt_token_mint_amount,
        wallet_provider,
        contribution_accrual_rate,
        fund_account: FundAccountInfo::new(
            fund_account.as_ref(),
            receipt_token_price,
            receipt_token_mint.supply,
        ),
    });

    Ok(())
}

pub fn deposit_supported_token<'info>(
    user: &Signer<'info>,
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    supported_token_mint: &InterfaceAccount<'info, Mint>,
    supported_token_account: &InterfaceAccount<'info, TokenAccount>,
    user_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
    fund_account: &mut FundAccount,
    reward_account: &mut RewardAccount,
    user_fund_account: &mut UserFundAccount,
    user_reward_account: &mut UserRewardAccount,
    user_reward_account_address: Pubkey,
    receipt_token_program: &Program<'info, Token2022>,
    supported_token_program: &Interface<'info, TokenInterface>,
    instructions_sysvar: &AccountInfo,
    pricing_sources: &[AccountInfo],
    supported_token_amount: u64,
    metadata: Option<DepositMetadata>,
    current_slot: u64,
) -> Result<()> {
    if let Some(metadata) = &metadata {
        ed25519::verify_preceding_ed25519_instruction(
            instructions_sysvar,
            metadata.try_to_vec()?.as_slice(),
        )?;
        metadata.verify_expiration()?;
    }
    let (wallet_provider, contribution_accrual_rate) = metadata
        .map(|metadata| (metadata.wallet_provider, metadata.contribution_accrual_rate))
        .unzip();

    fund_account.update_token_prices(pricing_sources)?;
    let receipt_token_mint_amount = fund_account.receipt_token_mint_amount_for(
        fund_account
            .supported_token(supported_token_mint.key())?
            .calculate_sol_from_tokens(supported_token_amount)?,
        receipt_token_mint.supply,
    )?;

    mint_receipt_token_to_user(
        receipt_token_mint,
        receipt_token_mint_authority,
        user_receipt_token_account,
        reward_account,
        user_fund_account,
        user_reward_account,
        user_reward_account_address,
        receipt_token_program,
        receipt_token_mint_amount,
        contribution_accrual_rate,
        current_slot,
    )?;

    transfer_supported_token_from_user_to_fund(
        user,
        supported_token_mint,
        supported_token_account,
        user_supported_token_account,
        fund_account,
        supported_token_program,
        supported_token_amount,
    )?;

    let receipt_token_price = fund_account.receipt_token_sol_value_per_token(
        receipt_token_mint.decimals,
        receipt_token_mint.supply,
    )?;

    emit!(events::UserDepositedSupportedTokenToFund {
        user: user.key(),
        user_receipt_token_account: user_receipt_token_account.key(),
        user_fund_account: user_fund_account.clone(),
        supported_token_mint: supported_token_mint.key(),
        supported_token_user_account: user_supported_token_account.key(),
        deposited_supported_token_amount: supported_token_amount,
        receipt_token_mint: receipt_token_mint.key(),
        minted_receipt_token_amount: receipt_token_mint_amount,
        wallet_provider,
        contribution_accrual_rate,
        fund_account: FundAccountInfo::new(
            fund_account,
            receipt_token_price,
            receipt_token_mint.supply,
        ),
    });

    Ok(())
}

fn transfer_sol_from_user_to_fund<'info>(
    user: &Signer<'info>,
    fund_account: &mut Account<'info, FundAccount>,
    system_program: &Program<'info, System>,
    sol_amount: u64,
) -> Result<()> {
    fund_account.deposit_sol(sol_amount)?;
    system_program::transfer(
        CpiContext::new(
            system_program.to_account_info(),
            system_program::Transfer {
                from: user.to_account_info(),
                to: fund_account.to_account_info(),
            },
        ),
        sol_amount,
    )
}

fn transfer_supported_token_from_user_to_fund<'info>(
    user: &Signer<'info>,
    supported_token_mint: &InterfaceAccount<'info, Mint>,
    supported_token_account: &InterfaceAccount<'info, TokenAccount>,
    user_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
    fund_account: &mut FundAccount,
    supported_token_program: &Interface<'info, TokenInterface>,
    supported_token_amount: u64,
) -> Result<()> {
    fund_account
        .supported_token_mut(supported_token_mint.key())?
        .deposit_token(supported_token_amount)?;
    token_interface::transfer_checked(
        CpiContext::new(
            supported_token_program.to_account_info(),
            token_interface::TransferChecked {
                from: user_supported_token_account.to_account_info(),
                to: supported_token_account.to_account_info(),
                mint: supported_token_mint.to_account_info(),
                authority: user.to_account_info(),
            },
        ),
        supported_token_amount,
        supported_token_mint.decimals,
    )
    .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))
}

fn mint_receipt_token_to_user<'info>(
    receipt_token_mint: &mut InterfaceAccount<'info, Mint>,
    receipt_token_mint_authority: &Account<'info, ReceiptTokenMintAuthority>,
    user_receipt_token_account: &mut InterfaceAccount<'info, TokenAccount>,
    reward_account: &mut RewardAccount,
    user_fund_account: &mut UserFundAccount,
    user_reward_account: &mut UserRewardAccount,
    user_reward_account_address: Pubkey,
    receipt_token_program: &Program<'info, Token2022>,
    receipt_token_mint_amount: u64,
    contribution_accrual_rate: Option<u8>,
    current_slot: u64,
) -> Result<()> {
    token_2022::mint_to(
        CpiContext::new_with_signer(
            receipt_token_program.to_account_info(),
            token_2022::MintTo {
                mint: receipt_token_mint.to_account_info(),
                to: user_receipt_token_account.to_account_info(),
                authority: receipt_token_mint_authority.to_account_info(),
            },
            &[receipt_token_mint_authority.signer_seeds().as_ref()],
        ),
        receipt_token_mint_amount,
    )
    .map_err(|_| error!(ErrorCode::FundTokenTransferFailedException))?;
    receipt_token_mint.reload()?;
    user_receipt_token_account.reload()?;
    user_fund_account.set_receipt_token_amount(user_receipt_token_account.amount);

    reward::update_reward_pools_token_allocation(
        reward_account,
        None,
        Some(user_reward_account),
        vec![user_reward_account_address],
        receipt_token_mint.key(),
        receipt_token_mint_amount,
        contribution_accrual_rate,
        current_slot,
    )
}

impl SupportedTokenInfo {
    pub fn deposit_token(&mut self, amount: u64) -> Result<()> {
        let new_accumulated_deposit_amount = self
            .accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        if self.capacity_amount < new_accumulated_deposit_amount {
            err!(ErrorCode::FundExceededTokenCapacityAmountError)?
        }

        self.accumulated_deposit_amount = new_accumulated_deposit_amount;
        self.operation_reserved_amount = self
            .operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}

impl FundAccount {
    pub fn deposit_sol(&mut self, amount: u64) -> Result<()> {
        let new_sol_accumulated_deposit_amount = self
            .sol_accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        if self.sol_capacity_amount < new_sol_accumulated_deposit_amount {
            err!(ErrorCode::FundExceededSOLCapacityAmountError)?
        }

        self.sol_accumulated_deposit_amount = new_sol_accumulated_deposit_amount;
        self.sol_operation_reserved_amount = self
            .sol_operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DepositMetadata {
    pub wallet_provider: String,
    pub contribution_accrual_rate: u8, // 100 is 1.0
    pub expired_at: i64,
}

impl DepositMetadata {
    pub fn verify_expiration(&self) -> Result<()> {
        let current_timestamp = crate::utils::timestamp_now()?;

        if current_timestamp > self.expired_at {
            err!(ErrorCode::FundDepositMetadataSignatureExpiredError)?
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::modules::fund::price::source::*;

    use super::*;

    #[test]
    fn test_deposit_sol() {
        let mut fund = FundAccount::new_uninitialized();
        fund.initialize(0, Pubkey::new_unique());
        fund.set_sol_capacity_amount(100_000).unwrap();

        assert_eq!(fund.sol_operation_reserved_amount, 0);
        assert_eq!(fund.sol_accumulated_deposit_amount, 0);

        fund.deposit_sol(100_000).unwrap();
        assert_eq!(fund.sol_operation_reserved_amount, 100_000);
        assert_eq!(fund.sol_accumulated_deposit_amount, 100_000);

        fund.deposit_sol(100_000).unwrap_err();
    }

    #[test]
    fn test_deposit_token() {
        let mut fund = FundAccount::new_uninitialized();
        fund.initialize(0, Pubkey::new_unique());

        let mut dummy_lamports = 0u64;
        let mut dummy_data = [0u8; std::mem::size_of::<SplStakePool>()];
        let pricing_sources = &[SplStakePool::dummy_pricing_source_account_info(
            &mut dummy_lamports,
            &mut dummy_data,
        )];
        let token = SupportedTokenInfo::dummy_spl_stake_pool_token_info(pricing_sources[0].key());

        fund.add_supported_token(
            token.mint,
            token.program,
            token.decimals,
            1_000,
            token.pricing_source,
        )
        .unwrap();

        assert_eq!(fund.supported_tokens[0].operation_reserved_amount, 0);
        assert_eq!(fund.supported_tokens[0].accumulated_deposit_amount, 0);

        fund.supported_tokens[0].deposit_token(1_000).unwrap();
        assert_eq!(fund.supported_tokens[0].operation_reserved_amount, 1_000);
        assert_eq!(fund.supported_tokens[0].accumulated_deposit_amount, 1_000);

        fund.supported_tokens[0].deposit_token(1_000).unwrap_err();
    }
}
