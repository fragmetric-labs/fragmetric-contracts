use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::system_program;
use anchor_spl::associated_token;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount};
use jito_vault_core::{
    config::Config, vault::Vault, vault_operator_delegation::VaultOperatorDelegation,
    vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket,
    vault_update_state_tracker::VaultUpdateStateTracker,
};

use crate::constants::{JITO_VAULT_CONFIG_ADDRESS, JITO_VAULT_PROGRAM_ID};
use crate::errors::ErrorCode;
use crate::modules::pricing::{PricingService, TokenValue, TokenValueProvider};
use crate::modules::restaking::JitoRestakingVaultValueProvider;
use crate::utils;
use crate::utils::AccountInfoExt;

use super::ValidateVault;

pub(in crate::modules) struct JitoRestakingVaultService<'info> {
    vault_program: &'info AccountInfo<'info>,
    vault_config_account: &'info AccountInfo<'info>,
    vault_account: &'info AccountInfo<'info>,
    num_operators: u64,
    current_slot: u64,
    current_epoch: u64,
    last_update_epoch: u64,
    epoch_length: u64,
    vault_program_fee_wallet: Pubkey,
}

impl ValidateVault for JitoRestakingVaultService<'_> {
    #[inline(never)]
    fn validate_vault(
        vault_account: &AccountInfo,
        vault_supported_token_mint: &InterfaceAccount<Mint>,
        vault_receipt_token_mint: &InterfaceAccount<Mint>,
        fund_account: &AccountInfo,
    ) -> Result<()> {
        let data = &Self::borrow_account_data(vault_account)?;
        let vault = Self::deserialize_account_data::<Vault>(data)?;

        require_keys_eq!(vault.supported_mint, vault_supported_token_mint.key());
        require_keys_eq!(vault.vrt_mint, vault_receipt_token_mint.key());
        require_keys_eq!(vault.delegation_admin, fund_account.key());

        Ok(())
    }
}

impl<'info> JitoRestakingVaultService<'info> {
    pub fn new(
        vault_program: &'info AccountInfo<'info>,
        vault_config_account: &'info AccountInfo<'info>,
        vault_account: &'info AccountInfo<'info>,
    ) -> Result<Self> {
        let vault_config_data = &Self::borrow_account_data(vault_config_account)?;
        let vault_config = Self::deserialize_account_data::<Config>(vault_config_data)?;
        let vault_data = &Self::borrow_account_data(vault_account)?;
        let vault = Self::deserialize_account_data::<Vault>(vault_data)?;

        require_keys_eq!(JITO_VAULT_PROGRAM_ID, vault_program.key());
        require_keys_eq!(JITO_VAULT_CONFIG_ADDRESS, vault_config_account.key());

        let num_operators = vault.operator_count();
        let current_slot = Clock::get()?.slot;
        let last_update_slot = vault.last_full_state_update_slot();
        let epoch_length = vault_config.epoch_length();
        let current_epoch = current_slot
            .checked_div(epoch_length)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;
        let last_update_epoch = last_update_slot
            .checked_div(epoch_length)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(Self {
            vault_program,
            vault_config_account,
            vault_account,
            num_operators,
            current_slot,
            current_epoch,
            last_update_epoch,
            epoch_length,
            vault_program_fee_wallet: vault_config.program_fee_wallet,
        })
    }

    /// returns [delegation_index, staked_amount, enqueued_for_cooldown_amount, cooling_down_amount]
    /// in other words [delegation_index, delegated_amount, undelegation_requested_amount, undelegating_amount]
    pub fn validate_vault_operator_delegation(
        vault_operator_delegation: &AccountInfo,
        vault_account: &AccountInfo,
        operator: &AccountInfo,
    ) -> Result<(u64, u64, u64, u64)> {
        let data = &Self::borrow_account_data(vault_operator_delegation)?;
        let delegation = Self::deserialize_account_data::<VaultOperatorDelegation>(data)?;

        require_keys_eq!(delegation.vault, vault_account.key());
        require_keys_eq!(delegation.operator, operator.key());

        Ok((
            delegation.index(),
            delegation.delegation_state.staked_amount(),
            delegation.delegation_state.enqueued_for_cooldown_amount(),
            delegation.delegation_state.cooling_down_amount(),
        ))
    }

    #[inline(always)]
    fn borrow_account_data<'a, 'b>(
        account: &'a AccountInfo<'b>,
    ) -> Result<std::cell::Ref<'a, &'b mut [u8]>> {
        require_keys_eq!(*account.owner, JITO_VAULT_PROGRAM_ID);
        Ok(account
            .data
            .try_borrow()
            .map_err(|_| ProgramError::AccountBorrowFailed)?)
    }

    #[inline(always)]
    fn deserialize_account_data<'a, T: jito_bytemuck::AccountDeserialize>(
        data: &'a std::cell::Ref<&mut [u8]>,
    ) -> Result<&'a T> {
        Ok(T::try_from_slice_unchecked(data)?)
    }

    fn find_vault_update_state_tracker_address(&self) -> Pubkey {
        Pubkey::find_program_address(
            &[
                b"vault_update_state_tracker",
                self.vault_account.key.as_ref(),
                &self.current_epoch.to_le_bytes(),
            ],
            self.vault_program.key,
        )
        .0
    }

    fn find_vault_operator_delegation_address(&self, operator: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[
                b"vault_operator_delegation",
                self.vault_account.key.as_ref(),
                operator.as_ref(),
            ],
            self.vault_program.key,
        )
        .0
    }

    /// gives max fee/expense ratio during a cycle of circulation
    /// returns (numerator, denominator)
    pub fn get_max_cycle_fee(vault_account: &AccountInfo) -> Result<(u64, u64)> {
        let data = &Self::borrow_account_data(vault_account)?;
        let vault = Self::deserialize_account_data::<Vault>(data)?;

        Ok((
            vault.deposit_fee_bps() as u64 // vault's deposit fee
            + vault.withdrawal_fee_bps() as u64 // vault's withdrawal fee
            + vault.program_fee_bps() as u64, // vault program's withdrawal fee
            10_000,
        ))
    }

    fn is_vault_up_to_date(&self) -> bool {
        self.last_update_epoch >= self.current_epoch
    }

    /// * (0) vault_program
    /// * (1) vault_config_account(writable)
    /// * (2) vault_account(writable)
    pub fn find_accounts_to_new(
        vault_address: Pubkey,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        Ok([
            (JITO_VAULT_PROGRAM_ID, false),
            (JITO_VAULT_CONFIG_ADDRESS, true), // well, idk why but deposit ix needs it to be writable.
            (vault_address, true),
        ]
        .into_iter())
    }

    pub fn get_current_epoch(&self) -> u64 {
        self.current_epoch
    }

    /// returns vault update indices in proper order.
    /// For example, when start index = 3,
    /// then order is [3, 4, ..., N-1, 0, 1, 2].
    pub fn get_ordered_vault_update_indices(&self) -> Vec<u64> {
        if self.num_operators == 0 {
            return vec![];
        }

        let start_index = self.current_epoch % self.num_operators;
        let mut indices: Vec<_> = (0..self.num_operators).collect();
        indices.rotate_left(start_index as usize);
        indices
    }

    /// * (0) vault_program
    /// * (1) vault_config_account(writable)
    /// * (2) vault_account(writable)
    /// * (3) vault_update_state_tracker(writable)
    pub fn find_accounts_to_update_vault_state(
        &self,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let accounts = Self::find_accounts_to_new(self.vault_account.key())?
            .chain([(self.find_vault_update_state_tracker_address(), true)]);

        Ok(accounts)
    }

    /// initialize vault_update_state_tracker if needed
    pub fn initialize_vault_update_state_tracker_if_needed(
        &self,
        // fixed
        system_program: &Program<'info, System>,
        vault_update_state_tracker: &AccountInfo<'info>,

        // variant
        payer: &AccountInfo<'info>,
        payer_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        if self.is_vault_up_to_date() {
            msg!(
                "RESTAKE#jito vault_update_state_tracker is up-to-date: current_epoch={}",
                self.current_epoch
            );

            return Ok(());
        }

        if vault_update_state_tracker.is_initialized() {
            msg!(
                "RESTAKE#jito vault_update_state_tracker already initialized: current_epoch={}",
                self.current_epoch
            );

            return Ok(());
        }

        let initialize_vault_update_state_tracker_ix =
            jito_vault_sdk::sdk::initialize_vault_update_state_tracker(
                self.vault_program.key,
                self.vault_config_account.key,
                self.vault_account.key,
                vault_update_state_tracker.key,
                payer.key,
                jito_vault_sdk::instruction::WithdrawalAllocationMethod::Greedy,
            );

        invoke_signed(
            &initialize_vault_update_state_tracker_ix,
            &[
                self.vault_program.to_account_info(),
                self.vault_config_account.to_account_info(),
                self.vault_account.to_account_info(),
                vault_update_state_tracker.to_account_info(),
                payer.to_account_info(),
                system_program.to_account_info(),
            ],
            payer_seeds,
        )?;

        msg!(
            "RESTAKE#jito vault_update_state_tracker initialized: current_epoch={}",
            self.current_epoch
        );

        Ok(())
    }

    /// * vault_operator_delegation(writable)
    /// * operator
    pub fn find_accounts_to_update_delegation_state(
        &self,
        operator: Pubkey,
    ) -> impl Iterator<Item = (Pubkey, bool)> {
        let accounts = [
            (self.find_vault_operator_delegation_address(&operator), true),
            (operator, false),
        ];

        accounts.into_iter()
    }

    /// updates vault_operator_delegation if needed
    /// since anyone can update vault_operator_delegation,
    /// it might already be updated.
    ///
    /// returns [staked_amount, enqueued_for_cooldown_amount, cooling_down_amount]
    /// in other words [delegated_amount, undelegation_requested_amount, undelegating_amount]
    pub fn update_operator_delegation_state_if_needed(
        &self,
        // fixed
        vault_update_state_tracker: &AccountInfo<'info>,

        // variant
        vault_operator_delegation: &AccountInfo<'info>,
        operator: &AccountInfo<'info>,

        next_index: u64,
    ) -> Result<(u64, u64, u64)> {
        let delegation_index = {
            let data = &Self::borrow_account_data(vault_operator_delegation)?;
            let delegation = Self::deserialize_account_data::<VaultOperatorDelegation>(data)?;
            delegation.index()
        };
        require_eq!(delegation_index, next_index);

        let next_update_index = self.get_vault_update_next_index(vault_update_state_tracker)?;
        if next_update_index.is_some_and(|index| index == next_index) {
            let crank_vault_update_state_tracker_ix =
                jito_vault_sdk::sdk::crank_vault_update_state_tracker(
                    self.vault_program.key,
                    self.vault_config_account.key,
                    self.vault_account.key,
                    operator.key,
                    vault_operator_delegation.key,
                    vault_update_state_tracker.key,
                );

            invoke_signed(
                &crank_vault_update_state_tracker_ix,
                &[
                    self.vault_program.to_account_info(),
                    self.vault_config_account.to_account_info(),
                    self.vault_account.to_account_info(),
                    operator.to_account_info(),
                    vault_operator_delegation.to_account_info(),
                    vault_update_state_tracker.to_account_info(),
                ],
                &[],
            )?;
        }

        let data = &Self::borrow_account_data(vault_operator_delegation)?;
        let delegation = Self::deserialize_account_data::<VaultOperatorDelegation>(data)?;

        let staked_amount = delegation.delegation_state.staked_amount();
        let enqueued_for_cooldown_amount =
            delegation.delegation_state.enqueued_for_cooldown_amount();
        let cooling_down_amount = delegation.delegation_state.cooling_down_amount();

        msg!("RESTAKE#jito vault_operator_delegation updated: current_epoch={}, operator={}, index={}, staked_amount={}, enqueued_for_cooldown_amount={}, cooling_down_amount={}", self.current_epoch, operator.key, delegation_index, staked_amount, enqueued_for_cooldown_amount, cooling_down_amount);

        Ok((
            staked_amount,
            enqueued_for_cooldown_amount,
            cooling_down_amount,
        ))
    }

    /// gets index of the delegation to be updated at next.
    /// this returns `None` if vault is up to date (no update needed),
    /// or all delegations are updated.
    fn get_vault_update_next_index(
        &self,
        vault_update_state_tracker: &AccountInfo,
    ) -> Result<Option<u64>> {
        if self.is_vault_up_to_date() {
            return Ok(None);
        }

        if self.num_operators == 0 {
            return Ok(None);
        }

        let data = &Self::borrow_account_data(vault_update_state_tracker)?;
        let tracker = Self::deserialize_account_data::<VaultUpdateStateTracker>(data)?;
        let start_index = tracker.ncn_epoch() % self.num_operators;

        if tracker.last_updated_index() == u64::MAX {
            return Ok(Some(start_index));
        }

        let next_index = (tracker.last_updated_index() + 1) % self.num_operators;
        if next_index == start_index {
            return Ok(None);
        }

        Ok(Some(next_index))
    }

    /// closes vault_update_state_tracker if needed
    pub fn close_vault_update_state_tracker_if_needed(
        &self,
        vault_update_state_tracker: &AccountInfo<'info>,
        payer: &AccountInfo<'info>,
        payer_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        if self.is_vault_up_to_date() {
            return Ok(());
        }

        let close_vault_update_state_tracker_ix =
            jito_vault_sdk::sdk::close_vault_update_state_tracker(
                self.vault_program.key,
                self.vault_config_account.key,
                self.vault_account.key,
                vault_update_state_tracker.key,
                payer.key,
                self.current_epoch,
            );
        invoke_signed(
            &close_vault_update_state_tracker_ix,
            &[
                self.vault_program.to_account_info(),
                self.vault_config_account.to_account_info(),
                self.vault_account.to_account_info(),
                vault_update_state_tracker.to_account_info(),
                payer.to_account_info(),
            ],
            payer_seeds,
        )?;

        msg!(
            "RESTAKE#jito vault_update_state_tracker closed: current_epoch={}",
            self.current_epoch
        );

        Ok(())
    }

    /// * (0) vault_program
    /// * (1) vault_config_account(writable)
    /// * (2) vault_account(writable)
    /// * (3) token_program
    /// * (4) vault_receipt_token_mint(writable)
    /// * (5) vault_receipt_token_fee_wallet_account(writable)
    /// * (6) vault_supported_token_reserve_account(writable),
    pub fn find_accounts_to_deposit(&self) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let data = &Self::borrow_account_data(self.vault_account)?;
        let vault = Self::deserialize_account_data::<Vault>(data)?;

        let accounts = Self::find_accounts_to_new(self.vault_account.key())?.chain([
            (anchor_spl::token::ID, false),
            (vault.vrt_mint, true),
            (
                associated_token::get_associated_token_address_with_program_id(
                    &vault.fee_wallet,
                    &vault.vrt_mint,
                    &anchor_spl::token::ID,
                ),
                true,
            ),
            (
                associated_token::get_associated_token_address_with_program_id(
                    self.vault_account.key,
                    &vault.supported_mint,
                    &anchor_spl::token::ID,
                ),
                true,
            ),
        ]);

        Ok(accounts)
    }

    /// returns [to_vault_receipt_token_account_amount, minted_vault_receipt_token_amount, deposited_supported_token_amount, deducted_supported_token_fee_amount]
    pub fn deposit(
        &self,
        // fixed
        token_program: &AccountInfo<'info>,
        vault_receipt_token_mint: &AccountInfo<'info>,
        vault_receipt_token_fee_wallet_account: &AccountInfo<'info>,
        vault_supported_token_reserve_account: &AccountInfo<'info>,

        // variant
        from_vault_supported_token_account: &AccountInfo<'info>,
        to_vault_receipt_token_account: &'info AccountInfo<'info>,
        signer: &AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],

        supported_token_amount: u64,
    ) -> Result<(u64, u64, u64, u64)> {
        // ensure update of fee related vault state
        let update_vault_balance_ix = jito_vault_sdk::sdk::update_vault_balance(
            self.vault_program.key,
            self.vault_config_account.key,
            self.vault_account.key,
            vault_supported_token_reserve_account.key,
            vault_receipt_token_mint.key,
            vault_receipt_token_fee_wallet_account.key,
            token_program.key,
        );
        invoke_signed(
            &update_vault_balance_ix,
            &[
                self.vault_program.to_account_info(),
                self.vault_config_account.to_account_info(),
                self.vault_account.to_account_info(),
                vault_supported_token_reserve_account.to_account_info(),
                vault_receipt_token_mint.to_account_info(),
                vault_receipt_token_fee_wallet_account.to_account_info(),
                token_program.to_account_info(),
            ],
            &[],
        )?;

        // do mint
        let mut to_vault_receipt_token_account =
            InterfaceAccount::<TokenAccount>::try_from(to_vault_receipt_token_account)?;
        let to_vault_receipt_token_account_amount_before = to_vault_receipt_token_account.amount;

        let (
            vault_deposit_capacity,
            vault_tokens_deposited,
            vault_deposit_fee_bps,
            vault_receipt_token_supply,
        ) = {
            let data = &Self::borrow_account_data(self.vault_account)?;
            let vault = Self::deserialize_account_data::<Vault>(data)?;
            (
                vault.deposit_capacity(),
                vault.tokens_deposited(),
                vault.deposit_fee_bps() as u64,
                vault.vrt_supply(),
            )
        };

        let supported_token_amount =
            supported_token_amount.min(vault_deposit_capacity - vault_tokens_deposited);

        let deducted_supported_token_fee_amount =
            utils::get_proportional_amount(supported_token_amount, vault_deposit_fee_bps, 10_000)?;

        let expected_minted_vault_receipt_token_amount = utils::get_proportional_amount(
            supported_token_amount - deducted_supported_token_fee_amount,
            vault_receipt_token_supply,
            vault_tokens_deposited,
        )?;

        let mint_to_ix = jito_vault_sdk::sdk::mint_to(
            self.vault_program.key,
            self.vault_config_account.key,
            self.vault_account.key,
            vault_receipt_token_mint.key,
            signer.key,
            from_vault_supported_token_account.key,
            vault_supported_token_reserve_account.key,
            &to_vault_receipt_token_account.key(),
            vault_receipt_token_fee_wallet_account.key,
            None, // mint_signer (?)
            supported_token_amount,
            expected_minted_vault_receipt_token_amount, // except fee
        );

        invoke_signed(
            &mint_to_ix,
            &[
                self.vault_program.to_account_info(),
                self.vault_config_account.to_account_info(),
                self.vault_account.to_account_info(),
                vault_receipt_token_mint.to_account_info(),
                signer.to_account_info(),
                from_vault_supported_token_account.to_account_info(),
                vault_supported_token_reserve_account.to_account_info(),
                to_vault_receipt_token_account.to_account_info(),
                vault_receipt_token_fee_wallet_account.to_account_info(),
                token_program.to_account_info(),
            ],
            signer_seeds,
        )?;

        to_vault_receipt_token_account.reload()?;
        let to_vault_receipt_token_account_amount = to_vault_receipt_token_account.amount;
        let minted_vault_receipt_token_amount =
            to_vault_receipt_token_account_amount - to_vault_receipt_token_account_amount_before;

        msg!("RESTAKE#jito deposited: vrt_mint={}, deposited_vst_amount={}, deducted_vst_amount={}, to_vrt_account_amount={}, minted_vrt_amount={}", vault_receipt_token_mint.key, supported_token_amount, deducted_supported_token_fee_amount, to_vault_receipt_token_account_amount, minted_vault_receipt_token_amount);

        Ok((
            to_vault_receipt_token_account_amount,
            minted_vault_receipt_token_amount,
            supported_token_amount,
            deducted_supported_token_fee_amount,
        ))
    }

    /// * (0) vault_program
    /// * (1) vault_config_account(writable)
    /// * (2) vault_account(writable)
    /// * (3) token_program
    /// * (4) associated_token_program
    /// * (5) system_program
    /// * (6) vault_receipt_token_mint
    pub fn find_accounts_to_request_withdraw(
        &self,
    ) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let data = &Self::borrow_account_data(self.vault_account)?;
        let vault = Self::deserialize_account_data::<Vault>(data)?;

        let accounts = Self::find_accounts_to_new(self.vault_account.key())?.chain([
            (anchor_spl::token::ID, false),
            (associated_token::ID, false),
            (system_program::ID, false),
            (vault.vrt_mint, false),
        ]);

        Ok(accounts)
    }

    pub fn find_withdrawal_ticket_account(&self, authority_account: &Pubkey) -> Pubkey {
        VaultStakerWithdrawalTicket::find_program_address(
            self.vault_program.key,
            self.vault_account.key,
            authority_account,
        )
        .0
    }

    /// returns [from_vault_receipt_token_account, enqueued_vault_receipt_token_amount]
    pub fn request_withdraw(
        &self,
        // fixed
        token_program: &AccountInfo<'info>,
        associated_token_program: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
        vault_receipt_token_mint: &AccountInfo<'info>,

        // variant
        from_vault_receipt_token_account: &'info AccountInfo<'info>,
        withdrawal_ticket_account: &AccountInfo<'info>,
        withdrawal_ticket_receipt_token_account: &'info AccountInfo<'info>,
        withdrawal_ticket_base_account: &AccountInfo<'info>,

        payer: &AccountInfo<'info>,
        payer_seeds: &[&[&[u8]]],
        signer: &AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],

        vault_receipt_token_amount: u64,
    ) -> Result<(u64, u64)> {
        let mut from_vault_receipt_token_account =
            InterfaceAccount::<TokenAccount>::try_from(from_vault_receipt_token_account)?;
        let from_vault_receipt_token_account_amount_before =
            from_vault_receipt_token_account.amount;

        // create ATA for withdrawal ticket account
        associated_token::create_idempotent(CpiContext::new_with_signer(
            associated_token_program.to_account_info(),
            associated_token::Create {
                payer: payer.to_account_info(),
                associated_token: withdrawal_ticket_receipt_token_account.to_account_info(),
                authority: withdrawal_ticket_account.to_account_info(),
                mint: vault_receipt_token_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
            payer_seeds,
        ))?;

        // pay withdrawal ticket rent
        let required_space = 8 + std::mem::size_of::<VaultUpdateStateTracker>();

        let current_lamports = withdrawal_ticket_account.get_lamports();
        let required_lamports = Rent::get()?
            .minimum_balance(required_space)
            .max(1)
            .saturating_sub(current_lamports);

        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: payer.to_account_info(),
                    to: withdrawal_ticket_account.to_account_info(),
                },
                payer_seeds,
            ),
            required_lamports,
        )?;

        // do request withdrawal
        let enqueue_withdraw_ix = jito_vault_sdk::sdk::enqueue_withdrawal(
            self.vault_program.key,
            self.vault_config_account.key,
            self.vault_account.key,
            withdrawal_ticket_account.key,
            withdrawal_ticket_receipt_token_account.key,
            signer.key,
            from_vault_receipt_token_account.to_account_info().key,
            withdrawal_ticket_base_account.key,
            None,
            vault_receipt_token_amount,
        );
        invoke_signed(
            &enqueue_withdraw_ix,
            &[
                self.vault_program.to_account_info(),
                self.vault_config_account.to_account_info(),
                self.vault_account.to_account_info(),
                withdrawal_ticket_account.to_account_info(),
                withdrawal_ticket_receipt_token_account.to_account_info(),
                signer.to_account_info(),
                from_vault_receipt_token_account.to_account_info(),
                withdrawal_ticket_base_account.to_account_info(),
                token_program.to_account_info(),
                system_program.to_account_info(),
            ],
            signer_seeds,
        )?;

        from_vault_receipt_token_account.reload()?;
        let from_vault_receipt_token_account_amount = from_vault_receipt_token_account.amount;
        let enqueued_vault_receipt_token_amount = from_vault_receipt_token_account_amount_before
            - from_vault_receipt_token_account_amount;

        msg!("UNRESTAKE#jito: receipt_token_mint={}, enqueued_vault_receipt_token_account={}, from_vault_receipt_token_account_amount={}", vault_receipt_token_mint.key, enqueued_vault_receipt_token_amount, from_vault_receipt_token_account_amount);

        let withdrawal_ticket_vault_receipt_token_account =
            InterfaceAccount::<TokenAccount>::try_from(withdrawal_ticket_receipt_token_account)?;
        require_eq!(
            withdrawal_ticket_vault_receipt_token_account.amount,
            enqueued_vault_receipt_token_amount,
        );

        Ok((
            from_vault_receipt_token_account_amount,
            enqueued_vault_receipt_token_amount,
        ))
    }

    pub fn is_claimable_withdrawal_ticket(&self, withdrawal_ticket: &AccountInfo) -> Result<bool> {
        Ok({
            if withdrawal_ticket.is_initialized() {
                let data = &Self::borrow_account_data(withdrawal_ticket)?;
                let ticket = Self::deserialize_account_data::<VaultStakerWithdrawalTicket>(data)?;
                let claimable = ticket.is_withdrawable(self.current_slot, self.epoch_length)?;
                msg!("CHECK_UNRESTAKED#jito: ticket={}, current_epoch={} ({}), unstaked_epoch={} ({}), withdrawalble={}", withdrawal_ticket.key, self.current_slot / self.epoch_length, self.current_slot, ticket.slot_unstaked() / self.epoch_length, ticket.slot_unstaked(), claimable);
                claimable
            } else {
                false
            }
        })
    }

    /// * (0) vault_program
    /// * (1) vault_config_account(writable)
    /// * (2) vault_account(writable)
    /// * (3) token_program
    /// * (4) system_program
    /// * (5) vault_receipt_token_mint(writable)
    /// * (6) vault_program_fee_wallet_receipt_token_account(writable)
    /// * (7) vault_fee_wallet_receipt_token_account(writable)
    /// * (8) vault_supported_token_account(writable)
    pub fn find_accounts_to_withdraw(&self) -> Result<impl Iterator<Item = (Pubkey, bool)>> {
        let data = &Self::borrow_account_data(self.vault_account)?;
        let vault = Self::deserialize_account_data::<Vault>(data)?;

        let accounts = Self::find_accounts_to_new(self.vault_account.key())?.chain([
            (anchor_spl::token::ID, false),
            (system_program::ID, false),
            (vault.vrt_mint, true),
            (
                associated_token::get_associated_token_address_with_program_id(
                    &self.vault_program_fee_wallet,
                    &vault.vrt_mint,
                    &anchor_spl::token::ID,
                ),
                true,
            ),
            (
                associated_token::get_associated_token_address_with_program_id(
                    &vault.fee_wallet,
                    &vault.vrt_mint,
                    &anchor_spl::token::ID,
                ),
                true,
            ),
            (
                associated_token::get_associated_token_address_with_program_id(
                    self.vault_account.key,
                    &vault.supported_mint,
                    &anchor_spl::token::ID,
                ),
                true,
            ),
        ]);

        Ok(accounts)
    }

    /// returns [to_vault_supported_token_account_amount, unrestaked_receipt_token_amount, claimed_supported_token_amount, deducted_program_fee_receipt_token_amount, deducted_vault_fee_receipt_token_amount, returned_rent_fee_sol_amount]
    pub fn withdraw(
        &self,
        // fixed
        token_program: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
        vault_receipt_token_mint: &AccountInfo<'info>,
        vault_program_fee_receipt_token_account: &'info AccountInfo<'info>,
        vault_fee_receipt_token_account: &'info AccountInfo<'info>,
        vault_supported_token_reserve_account: &'info AccountInfo<'info>,

        // variant
        withdrawal_ticket: &AccountInfo<'info>,
        withdrawal_ticket_receipt_token_account: &'info AccountInfo<'info>,
        to_vault_supported_token_account: &'info AccountInfo<'info>,

        signer: &AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],

        to_rent_fee_return_account: &AccountInfo<'info>,
    ) -> Result<(u64, u64, u64, u64, u64, u64)> {
        let mut vault_program_fee_receipt_token_account =
            InterfaceAccount::<TokenAccount>::try_from(vault_program_fee_receipt_token_account)?;
        let vault_program_fee_receipt_token_account_amount_before =
            vault_program_fee_receipt_token_account.amount;

        let mut vault_fee_receipt_token_account =
            InterfaceAccount::<TokenAccount>::try_from(vault_fee_receipt_token_account)?;
        let vault_fee_receipt_token_account_amount_before = vault_fee_receipt_token_account.amount;

        let mut vault_supported_token_reserve_account =
            InterfaceAccount::<TokenAccount>::try_from(vault_supported_token_reserve_account)?;
        let vault_supported_token_reserve_account_amount_before =
            vault_supported_token_reserve_account.amount;

        let withdrawal_ticket_receipt_token_account =
            InterfaceAccount::<TokenAccount>::try_from(withdrawal_ticket_receipt_token_account)?;
        let unrestaked_receipt_token_amount = withdrawal_ticket_receipt_token_account.amount;

        let mut to_vault_supported_token_account =
            InterfaceAccount::<TokenAccount>::try_from(to_vault_supported_token_account)?;
        let to_vault_supported_token_account_amount_before =
            to_vault_supported_token_account.amount;

        let signer_sol_amount_before = signer.lamports();

        let withdraw_ix = jito_vault_sdk::sdk::burn_withdrawal_ticket(
            self.vault_program.key,
            self.vault_config_account.key,
            self.vault_account.key,
            vault_supported_token_reserve_account.to_account_info().key,
            vault_receipt_token_mint.key,
            signer.key,
            to_vault_supported_token_account.to_account_info().key,
            withdrawal_ticket.key,
            withdrawal_ticket_receipt_token_account
                .to_account_info()
                .key,
            vault_fee_receipt_token_account.to_account_info().key,
            vault_program_fee_receipt_token_account
                .to_account_info()
                .key,
            None,
        );

        invoke_signed(
            &withdraw_ix,
            &[
                self.vault_program.to_account_info(),
                self.vault_config_account.to_account_info(),
                self.vault_account.to_account_info(),
                vault_supported_token_reserve_account.to_account_info(),
                vault_receipt_token_mint.to_account_info(),
                signer.to_account_info(),
                to_vault_supported_token_account.to_account_info(),
                withdrawal_ticket.to_account_info(),
                withdrawal_ticket_receipt_token_account.to_account_info(),
                vault_fee_receipt_token_account.to_account_info(),
                vault_program_fee_receipt_token_account.to_account_info(),
                token_program.to_account_info(),
                system_program.to_account_info(),
            ],
            signer_seeds,
        )?;

        vault_program_fee_receipt_token_account.reload()?;
        let vault_program_fee_receipt_token_account_amount =
            vault_program_fee_receipt_token_account.amount;
        let deducted_program_fee_receipt_token_amount =
            vault_program_fee_receipt_token_account_amount
                - vault_program_fee_receipt_token_account_amount_before;

        vault_fee_receipt_token_account.reload()?;
        let vault_fee_receipt_token_account_amount = vault_fee_receipt_token_account.amount;
        let deducted_vault_fee_receipt_token_amount =
            vault_fee_receipt_token_account_amount - vault_fee_receipt_token_account_amount_before;

        vault_supported_token_reserve_account.reload()?;
        let vault_supported_token_reserve_account_amount =
            vault_supported_token_reserve_account.amount;
        let vault_reduced_supported_token_amount =
            vault_supported_token_reserve_account_amount_before
                - vault_supported_token_reserve_account_amount;

        to_vault_supported_token_account.reload()?;
        let to_vault_supported_token_account_amount = to_vault_supported_token_account.amount;
        let claimed_supported_token_amount = to_vault_supported_token_account_amount
            - to_vault_supported_token_account_amount_before;
        require_eq!(
            vault_reduced_supported_token_amount,
            claimed_supported_token_amount
        );

        let returned_rent_fee_sol_amount = signer.lamports() - signer_sol_amount_before;
        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: signer.to_account_info(),
                    to: to_rent_fee_return_account.to_account_info(),
                },
                signer_seeds,
            ),
            returned_rent_fee_sol_amount,
        )?;

        msg!("CLAIM_UNRESTAKED#jito: receipt_token_mint={}, to_vault_supported_token_account_amount={}, unrestaked_receipt_token_amount={}, claimed_supported_token_amount={}, deducted_program_fee_receipt_token_amount={}, deducted_vault_fee_receipt_token_amount={}, returned_rent_fee_sol_amount={}", vault_receipt_token_mint.key,
            to_vault_supported_token_account_amount,
            unrestaked_receipt_token_amount,
            claimed_supported_token_amount,
            deducted_program_fee_receipt_token_amount,
            deducted_vault_fee_receipt_token_amount,
            returned_rent_fee_sol_amount,
        );

        Ok((
            to_vault_supported_token_account_amount,
            unrestaked_receipt_token_amount,
            claimed_supported_token_amount,
            deducted_program_fee_receipt_token_amount,
            deducted_vault_fee_receipt_token_amount,
            returned_rent_fee_sol_amount,
        ))
    }

    /// If there is more idle(not staked) vst than required amount for withdrawals, delegate them
    /// ref: https://github.com/jito-foundation/restaking/blob/df9f051/vault_core/src/vault.rs#L1200
    pub fn get_available_amount_to_delegate(&self) -> Result<u64> {
        let data = &Self::borrow_account_data(self.vault_account)?;

        let vault = Self::deserialize_account_data::<Vault>(data)?;
        // ref: just check vault.delegate fn for constraints

        // there is some protection built-in to the vault to avoid over delegating assets
        // this number is denominated in the supported token units
        let amount_to_reserve_for_vrts = vault
            .calculate_supported_assets_requested_for_withdrawal()
            .map_err(|_| error!(ErrorCode::CalculationArithmeticException))?;

        let amount_available_for_delegation = vault
            .tokens_deposited()
            .saturating_sub(
                vault
                    .delegation_state
                    .total_security()
                    .map_err(|_| error!(ErrorCode::CalculationArithmeticException))?,
            )
            .saturating_sub(amount_to_reserve_for_vrts);

        Ok(amount_available_for_delegation)
    }

    pub fn add_delegation(
        &self,
        // fixed
        vault_operator_delegation: &AccountInfo<'info>,
        operator: &AccountInfo<'info>,
        vault_delegation_admin: &AccountInfo<'info>,
        vault_delegation_admin_seeds: &[&[&[u8]]],

        supported_token_amount: u64,
    ) -> Result<()> {
        if supported_token_amount == 0 {
            return Ok(());
        }

        let add_delegation_ix = jito_vault_sdk::sdk::add_delegation(
            self.vault_program.key,
            self.vault_config_account.key,
            self.vault_account.key,
            operator.key,
            vault_operator_delegation.key,
            vault_delegation_admin.key,
            supported_token_amount,
        );

        let total_delegated_amount_before = {
            let data = &Self::borrow_account_data(self.vault_account)?;
            let vault = Self::deserialize_account_data::<Vault>(data)?;
            vault.delegation_state.staked_amount()
        };

        invoke_signed(
            &add_delegation_ix,
            &[
                self.vault_config_account.to_account_info(),
                self.vault_account.to_account_info(),
                operator.to_account_info(),
                vault_operator_delegation.to_account_info(),
                vault_delegation_admin.to_account_info(),
            ],
            vault_delegation_admin_seeds,
        )?;

        let total_delegated_amount = {
            let data = &Self::borrow_account_data(self.vault_account)?;
            let vault = Self::deserialize_account_data::<Vault>(data)?;
            vault.delegation_state.staked_amount()
        };
        let delegated_amount = total_delegated_amount - total_delegated_amount_before;

        msg!("DELEGATE#jito: operator={}, delegated_supported_token_amount={}, total_delegated_supported_token_amount={}",
            operator.key(),
            delegated_amount,
            total_delegated_amount,
        );

        Ok(())
    }

    /// If there is a shortage of required vst amount for withdrawals, undelegate them
    /// ref: https://github.com/jito-foundation/restaking/blob/master/vault_core/src/vault.rs#L1110
    pub fn get_additional_undelegation_amount_needed(&self) -> Result<u64> {
        let data = &Self::borrow_account_data(self.vault_account)?;
        let vault = Self::deserialize_account_data::<Vault>(data)?;

        // amount_requested_for_withdrawal = vrt_to_vst(enqueued + cooling_down + ready_to_claim)
        let amount_requested_for_withdrawals = vault
            .calculate_supported_assets_requested_for_withdrawal()
            .map_err(|_| error!(ErrorCode::CalculationArithmeticException))?;

        // available_for_withdrawal = tokens_deposited - staked
        // note that this included undelegating amount
        let available_amount_for_withdrawal =
            vault.tokens_deposited() - vault.delegation_state.staked_amount();

        Ok(amount_requested_for_withdrawals.saturating_sub(available_amount_for_withdrawal))
    }

    pub fn cooldown_delegation(
        &self,
        // fixed
        vault_operator_delegation: &AccountInfo<'info>,
        operator: &AccountInfo<'info>,
        vault_delegation_admin: &AccountInfo<'info>,
        vault_delegation_admin_seeds: &[&[&[u8]]],

        supported_token_amount: u64,
    ) -> Result<()> {
        if supported_token_amount == 0 {
            return Ok(());
        }

        let cooldown_delegation_ix = jito_vault_sdk::sdk::cooldown_delegation(
            self.vault_program.key,
            self.vault_config_account.key,
            self.vault_account.key,
            operator.key,
            vault_operator_delegation.key,
            vault_delegation_admin.key,
            supported_token_amount,
        );

        let total_undelegating_amount_before = {
            let data = &Self::borrow_account_data(self.vault_account)?;
            let vault = Self::deserialize_account_data::<Vault>(data)?;
            vault.delegation_state.enqueued_for_cooldown_amount()
                + vault.delegation_state.cooling_down_amount()
        };

        invoke_signed(
            &cooldown_delegation_ix,
            &[
                self.vault_config_account.clone(),
                self.vault_account.clone(),
                operator.clone(),
                vault_operator_delegation.clone(),
                vault_delegation_admin.clone(),
            ],
            vault_delegation_admin_seeds,
        )?;

        let total_undelegating_amount = {
            let data = &Self::borrow_account_data(self.vault_account)?;
            let vault = Self::deserialize_account_data::<Vault>(data)?;
            vault.delegation_state.enqueued_for_cooldown_amount()
                + vault.delegation_state.cooling_down_amount()
        };
        let undelegating_amount = total_undelegating_amount - total_undelegating_amount_before;

        msg!("UNDELEGATE#jito: operator={}, enqueued_supported_token_amount={}, total_undelegating_supported_token_amount={}",
            operator.key(),
            undelegating_amount,
            total_undelegating_amount,
        );

        Ok(())
    }

    pub fn get_supported_token_to_receipt_token_exchange_ratio(&self) -> Result<(u64, u64)> {
        let data = &Self::borrow_account_data(self.vault_account)?;
        let vault = Self::deserialize_account_data::<Vault>(data)?;

        Ok((vault.tokens_deposited(), vault.vrt_supply()))
    }
}
