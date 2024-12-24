use crate::constants::{
    ADMIN_PUBKEY, JITO_VAULT_CONFIG_ADDRESS, JITO_VAULT_PROGRAM_FEE_WALLET, JITO_VAULT_PROGRAM_ID,
};
use crate::errors;
use crate::modules::restaking::jito::{
    JitoRestakingVault, JitoRestakingVaultContext, JitoVaultOperatorDelegation,
};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::associated_token;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token_interface::TokenAccount;
use jito_bytemuck::types::PodU64;
use jito_bytemuck::AccountDeserialize;
use jito_vault_core::vault::{BurnSummary, Vault};
use jito_vault_core::vault_operator_delegation::VaultOperatorDelegation;
use jito_vault_core::vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket;
use jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker;
use jito_vault_sdk::error::VaultError;

pub struct JitoRestakingVaultStatus {
    pub vault_receipt_token_claimable_amount: u64,
    pub vault_receipt_token_unrestake_pending_amount: u64,
    pub vault_supported_token_restaked_amount: u64,
    pub supported_token_undelegate_pending_amount: u64,
    pub supported_token_delegated_amount: u64,
    pub vault_operators_delegation_status: Vec<JitoVaultOperatorDelegation>,
}

pub struct JitoRestakingVaultService<'info> {
    vault_program: &'info AccountInfo<'info>,
    vault_config_account: &'info AccountInfo<'info>,
    vault_config: jito_vault_core::config::Config,
    vault_account: &'info AccountInfo<'info>,
    vault: jito_vault_core::vault::Vault,
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
        let vault = Self::deserialize_vault(vault_account)?;

        let current_slot = Clock::get()?.slot;
        let current_epoch = current_slot
            .checked_div(vault_config.epoch_length())
            .ok_or_else(|| error!(errors::ErrorCode::CalculationArithmeticException))?;

        Ok(Self {
            vault_program,
            vault_config_account,
            vault_config,
            vault_account,
            vault,
            current_slot,
            current_epoch,
        })
    }

    pub fn deserialize_vault(
        vault_account: &'info AccountInfo<'info>,
    ) -> Result<jito_vault_core::vault::Vault> {
        Ok(*jito_vault_core::vault::Vault::try_from_slice_unchecked(
            vault_account.try_borrow_data()?.as_ref(),
        )?)
    }

    pub fn deserialize_vault_config(
        vault_config: &'info AccountInfo<'info>,
    ) -> Result<jito_vault_core::config::Config> {
        Ok(*jito_vault_core::config::Config::try_from_slice_unchecked(
            vault_config.try_borrow_data()?.as_ref(),
        )?)
    }

    pub fn deserialize_vault_update_state_tracker(
        vault_update_state_tracker: &'info AccountInfo<'info>,
    ) -> Result<jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker> {
        Ok(*jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker::try_from_slice_unchecked(
            vault_update_state_tracker.try_borrow_data()?.as_ref(),
        )?)
    }

    pub fn deserialize_vault_operator_delegation(
        vault_operator_delegation: &'info AccountInfo<'info>,
    ) -> Result<jito_vault_core::vault_operator_delegation::VaultOperatorDelegation> {
        Ok(*jito_vault_core::vault_operator_delegation::VaultOperatorDelegation::try_from_slice_unchecked(
            vault_operator_delegation.try_borrow_data()?.as_ref(),
        )?)
    }

    fn find_vault_update_state_tracker_address(&self) -> Pubkey {
        jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker::find_program_address(
            self.vault_program.key,
            self.vault_account.key,
            self.current_epoch,
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
    pub(in crate::modules) fn get_max_cycle_fee(
        vault_account: &'info AccountInfo<'info>,
    ) -> Result<(u64, u64)> {
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
            (JITO_VAULT_CONFIG_ADDRESS, false),
            (vault_address, true),
        ])
    }

    /// returns (pubkey, writable) of [vault_program, vault_config, vault_account, system_program, vault_update_state_tracker]
    pub fn find_account_to_ensure_state_update_required(&self) -> Result<Vec<(Pubkey, bool)>> {
        let mut accounts = Self::find_accounts_to_new(self.vault_account.key())?;
        accounts.extend(vec![
            (anchor_lang::solana_program::system_program::id(), false),
            (self.find_vault_update_state_tracker_address(), true),
        ]);
        Ok(accounts)
    }

    /// Check whether vault epoch-process should be fulfilled or not.
    /// After run update_operator_delegation_state for all operators, this method needs to be called again to finalize it.
    pub fn ensure_state_update_required(
        &self,
        system_program: &'info AccountInfo<'info>,
        vault_update_state_tracker: &'info AccountInfo<'info>,

        payer: &Signer<'info>,
        payer_seeds: &[&[&[u8]]],
    ) -> Result<bool> {
        let last_updated_slot = self.vault.last_full_state_update_slot();
        let last_updated_epoch = last_updated_slot
            .checked_div(self.vault_config.epoch_length())
            .ok_or_else(|| error!(errors::ErrorCode::CalculationArithmeticException))?;

        if self.current_epoch > last_updated_epoch {
            // check new tracker is required
            if match Self::deserialize_vault_update_state_tracker(vault_update_state_tracker) {
                Ok(old_update_state_tracker) => {
                    if old_update_state_tracker.ncn_epoch() != self.current_epoch {
                        // just close out-dated state tracker
                        let close_vault_update_state_tracker_ix =
                            jito_vault_sdk::sdk::close_vault_update_state_tracker(
                                self.vault_program.key,
                                self.vault_config_account.key,
                                self.vault_account.key,
                                vault_update_state_tracker.key,
                                payer.key,
                                old_update_state_tracker.ncn_epoch(),
                            );
                        invoke_signed(
                            &close_vault_update_state_tracker_ix,
                            &[
                                self.vault_program.clone(),
                                self.vault_config_account.clone(),
                                self.vault_account.clone(),
                                vault_update_state_tracker.to_account_info(),
                                payer.to_account_info(),
                            ],
                            payer_seeds,
                        )?;
                        msg!("RESTAKING#jito vault_update_state_tracker needs to be initialized: current_epoch={}, closed_epoch={}", self.current_epoch, old_update_state_tracker.ncn_epoch());
                        true
                    } else {
                        msg!("RESTAKING#jito vault_update_state_tracker needs to be cranked/closed: current_epoch={}", self.current_epoch);
                        false
                    }
                }
                Err(err) => {
                    msg!("RESTAKING#jito vault_update_state_tracker needs to be initialized: current_epoch={}, error={}", self.current_epoch, err);
                    true
                }
            } {
                let required_space = 8 + std::mem::size_of::<
                    jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker,
                >();
                let current_lamports = vault_update_state_tracker.get_lamports();
                let required_lamports = Rent::get()?
                    .minimum_balance(required_space)
                    .max(1)
                    .saturating_sub(current_lamports);

                anchor_lang::system_program::transfer(
                    CpiContext::new_with_signer(
                        system_program.to_account_info(),
                        anchor_lang::system_program::Transfer {
                            from: payer.to_account_info(),
                            to: vault_update_state_tracker.to_account_info(),
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
                    "RESTAKING#jito initialized vault_update_state_tracker: current_epoch={}",
                    self.current_epoch
                );
            }

            // check all operator has been updated
            let update_state_tracker =
                Self::deserialize_vault_update_state_tracker(vault_update_state_tracker)?;
            let all_operators_updated = update_state_tracker
                .all_operators_updated(self.vault.operator_count())
                .unwrap_or_else(|err| {
                    msg!(
                        "RESTAKING#jito failed to compute all_operators_updated: {}",
                        err
                    );
                    false
                });

            // operators still need to be cranked
            if !all_operators_updated {
                return Ok(true);
            }

            // close the tracker and finalize current epoch process
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
                "RESTAKING#jito closed vault_update_state_tracker: current_epoch={}",
                self.current_epoch
            );
        }

        // epoch process not required
        msg!("RESTAKING#jito vault state update (epoch process) not required: current_epoch={}, last_updated_epoch={}", self.current_epoch, last_updated_epoch);
        Ok(false)
    }

    /// returns [staked_amount, enqueued_for_cooldown_amount, cooling_down_amount]
    /// in other words [restaked_amount, undelegation_requested_amount, undelegating_amount]
    pub fn update_operator_delegation_state_if_needed(
        self: &Self,
        vault_update_state_tracker: &'info AccountInfo<'info>,
        vault_operator_delegation: &'info AccountInfo<'info>,
        operator: &'info AccountInfo<'info>,

        payer: &Signer<'info>,
        payer_seeds: &[&[&[u8]]],
    ) -> Result<(u64, u64, u64)> {
        let mut delegation =
            Self::deserialize_vault_operator_delegation(vault_operator_delegation)?;
        if !match delegation.check_is_already_updated(self.current_slot, self.current_epoch) {
            Ok(_) => false,
            Err(err) => {
                msg!("RESTAKING#jito already updated operator_delegation: current_epoch={}, operator={}, error={}", self.current_epoch, operator.key(), err);
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

        msg!("RESTAKING#jito crank_vault_update_state_tracker: current_epoch={}, operator={}, staked_amount={}, enqueued_for_cooldown_amount={}, cooling_down_amount={}", self.current_epoch, operator.key, staked_amount, enqueued_for_cooldown_amount, cooling_down_amount);
        Ok((
            staked_amount,
            enqueued_for_cooldown_amount,
            cooling_down_amount,
        ))
    }

    pub fn deposit(
        self: &Self,
        fund_supported_token_account: &AccountInfo<'info>,
        vault_fee_receipt_token_account: &AccountInfo<'info>,
        vault_receipt_token_account: &'info AccountInfo<'info>,
        supported_token_amount_in: u64,
        vault_receipt_token_min_amount_out: u64,
        signer: &AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],
    ) -> Result<u64> {
        let mut vault_receipt_token_account_parsed =
            InterfaceAccount::<TokenAccount>::try_from(vault_receipt_token_account)?;
        let vault_receipt_token_account_amount_before = vault_receipt_token_account_parsed.amount;

        // let mint_to_ix = jito_vault_sdk::sdk::mint_to(
        //     self.vault_program.key,
        //     self.vault_config.key,
        //     self.vault.key,
        //     self.vault_receipt_token_mint.key,
        //     signer.key,
        //     fund_supported_token_account.key,
        //     self.vault_supported_token_account.key,
        //     vault_receipt_token_account.key,
        //     // TODO: read fee admin ata from vault state account
        //     vault_fee_receipt_token_account.key,
        //     None,
        //     supported_token_amount_in,
        //     vault_receipt_token_min_amount_out,
        // );
        //
        // invoke_signed(
        //     &mint_to_ix,
        //     &[
        //         self.vault_program.clone(),
        //         self.vault_config.clone(),
        //         self.vault.clone(),
        //         self.vault_receipt_token_mint.clone(),
        //         signer.clone(),
        //         fund_supported_token_account.clone(),
        //         self.vault_supported_token_account.clone(),
        //         vault_receipt_token_account.clone(),
        //         vault_fee_receipt_token_account.clone(),
        //         self.vault_receipt_token_program.clone(),
        //     ],
        //     signer_seeds,
        // )?;
        //
        // vault_receipt_token_account_parsed.reload()?;
        // let vault_receipt_token_account_amount = vault_receipt_token_account_parsed.amount;
        // let minted_vrt_amount =
        //     vault_receipt_token_account_amount - vault_receipt_token_account_amount_before;
        //
        // require_gte!(minted_vrt_amount, supported_token_amount_in);
        // Ok(minted_vrt_amount)
        todo!()
    }

    fn get_status(
        vault: AccountInfo,
        vault_operators_delegation: &[AccountInfo],
    ) -> Result<JitoRestakingVaultStatus> {
        let vault_data_ref = vault.data.borrow();
        let vault_data = vault_data_ref.as_ref();
        let vault = Vault::try_from_slice_unchecked(vault_data)?;

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
        let mut vault_operators_delegation_status = Vec::new();
        for vault_operator_delegation in vault_operators_delegation {
            if vault_operator_delegation.owner != &vault.operator_admin {
                msg!("owner and operator admin do not match.");
                return Err(ProgramError::InvalidAccountData.into());
            }
            let vault_operator_delegation_data_ref = vault_operator_delegation.data.borrow();
            let vault_operator_delegation_data = vault_operator_delegation_data_ref.as_ref();
            let vault_operator_delegation =
                VaultOperatorDelegation::try_from_slice_unchecked(vault_operator_delegation_data)?;

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

            vault_operators_delegation_status.push(JitoVaultOperatorDelegation {
                operator_supported_token_delegated_amount,
                operator_supported_token_undelegate_pending_amount,
            });
        }

        Ok(JitoRestakingVaultStatus {
            vault_receipt_token_claimable_amount,
            vault_receipt_token_unrestake_pending_amount,
            vault_supported_token_restaked_amount,
            supported_token_undelegate_pending_amount,
            supported_token_delegated_amount,
            vault_operators_delegation_status,
        })
    }

    fn get_vault_operator_delegation_key(self: &Self, operator: &Pubkey) -> Pubkey {
        let (vault_operator_delegation_key, _, _) = VaultOperatorDelegation::find_program_address(
            self.vault_program.key,
            self.vault_account.key,
            operator,
        );
        vault_operator_delegation_key
    }

    pub fn find_accounts_for_restaking_vault(
        fund_account: &AccountInfo,
        jito_vault_program: &AccountInfo,
        jito_vault_config: &AccountInfo,
        jito_vault_account: &AccountInfo,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let vault_data_ref = jito_vault_account.data.borrow();
        let vault_data = vault_data_ref.as_ref();
        let vault = Vault::try_from_slice_unchecked(vault_data)?;

        let vault_vrt_mint = vault.vrt_mint;
        let vault_vst_mint = vault.supported_mint;
        let token_program = anchor_spl::token::ID;
        let system_program = System::id();
        let fund_supported_token_account =
            anchor_spl::associated_token::get_associated_token_address_with_program_id(
                &fund_account.key(),
                &vault_vst_mint,
                &token_program,
            );

        let vault_fee_wallet_token_account =
            associated_token::get_associated_token_address_with_program_id(
                &vault.fee_wallet,
                &vault_vrt_mint,
                &token_program,
            );
        let fund_receipt_token_account =
            associated_token::get_associated_token_address_with_program_id(
                &fund_account.key(),
                &vault_vrt_mint,
                &token_program,
            );
        let vault_supported_token_account =
            associated_token::get_associated_token_address_with_program_id(
                &jito_vault_account.key(),
                &vault.supported_mint,
                &token_program,
            );

        Ok(vec![
            (*jito_vault_program.key, false),
            (*jito_vault_config.key, true),
            (*jito_vault_account.key, true),
            (vault_vrt_mint, true),
            (vault_vst_mint, true),
            (fund_supported_token_account, true),
            (fund_receipt_token_account, true),
            (vault_supported_token_account, true),
            (vault_fee_wallet_token_account, true),
            (token_program, false),
            (system_program, false),
        ])
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
        let vault = Vault::try_from_slice_unchecked(&vault_data_ref)?;

        let vault_receipt_token_mint = vault.vrt_mint;
        let vault_supported_token_mint = vault.supported_mint;
        let token_program = anchor_spl::token::ID;
        let vault_supported_token_account =
            associated_token::get_associated_token_address_with_program_id(
                &vault_account.key(),
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
            let ticket = VaultStakerWithdrawalTicket::try_from_slice_unchecked(ticket_data)?;
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
        let vault = Vault::try_from_slice_unchecked(vault_data)?;

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
}
