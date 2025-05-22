use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::errors::ErrorCode;

pub struct VirtualRestakingVaultService<'info> {
    vault_account: &'info AccountInfo<'info>,
}

impl<'info> VirtualRestakingVaultService<'info> {
    pub fn new(vault_account: &'info AccountInfo<'info>) -> Result<Self> {
        require_keys_eq!(*vault_account.owner, System::id());

        Ok(Self { vault_account })
    }

    pub fn validate_vault(
        vault_account: &AccountInfo,
        vault_receipt_token_mint: &AccountInfo,
    ) -> Result<()> {
        require_keys_eq!(*vault_account.owner, System::id()); // do again? it is checked at new method -> but should also be here, because this method doesn't access new().

        let vault_receipt_token_mint_data =
            Mint::try_deserialize(&mut &vault_receipt_token_mint.data.borrow()[..])?;

        require_eq!(vault_receipt_token_mint_data.supply, 0);
        require!(
            vault_receipt_token_mint_data.mint_authority.is_none(),
            ErrorCode::FundRestakingVaultAuthorityNotMatchedError
        );

        Ok(())
    }
}
