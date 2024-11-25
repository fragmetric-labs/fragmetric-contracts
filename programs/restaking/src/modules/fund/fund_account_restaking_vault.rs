use anchor_lang::prelude::*;

use crate::constants::JITO_VAULT_PROGRAM_ID;
use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;

const MAX_RESTAKING_VAULT_OPERATORS: usize = 30;

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct RestakingVault {
    pub vault: Pubkey,
    pub program: Pubkey,

    pub supported_token_mint: Pubkey,
    pub receipt_token_mint: Pubkey,
    pub receipt_token_program: Pubkey,
    pub receipt_token_decimals: u8,
    /// transient price
    pub one_receipt_token_as_sol: u64,
    pub receipt_token_pricing_source: TokenPricingSource,
    pub receipt_token_operation_reserved_amount: u64,
    /// the amount of vrt being unrestaked
    pub receipt_token_operating_amount: u64,

    /// configuration: used for restaking allocation strategy.
    pub sol_allocation_weight: u64,
    pub sol_allocation_capacity_amount: u64,

    #[max_len(MAX_RESTAKING_VAULT_OPERATORS)]
    pub operators: Vec<RestakingVaultOperator>,

    _reserved: [u8; 128],
}

impl RestakingVault {
    pub(super) fn new(
        vault: Pubkey,
        program: Pubkey,

        supported_token_mint: Pubkey,

        receipt_token_mint: Pubkey,
        receipt_token_program: Pubkey,
        receipt_token_decimals: u8,

        receipt_token_operation_reserved_amount: u64,
    ) -> Result<Self> {
        let receipt_token_pricing_source = match program {
            JITO_VAULT_PROGRAM_ID => Ok(TokenPricingSource::JitoRestakingVault { address: vault }),
            _ => {
                err!(ErrorCode::FundNotSupportedRestakingVaultError)
            }
        }?;

        Ok(Self {
            vault,
            program,

            supported_token_mint,

            receipt_token_mint,
            receipt_token_program,
            receipt_token_decimals,
            one_receipt_token_as_sol: 0,
            receipt_token_pricing_source,
            receipt_token_operation_reserved_amount,
            receipt_token_operating_amount: 0,

            sol_allocation_weight: 0,
            sol_allocation_capacity_amount: 0,

            operators: Vec::new(),

            _reserved: [0; 128],
        })
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub(super) struct RestakingVaultOperator {
    pub operator: Pubkey,

    /// configuration: used for delegation strategy.
    pub supported_token_allocation_weight: u64,
    pub supported_token_allocation_capacity_amount: u64,

    /// just informative field
    pub supported_token_delegated_amount: u64,

    /// configuration: the amount requested to be undelegated as soon as possible regardless of current state, this value should be decreased by each undelegation requested amount.
    pub supported_token_redelegation_amount: u64,

    _reserved: [u8; 32],
}
