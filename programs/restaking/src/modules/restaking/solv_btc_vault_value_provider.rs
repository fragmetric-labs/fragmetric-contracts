use anchor_lang::prelude::*;
use solv::states::VaultAccount;

use crate::errors::ErrorCode;
use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};

pub struct SolvBTCVaultValueProvider;

impl TokenValueProvider for SolvBTCVaultValueProvider {
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
        result: &mut TokenValue,
    ) -> Result<()> {
        require_eq!(pricing_source_accounts.len(), 1);

        let vault_loader = AccountLoader::<VaultAccount>::try_from(pricing_source_accounts[0])?;
        let vault = vault_loader.load()?;

        require_keys_eq!(vault.get_vst_mint(), *token_mint);

        result.numerator.clear();
        result.numerator.reserve_exact(1);

        result.numerator.extend([Asset::Token(
            vault.get_vst_mint(),
            None,
            vault
                .get_total_operation_reserved_amount_as_vst()
                .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?,
        )]);

        result.denominator = vault.get_vrt_circulating_amount();

        Ok(())
    }
}
