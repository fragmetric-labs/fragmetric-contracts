use anchor_lang::prelude::*;

use crate::modules::fund::FundAccount;
use crate::modules::pricing::{Asset, TokenValue, TokenValueProvider};

pub struct FundReceiptTokenValueProvider;

impl TokenValueProvider for FundReceiptTokenValueProvider {
    #[inline(never)]
    fn resolve_underlying_assets<'info>(
        self,
        token_mint: &Pubkey,
        pricing_source_accounts: &[&'info AccountInfo<'info>],
        result: &mut TokenValue,
    ) -> Result<()> {
        require_eq!(pricing_source_accounts.len(), 1);

        let fund_account_loader =
            AccountLoader::<FundAccount>::try_from(pricing_source_accounts[0])?;
        let fund_account = fund_account_loader.load()?;

        require_keys_eq!(fund_account.receipt_token_mint, *token_mint);

        result.numerator.clear();
        result
            .numerator
            .reserve_exact(TokenValue::MAX_NUMERATOR_SIZE);

        // sol_operation_reserved_amount + sol_operation_receivable_amount
        result.numerator.push(Asset::SOL(
            fund_account.sol.operation_reserved_amount
                + fund_account.sol.operation_receivable_amount,
        ));

        // lst_operation_reserved_amount + operation_receivable_amount
        for supported_token in fund_account.get_supported_tokens_iter() {
            result.numerator.push(Asset::Token(
                supported_token.mint,
                supported_token.pricing_source.try_deserialize()?,
                supported_token.token.operation_reserved_amount
                    + supported_token.token.operation_receivable_amount,
            ));
        }

        // nt_operation_reserved_amount
        if let Some(normalized_token) = fund_account.get_normalized_token() {
            result.numerator.push(Asset::Token(
                normalized_token.mint,
                normalized_token.pricing_source.try_deserialize()?,
                normalized_token.operation_reserved_amount,
            ));
        }

        // vrt_operation_reserved + vrt_operation_receivable_amount
        for restaking_vault in fund_account.get_restaking_vaults_iter() {
            result.numerator.push(Asset::Token(
                restaking_vault.receipt_token_mint,
                restaking_vault
                    .receipt_token_pricing_source
                    .try_deserialize()?,
                restaking_vault.receipt_token_operation_reserved_amount
                    + restaking_vault.receipt_token_operation_receivable_amount,
            ));
        }

        result.denominator = fund_account.receipt_token_supply_amount;

        Ok(())
    }
}
