use anchor_lang::{
    prelude::*,
    solana_program::{ed25519_program, instruction::Instruction, sysvar::instructions},
};

use crate::error::ErrorCode;

pub(crate) struct Ed25519Instruction(Instruction);

impl Ed25519Instruction {
    pub(crate) fn new_from_instruction_sysvar(instruction_sysvar: &AccountInfo) -> Result<Self> {
        let current_ix_index: usize =
            instructions::load_current_index_checked(instruction_sysvar)?.into();
        let ix =
            instructions::load_instruction_at_checked(current_ix_index - 1, instruction_sysvar)?;
        require_eq!(ix.program_id, ed25519_program::ID);
        require_eq!(ix.accounts.len(), 0);

        Ok(Self(ix))
    }

    /// Verify serialized Ed25519Program instruction data
    pub(crate) fn verify(&self, pubkey: &[u8], payload: &[u8]) -> Result<()> {
        // According to this layout used by the Ed25519Program
        // https://github.com/solana-labs/solana-web3.js/blob/master/src/ed25519-program.ts#L33
        // "Deserializing" byte slices
        let expected_payload_size =
            u16::try_from(payload.len()).map_err(|_| error!(ErrorCode::SigVerificationFailed))?;
        self.check_data_len(payload)?;
        self.check_data_header(expected_payload_size)?;
        self.check_data_pubkey(pubkey)?;
        self.check_data_payload(payload)?;

        Ok(())
    }

    fn check_data_len(&self, payload: &[u8]) -> Result<()> {
        let actual = self.0.data.len();
        let expected = 16 + 64 + 32 + payload.len();
        if actual != expected {
            msg!(
                "Invalid data length: actual {}, expected {}",
                actual,
                expected
            );
            err!(ErrorCode::SigVerificationFailed)?;
        }

        Ok(())
    }

    fn check_data_header(&self, expected_payload_size: u16) -> Result<()> {
        let header = &self.0.data[0..16];
        let num_signatures = header[0];
        let padding = header[1];
        let signature_offset = u16::from_le_bytes([header[2], header[3]]);
        let signature_instruction_index = u16::from_le_bytes([header[4], header[5]]);
        let public_key_offset = u16::from_le_bytes([header[6], header[7]]);
        let public_key_instruction_index = u16::from_le_bytes([header[8], header[9]]);
        let payload_offset = u16::from_le_bytes([header[10], header[11]]);
        let payload_size = u16::from_le_bytes([header[12], header[13]]);
        let payload_instruction_index = u16::from_le_bytes([header[14], header[15]]);

        require_eq!(num_signatures, 1, ErrorCode::SigVerificationFailed);
        require_eq!(padding, 0, ErrorCode::SigVerificationFailed);
        require_eq!(signature_offset, 48, ErrorCode::SigVerificationFailed);
        require_eq!(
            signature_instruction_index,
            u16::MAX,
            ErrorCode::SigVerificationFailed
        );
        require_eq!(public_key_offset, 16, ErrorCode::SigVerificationFailed);
        require_eq!(
            public_key_instruction_index,
            u16::MAX,
            ErrorCode::SigVerificationFailed
        );
        require_eq!(payload_offset, 112, ErrorCode::SigVerificationFailed);
        require_eq!(
            payload_size,
            expected_payload_size,
            ErrorCode::SigVerificationFailed
        );
        require_eq!(
            payload_instruction_index,
            u16::MAX,
            ErrorCode::SigVerificationFailed
        );

        Ok(())
    }

    fn check_data_pubkey(&self, pubkey: &[u8]) -> Result<()> {
        let data_pubkey = &self.0.data[16..48];

        if data_pubkey != pubkey {
            msg!("Data pubkey mismatch");
            err!(ErrorCode::SigVerificationFailed)?;
        }

        Ok(())
    }

    fn check_data_payload(&self, payload: &[u8]) -> Result<()> {
        let data_payload = &self.0.data[112..];

        if data_payload != payload {
            msg!("Data payload mismatch");
            err!(ErrorCode::SigVerificationFailed)?;
        }

        Ok(())
    }
}
