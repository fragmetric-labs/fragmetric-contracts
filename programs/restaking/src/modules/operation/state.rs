use super::command::{InitializationCommand, OperationCommand, OperationCommandContext, OperationCommandEntry, SelfExecutable};
use crate::errors::ErrorCode;
use anchor_lang::prelude::*;
use std::collections::HashMap;

const OPERATION_COMMANDS_STACK_SIZE: usize = 16;

const OPERATION_COMMANDS_EXPIRATION_SECONDS: i64 = 1800;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize)]
pub(in crate::modules) struct OperationState {
    #[max_len(OPERATION_COMMANDS_STACK_SIZE)]
    commands: Vec<OperationCommandEntry>,
    updated_at: i64,
    expired_at: i64,
}

impl OperationState {
    pub(in crate::modules) fn new() -> Self {
        OperationState {
            commands: vec![],
            updated_at: 0,
            expired_at: 0,
        }
    }

    fn initialize(&mut self, context: &OperationCommandContext, current_timestamp: i64) -> Result<()> {
        self.commands.clear();

        let init_command_entry = OperationCommand::Initialization(InitializationCommand {}).build(context)?;
        self.push_commands(vec![init_command_entry], current_timestamp);
        self.expired_at = current_timestamp + OPERATION_COMMANDS_EXPIRATION_SECONDS;
        Ok(())
    }

    fn push_commands(&mut self, commands: Vec<OperationCommandEntry>, current_timestamp: i64) {
        self.commands.extend(commands);
        self.updated_at = current_timestamp;
    }

    fn pop_command(&mut self) -> Option<OperationCommandEntry> {
        self.commands.pop()
    }

    fn initialize_if_needed(&mut self, context: &OperationCommandContext, current_timestamp: i64, forced_reset: bool) -> Result<()> {
        if forced_reset || current_timestamp > self.expired_at || self.commands.is_empty() {
            self.initialize(context, current_timestamp)?
        }
        Ok(())
    }
}

fn run(state: &mut OperationState, context: &OperationCommandContext, remaining_accounts: Vec<AccountInfo>, current_timestamp: i64, forced_reset: bool) -> Result<()> {
    state.initialize_if_needed(context, current_timestamp, forced_reset)?;

    let mut run_count = 0;
    let remaining_accounts_map: HashMap<Pubkey, &AccountInfo> = remaining_accounts.iter()
        .map(|info| (info.key.clone(), info)).collect();

    while let Some(command_entry) = state.pop_command() {
        let OperationCommandEntry { command, required_accounts } = command_entry;
        let given_accounts: std::result::Result<Vec<&AccountInfo>, ProgramError> = required_accounts
            .iter()
            .map(|key| remaining_accounts_map.get(key).copied().ok_or(ProgramError::NotEnoughAccountKeys))
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
                state.push_commands(next_commands, current_timestamp);
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