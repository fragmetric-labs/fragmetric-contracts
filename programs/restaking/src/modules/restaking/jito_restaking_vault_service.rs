use crate::constants::{
    ADMIN_PUBKEY, FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS, FRAGSOL_JITO_VAULT_CONFIG_ADDRESS,
    FRAGSOL_MINT_ADDRESS, JITO_VAULT_PROGRAM_FEE_WALLET, JITO_VAULT_PROGRAM_ID, NSOL_MINT_ADDRESS,
};
use crate::modules::restaking::jito::{
    JitoRestakingVault, JitoRestakingVaultContext, JitoVaultOperatorDelegation,
};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::associated_token;
use anchor_spl::associated_token::spl_associated_token_account;
use anchor_spl::token_interface::TokenAccount;
use jito_bytemuck::AccountDeserialize;
use jito_vault_core::config::Config;
use jito_vault_core::vault::{BurnSummary, Vault};
use jito_vault_core::vault_operator_delegation::VaultOperatorDelegation;
use jito_vault_core::vault_staker_withdrawal_ticket::VaultStakerWithdrawalTicket;
use jito_vault_core::vault_update_state_tracker::VaultUpdateStateTracker;
use crate::errors;

pub struct JitoRestakingVaultStatus {
    pub vault_receipt_token_claimable_amount: u64,
    pub vault_receipt_token_unrestake_pending_amount: u64,
    pub vault_supported_token_restaked_amount: u64,
    pub supported_token_undelegate_pending_amount: u64,
    pub supported_token_delegated_amount: u64,
    pub vault_operators_delegation_status: Vec<JitoVaultOperatorDelegation>,
}

pub struct JitoRestakingVaultService<'info> {
    pub vault_program: AccountInfo<'info>,
    pub vault_config: AccountInfo<'info>,
    pub vault: AccountInfo<'info>,
    pub vault_receipt_token_mint: AccountInfo<'info>,
    pub vault_receipt_token_program: AccountInfo<'info>,
    pub vault_supported_token_mint: AccountInfo<'info>,
    pub vault_supported_token_program: AccountInfo<'info>,
    pub vault_supported_token_account: AccountInfo<'info>,
}

impl<'info> JitoRestakingVaultService<'info> {
    pub const VAULT_BASE_ACCOUNT_SEED: &'static [u8] = b"vault_base_account";
    pub const VAULT_WITHDRAWAL_TICKET_SEED: &'static [u8] = b"vault_staker_withdrawal_ticket";
    pub const BASE_ACCOUNTS_LENGTH: u8 = 5;
    pub fn new(
        vault_program: AccountInfo<'info>,
        vault_config: AccountInfo<'info>,
        vault: AccountInfo<'info>,
        vault_receipt_token_mint: AccountInfo<'info>,
        vault_receipt_token_program: AccountInfo<'info>,
        vault_supported_token_mint: AccountInfo<'info>,
        vault_supported_token_program: AccountInfo<'info>,
        vault_supported_token_account: AccountInfo<'info>,
    ) -> Result<Self> {
        require_eq!(JITO_VAULT_PROGRAM_ID, vault_program.key());
        Ok(Self {
            vault_program,
            vault_config,
            vault,
            vault_receipt_token_mint,
            vault_receipt_token_program,
            vault_supported_token_mint,
            vault_supported_token_program,
            vault_supported_token_account,
        })
    }


    pub fn get_status(
        vault: AccountInfo,
        vault_operators_delegation: &[AccountInfo],
    ) -> Result<JitoRestakingVaultStatus> {
        require_eq!(FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS, vault.key());

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

    pub fn get_vault_operator_delegation_key(self: &Self, operator: &Pubkey) -> Pubkey {
        let (vault_operator_delegation_key, _, _) = VaultOperatorDelegation::find_program_address(
            self.vault_program.key,
            self.vault.key,
            operator,
        );
        vault_operator_delegation_key
    }

    pub fn find_accounts_for_vault(vault: Pubkey) -> Result<Vec<(Pubkey, bool)>> {
        require_eq!(vault, FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS);
        Ok(vec![
            (JITO_VAULT_PROGRAM_ID, false),
            (FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS, false),
            (FRAGSOL_JITO_VAULT_CONFIG_ADDRESS, false),
        ])
    }

    pub fn find_accounts_for_restaking_vault(
        fund_account: &AccountInfo,
        jito_vault_program: &AccountInfo,
        jito_vault_account: &AccountInfo,
        jito_vault_config: &AccountInfo,
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
        let clock = Clock::get()?;
        let vault_update_state_tracker =
            Self::get_vault_update_state_tracker(jito_vault_config, clock.slot, false)?;

        let vault_update_state_tracker_prepare_for_delaying =
            Self::get_vault_update_state_tracker(jito_vault_config, clock.slot, true)?;

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
            (*jito_vault_account.key, true),
            (*jito_vault_config.key, true),
            (vault_update_state_tracker.key(), false),
            (vault_update_state_tracker_prepare_for_delaying.key(), false),
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

    pub fn get_vault_update_state_tracker(
        jito_vault_config: &AccountInfo,
        current_slot: u64,
        delayed: bool,
    ) -> Result<Pubkey> {
        let data = jito_vault_config
            .try_borrow_data()
            .map_err(|e| Error::from(e).with_account_name("jito_vault_update_state_tracker"))?;
        let config = Config::try_from_slice_unchecked(&data)
            .map_err(|e| Error::from(e).with_account_name("jito_vault_update_state_tracker"))?;
        let mut ncn_epoch = current_slot
            .checked_div(config.epoch_length())
            .ok_or_else(|| error!(crate::errors::ErrorCode::CalculationArithmeticException))?;

        if delayed {
            ncn_epoch += 1
        }

        let (vault_update_state_tracker, _) = Pubkey::find_program_address(
            &[
                b"vault_update_state_tracker",
                FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS.as_ref(),
                &ncn_epoch.to_le_bytes(),
            ],
            &JITO_VAULT_PROGRAM_ID,
        );

        Ok(vault_update_state_tracker)
    }

    pub fn find_current_vault_update_state_tracker(
        vault_update_state_tracker: &'info AccountInfo<'info>,
        next_vault_update_state_tracker: &'info AccountInfo<'info>,
    ) -> Result<&'info AccountInfo<'info>> {
        let data = vault_update_state_tracker
            .try_borrow_data()
            .map_err(|e| Error::from(e).with_account_name("jito_vault_update_state_tracker"))?;
        let tracker = VaultUpdateStateTracker::try_from_slice_unchecked(&data)
            .map_err(|e| Error::from(e).with_account_name("jito_vault_update_state_tracker"))?;
        if Clock::get()?.epoch == tracker.ncn_epoch() {
            Ok(vault_update_state_tracker)
        } else {
            Ok(next_vault_update_state_tracker)
        }
    }

    pub fn update_vault_if_needed(
        self: &Self,
        operator: &Signer<'info>,
        vault_update_state_tracker: &'info AccountInfo<'info>,
        next_vault_update_state_tracker: &'info AccountInfo<'info>,
        current_slot: u64,
        system_program: &AccountInfo<'info>,
        signer: &AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],
    ) -> Result<&Self> {
        let epoch_length =
            Config::try_from_slice_unchecked(&self.vault_config.try_borrow_data()?)?.epoch_length();

        let current_epoch = current_slot
            .checked_div(epoch_length)
            .ok_or_else(|| error!(crate::errors::ErrorCode::CalculationArithmeticException))?;

        let updated_slot = Vault::try_from_slice_unchecked(&self.vault.try_borrow_data()?)?
            .last_full_state_update_slot();
        let updated_epoch = updated_slot
            .checked_div(epoch_length)
            .ok_or_else(|| error!(crate::errors::ErrorCode::CalculationArithmeticException))?;

        if current_epoch > updated_epoch {
            let current_vault_update_state_tracker = Self::find_current_vault_update_state_tracker(
                vault_update_state_tracker,
                next_vault_update_state_tracker,
            )?;
            let rent = Rent::get()?;
            let current_lamports = current_vault_update_state_tracker.get_lamports();
            let space = 8 + std::mem::size_of::<VaultUpdateStateTracker>();
            let required_lamports = rent
                .minimum_balance(space)
                .max(1)
                .saturating_sub(current_lamports);

            anchor_lang::system_program::transfer(
                CpiContext::new(
                    system_program.clone(),
                    anchor_lang::system_program::Transfer {
                        from: operator.to_account_info(),
                        to: current_vault_update_state_tracker.clone(),
                    },
                ),
                required_lamports,
            )?;

            Self::initialize_vault_update_state_tracker(
                self,
                signer,
                signer_seeds,
                current_vault_update_state_tracker,
                system_program,
            )?;

            // TODO : need to crank_update_state_tracker for each operator

            Self::close_vault_update_state_tracker(
                self,
                signer,
                signer_seeds,
                current_vault_update_state_tracker,
                current_epoch,
            )?;
        }
        Ok(&self)
    }

    pub fn initialize_vault_update_state_tracker(
        self: &Self,
        signer: &AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],
        vault_update_state_tracker: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
    ) -> Result<()> {
        let initialize_vault_update_state_tracker_ix =
            jito_vault_sdk::sdk::initialize_vault_update_state_tracker(
                self.vault_program.key,
                self.vault_config.key,
                self.vault.key,
                vault_update_state_tracker.key,
                signer.key,
                TryFrom::try_from(0u8).unwrap(), // WithdrawalAllocationMethod
            );

        invoke_signed(
            &initialize_vault_update_state_tracker_ix,
            &[
                self.vault_program.clone(),
                self.vault_config.clone(),
                self.vault.clone(),
                vault_update_state_tracker.clone(),
                signer.clone(),
                system_program.clone(),
            ],
            signer_seeds,
        )?;

        Ok(())
    }
    pub fn crank_vault_update_state_tracker(
        self: &Self,
        signer: &AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],
        restaking_operator: &AccountInfo<'info>,
        vault_operator_delegation: &AccountInfo<'info>,
        vault_update_state_tracker: &AccountInfo<'info>,
    ) -> Result<()> {
        let initialize_vault_update_state_tracker_ix =
            jito_vault_sdk::sdk::crank_vault_update_state_tracker(
                self.vault_program.key,
                self.vault_config.key,
                self.vault.key,
                restaking_operator.key,
                vault_operator_delegation.key,
                vault_update_state_tracker.key,
            );

        invoke_signed(
            &initialize_vault_update_state_tracker_ix,
            &[
                self.vault_program.clone(),
                self.vault_config.clone(),
                self.vault.clone(),
                restaking_operator.clone(),
                vault_operator_delegation.clone(),
                vault_update_state_tracker.clone(),
                signer.clone(),
            ],
            signer_seeds,
        )?;

        Ok(())
    }

    pub fn close_vault_update_state_tracker(
        self: &Self,
        signer: &AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],
        vault_update_state_tracker: &AccountInfo<'info>,
        ncn_epoch: u64,
    ) -> Result<()> {
        let close_vault_update_state_tracker_ix =
            jito_vault_sdk::sdk::close_vault_update_state_tracker(
                self.vault_program.key,
                self.vault_config.key,
                self.vault.key,
                vault_update_state_tracker.key,
                signer.key,
                ncn_epoch, // Clock::get()?.slot.checked_div(432000).unwrap(), need to change 432000 -> config.epoch_length
            );

        invoke_signed(
            &close_vault_update_state_tracker_ix,
            &[
                self.vault_program.clone(),
                self.vault_config.clone(),
                self.vault.clone(),
                vault_update_state_tracker.clone(),
                signer.clone(),
            ],
            signer_seeds,
        )?;

        Ok(())
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

        let mint_to_ix = jito_vault_sdk::sdk::mint_to(
            self.vault_program.key,
            self.vault_config.key,
            self.vault.key,
            self.vault_receipt_token_mint.key,
            signer.key,
            fund_supported_token_account.key,
            self.vault_supported_token_account.key,
            vault_receipt_token_account.key,
            // TODO: read fee admin ata from vault state account
            vault_fee_receipt_token_account.key,
            None,
            supported_token_amount_in,
            vault_receipt_token_min_amount_out,
        );

        invoke_signed(
            &mint_to_ix,
            &[
                self.vault_program.clone(),
                self.vault_config.clone(),
                self.vault.clone(),
                self.vault_receipt_token_mint.clone(),
                signer.clone(),
                fund_supported_token_account.clone(),
                self.vault_supported_token_account.clone(),
                vault_receipt_token_account.clone(),
                vault_fee_receipt_token_account.clone(),
                self.vault_receipt_token_program.clone(),
            ],
            signer_seeds,
        )?;

        vault_receipt_token_account_parsed.reload()?;
        let vault_receipt_token_account_amount = vault_receipt_token_account_parsed.amount;
        let minted_vrt_amount =
            vault_receipt_token_account_amount - vault_receipt_token_account_amount_before;

        require_gte!(minted_vrt_amount, supported_token_amount_in);
        Ok(minted_vrt_amount)
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
        vault_account: &AccountInfo,
        vault_config: &AccountInfo,
    ) -> Result<Vec<(Pubkey, bool)>> {
        let vault_data_ref = vault_account.data.borrow();
        let vault_data = vault_data_ref.as_ref();
        let vault = Vault::try_from_slice_unchecked(vault_data)?;

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
            (vault_account.key(), false),
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
        system_program: &AccountInfo<'info>,
        signer: &AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],
        vrt_token_amount_out: u64,
    ) -> Result<()> {
        anchor_spl::associated_token::create(CpiContext::new(
            self.vault_receipt_token_program.clone(),
            anchor_spl::associated_token::Create {
                payer: operator.to_account_info(),
                associated_token: vault_withdrawal_ticket_token_account.clone(),
                authority: vault_withdrawal_ticket.clone(),
                mint: self.vault_receipt_token_mint.clone(),
                system_program: system_program.clone(),
                token_program: self.vault_receipt_token_program.clone(),
            },
        ))?;

        let rent = Rent::get()?;
        let current_lamports = vault_withdrawal_ticket.lamports();
        let space = 8 + std::mem::size_of::<VaultStakerWithdrawalTicket>();
        let required_lamports = rent
            .minimum_balance(space)
            .max(1)
            .saturating_sub(current_lamports);

        if required_lamports > 0 {
            anchor_lang::system_program::transfer(
                CpiContext::new(
                    system_program.clone(),
                    anchor_lang::system_program::Transfer {
                        from: operator.to_account_info(),
                        to: vault_withdrawal_ticket.clone(),
                    },
                ),
                required_lamports,
            )?;
        }

        let enqueue_withdraw_ix = jito_vault_sdk::sdk::enqueue_withdrawal(
            self.vault_program.key,
            self.vault_config.key,
            self.vault.key,
            vault_withdrawal_ticket.key,
            vault_withdrawal_ticket_token_account.key,
            signer.key,
            vault_receipt_token_account.key,
            vault_base_account.key,
            vrt_token_amount_out,
        );

        invoke_signed(
            &enqueue_withdraw_ix,
            &[
                self.vault_program.clone(),
                self.vault_config.clone(),
                self.vault.clone(),
                vault_withdrawal_ticket.clone(),
                vault_withdrawal_ticket_token_account.clone(),
                signer.clone(),
                vault_receipt_token_account.clone(),
                vault_base_account.clone(),
                self.vault_receipt_token_program.clone(),
                system_program.clone(),
            ],
            signer_seeds,
        )?;

        Ok(())
    }

    pub fn find_withdrawal_tickets() -> Vec<(Pubkey, bool)> {
        let mut withdrawal_tickets = Vec::with_capacity(Self::BASE_ACCOUNTS_LENGTH as usize);
        for i in 1..=Self::BASE_ACCOUNTS_LENGTH {
            withdrawal_tickets.push((
                Self::find_withdrawal_ticket_account(&Self::find_vault_base_account(i)),
                false,
            ));
        }
        withdrawal_tickets
    }

    pub fn find_vault_base_account(index: u8) -> Pubkey {
        let (base, _) = Pubkey::find_program_address(
            &[
                Self::VAULT_BASE_ACCOUNT_SEED,
                (FRAGSOL_MINT_ADDRESS as Pubkey).as_ref(),
                &[index],
            ],
            &JITO_VAULT_PROGRAM_ID,
        );

        base
    }

    pub fn find_withdrawal_ticket_account(base: &Pubkey) -> Pubkey {
        let vault_account: Pubkey = FRAGSOL_JITO_VAULT_ACCOUNT_ADDRESS;
        let (withdrawal_ticket_account, _) = Pubkey::find_program_address(
            &[
                Self::VAULT_WITHDRAWAL_TICKET_SEED,
                vault_account.as_ref(),
                base.as_ref(),
            ],
            &JITO_VAULT_PROGRAM_ID,
        );
        withdrawal_ticket_account
    }

    pub fn find_withdrawal_ticket_token_account(withdrawal_ticket_account: &Pubkey) -> Pubkey {
        let withdrawal_ticket_token_account = associated_token::get_associated_token_address(
            &withdrawal_ticket_account,
            &NSOL_MINT_ADDRESS,
        );
        withdrawal_ticket_token_account
    }

    pub fn check_ready_to_burn_withdrawal_ticket(
        vault_config: &'info AccountInfo<'info>,
        vault_withdrawal_ticket: &'info AccountInfo<'info>,
        slot: u64,
    ) -> Result<bool> {
        let vault_config_data = &vault_config.try_borrow_data()?;
        let vault_config = Config::try_from_slice_unchecked(vault_config_data)?;
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
        withdrawal_tickets: Vec<&'info AccountInfo<'info>>,
    ) -> Result<(Vec<(Pubkey, Pubkey)>, i32)> {
        let clock = Clock::get().unwrap();
        let mut claimable_unrestaked_tickets_len: i32 = 0;
        let mut claimable_unrestaked_tickets = vec![];

        for withdrawal_ticket in withdrawal_tickets {
            if JitoRestakingVaultService::check_ready_to_burn_withdrawal_ticket(
                &vault_config,
                &withdrawal_ticket,
                clock.slot,
            ).unwrap() {
                let withdrawal_ticket_token_account =
                    JitoRestakingVaultService::find_withdrawal_ticket_token_account(
                        &withdrawal_ticket.key(),
                    );
                claimable_unrestaked_tickets.push((withdrawal_ticket.key(), withdrawal_ticket_token_account));
                claimable_unrestaked_tickets_len += 1;
            }
        }

        Ok((claimable_unrestaked_tickets, claimable_unrestaked_tickets_len))
    }


    pub fn find_accounts_for_unrestaking_vault(
        fund_account: &AccountInfo,
        jito_vault_program: &AccountInfo,
        jito_vault_account: &AccountInfo,
        jito_vault_config: &AccountInfo,
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


        let clock = Clock::get()?;
        let vault_update_state_tracker =
            Self::get_vault_update_state_tracker(jito_vault_config, clock.slot, false)?;

        let vault_update_state_tracker_prepare_for_delaying =
            Self::get_vault_update_state_tracker(jito_vault_config, clock.slot, true)?;

        Ok(vec![
            (*jito_vault_program.key, false),
            (*jito_vault_account.key, false),
            (*jito_vault_config.key, false),
            (vault_vrt_mint, false),
            (vault_vst_mint, false),
            (fund_supported_token_account, false),
            (fund_receipt_token_account, false),
            (vault_supported_token_account, false),
            (vault_fee_receipt_token_account, false),
            (vault_program_fee_wallet_vrt_account, false),
            (vault_update_state_tracker, false),
            (vault_update_state_tracker_prepare_for_delaying, false),
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
        signer: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
    ) -> Result<u64> {
        let mut fund_vault_supported_token_account_parsed =
            InterfaceAccount::<TokenAccount>::try_from(fund_vault_supported_token_account)?;
        let fund_vault_supported_token_account_amount_before = fund_vault_supported_token_account_parsed.amount;
        let ticket_data_ref = vault_withdrawal_ticket.data.borrow();
        let ticket_data = ticket_data_ref.as_ref();
        let vault_staker_withdrawal_ticket = VaultStakerWithdrawalTicket::try_from_slice_unchecked(ticket_data)?;
        let claimed_vrt_amount = vault_staker_withdrawal_ticket.vrt_amount();;

        let enqueue_withdraw_ix = jito_vault_sdk::sdk::burn_withdrawal_ticket(
            self.vault_program.key,
            self.vault_config.key,
            self.vault.key,
            self.vault_supported_token_account.key,
            self.vault_receipt_token_mint.key,
            signer.key,
            fund_vault_supported_token_account.key,
            vault_withdrawal_ticket.key,
            vault_withdrawal_ticket_token_account.key,
            vault_fee_receipt_token_account.key,
            vault_program_fee_wallet_vrt_account.key,
        );

        invoke_signed(
            &enqueue_withdraw_ix,
            &[
                self.vault_program.clone(),
                self.vault_config.clone(),
                self.vault.clone(),
                self.vault_supported_token_account.clone(),
                self.vault_receipt_token_mint.clone(),
                signer.clone(),
                fund_vault_supported_token_account.clone(),
                vault_withdrawal_ticket.clone(),
                vault_withdrawal_ticket_token_account.clone(),
                vault_fee_receipt_token_account.clone(),
                vault_program_fee_wallet_vrt_account.clone(),
                self.vault_receipt_token_program.clone(),
                system_program.clone(),
            ],
            &[],
        )?;


        fund_vault_supported_token_account_parsed.reload()?;
        let fund_vault_supported_token_account_amount = fund_vault_supported_token_account_parsed.amount;
        let unrestaked_vst_amount =
            fund_vault_supported_token_account_amount - fund_vault_supported_token_account_amount_before;

        Ok(unrestaked_vst_amount)
    }
}
