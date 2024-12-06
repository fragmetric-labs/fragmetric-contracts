use anchor_lang::prelude::*;

use crate::constants::JITO_VAULT_PROGRAM_ID;
use crate::errors::ErrorCode;
use crate::modules::fund::SupportedToken;
use crate::modules::pricing::{TokenPricingSource, TokenPricingSourcePod};
use crate::utils::OptionPod;

const MAX_RESTAKING_VAULT_OPERATORS: usize = 30;

#[derive(Default)]
#[zero_copy]
pub(super) struct RestakingVault {
    pub vault: Pubkey,
    pub program: Pubkey,

    pub supported_token_mint: Pubkey,
    pub receipt_token_mint: Pubkey,
    pub receipt_token_program: Pubkey,
    pub receipt_token_decimals: u8,
    /// transient price
    pub one_receipt_token_as_sol: u64,
    pub receipt_token_pricing_source: TokenPricingSourcePod,
    pub receipt_token_operation_reserved_amount: u64,
    /// the amount of vrt being unrestaked
    pub receipt_token_operation_receivable_amount: u64,

    /// configuration: used for restaking allocation strategy.
    pub sol_allocation_weight: u64,
    pub sol_allocation_capacity_amount: u64,

    pub operators: [RestakingVaultOperator; MAX_RESTAKING_VAULT_OPERATORS],

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
                err!(ErrorCode::FundRestakingNotSupportedVaultError)
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
            receipt_token_pricing_source: receipt_token_pricing_source.into(),
            receipt_token_operation_reserved_amount,
            receipt_token_operation_receivable_amount: 0,

            sol_allocation_weight: 0,
            sol_allocation_capacity_amount: 0,

            operators: [RestakingVaultOperator::default(); MAX_RESTAKING_VAULT_OPERATORS],

            _reserved: [0; 128],
        })
    }

    pub(super) fn set_sol_allocation_strategy(
        &mut self,
        weight: u64,
        sol_capacity_amount: u64,
    ) -> Result<()> {
        self.sol_allocation_weight = weight;
        self.sol_allocation_capacity_amount = sol_capacity_amount;

        Ok(())
    }

    pub(super) fn add_operator(&mut self, operator: &Pubkey) -> Result<()> {
        if self.operators.iter().any(|op| op.operator == *operator) {
            err!(ErrorCode::FundRestakingVaultOperatorAlreadyRegisteredError)?
        }

        require_gt!(
            MAX_RESTAKING_VAULT_OPERATORS,
            self.operators.len(),
            ErrorCode::FundExceededMaxRestakingVaultOperatorsError
        );

        self.operators.push(RestakingVaultOperator::new(*operator));

        Ok(())
    }

    pub(super) fn get_operator(&self, operator: &Pubkey) -> Result<&RestakingVaultOperator> {
        self.operators
            .iter()
            .find(|op| op.operator == *operator)
            .ok_or_else(|| error!(ErrorCode::FundRestakingVaultOperatorNotFoundError))
    }

    pub(super) fn get_operator_mut(
        &mut self,
        operator: &Pubkey,
    ) -> Result<&mut RestakingVaultOperator> {
        self.operators
            .iter_mut()
            .find(|op| op.operator == *operator)
            .ok_or_else(|| error!(ErrorCode::FundRestakingVaultOperatorNotFoundError))
    }
}

#[derive(Default)]
#[zero_copy]
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

impl RestakingVaultOperator {
    pub(super) fn new(operator: Pubkey) -> Self {
        Self {
            operator,
            supported_token_allocation_capacity_amount: 0,
            supported_token_redelegation_amount: 0,
            supported_token_allocation_weight: 0,
            supported_token_delegated_amount: 0,
            _reserved: [0; 32],
        }
    }

    pub(super) fn set_supported_token_allocation_strategy(
        &mut self,
        weight: u64,
        supported_token_capacity_amount: u64,
    ) -> Result<()> {
        self.supported_token_allocation_weight = weight;
        self.supported_token_allocation_capacity_amount = supported_token_capacity_amount;

        Ok(())
    }

    pub(super) fn set_supported_token_redelegation_amount(
        &mut self,
        token_amount: u64,
    ) -> Result<()> {
        require_gte!(
            token_amount,
            self.supported_token_delegated_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.supported_token_redelegation_amount = token_amount;

        Ok(())
    }
}
