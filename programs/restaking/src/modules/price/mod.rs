use anchor_lang::prelude::*;

mod source;

pub use source::*;

use crate::errors::ErrorCode;
use crate::modules::fund::{FundAccount, SupportedTokenInfo};

impl SupportedTokenInfo {
    /// Simply it returns 10^token_decimals.
    fn token_lamports_per_token(&self) -> Result<u64> {
        10u64
            .checked_pow(self.decimals as u32)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    pub fn calculate_sol_from_tokens(&self, token_amount: u64) -> Result<u64> {
        crate::utils::proportional_amount(
            token_amount,
            self.price,
            self.token_lamports_per_token()?,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }
}

impl FundAccount {
    // TODO move to fund module
    pub fn update_token_prices<'info>(
        &mut self,
        sources: &'info [AccountInfo<'info>],
    ) -> Result<()> {
        for token in self.supported_tokens_iter_mut() {
            let token_lamports_per_token = token.token_lamports_per_token()?;
            let token_price_calculator = TokenPriceCalculatorFactory
                .to_calculator_checked(&token.pricing_source, sources)?;
            token.price = token_price_calculator.calculate_token_price(token_lamports_per_token)?;
        }

        Ok(())
    }

    pub fn receipt_token_sol_value_per_token(
        &self,
        receipt_token_decimals: u8,
        receipt_token_total_supply: u64,
    ) -> Result<u64> {
        self.receipt_token_sol_value_for(
            10u64
                .checked_pow(receipt_token_decimals as u32)
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
            receipt_token_total_supply,
        )
    }

    pub fn receipt_token_mint_amount_for(
        &self,
        sol_amount: u64,
        receipt_token_total_supply: u64,
    ) -> Result<u64> {
        crate::utils::proportional_amount(
            sol_amount,
            receipt_token_total_supply,
            self.assets_total_sol_value()?,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    pub fn receipt_token_sol_value_for(
        &self,
        receipt_token_amount: u64,
        receipt_token_total_supply: u64,
    ) -> Result<u64> {
        crate::utils::proportional_amount(
            receipt_token_amount,
            self.assets_total_sol_value()?,
            receipt_token_total_supply,
        )
        .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
    }

    // TODO move to fund module
    pub fn assets_total_sol_value(&self) -> Result<u64> {
        // TODO: need to add the sum(operating sol/tokens) after supported_restaking_protocols add
        self.supported_tokens_iter()
            .try_fold(self.sol_operation_reserved_amount, |sum, token| {
                sum.checked_add(token.calculate_sol_from_tokens(token.operation_reserved_amount)?)
                    .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))
            })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_update_price() {
//         let mut fund = FundAccount::new_uninitialized();
//         fund.initialize(0, Pubkey::new_unique());

//         let mut dummy_lamports = 0u64;
//         let mut dummy_data = [0u8; std::mem::size_of::<SplStakePool>()];
//         let mut dummy_lamports2 = 0u64;
//         let mut dummy_data2 = [0u8; 8 + MarinadeStakePool::INIT_SPACE];
//         let pricing_sources = &[
//             // 1 Token = 1.4 SOL
//             SplStakePool::placeholder(&mut dummy_lamports, &mut dummy_data),
//             // 1 Token = 1.2 SOL
//             MarinadeStakePool::placeholder(&mut dummy_lamports2, &mut dummy_data2),
//         ];
//         let token1 = SupportedTokenInfo::dummy_spl_stake_pool_token_info(pricing_sources[0].key());
//         let token2 =
//             SupportedTokenInfo::dummy_marinade_stake_pool_token_info(pricing_sources[1].key());

//         fund.add_supported_token(
//             token1.mint,
//             token1.program,
//             token1.decimals,
//             token1.capacity_amount,
//             token1.pricing_source,
//         )
//         .unwrap();
//         fund.add_supported_token(
//             token2.mint,
//             token2.program,
//             token2.decimals,
//             token2.capacity_amount,
//             token2.pricing_source,
//         )
//         .unwrap();

//         fund.update_token_prices(pricing_sources).unwrap();

//         assert_eq!(fund.supported_tokens[0].price, 1_400_000_000);
//         assert_eq!(fund.supported_tokens[1].price, 1_200_000_000);
//     }
// }
