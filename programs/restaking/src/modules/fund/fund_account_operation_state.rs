use super::command::{
    InitializeCommand, OperationCommand, OperationCommandContext, OperationCommandEntry,
    SelfExecutable,
};
use crate::errors::ErrorCode;
use crate::modules::operation;
use anchor_lang::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

const OPERATION_COMMANDS_STACK_SIZE: usize = 2;
const OPERATION_COMMANDS_EXPIRATION_SECONDS: i64 = 600;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub(super) struct OperationState {
    updated_at: i64,
    expired_at: i64,
    #[max_len(OPERATION_COMMANDS_STACK_SIZE)]
    commands: Vec<OperationCommandEntry>,
    _reserved: [[u8; 32]; 8],
}

impl OperationState {
    pub(super) fn initialize(&mut self, fund_account_data_version: u16) {
        if fund_account_data_version == 4 {
            self.updated_at = 0;
            self.expired_at = 0;
            self.commands = Default::default();
            self._reserved = Default::default();
        }
    }

    fn reset_commands_if_needed(
        &mut self,
        current_timestamp: i64,
        forced_reset: bool,
    ) -> Result<()> {
        if forced_reset || current_timestamp > self.expired_at || self.commands.is_empty() {
            self.commands.clear();

            let init_command_entry =
                OperationCommand::Initialize(InitializeCommand {}).with_required_accounts(vec![]);
            self.push_commands(vec![init_command_entry], current_timestamp);
        }
        Ok(())
    }

    fn push_commands(&mut self, commands: Vec<OperationCommandEntry>, current_timestamp: i64) {
        self.commands.extend(commands);
        self.updated_at = current_timestamp;
        self.expired_at = current_timestamp + OPERATION_COMMANDS_EXPIRATION_SECONDS;
    }

    pub(super) fn get_remaining_commands_count(&self) -> u8 {
        self.commands.len() as u8
    }

    pub(super) fn run_commands(
        &mut self,
        ctx: &mut OperationCommandContext,
        remaining_accounts: &[AccountInfo],
        current_timestamp: i64,
        _current_slot: u64,
    ) -> Result<()> {
        self.reset_commands_if_needed(current_timestamp, false)?;

        let mut run_count = 0;
        let remaining_accounts_map: BTreeMap<Pubkey, &AccountInfo> = remaining_accounts
            .iter()
            .map(|info| (info.key.clone(), info))
            .collect();

        while let Some(command_entry) = self.commands.pop() {
            let OperationCommandEntry {
                command,
                required_accounts,
            } = command_entry;

            // rearrange given accounts in required order
            let mut required_account_infos = Vec::new();
            let mut unused_account_keys = BTreeSet::new();
            remaining_accounts_map.keys().for_each(|key| {
                unused_account_keys.insert(*key);
            });

            for account_key in required_accounts.iter() {
                // reject invalid requirements from command to prevent redundant account info creation
                if *account_key == ctx.fund_account.key()
                    || *account_key == ctx.receipt_token_mint.key()
                {
                    return err!(ErrorCode::OperationCommandAccountComputationException);
                }

                // append required accounts in exact order
                match remaining_accounts_map.get(&account_key) {
                    Some(account) => {
                        required_account_infos.push((*account).clone());
                        unused_account_keys.remove(&account_key);
                    }
                    None => {
                        if run_count > 0 {
                            // restore the current command and gracefully stop executing commands
                            msg!(
                                "COMMAND: {:?} has not enough accounts after {} run(s)",
                                command,
                                run_count
                            );
                            self.commands.insert(
                                0,
                                OperationCommandEntry {
                                    command,
                                    required_accounts,
                                },
                            );
                            return Ok(());
                        }
                        // error if it is the first command in this tx
                        msg!(
                            "COMMAND: {:?} has not enough accounts at the first run",
                            command
                        );
                        return err!(ErrorCode::OperationCommandAccountComputationException);
                    }
                }
            }

            // append all unused accounts
            for unused_account_key in unused_account_keys.iter() {
                let remaining_account = remaining_accounts_map.get(unused_account_key).unwrap();
                required_account_infos.push((*remaining_account).clone().clone());
            }

            match command.execute(ctx, required_account_infos.as_slice()) {
                Ok(next_commands) => {
                    // msg!("COMMAND: {:?} with {:?} passed", command, required_accounts);
                    msg!("COMMAND: {:?} passed", command);
                    self.push_commands(next_commands, current_timestamp);
                    run_count += 1;
                }
                Err(error) => {
                    // msg!("COMMAND: {:?} with {:?} failed", command, required_accounts);
                    msg!("COMMAND: {:?} failed", command);
                    return Err(error);
                }
            };
        }

        // there is no commands to execute
        Ok(())
    }
}
