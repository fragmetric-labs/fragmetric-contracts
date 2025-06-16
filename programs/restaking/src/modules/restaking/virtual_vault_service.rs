use anchor_lang::prelude::*;
use anchor_spl::{associated_token, token_interface::Mint};

use crate::errors::ErrorCode;

use super::ValidateVault;

#[allow(dead_code)]
pub(in crate::modules) struct VirtualVaultService<'info> {
    vault_account: &'info AccountInfo<'info>,
    vault_receipt_token_mint: &'info AccountInfo<'info>,
}

impl ValidateVault for VirtualVaultService<'_> {
    fn validate_vault<'info>(
        vault_account: &'info AccountInfo<'info>,
        _vault_supported_token_mint: &AccountInfo,
        vault_receipt_token_mint: &'info AccountInfo<'info>,
        fund_account: &AccountInfo,
    ) -> Result<()> {
        // validate vault address
        let vault_address =
            Self::find_vault_address(vault_receipt_token_mint.key, fund_account.key).vault_address;
        require_keys_eq!(*vault_account.key, vault_address);
        require_keys_eq!(*vault_account.owner, System::id());

        // validate vrt mint
        let vault_receipt_token_mint_data =
            InterfaceAccount::<Mint>::try_from(vault_receipt_token_mint)?;

        require_eq!(vault_receipt_token_mint_data.supply, 0);
        require!(
            vault_receipt_token_mint_data.mint_authority.is_none(),
            ErrorCode::FundRestakingVaultAuthorityNotMatchedError
        );

        Ok(())
    }
}

impl<'info> VirtualVaultService<'info> {
    #[allow(dead_code)]
    pub fn new(
        vault_account: &'info AccountInfo<'info>,
        vault_receipt_token_mint: &'info AccountInfo<'info>,
    ) -> Result<Self> {
        require_keys_eq!(*vault_account.owner, System::id());

        Ok(Self {
            vault_account,
            vault_receipt_token_mint,
        })
    }

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
            std::slice::from_ref(&self.bump),
        ]
    }
}
