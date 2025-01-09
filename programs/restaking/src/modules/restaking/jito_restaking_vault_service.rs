use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::associated_token;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token_interface::TokenAccount;
use jito_bytemuck::AccountDeserialize;

use crate::constants::{
    ADMIN_PUBKEY, JITO_VAULT_CONFIG_ADDRESS, JITO_VAULT_PROGRAM_FEE_WALLET, JITO_VAULT_PROGRAM_ID,
};
use crate::errors;
use crate::utils;
use crate::utils::AccountInfoExt;

pub struct JitoRestakingVaultService<'info> {
    vault_program: &'info AccountInfo<'info>,
    vault_config_account: &'info AccountInfo<'info>,
    vault_account: &'info AccountInfo<'info>,
    current_slot: u64,
    current_epoch: u64,
}

impl<'info> JitoRestakingVaultService<'info> {
    const VAULT_BASE_ACCOUNT_SEED: &'static [u8] = b"vault_base_account";
    const VAULT_WITHDRAWAL_TICKET_SEED: &'static [u8] = b"vault_staker_withdrawal_ticket";
    const BASE_ACCOUNTS_LENGTH: u8 = 5;

    pub fn new(
        vault_program: &'info AccountInfo<'info>,
        vault_config_account: &'info AccountInfo<'info>,
        vault_account: &'info AccountInfo<'info>,
    ) -> Result<Self> {
        require_eq!(JITO_VAULT_PROGRAM_ID, vault_program.key());
        let vault_config = Self::deserialize_vault_config(vault_config_account)?;

        let current_slot = Clock::get()?.slot;
        let current_epoch = current_slot
            .checked_div(vault_config.epoch_length())
            .ok_or_else(|| error!(errors::ErrorCode::CalculationArithmeticException))?;

        Ok(Self {
            vault_program,
            vault_config_account,
            vault_account,
            current_slot,
            current_epoch,
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

    pub fn check_withdrawal_ticket_is_empty(
        vault_withdrawal_ticket: &'info AccountInfo<'info>,
    ) -> Result<bool> {
        if vault_withdrawal_ticket.data_is_empty() && vault_withdrawal_ticket.lamports() == 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn find_initialize_vault_accounts(
        vault_program: &AccountInfo,
        vault_config: &AccountInfo,
        vault_account: &AccountInfo,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let vault_data_ref = vault_account.try_borrow_data()?;
        let vault = jito_vault_core::vault::Vault::try_from_slice_unchecked(&vault_data_ref)?;

        let vault_receipt_token_mint = vault.vrt_mint;
        let vault_supported_token_mint = vault.supported_mint;
        let token_program = anchor_spl::token::ID;
        let vault_supported_token_account =
            associated_token::get_associated_token_address_with_program_id(
                &vault_account.key,
                &vault.supported_mint,
                &token_program,
            );

        Ok(vec![
            (vault_program.key(), false),
            (vault_config.key(), false),
            (vault_account.key(), true),
            (vault_receipt_token_mint, false),
            (token_program, false),
            (vault_supported_token_mint, false),
            (token_program, false),
            (vault_supported_token_account, false),
        ])
    }

    pub fn request_withdraw(
        self: &Self,
        operator: &Signer<'info>,
        vault_withdrawal_ticket: &AccountInfo<'info>,
        vault_withdrawal_ticket_token_account: &AccountInfo<'info>,
        vault_receipt_token_account: &AccountInfo<'info>, // fund ata
        vault_base_account: &AccountInfo<'info>,
        associated_token_program: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
        signer: &Signer<'info>,
        signer_seeds: &[&[&[u8]]],
        vrt_token_amount_out: u64,
    ) -> Result<()> {
        // associated_token::create(CpiContext::new(
        //     associated_token_program.clone(),
        //     associated_token::Create {
        //         payer: operator.to_account_info(),
        //         associated_token: vault_withdrawal_ticket_token_account.clone(),
        //         authority: vault_withdrawal_ticket.clone(),
        //         mint: self.vault_receipt_token_mint.clone(),
        //         system_program: system_program.clone(),
        //         token_program: self.vault_receipt_token_program.clone(),
        //     },
        // ))?;
        //
        // let rent = Rent::get()?;
        // let current_lamports = vault_withdrawal_ticket.lamports();
        // let space = 8 + std::mem::size_of::<VaultStakerWithdrawalTicket>();
        // let required_lamports = rent
        //     .minimum_balance(space)
        //     .max(1)
        //     .saturating_sub(current_lamports);
        //
        // if required_lamports > 0 {
        //     anchor_lang::system_program::transfer(
        //         CpiContext::new(
        //             system_program.clone(),
        //             anchor_lang::system_program::Transfer {
        //                 from: operator.to_account_info(),
        //                 to: vault_withdrawal_ticket.clone(),
        //             },
        //         ),
        //         required_lamports,
        //     )?;
        // }
        // let enqueue_withdraw_ix = jito_vault_sdk::sdk::enqueue_withdrawal(
        //     self.vault_program.key,
        //     self.vault_config.key,
        //     self.vault.key,
        //     vault_withdrawal_ticket.key,
        //     vault_withdrawal_ticket_token_account.key,
        //     signer.key,
        //     vault_receipt_token_account.key,
        //     vault_base_account.key,
        //     vrt_token_amount_out,
        // );
        // invoke_signed(
        //     &enqueue_withdraw_ix,
        //     &[
        //         self.vault_program.clone(),
        //         self.vault_config.clone(),
        //         self.vault.clone(),
        //         vault_withdrawal_ticket.clone(),
        //         vault_withdrawal_ticket_token_account.clone(),
        //         signer.clone(),
        //         vault_receipt_token_account.clone(),
        //         vault_base_account.clone(),
        //         self.vault_receipt_token_program.clone(),
        //         system_program.clone(),
        //     ],
        //     signer_seeds,
        // )?;

        Ok(())
    }

    pub fn find_withdrawal_tickets(
        vault_account_address: &Pubkey,
        receipt_token_mint: &Pubkey,
    ) -> Vec<(Pubkey, bool)> {
        let mut withdrawal_tickets = Vec::with_capacity(Self::BASE_ACCOUNTS_LENGTH as usize);
        for i in 0..=Self::BASE_ACCOUNTS_LENGTH - 1 {
            withdrawal_tickets.push((
                Self::find_withdrawal_ticket_account(
                    vault_account_address,
                    &Self::find_vault_base_account(receipt_token_mint, i).0,
                ),
                true,
            ));
        }
        withdrawal_tickets
    }

    pub fn find_vault_base_account(receipt_token_mint: &Pubkey, index: u8) -> (Pubkey, u8) {
        let (base, bump) = Pubkey::find_program_address(
            &[
                Self::VAULT_BASE_ACCOUNT_SEED,
                receipt_token_mint.as_ref(),
                &[index],
            ],
            &crate::ID,
        );
        (base, bump)
    }

    pub fn find_withdrawal_ticket_account(vault_account_address: &Pubkey, base: &Pubkey) -> Pubkey {
        let (withdrawal_ticket_account, _) = Pubkey::find_program_address(
            &[
                Self::VAULT_WITHDRAWAL_TICKET_SEED,
                vault_account_address.as_ref(),
                base.as_ref(),
            ],
            &JITO_VAULT_PROGRAM_ID,
        );
        withdrawal_ticket_account
    }

    pub fn find_withdrawal_ticket_token_account(
        withdrawal_ticket_account: &Pubkey,
        vault_vrt_mint: &Pubkey,
        token_program: &Pubkey,
    ) -> Pubkey {
        let withdrawal_ticket_token_account =
            associated_token::get_associated_token_address_with_program_id(
                &withdrawal_ticket_account,
                vault_vrt_mint,
                token_program,
            );
        withdrawal_ticket_token_account
    }

    pub fn check_ready_to_burn_withdrawal_ticket(
        vault_config: &'info AccountInfo<'info>,
        vault_withdrawal_ticket: &'info AccountInfo<'info>,
        slot: u64,
    ) -> Result<bool> {
        let vault_config_data = &vault_config.try_borrow_data()?;
        let vault_config =
            jito_vault_core::config::Config::try_from_slice_unchecked(vault_config_data)?;
        let epoch_length = vault_config.epoch_length();
        if vault_withdrawal_ticket.data_is_empty() && vault_withdrawal_ticket.lamports() == 0 {
            Ok(false)
        } else {
            let ticket_data_ref = vault_withdrawal_ticket.data.borrow();
            let ticket_data = ticket_data_ref.as_ref();
            let ticket = jito_vault_core::vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket::try_from_slice_unchecked(ticket_data)?;
            if ticket.is_withdrawable(slot, epoch_length)? {
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    pub fn get_claimable_withdrawal_tickets(
        vault_config: &'info AccountInfo<'info>,
        vault_vrt_mint: &Pubkey,
        vault_vrt_program: &Pubkey,
        withdrawal_tickets: Vec<&'info AccountInfo<'info>>,
    ) -> Result<(Vec<(Pubkey, Pubkey)>, i32)> {
        let clock = Clock::get()?;
        let mut claimable_unrestaked_tickets_len: i32 = 0;
        let mut claimable_unrestaked_tickets = vec![];

        for withdrawal_ticket in withdrawal_tickets {
            if JitoRestakingVaultService::check_ready_to_burn_withdrawal_ticket(
                &vault_config,
                &withdrawal_ticket,
                clock.slot,
            )? {
                let withdrawal_ticket_token_account =
                    JitoRestakingVaultService::find_withdrawal_ticket_token_account(
                        &withdrawal_ticket.key(),
                        vault_vrt_mint,
                        vault_vrt_program,
                    );
                claimable_unrestaked_tickets
                    .push((withdrawal_ticket.key(), withdrawal_ticket_token_account));
                claimable_unrestaked_tickets_len += 1;
            }
        }

        Ok((
            claimable_unrestaked_tickets,
            claimable_unrestaked_tickets_len,
        ))
    }

    pub fn find_accounts_for_unrestaking_vault(
        fund_account: &AccountInfo,
        jito_vault_program: &AccountInfo,
        jito_vault_config: &AccountInfo,
        jito_vault_account: &AccountInfo,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let vault_data_ref = jito_vault_account.data.borrow();
        let vault_data = vault_data_ref.as_ref();
        let vault = jito_vault_core::vault::Vault::try_from_slice_unchecked(vault_data)?;

        let vault_vrt_mint = vault.vrt_mint;
        let vault_vst_mint = vault.supported_mint;
        let token_program = anchor_spl::token::ID;
        let system_program = System::id();

        let fund_supported_token_account =
            spl_associated_token_account::get_associated_token_address(
                &fund_account.key(),
                &vault_vst_mint,
            );
        let fund_receipt_token_account =
            associated_token::get_associated_token_address_with_program_id(
                &fund_account.key(),
                &vault_vrt_mint,
                &token_program,
            );

        let vault_fee_receipt_token_account =
            spl_associated_token_account::get_associated_token_address(
                &ADMIN_PUBKEY,
                &vault_vrt_mint,
            );
        let vault_program_fee_wallet_vrt_account =
            spl_associated_token_account::get_associated_token_address(
                &JITO_VAULT_PROGRAM_FEE_WALLET,
                &vault_vrt_mint,
            );

        let vault_supported_token_account =
            associated_token::get_associated_token_address_with_program_id(
                &jito_vault_account.key(),
                &vault.supported_mint,
                &token_program,
            );

        Ok(vec![
            (*jito_vault_program.key, false),
            (*jito_vault_config.key, false),
            (*jito_vault_account.key, true),
            (vault_vrt_mint, true),
            (vault_vst_mint, true),
            (fund_supported_token_account, true),
            (fund_receipt_token_account, true),
            (vault_supported_token_account, true),
            (vault_fee_receipt_token_account, true),
            (vault_program_fee_wallet_vrt_account, true),
            (token_program, false),
            (system_program, false),
        ])
    }

    pub fn withdraw(
        self: &Self,
        vault_withdrawal_ticket: &AccountInfo<'info>,
        vault_withdrawal_ticket_token_account: &AccountInfo<'info>,
        fund_vault_supported_token_account: &'info AccountInfo<'info>,
        vault_fee_receipt_token_account: &AccountInfo<'info>,
        vault_program_fee_wallet_vrt_account: &AccountInfo<'info>,
        signer: &Signer<'info>,
        system_program: &AccountInfo<'info>,
    ) -> Result<u64> {
        let mut fund_vault_supported_token_account_parsed =
            InterfaceAccount::<TokenAccount>::try_from(fund_vault_supported_token_account)?;
        let fund_vault_supported_token_account_amount_before =
            fund_vault_supported_token_account_parsed.amount;
        // let vault_withdrawal_ticket_copied = vault_withdrawal_ticket.clone();
        // let vault_staker_withdrawal_ticket_data = vault_withdrawal_ticket_copied.try_borrow_data()?;
        // let vault_staker_withdrawal_ticket =
        //     VaultStakerWithdrawalTicket::try_from_slice_unchecked(&vault_staker_withdrawal_ticket_data)?;
        // let claimed_vrt_amount = vault_staker_withdrawal_ticket.vrt_amount();

        // let enqueue_withdraw_ix = jito_vault_sdk::sdk::burn_withdrawal_ticket(
        //     self.vault_program.key,
        //     self.vault_config.key,
        //     self.vault.key,
        //     self.vault_supported_token_account.key,
        //     self.vault_receipt_token_mint.key,
        //     signer.key,
        //     fund_vault_supported_token_account.key,
        //     vault_withdrawal_ticket.key,
        //     vault_withdrawal_ticket_token_account.key,
        //     vault_fee_receipt_token_account.key,
        //     vault_program_fee_wallet_vrt_account.key,
        // );
        //
        // invoke_signed(
        //     &enqueue_withdraw_ix,
        //     &[
        //         self.vault_program.clone(),
        //         self.vault_config.clone(),
        //         self.vault.clone(),
        //         self.vault_supported_token_account.clone(),
        //         self.vault_receipt_token_mint.clone(),
        //         signer.clone(),
        //         fund_vault_supported_token_account.clone(),
        //         vault_withdrawal_ticket.clone(),
        //         vault_withdrawal_ticket_token_account.clone(),
        //         vault_fee_receipt_token_account.clone(),
        //         vault_program_fee_wallet_vrt_account.clone(),
        //         self.vault_receipt_token_program.clone(),
        //         system_program.clone(),
        //     ],
        //     &[],
        // )?;
        //
        // fund_vault_supported_token_account_parsed.reload()?;
        // let fund_vault_supported_token_account_amount =
        //     fund_vault_supported_token_account_parsed.amount;
        // let unrestaked_vst_amount = fund_vault_supported_token_account_amount
        //     - fund_vault_supported_token_account_amount_before;
        // Ok(unrestaked_vst_amount)
        todo!()
    }

    fn get_status(vault: AccountInfo, vault_operators_delegation: &[AccountInfo]) -> Result<()> {
        let vault_data_ref = vault.data.borrow();
        let vault_data = vault_data_ref.as_ref();
        let vault = jito_vault_core::vault::Vault::try_from_slice_unchecked(vault_data)?;

        // Vault's status
        let vault_receipt_token_withdrawal_pending_amount = vault.vrt_cooling_down_amount();
        let vault_receipt_token_enqueued_for_withdrawal_amount =
            vault.vrt_enqueued_for_cooldown_amount();
        let vault_receipt_token_claimable_amount = vault.vrt_ready_to_claim_amount();
        let vault_receipt_token_unrestake_pending_amount =
            vault_receipt_token_withdrawal_pending_amount
                .checked_add(vault_receipt_token_enqueued_for_withdrawal_amount)
                .unwrap();
        let vault_supported_token_restaked_amount = vault.tokens_deposited();

        // Vault::DelegationState's status
        let vault_delegation_state = vault.delegation_state;
        let supported_token_cooling_down_amount = vault_delegation_state.cooling_down_amount();
        let supported_token_enqueued_for_undelegate_amount =
            vault_delegation_state.enqueued_for_cooldown_amount();
        let supported_token_undelegate_pending_amount = supported_token_cooling_down_amount
            .checked_add(supported_token_enqueued_for_undelegate_amount)
            .unwrap();
        let supported_token_delegated_amount = vault_delegation_state.staked_amount();

        // Check Validation
        let operator_count = vault.operator_count();
        if vault_operators_delegation.len() != operator_count as usize {
            msg!("the number of vault operators delegation does not match the operator count in the vault.");
            return Err(ProgramError::InvalidAccountData.into());
        }

        // VaultOperatorDelegation's status
        // let mut vault_operators_delegation_status = Vec::new();
        for vault_operator_delegation in vault_operators_delegation {
            if vault_operator_delegation.owner != &vault.operator_admin {
                msg!("owner and operator admin do not match.");
                return Err(ProgramError::InvalidAccountData.into());
            }
            let vault_operator_delegation_data_ref = vault_operator_delegation.data.borrow();
            let vault_operator_delegation_data = vault_operator_delegation_data_ref.as_ref();
            let vault_operator_delegation =
                jito_vault_core::vault_operator_delegation::VaultOperatorDelegation::try_from_slice_unchecked(vault_operator_delegation_data)?;

            let vault_operator_delegation_state = vault_operator_delegation.delegation_state;
            let operator_supported_token_delegated_amount =
                vault_operator_delegation_state.staked_amount();
            let operator_supported_token_cooling_down_amount =
                vault_operator_delegation_state.cooling_down_amount();
            let operator_supported_token_enqueued_for_undelegate_amount =
                vault_operator_delegation_state.enqueued_for_cooldown_amount();
            let operator_supported_token_undelegate_pending_amount =
                operator_supported_token_cooling_down_amount
                    .checked_add(operator_supported_token_enqueued_for_undelegate_amount)
                    .unwrap();

            // vault_operators_delegation_status.push(JitoVaultOperatorDelegation {
            //     operator_supported_token_delegated_amount,
            //     operator_supported_token_undelegate_pending_amount,
            // });
        }

        // Ok(JitoRestakingVaultStatus {
        //     vault_receipt_token_claimable_amount,
        //     vault_receipt_token_unrestake_pending_amount,
        //     vault_supported_token_restaked_amount,
        //     supported_token_undelegate_pending_amount,
        //     supported_token_delegated_amount,
        //     vault_operators_delegation_status,
        // })
        Ok(())
    }
}
