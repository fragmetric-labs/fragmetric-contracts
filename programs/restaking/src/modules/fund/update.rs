use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::errors::ErrorCode;
use crate::events;

use super::*;

pub fn update_fund_account_if_needed(
    fund_account: &mut FundAccount,
    receipt_token_mint: Pubkey,
) -> Result<()> {
    fund_account.update_if_needed(receipt_token_mint);
    Ok(())
}

pub fn update_user_fund_account_if_needed(
    user_fund_account: &mut UserFundAccount,
    receipt_token_mint: Pubkey,
    user: Pubkey,
) -> Result<()> {
    user_fund_account.update_if_needed(receipt_token_mint, user);
    Ok(())
}

pub fn update_extra_account_meta_list_if_needed(
    extra_account_meta_list: &AccountInfo,
) -> Result<()> {
    ExtraAccountMetaList::update::<ExecuteInstruction>(
        &mut extra_account_meta_list.try_borrow_mut_data()?,
        &extra_account_metas()?,
    )?;
    Ok(())
}

pub fn update_sol_capacity_amount(
    fund_account: &mut FundAccount,
    receipt_token_mint: &Mint,
    capacity_amount: u64,
) -> Result<()> {
    fund_account.set_sol_capacity_amount(capacity_amount)?;
    emit_fund_manager_updated_fund_event(
        fund_account,
        receipt_token_mint,
        fund_account.receipt_token_mint,
    )
}

pub fn update_supported_token_capacity_amount(
    fund_account: &mut FundAccount,
    receipt_token_mint: &Mint,
    token: Pubkey,
    capacity_amount: u64,
) -> Result<()> {
    fund_account
        .supported_token_mut(token)?
        .set_capacity_amount(capacity_amount)?;
    emit_fund_manager_updated_fund_event(
        fund_account,
        receipt_token_mint,
        fund_account.receipt_token_mint,
    )
}

pub fn update_withdrawal_enabled_flag(
    fund_account: &mut FundAccount,
    receipt_token_mint: &Mint,
    enabled: bool,
) -> Result<()> {
    fund_account
        .withdrawal_status
        .set_withdrawal_enabled_flag(enabled);
    emit_fund_manager_updated_fund_event(
        fund_account,
        receipt_token_mint,
        fund_account.receipt_token_mint,
    )
}

pub fn update_sol_withdrawal_fee_rate(
    fund_account: &mut FundAccount,
    receipt_token_mint: &Mint,
    sol_withdrawal_fee_rate: u16,
) -> Result<()> {
    fund_account
        .withdrawal_status
        .set_sol_withdrawal_fee_rate(sol_withdrawal_fee_rate);
    emit_fund_manager_updated_fund_event(
        fund_account,
        receipt_token_mint,
        fund_account.receipt_token_mint,
    )
}

pub fn update_batch_processing_threshold(
    fund_account: &mut FundAccount,
    receipt_token_mint: &Mint,
    amount: Option<u64>,
    duration: Option<i64>,
) -> Result<()> {
    fund_account
        .withdrawal_status
        .set_batch_processing_threshold(amount, duration);
    emit_fund_manager_updated_fund_event(
        fund_account,
        receipt_token_mint,
        fund_account.receipt_token_mint,
    )
}

pub fn add_supported_token(
    fund_account: &mut FundAccount,
    receipt_token_mint: &Mint,
    supported_token_mint: &Mint,
    supported_token_mint_address: Pubkey,
    supported_token_program: Pubkey,
    capacity_amount: u64,
    pricing_source: TokenPricingSource,
    pricing_sources: &[AccountInfo],
) -> Result<()> {
    fund_account.add_supported_token(
        supported_token_mint_address,
        supported_token_program,
        supported_token_mint.decimals,
        capacity_amount,
        pricing_source,
    )?;
    fund_account.update_token_prices(pricing_sources)?;
    emit_fund_manager_updated_fund_event(
        fund_account,
        receipt_token_mint,
        fund_account.receipt_token_mint,
    )
}

pub fn update_prices(
    fund_account: &mut FundAccount,
    receipt_token_mint: &Mint,
    pricing_sources: &[AccountInfo],
) -> Result<()> {
    fund_account.update_token_prices(pricing_sources)?;

    let receipt_token_price = fund_account.receipt_token_sol_value_per_token(
        receipt_token_mint.decimals,
        receipt_token_mint.supply,
    )?;

    emit!(events::OperatorUpdatedFundPrice {
        receipt_token_mint: fund_account.receipt_token_mint,
        fund_account: FundAccountInfo::new(
            fund_account,
            receipt_token_price,
            receipt_token_mint.supply,
        ),
    });

    Ok(())
}

/// Receipt token price & supply might be outdated.
fn emit_fund_manager_updated_fund_event(
    fund_account: &FundAccount,
    receipt_token_mint: &Mint,
    receipt_token_mint_address: Pubkey,
) -> Result<()> {
    let receipt_token_price = fund_account.receipt_token_sol_value_per_token(
        receipt_token_mint.decimals,
        receipt_token_mint.supply,
    )?;

    emit!(events::FundManagerUpdatedFund {
        receipt_token_mint: receipt_token_mint_address,
        fund_account: FundAccountInfo::new(
            fund_account,
            receipt_token_price,
            receipt_token_mint.supply,
        ),
    });

    Ok(())
}

impl SupportedTokenInfo {
    pub fn set_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        if capacity_amount < self.accumulated_deposit_amount {
            err!(ErrorCode::FundInvalidUpdateError)?
        }
        self.capacity_amount = capacity_amount;

        Ok(())
    }
}

impl FundAccount {
    pub fn set_sol_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        if capacity_amount < self.sol_accumulated_deposit_amount {
            err!(ErrorCode::FundInvalidUpdateError)?
        }
        self.sol_capacity_amount = capacity_amount;

        Ok(())
    }

    pub fn add_supported_token(
        &mut self,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
    ) -> Result<()> {
        if self.supported_tokens.iter().any(|info| info.mint == mint) {
            err!(ErrorCode::FundAlreadySupportedTokenError)?
        }
        let token_info =
            SupportedTokenInfo::new(mint, program, decimals, capacity_amount, pricing_source);
        self.supported_tokens.push(token_info);

        Ok(())
    }
}

impl WithdrawalStatus {
    pub fn set_sol_withdrawal_fee_rate(&mut self, sol_withdrawal_fee_rate: u16) {
        self.sol_withdrawal_fee_rate = sol_withdrawal_fee_rate;
    }

    pub fn set_withdrawal_enabled_flag(&mut self, flag: bool) {
        self.withdrawal_enabled_flag = flag;
    }

    pub fn set_batch_processing_threshold(&mut self, amount: Option<u64>, duration: Option<i64>) {
        if let Some(amount) = amount {
            self.batch_processing_threshold_amount = amount;
        }
        if let Some(duration) = duration {
            self.batch_processing_threshold_duration = duration;
        }
    }
}

impl UserFundAccount {
    pub fn set_receipt_token_amount(&mut self, total_amount: u64) {
        self.receipt_token_amount = total_amount;
    }
}

#[cfg(test)]
mod tests {
    use crate::modules::fund::price::source::*;

    use super::*;

    #[test]
    fn test_update_fund() {
        let mut fund = FundAccount::new_uninitialized();
        fund.initialize(0, Pubkey::new_unique());

        assert_eq!(fund.sol_capacity_amount, 0);
        assert_eq!(fund.withdrawal_status.sol_withdrawal_fee_rate, 0);
        assert!(fund.withdrawal_status.withdrawal_enabled_flag);
        assert_eq!(fund.withdrawal_status.batch_processing_threshold_amount, 0);
        assert_eq!(
            fund.withdrawal_status.batch_processing_threshold_duration,
            0
        );

        let new_sol_capacity_amount = 1_000_000_000 * 60_000;
        fund.set_sol_capacity_amount(new_sol_capacity_amount)
            .unwrap();
        assert_eq!(fund.sol_capacity_amount, new_sol_capacity_amount);

        let new_sol_withdrawal_fee_rate = 20;
        fund.withdrawal_status
            .set_sol_withdrawal_fee_rate(new_sol_withdrawal_fee_rate);
        assert_eq!(
            fund.withdrawal_status.sol_withdrawal_fee_rate,
            new_sol_withdrawal_fee_rate
        );

        fund.withdrawal_status.set_withdrawal_enabled_flag(false);
        assert!(!fund.withdrawal_status.withdrawal_enabled_flag);

        let new_amount = 10;
        let new_duration = 10;
        fund.withdrawal_status
            .set_batch_processing_threshold(Some(new_amount), None);
        assert_eq!(
            fund.withdrawal_status.batch_processing_threshold_amount,
            new_amount
        );
        assert_eq!(
            fund.withdrawal_status.batch_processing_threshold_duration,
            0
        );

        fund.withdrawal_status
            .set_batch_processing_threshold(None, Some(new_duration));
        assert_eq!(
            fund.withdrawal_status.batch_processing_threshold_amount,
            new_amount
        );
        assert_eq!(
            fund.withdrawal_status.batch_processing_threshold_duration,
            new_duration
        );
    }

    #[test]
    fn test_update_token() {
        let mut fund = FundAccount::new_uninitialized();
        fund.initialize(0, Pubkey::new_unique());

        let mut dummy_lamports = 0u64;
        let mut dummy_data = [0u8; std::mem::size_of::<SplStakePool>()];
        let mut dummy_lamports2 = 0u64;
        let mut dummy_data2 = [0u8; 8 + MarinadeStakePool::INIT_SPACE];
        let pricing_sources = &[
            SplStakePool::dummy_pricing_source_account_info(&mut dummy_lamports, &mut dummy_data),
            MarinadeStakePool::dummy_pricing_source_account_info(
                &mut dummy_lamports2,
                &mut dummy_data2,
            ),
        ];
        let token1 = SupportedTokenInfo::dummy_spl_stake_pool_token_info(pricing_sources[0].key());
        let token2 =
            SupportedTokenInfo::dummy_marinade_stake_pool_token_info(pricing_sources[1].key());

        fund.add_supported_token(
            token1.mint,
            token1.program,
            token1.decimals,
            token1.capacity_amount,
            token1.pricing_source,
        )
        .unwrap();
        fund.add_supported_token(
            token2.mint,
            token2.program,
            token2.decimals,
            token2.capacity_amount,
            token2.pricing_source,
        )
        .unwrap();
        assert_eq!(fund.supported_tokens.len(), 2);
        assert_eq!(
            fund.supported_tokens[0].capacity_amount,
            token1.capacity_amount
        );

        let new_token1_capacity_amount = 1_000_000_000 * 3000;
        fund.supported_token_mut(token1.mint)
            .unwrap()
            .set_capacity_amount(new_token1_capacity_amount)
            .unwrap();
        assert_eq!(
            fund.supported_tokens[0].capacity_amount,
            new_token1_capacity_amount
        );
    }
}
