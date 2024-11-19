use anchor_lang::prelude::*;

use super::command::{InitializeCommand, OperationCommand, OperationCommandEntry};

const OPERATION_COMMANDS_EXPIRATION_SECONDS: i64 = 600;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub(super) struct OperationState {
    updated_at: i64,
    expired_at: i64,
    sequence: u16,
    command: Option<OperationCommandEntry>,
    _reserved: [[u8; 32]; 8],
}

impl OperationState {
    pub(super) fn initialize(&mut self, fund_account_data_version: u16) {
        if fund_account_data_version == 4 {
            self.updated_at = 0;
            self.expired_at = 0;
            self.sequence = 0;
            self.command = None;
            self._reserved = Default::default();
        }
    }

    /// Initialize current operation command to `reset_command` or default.
    pub(super) fn initialize_command_if_needed(
        &mut self,
        current_timestamp: i64,
        reset_command: Option<OperationCommandEntry>,
    ) -> Result<()> {
        if reset_command.is_some() || current_timestamp > self.expired_at || self.command.is_none()
        {
            self.set_command(
                reset_command.or_else(|| {
                    Some(
                        OperationCommand::Initialize(InitializeCommand {})
                            .with_required_accounts(vec![]),
                    )
                }),
                current_timestamp,
            );
        }
        Ok(())
    }

    /// Sets next operation command and increment sequence number.
    pub(super) fn set_command(
        &mut self,
        command: Option<OperationCommandEntry>,
        current_timestamp: i64,
    ) {
        self.updated_at = current_timestamp;
        self.expired_at = current_timestamp + OPERATION_COMMANDS_EXPIRATION_SECONDS;
        self.sequence = match command {
            Some(_) => self.sequence + 1,
            None => 0,
        };
        self.command = command;
    }

    pub(super) fn get_command(&self) -> Option<(&OperationCommand, &[Pubkey])> {
        match self.command {
            Some(ref command) => Some(command.into()),
            None => None,
        }
    }

    pub(super) fn get_sequence(&self) -> u16 {
        self.sequence
    }
}
