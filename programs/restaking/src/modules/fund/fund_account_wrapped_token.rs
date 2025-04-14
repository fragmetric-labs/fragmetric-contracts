use anchor_lang::prelude::*;
use bytemuck::Zeroable;

use crate::errors::ErrorCode;

const FUND_ACCOUNT_WRAPPED_TOKEN_MAX_HOLDERS: usize = 30;

#[zero_copy]
#[repr(C)]
pub(super) struct WrappedToken {
    pub mint: Pubkey,
    pub program: Pubkey,
    pub decimals: u8,
    pub enabled: u8,
    num_holders: u8,
    _padding: [u8; 5],
    pub supply: u64,
    /// List of wrapped token holders who will receive reward for wrapped tokens
    holders: [Pubkey; FUND_ACCOUNT_WRAPPED_TOKEN_MAX_HOLDERS],
    _reserved: [u8; 1024],
}

impl WrappedToken {
    pub fn initialize(
        &mut self,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        supply: u64,
    ) -> Result<()> {
        require_eq!(self.enabled, 0);

        *self = Zeroable::zeroed();

        self.enabled = 1;
        self.mint = mint;
        self.program = program;
        self.decimals = decimals;
        self.supply = supply;

        Ok(())
    }

    pub fn get_holders_iter(&self) -> impl Iterator<Item = &Pubkey> {
        self.holders[..self.num_holders as usize].iter()
    }

    pub fn add_holder(&mut self, wrapped_token_account: Pubkey) -> Result<()> {
        if self
            .get_holders_iter()
            .any(|holder| *holder == wrapped_token_account)
        {
            err!(ErrorCode::FundWrappedTokenHolderAlreadyRegisteredError)?
        }

        require_gt!(
            FUND_ACCOUNT_WRAPPED_TOKEN_MAX_HOLDERS,
            self.num_holders as usize,
            ErrorCode::FundExceededMaxWrappedTokenHoldersError,
        );

        self.holders[self.num_holders as usize] = wrapped_token_account;
        self.num_holders += 1;

        Ok(())
    }
}
