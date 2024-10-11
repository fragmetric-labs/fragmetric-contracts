use anchor_lang::{
    prelude::*,
    solana_program::{ed25519_program, sysvar::instructions},
};

use crate::{constants::ADMIN_PUBKEY, errors::ErrorCode};

/// Verify serialized Ed25519Program instruction data with ADMIN_PUBKEY
pub fn verify_preceding_ed25519_instruction(
    instructions_sysvar: &UncheckedAccount,
    payload: &[u8],
) -> Result<()> {
    // load prev instruction
    let current_ix_index: usize =
        instructions::load_current_index_checked(instructions_sysvar)?.into();
    let ix = instructions::load_instruction_at_checked(current_ix_index - 1, instructions_sysvar)?;
    require_eq!(ix.program_id, ed25519_program::ID);
    require_eq!(ix.accounts.len(), 0);

    // According to this layout used by the Ed25519Program
    // https://github.com/solana-labs/solana-web3.js/blob/master/src/ed25519-program.ts#L33
    // "Deserializing" byte slices
    let expected_payload_size =
        u16::try_from(payload.len()).map_err(|_| error!(ErrorCode::InvalidSignatureError))?;

    // check_data_len
    let actual = ix.data.len();
    let expected = 16 + 64 + 32 + payload.len();
    if actual != expected {
        // msg!(
        //     "Invalid data length: actual {}, expected {}",
        //     actual,
        //     expected
        // );
        err!(ErrorCode::InvalidSignatureError)?
    }

    // check_data_header
    let header = &ix.data[0..16];
    let num_signatures = header[0];
    let padding = header[1];
    let signature_offset = u16::from_le_bytes([header[2], header[3]]);
    let signature_instruction_index = u16::from_le_bytes([header[4], header[5]]);
    let public_key_offset = u16::from_le_bytes([header[6], header[7]]);
    let public_key_instruction_index = u16::from_le_bytes([header[8], header[9]]);
    let payload_offset = u16::from_le_bytes([header[10], header[11]]);
    let payload_size = u16::from_le_bytes([header[12], header[13]]);
    let payload_instruction_index = u16::from_le_bytes([header[14], header[15]]);

    require_eq!(num_signatures, 1, ErrorCode::InvalidSignatureError);
    require_eq!(padding, 0, ErrorCode::InvalidSignatureError);
    require_eq!(signature_offset, 48, ErrorCode::InvalidSignatureError);
    require_eq!(
        signature_instruction_index,
        u16::MAX,
        ErrorCode::InvalidSignatureError
    );
    require_eq!(public_key_offset, 16, ErrorCode::InvalidSignatureError);
    require_eq!(
        public_key_instruction_index,
        u16::MAX,
        ErrorCode::InvalidSignatureError
    );
    require_eq!(payload_offset, 112, ErrorCode::InvalidSignatureError);
    require_eq!(
        payload_size,
        expected_payload_size,
        ErrorCode::InvalidSignatureError
    );
    require_eq!(
        payload_instruction_index,
        u16::MAX,
        ErrorCode::InvalidSignatureError
    );

    // check_data_pubkey
    let data_pubkey = &ix.data[16..48];
    if data_pubkey != ADMIN_PUBKEY.to_bytes() {
        // msg!("Data pubkey mismatch");
        err!(ErrorCode::InvalidSignatureError)?
    }

    let data_payload = &ix.data[112..];
    if data_payload != payload {
        // msg!("Data payload mismatch");
        err!(ErrorCode::InvalidSignatureError)?
    }

    Ok(())
}

pub fn verify_expiration(expiration: i64) -> Result<()> {
    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    if current_timestamp > expiration {
        err!(ErrorCode::ExpiredSignatureError)?
    }

    Ok(())
}
