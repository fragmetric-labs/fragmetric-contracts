use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::errors::ErrorCode;
use crate::utils::AccountInfoExt;

use super::ValidateVault;

pub(in crate::modules) struct VirtualVaultService;

impl ValidateVault for VirtualVaultService {
    fn validate_vault<'info>(
        vault_vault_supported_token_account: &InterfaceAccount<TokenAccount>,
        vault_account: &'info AccountInfo<'info>,
        vault_supported_token_mint: &InterfaceAccount<Mint>,
        vault_receipt_token_mint: &InterfaceAccount<Mint>,
        fund_account: &AccountInfo,
    ) -> Result<()> {
        // Verify whether vault's supported token account conforms to the ATA specification
        require_keys_eq!(
            vault_vault_supported_token_account.key(),
            get_associated_token_address_with_program_id(
                vault_account.key,
                &vault_supported_token_mint.key(),
                &Token::id(),
            )
        );

        // validate vault address
        let vault_address =
            Self::find_vault_address(&vault_receipt_token_mint.key(), fund_account.key)
                .vault_address;
        require_keys_eq!(*vault_account.key, vault_address);
        require_eq!(vault_account.is_initialized(), false);

        require_keys_eq!(
            vault_supported_token_mint.key(),
            vault_receipt_token_mint.key(),
        );

        require_eq!(vault_receipt_token_mint.supply, 0);
        require!(
            vault_receipt_token_mint.mint_authority.is_none(),
            ErrorCode::RestakingVaultAuthorityNotMatchedError
        );

        Ok(())
    }
}

impl VirtualVaultService {
    pub fn find_vault_address<'a>(
        vault_receipt_token_mint: &'a Pubkey,
        fund_account: &'a Pubkey,
    ) -> VirtualVaultAddress<'a> {
        VirtualVaultAddress::new(vault_receipt_token_mint, fund_account)
    }
}

pub(in crate::modules) struct VirtualVaultAddress<'a> {
    vault_address: Pubkey,
    vault_receipt_token_mint: &'a Pubkey,
    fund_account: &'a Pubkey,
    bump: u8,
}

impl<'a> VirtualVaultAddress<'a> {
    const SEED: &'static [u8] = b"virtual_vault";

    fn new(vault_receipt_token_mint: &'a Pubkey, fund_account: &'a Pubkey) -> Self {
        let (vault_address, bump) = Pubkey::find_program_address(
            &[
                Self::SEED,
                vault_receipt_token_mint.as_ref(),
                fund_account.as_ref(),
            ],
            &crate::ID,
        );
        Self {
            vault_address,
            vault_receipt_token_mint,
            fund_account,
            bump,
        }
    }

    pub fn get_seeds(&self) -> [&[u8]; 4] {
        [
            Self::SEED,
            self.vault_receipt_token_mint.as_ref(),
            self.fund_account.as_ref(),
            core::slice::from_ref(&self.bump),
        ]
    }
}
