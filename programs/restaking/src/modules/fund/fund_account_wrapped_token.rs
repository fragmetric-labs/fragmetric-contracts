use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
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

    /// Total amount of wrapped token held by holders.
    /// This value is always equal to the sum of each wrapped token balance of holders.
    pub total_amount_held_by_holders: u64,

    /// An amount of wrapped token that is not held by any holders.
    /// This value is not always equal to `supply - total_amount_held_by_holders`.
    ///
    /// Wrapped token amount of holders are updated via snapshot during operation cycle.
    /// Unless all snapshots are captured in a single instruction,
    /// their total amount might be inaccurate due to concurrency.
    ///
    /// Therefore, retained_amount is adjusted to max(0, supply - total_amount_held_by_holders)
    pub retained_amount: u64,

    /// List of wrapped token holders who will receive reward for their wrapped token balance.
    holders: [WrappedTokenHolder; FUND_ACCOUNT_WRAPPED_TOKEN_MAX_HOLDERS],

    _reserved: [u8; 768], // 768 = 32 * 24
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

    /// returns old_wrapped_token_retained_amount
    pub fn reload_supply(
        &mut self,
        wrapped_token_mint: &mut InterfaceAccount<Mint>,
    ) -> Result<u64> {
        require_keys_eq!(self.mint, wrapped_token_mint.key());

        wrapped_token_mint.reload()?;
        self.supply = wrapped_token_mint.supply;
        let old_wrapped_token_retained_amount = self.update_retained_amount();

        Ok(old_wrapped_token_retained_amount)
    }

    pub fn get_holders_iter(&self) -> impl Iterator<Item = &WrappedTokenHolder> {
        self.holders[..self.num_holders as usize].iter()
    }

    pub fn get_holders_iter_mut(&mut self) -> impl Iterator<Item = &mut WrappedTokenHolder> {
        self.holders[..self.num_holders as usize].iter_mut()
    }

    pub fn get_holder_mut(
        &mut self,
        wrapped_token_account: &Pubkey,
    ) -> Result<&mut WrappedTokenHolder> {
        Ok(self
            .get_holders_iter_mut()
            .find(|holder| holder.token_account == *wrapped_token_account)
            .ok_or_else(|| ErrorCode::FundWrappedTokenHolderNotFoundError)?)
    }

    pub fn add_holder(&mut self, wrapped_token_account: Pubkey) -> Result<()> {
        if self
            .get_holders_iter()
            .any(|holder| holder.token_account == wrapped_token_account)
        {
            err!(ErrorCode::FundWrappedTokenHolderAlreadyRegisteredError)?
        }

        require_gt!(
            FUND_ACCOUNT_WRAPPED_TOKEN_MAX_HOLDERS,
            self.num_holders as usize,
            ErrorCode::FundExceededMaxWrappedTokenHoldersError,
        );

        self.holders[self.num_holders as usize].initialize(wrapped_token_account);
        self.num_holders += 1;

        Ok(())
    }

    /// returns [old_wrapped_token_holder_amount, old_wrapped_token_retained_amount]
    pub fn remove_holder(&mut self, wrapped_token_account: &Pubkey) -> Result<(u64, u64)> {
        // find holder
        let (idx, holder) = self
            .get_holders_iter_mut()
            .enumerate()
            .find(|(_, holder)| holder.token_account == *wrapped_token_account)
            .ok_or_else(|| error!(ErrorCode::FundWrappedTokenHolderNotFoundError))?;

        // deduct the wrapped token amount of this holder
        let old_wrapped_token_holder_amount = holder.update_amount(0);
        self.total_amount_held_by_holders -= old_wrapped_token_holder_amount;
        let old_wrapped_token_retained_amount = self.update_retained_amount();

        // remove holder: list of holders need not preserve the order
        self.num_holders -= 1;
        self.holders.swap(idx, self.num_holders as usize);
        self.holders[self.num_holders as usize] = Zeroable::zeroed();

        Ok((
            old_wrapped_token_holder_amount,
            old_wrapped_token_retained_amount,
        ))
    }

    /// returns [old_wrapped_token_holder_amount, old_wrapped_token_retained_amount]
    pub fn update_holder_amount(
        &mut self,
        wrapped_token_account: &Pubkey,
        wrapped_token_amount: u64,
    ) -> Result<(u64, u64)> {
        let holder = self.get_holder_mut(wrapped_token_account)?;

        let old_wrapped_token_holder_amount = holder.update_amount(wrapped_token_amount);
        self.total_amount_held_by_holders -= old_wrapped_token_holder_amount;
        self.total_amount_held_by_holders += wrapped_token_amount;
        let old_wrapped_token_receivable_amount = self.update_retained_amount();

        Ok((
            old_wrapped_token_holder_amount,
            old_wrapped_token_receivable_amount,
        ))
    }

    /// returns old_wrapped_token_retained_amount
    fn update_retained_amount(&mut self) -> u64 {
        let old_retained_amount = self.retained_amount;
        self.retained_amount = self
            .supply
            .saturating_sub(self.total_amount_held_by_holders);

        old_retained_amount
    }
}

#[zero_copy]
#[repr(C)]
pub(super) struct WrappedTokenHolder {
    pub token_account: Pubkey,
    pub amount: u64,
}

impl WrappedTokenHolder {
    fn initialize(&mut self, token_account: Pubkey) {
        *self = Zeroable::zeroed();

        self.token_account = token_account;
    }

    /// returns old_amount
    fn update_amount(&mut self, amount: u64) -> u64 {
        let old_amount = self.amount;
        self.amount = amount;

        old_amount
    }
}
