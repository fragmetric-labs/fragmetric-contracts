use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::system_program;
use anchor_spl::associated_token;
use anchor_spl::token_interface::TokenAccount;
use jito_bytemuck::AccountDeserialize;

use crate::constants::{JITO_VAULT_CONFIG_ADDRESS, JITO_VAULT_PROGRAM_ID};
use crate::errors;
use crate::utils;
use crate::utils::AccountInfoExt;

pub struct JitoRestakingVaultService<'info> {
    vault_program: &'info AccountInfo<'info>,
    vault_config_account: &'info AccountInfo<'info>,
    vault_account: &'info AccountInfo<'info>,
    current_slot: u64,
    current_epoch: u64,
    epoch_length: u64,
    vault_program_fee_wallet: Pubkey,
}

impl<'info> JitoRestakingVaultService<'info> {
    pub fn new(
        vault_program: &'info AccountInfo<'info>,
        vault_config_account: &'info AccountInfo<'info>,
        vault_account: &'info AccountInfo<'info>,
    ) -> Result<Self> {
        require_eq!(JITO_VAULT_PROGRAM_ID, vault_program.key());
        let vault_config = Self::deserialize_vault_config(vault_config_account)?;

        let current_slot = Clock::get()?.slot;
        let epoch_length = vault_config.epoch_length();
        let current_epoch = current_slot
            .checked_div(epoch_length)
            .ok_or_else(|| error!(errors::ErrorCode::CalculationArithmeticException))?;

        Ok(Self {
            vault_program,
            vault_config_account,
            vault_account,
            current_slot,
            current_epoch,
            epoch_length,
            vault_program_fee_wallet: vault_config.program_fee_wallet,
        })
    }

    pub(super) fn deserialize_vault(
        vault_account: &'info AccountInfo<'info>,
    ) -> Result<jito_vault_core::vault::Vault> {
        if !vault_account.is_initialized() {
            Err(ProgramError::InvalidAccountData.into())
        } else {
            Ok(*jito_vault_core::vault::Vault::try_from_slice_unchecked(
                vault_account.try_borrow_data()?.as_ref(),
            )?)
        }
    }

    fn deserialize_vault_config(
        vault_config: &'info AccountInfo<'info>,
    ) -> Result<jito_vault_core::config::Config> {
        if !vault_config.is_initialized() {
            Err(ProgramError::InvalidAccountData.into())
        } else {
            Ok(*jito_vault_core::config::Config::try_from_slice_unchecked(
                vault_config.try_borrow_data()?.as_ref(),
            )?)
        }
    }

    fn deserialize_vault_update_state_tracker(
        vault_update_state_tracker: &'info AccountInfo<'info>,
    ) -> Result<jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker> {
        if !vault_update_state_tracker.is_initialized() {
            Err(ProgramError::InvalidAccountData.into())
        } else {
            Ok(*jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker::try_from_slice_unchecked(
                vault_update_state_tracker.try_borrow_data()?.as_ref(),
            )?)
        }
    }

    fn deserialize_vault_operator_delegation(
        vault_operator_delegation: &'info AccountInfo<'info>,
    ) -> Result<jito_vault_core::vault_operator_delegation::VaultOperatorDelegation> {
        if !vault_operator_delegation.is_initialized() {
            Err(ProgramError::InvalidAccountData.into())
        } else {
            Ok(*jito_vault_core::vault_operator_delegation::VaultOperatorDelegation::try_from_slice_unchecked(
                vault_operator_delegation.try_borrow_data()?.as_ref(),
            )?)
        }
    }

    fn find_vault_update_state_tracker_address(&self, epoch: Option<u64>) -> Pubkey {
        jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker::find_program_address(
            self.vault_program.key,
            self.vault_account.key,
            epoch.unwrap_or(self.current_epoch),
        )
        .0
    }

    pub fn find_vault_operator_delegation_address(&self, operator: &Pubkey) -> Pubkey {
        jito_vault_core::vault_operator_delegation::VaultOperatorDelegation::find_program_address(
            self.vault_program.key,
            self.vault_account.key,
            operator,
        )
        .0
    }

    /// gives max fee/expense ratio during a cycle of circulation
    /// returns (numerator, denominator)
    pub fn get_max_cycle_fee(vault_account: &'info AccountInfo<'info>) -> Result<(u64, u64)> {
        let vault = Self::deserialize_vault(vault_account)?;

        Ok((
            vault.deposit_fee_bps() as u64 // vault's deposit fee
            + vault.withdrawal_fee_bps() as u64 // vault's withdrawal fee
            + vault.program_fee_bps() as u64, // vault program's withdrawal fee
            10_000,
        ))
    }

    /// returns (pubkey, writable) of [vault_program, vault_config, vault_account]
    pub fn find_accounts_to_new(vault_address: Pubkey) -> Result<Vec<(Pubkey, bool)>> {
        Ok(vec![
            (JITO_VAULT_PROGRAM_ID, false),
            (JITO_VAULT_CONFIG_ADDRESS, true), // well, idk why but deposit ix needs it to be writable.
            (vault_address, true),
        ])
    }

    /// returns (pubkey, writable) of [vault_program, vault_config, vault_account, system_program, vault_update_state_tracker_for_current_epoch, vault_update_state_tracker_for_next_epoch]
    pub fn find_account_to_ensure_state_update_required(&self) -> Result<Vec<(Pubkey, bool)>> {
        let mut accounts = Self::find_accounts_to_new(self.vault_account.key())?;
        accounts.extend(vec![
            (anchor_lang::solana_program::system_program::id(), false),
            (self.find_vault_update_state_tracker_address(None), true),
            (
                self.find_vault_update_state_tracker_address(Some(self.current_epoch + 1)),
                true,
            ),
        ]);
        Ok(accounts)
    }

    pub fn get_current_epoch(&self) -> u64 {
        self.current_epoch
    }

    /// check whether vault epoch-process should be fulfilled or not.
    /// returns valid [vault_update_state_tracker] among [vault_update_state_tracker1 or vault_update_state_tracker2] if state update is required.
    /// after run [update_delegation_state] for all operators, this method should to be called to finalize it.
    pub fn ensure_state_update_required(
        &self,
        system_program: &'info AccountInfo<'info>,
        vault_update_state_tracker1: &'info AccountInfo<'info>,
        vault_update_state_tracker2: &'info AccountInfo<'info>,

        payer: &AccountInfo<'info>,
        payer_seeds: &[&[&[u8]]],
    ) -> Result<Option<&'info AccountInfo<'info>>> {
        let (last_updated_epoch, vault_operator_count) = {
            let vault = Self::deserialize_vault(self.vault_account)?;
            let vault_config = Self::deserialize_vault_config(self.vault_config_account)?;
            let last_updated_slot = vault.last_full_state_update_slot();
            let last_updated_epoch = last_updated_slot
                .checked_div(vault_config.epoch_length())
                .ok_or_else(|| error!(errors::ErrorCode::CalculationArithmeticException))?;
            let vault_operator_count = vault.operator_count();
            (last_updated_epoch, vault_operator_count)
        };

        if self.current_epoch == last_updated_epoch {
            // epoch process not required
            msg!(
                "RESTAKE#jito vault_update_state_tracker is up-to-date: current_epoch={}",
                self.current_epoch
            );
            return Ok(None);
        }

        // check new tracker is required
        let initializing_tracker_account = match Self::deserialize_vault_update_state_tracker(
            vault_update_state_tracker1,
        ) {
            Ok(current_tracker) => {
                if current_tracker.ncn_epoch() != self.current_epoch {
                    // just close out-dated state tracker
                    let closing_epoch = current_tracker.ncn_epoch();
                    let close_vault_update_state_tracker_ix =
                        jito_vault_sdk::sdk::close_vault_update_state_tracker(
                            self.vault_program.key,
                            self.vault_config_account.key,
                            self.vault_account.key,
                            vault_update_state_tracker1.key,
                            payer.key,
                            closing_epoch,
                        );
                    invoke_signed(
                        &close_vault_update_state_tracker_ix,
                        &[
                            self.vault_program.to_account_info(),
                            self.vault_config_account.to_account_info(),
                            self.vault_account.to_account_info(),
                            vault_update_state_tracker1.to_account_info(),
                            payer.to_account_info(),
                        ],
                        payer_seeds,
                    )?;
                    msg!("RESTAKE#jito vault_update_state_tracker needs to be initialized: closed_epoch={}, current_epoch={}", closing_epoch, self.current_epoch);
                    Some(vault_update_state_tracker2)
                } else {
                    None
                }
            }
            Err(err) => {
                msg!("RESTAKE#jito vault_update_state_tracker needs to be initialized: current_epoch={}, error={}", self.current_epoch, err);
                Some(
                    if vault_update_state_tracker1.key()
                        == self.find_vault_update_state_tracker_address(None)
                    {
                        vault_update_state_tracker1
                    } else {
                        vault_update_state_tracker2
                    },
                )
            }
        };

        // initialize new tracker and return
        if let Some(tracker_account) = initializing_tracker_account {
            let required_space = 8 + std::mem::size_of::<
                jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker,
            >();
            let current_lamports = tracker_account.get_lamports();
            let required_lamports = Rent::get()?
                .minimum_balance(required_space)
                .max(1)
                .saturating_sub(current_lamports);

            anchor_lang::system_program::transfer(
                CpiContext::new_with_signer(
                    system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: payer.to_account_info(),
                        to: tracker_account.to_account_info(),
                    },
                    payer_seeds,
                ),
                required_lamports,
            )?;

            let initialize_vault_update_state_tracker_ix =
                jito_vault_sdk::sdk::initialize_vault_update_state_tracker(
                    self.vault_program.key,
                    self.vault_config_account.key,
                    self.vault_account.key,
                    tracker_account.key,
                    payer.key,
                    jito_vault_sdk::instruction::WithdrawalAllocationMethod::Greedy,
                );

            invoke_signed(
                &initialize_vault_update_state_tracker_ix,
                &[
                    self.vault_program.to_account_info(),
                    self.vault_config_account.to_account_info(),
                    self.vault_account.to_account_info(),
                    tracker_account.to_account_info(),
                    payer.to_account_info(),
                    system_program.to_account_info(),
                ],
                payer_seeds,
            )?;

            msg!(
                "RESTAKE#jito vault_update_state_tracker initialized: current_epoch={}",
                self.current_epoch
            );

            return Ok(Some(tracker_account));
        }

        // check current tracker is ready to close
        let current_tracker_account = vault_update_state_tracker1;

        // check all operator has been updated
        let current_tracker =
            Self::deserialize_vault_update_state_tracker(current_tracker_account)?;
        let all_cranked = vault_operator_count == 0 || current_tracker
            .all_operators_updated(vault_operator_count)
            .unwrap_or_else(|err| {
                msg!(
                        "RESTAKE#jito failed to compute all_operators_updated: vault_operator_count={}, error={}",
                        vault_operator_count,
                        err
                    );
                false
            });

        // update still need to be cranked then closed
        if !all_cranked {
            msg!("RESTAKE#jito vault_update_state_tracker needs to be cranked then closed: vault_operator_count={}, current_epoch={}", vault_operator_count, self.current_epoch);
            return Ok(Some(current_tracker_account));
        }

        // close the tracker and finalize current epoch process
        let close_vault_update_state_tracker_ix =
            jito_vault_sdk::sdk::close_vault_update_state_tracker(
                self.vault_program.key,
                self.vault_config_account.key,
                self.vault_account.key,
                current_tracker_account.key,
                payer.key,
                self.current_epoch,
            );

        invoke_signed(
            &close_vault_update_state_tracker_ix,
            &[
                self.vault_program.to_account_info(),
                self.vault_config_account.to_account_info(),
                self.vault_account.to_account_info(),
                current_tracker_account.to_account_info(),
                payer.to_account_info(),
            ],
            payer_seeds,
        )?;

        msg!(
            "RESTAKE#jito vault_update_state_tracker closed: current_epoch={}",
            self.current_epoch
        );
        Ok(None)
    }

    /// returns [staked_amount, enqueued_for_cooldown_amount, cooling_down_amount]
    /// in other words [restaked_amount, undelegation_requested_amount, undelegating_amount]
    pub fn update_delegation_state(
        self: &Self,
        vault_update_state_tracker: &'info AccountInfo<'info>,
        vault_operator_delegation: &'info AccountInfo<'info>,
        operator: &'info AccountInfo<'info>,

        payer: &AccountInfo<'info>,
        payer_seeds: &[&[&[u8]]],
    ) -> Result<(u64, u64, u64)> {
        let mut delegation =
            Self::deserialize_vault_operator_delegation(vault_operator_delegation)?;

        // assertion: `check_is_already_updated` returns Err when already updated
        if !match delegation.check_is_already_updated(self.current_slot, self.current_epoch) {
            Ok(_) => false,
            Err(err) => {
                msg!("RESTAKE#jito vault_operator_delegation is up-to-date: current_epoch={}, operator={}, error={}", self.current_epoch, operator.key(), err);
                true
            }
        } {
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
                    payer.to_account_info(),
                ],
                payer_seeds,
            )?;

            delegation = Self::deserialize_vault_operator_delegation(vault_operator_delegation)?;
        }

        let staked_amount = delegation.delegation_state.staked_amount();
        let enqueued_for_cooldown_amount =
            delegation.delegation_state.enqueued_for_cooldown_amount();
        let cooling_down_amount = delegation.delegation_state.cooling_down_amount();

        msg!("RESTAKE#jito vault_update_state_tracker cranked: current_epoch={}, operator={}, staked_amount={}, enqueued_for_cooldown_amount={}, cooling_down_amount={}", self.current_epoch, operator.key, staked_amount, enqueued_for_cooldown_amount, cooling_down_amount);
        Ok((
            staked_amount,
            enqueued_for_cooldown_amount,
            cooling_down_amount,
        ))
    }

    /// returns (pubkey, writable) of [vault_program, vault_config, vault_account, token_program, vault_receipt_token_mint, vault_receipt_token_fee_wallet_account, vault_supported_token_reserve_account]
    pub fn find_accounts_to_deposit(&self) -> Result<Vec<(Pubkey, bool)>> {
        let mut accounts = Self::find_accounts_to_new(self.vault_account.key())?;
        let vault = Self::deserialize_vault(self.vault_account)?;
        accounts.extend(vec![
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
        token_program: &'info AccountInfo<'info>,
        vault_receipt_token_mint: &'info AccountInfo<'info>,
        vault_receipt_token_fee_wallet_account: &'info AccountInfo<'info>,
        vault_supported_token_reserve_account: &'info AccountInfo<'info>,

        // variant
        from_vault_supported_token_account: &'info AccountInfo<'info>,
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
            let vault = Self::deserialize_vault(self.vault_account)?;
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

    /// returns (pubkey, writable) of [vault_program, vault_config, vault_account, token_program, associated_token_program, system_program, vault_receipt_token_mint]
    pub fn find_accounts_to_request_withdraw(&self) -> Result<Vec<(Pubkey, bool)>> {
        let mut accounts = Self::find_accounts_to_new(self.vault_account.key())?;
        let vault = Self::deserialize_vault(self.vault_account)?;
        accounts.extend(vec![
            (anchor_spl::token::ID, false),
            (associated_token::ID, false),
            (system_program::ID, false),
            (vault.vrt_mint, false),
        ]);
        Ok(accounts)
    }

    pub fn find_withdrawal_ticket_account(&self, authority_account: &Pubkey) -> Pubkey {
        jito_vault_core::vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket::find_program_address(self.vault_program.key, self.vault_account.key, authority_account).0
    }

    /// returns [from_vault_receipt_token_account, enqueued_vault_receipt_token_amount]
    pub fn request_withdraw(
        &self,
        // fixed
        token_program: &'info AccountInfo<'info>,
        associated_token_program: &'info AccountInfo<'info>,
        system_program: &'info AccountInfo<'info>,
        vault_receipt_token_mint: &'info AccountInfo<'info>,

        // variant
        from_vault_receipt_token_account: &'info AccountInfo<'info>,
        withdrawal_ticket_account: &'info AccountInfo<'info>,
        withdrawal_ticket_receipt_token_account: &'info AccountInfo<'info>,
        withdrawal_ticket_base_account: &'info AccountInfo<'info>,

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
        let required_space = 8 + std::mem::size_of::<
            jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker,
        >();

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

    pub fn is_claimable_withdrawal_ticket(
        &self,
        withdrawal_ticket: &'info AccountInfo<'info>,
    ) -> Result<bool> {
        Ok({
            if withdrawal_ticket.is_initialized() {
                let ticket_data_ref = withdrawal_ticket.data.borrow();
                let ticket_data = ticket_data_ref.as_ref();
                let ticket = jito_vault_core::vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket::try_from_slice_unchecked(ticket_data)?;
                let claimable = ticket.is_withdrawable(self.current_slot, self.epoch_length)?;
                msg!("CHECK_UNRESTAKED#jito: ticket={}, current_epoch={} ({}), unstaked_epoch={} ({}), withdrawalble={}", withdrawal_ticket.key, self.current_slot / self.epoch_length, self.current_slot, ticket.slot_unstaked() / self.epoch_length, ticket.slot_unstaked(), claimable);
                claimable
            } else {
                false
            }
        })
    }

    /// returns (pubkey, writable) of [vault_program, vault_config, vault_account, token_program, system_program, vault_receipt_token_mint, vault_program_fee_receipt_token_account, vault_fee_receipt_token_account, vault_supported_token_reserve_account]
    pub fn find_accounts_to_withdraw(&self) -> Result<Vec<(Pubkey, bool)>> {
        let mut accounts = Self::find_accounts_to_new(self.vault_account.key())?;
        let vault = Self::deserialize_vault(self.vault_account)?;
        accounts.extend(vec![
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
        token_program: &'info AccountInfo<'info>,
        system_program: &'info AccountInfo<'info>,
        vault_receipt_token_mint: &'info AccountInfo<'info>,
        vault_program_fee_receipt_token_account: &'info AccountInfo<'info>,
        vault_fee_receipt_token_account: &'info AccountInfo<'info>,
        vault_supported_token_reserve_account: &'info AccountInfo<'info>,

        // variant
        withdrawal_ticket: &'info AccountInfo<'info>,
        withdrawal_ticket_receipt_token_account: &'info AccountInfo<'info>,
        to_vault_supported_token_account: &'info AccountInfo<'info>,

        signer: &'info AccountInfo<'info>,
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

    pub fn find_accounts_to_add_delegation(&self, operator: Pubkey) -> Result<Vec<(Pubkey, bool)>> {
        let mut accounts = Self::find_accounts_to_new(self.vault_account.key())?;
        let vault = Self::deserialize_vault(self.vault_account)?;
        accounts.extend(vec![
            (operator, false),
            (jito_vault_core::vault_operator_delegation::VaultOperatorDelegation::find_program_address(self.vault_program.key, self.vault_account.key, &operator).0, true),
            (vault.delegation_admin, false),
        ]);
        Ok(accounts)
    }

    pub fn add_delegation(
        &self,
        operator_account: &AccountInfo<'info>,
        vault_operator_delegation_account: &AccountInfo<'info>,
        vault_delegation_admin_account: &AccountInfo<'info>, // fund_account
        vault_delegation_admin_signer_seeds: &[&[u8]],

        delegation_amount: u64,
    ) -> Result<()> {
        let add_delegation_ix = jito_vault_sdk::sdk::add_delegation(
            self.vault_program.key,
            self.vault_config_account.key,
            self.vault_account.key,
            operator_account.key,
            vault_operator_delegation_account.key,
            vault_delegation_admin_account.key,
            delegation_amount,
        );

        let vault = Self::deserialize_vault(self.vault_account)?;
        msg!(
            "Before vault delegation_state staked_amount {}",
            vault.delegation_state.staked_amount()
        );

        invoke_signed(
            &add_delegation_ix,
            &[
                self.vault_config_account.clone(),
                self.vault_account.clone(),
                operator_account.clone(),
                vault_operator_delegation_account.clone(),
                vault_delegation_admin_account.clone(),
            ],
            &[vault_delegation_admin_signer_seeds],
        )?;

        let vault = Self::deserialize_vault(self.vault_account)?;
        msg!(
            "After vault delegation_state staked_amount {}",
            vault.delegation_state.staked_amount()
        );

        Ok(())
    }
}
