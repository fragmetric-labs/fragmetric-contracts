use super::ValidateVault;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;

pub(in crate::modules) struct DriftVaultService;

impl ValidateVault for DriftVaultService {
    fn validate_vault<'info>(
        vault_vault_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        vault_account: &'info AccountInfo<'info>,
        vault_supported_token_mint: &InterfaceAccount<anchor_spl::token_interface::Mint>,
        vault_receipt_token_mint: &InterfaceAccount<anchor_spl::token_interface::Mint>,
        fund_account: &AccountInfo,
    ) -> anchor_lang::Result<()> {
        // Verify whether vault's supported token account conforms to the Drift Vault's token account specification
        let (expected_token_account_address, _) = Pubkey::find_program_address(
            &[b"vault_token_account".as_ref(), vault_account.key.as_ref()],
            &drift_vault_cpi::drift_vault::ID,
        );

        require_keys_eq!(
            vault_vault_supported_token_account.key(),
            expected_token_account_address,
        );

        Ok(())
    }
}
