use std::mem::discriminant;

use anchor_lang::prelude::*;

use super::command::*;

const OPERATION_COMMANDS_EXPIRATION_SECONDS: i64 = 600;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Default)]
pub struct OperationState {
    updated_at: i64,
    expired_at: i64,
    pub(super) next_sequence: u16,
    next_command: Option<OperationCommandEntry>,
    /// when the no_transition flag turned on, current command should not be transitioned to other command.
    /// the purpose of this flag is for internal testing by set boundary of the reset command operation.
    no_transition: bool,
    _reserved: [[u8; 8]; 32],
}

impl OperationState {
    pub(super) fn initialize(&mut self, fund_account_data_version: u16) {
        if fund_account_data_version == 3 {
            self.updated_at = 0;
            self.expired_at = 0;
            self.next_sequence = 0;
            self.next_command = None;
            self.no_transition = false;
            self._reserved = Default::default();
        }
    }

    /// Initialize current operation command to `reset_command` or default.
    pub(super) fn initialize_command_if_needed(
        &mut self,
        current_timestamp: i64,
        reset_command: Option<OperationCommandEntry>,
    ) -> Result<()> {
        let has_reset_command = reset_command.is_some();

        if has_reset_command || self.next_command.is_none() || current_timestamp > self.expired_at {
            self.no_transition = false;
            self.next_sequence = 0;
            self.set_command(
                reset_command.or_else(|| Some(InitializeCommand {}.with_required_accounts([]))),
                current_timestamp,
            );
            self.no_transition = has_reset_command;
        }

        Ok(())
    }

    /// Sets next operation command and increment sequence number.
    pub(super) fn set_command(
        &mut self,
        mut command: Option<OperationCommandEntry>,
        current_timestamp: i64,
    ) {
        // deal with no_transition state, to adjust next command.
        if self.no_transition {
            if let (Some(prev_entry), Some(next_entry)) = (&self.next_command, &command) {
                if discriminant(&prev_entry.command) != discriminant(&next_entry.command) {
                    // when the type of the command changes on no_transition state, ignore the next command and clear no_transition state.
                    msg!(
                        "COMMAND#{} reset due to no_transition state",
                        self.next_sequence
                    );
                    self.no_transition = false;
                    command = None;
                }
                // otherwise, retaining on the same command type, still maintains no_transition state.
            } else {
                // if there is no previous command (unexpected flow) or next command, clear no_transition state.
                self.no_transition = false;
            }
        }

        self.updated_at = current_timestamp;
        self.expired_at = current_timestamp + OPERATION_COMMANDS_EXPIRATION_SECONDS;
        self.next_sequence = match command {
            Some(_) => self.next_sequence + 1,
            None => 0,
        };
        self.next_command = command;
    }

    #[inline(always)]
    pub(super) fn get_command(
        &self,
    ) -> Option<(&OperationCommand, &[OperationCommandAccountMeta])> {
        self.next_command.as_ref().map(Into::into)
    }
}
