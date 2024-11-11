use super::command::{
    InitializeCommand, OperationCommand, OperationCommandContext, OperationCommandEntry,
    SelfExecutable,
};
use crate::errors::ErrorCode;
use crate::modules::operation;
use anchor_lang::prelude::*;
use std::collections::BTreeMap;

const OPERATION_COMMANDS_STACK_SIZE: usize = 4;
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
        context: &OperationCommandContext,
        current_timestamp: i64,
        forced_reset: bool,
    ) -> Result<()> {
        if forced_reset || current_timestamp > self.expired_at || self.commands.is_empty() {
            self.commands.clear();

            let init_command_entry =
                OperationCommand::Initialize(InitializeCommand {}).build(context)?;
            self.push_commands(vec![init_command_entry], current_timestamp);
        }
        Ok(())
    }

    fn push_commands(&mut self, commands: Vec<OperationCommandEntry>, current_timestamp: i64) {
        self.commands.extend(commands);
        self.updated_at = current_timestamp;
        self.expired_at = current_timestamp + OPERATION_COMMANDS_EXPIRATION_SECONDS;
    }

    pub(super) fn run_commands(
        &mut self,
        context: &OperationCommandContext,
        remaining_accounts: Vec<AccountInfo>,
        current_timestamp: i64,
        _current_slot: u64,
        forced_reset: bool,
    ) -> Result<()> {
        self.reset_commands_if_needed(context, current_timestamp, forced_reset)?;

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
            let given_accounts: std::result::Result<Vec<&AccountInfo>, ProgramError> =
                required_accounts
                    .iter()
                    .map(|key| {
                        remaining_accounts_map
                            .get(key)
                            .copied()
                            .ok_or(ProgramError::NotEnoughAccountKeys)
                    })
                    .collect();

            if let Err(err) = given_accounts {
                if run_count > 0 {
                    // gracefully stop executing commands
                    return Ok(());
                }
                // fail if it is the first command in this tx
                return Err(err.into());
            }

            match command.execute(context, given_accounts?) {
                Ok(next_commands) => {
                    msg!("succeeded to execute command: {:?}", command);
                    self.push_commands(next_commands, current_timestamp);
                    run_count += 1;
                }
                Err(error) => {
                    msg!("failed to execute command: {}", error);
                    err!(ErrorCode::OperationCommandExecutionFailedException)?;
                }
            };
        }

        // there is no commands to execute
        Ok(())
    }
}
