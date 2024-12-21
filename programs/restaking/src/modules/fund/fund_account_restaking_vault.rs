use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

use crate::constants::JITO_VAULT_PROGRAM_ID;
use crate::errors::ErrorCode;
use crate::modules::fund::SupportedToken;
use crate::modules::pricing::{TokenPricingSource, TokenPricingSourcePod};

const FUND_ACCOUNT_MAX_RESTAKING_VAULT_OPERATORS: usize = 30;
const FUND_ACCOUNT_RESTAKING_VAULT_MAX_COMPOUNDING_TOKENS: usize = 10;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
pub(super) struct RestakingVault {
    pub vault: Pubkey,
    pub program: Pubkey,

    pub supported_token_mint: Pubkey,
    pub receipt_token_mint: Pubkey,
    pub receipt_token_program: Pubkey,
    pub receipt_token_decimals: u8,
    _padding: [u8; 7],

    /// transient price
    pub one_receipt_token_as_sol: u64,
    pub receipt_token_pricing_source: TokenPricingSourcePod,
    pub receipt_token_operation_reserved_amount: u64,
    /// the amount of vrt being unrestaked
    pub receipt_token_operation_receivable_amount: u64,

    /// configuration: used for restaking allocation strategy.
    pub sol_allocation_weight: u64,
    pub sol_allocation_capacity_amount: u64,

    _padding2: [u8; 7],
    num_operators: u8,
    operators: [RestakingVaultOperator; FUND_ACCOUNT_MAX_RESTAKING_VAULT_OPERATORS],

    /// auto-compounding
    compounding_token_mints: [Pubkey; FUND_ACCOUNT_RESTAKING_VAULT_MAX_COMPOUNDING_TOKENS],

    _reserved: [u8; 128],
}

impl RestakingVault {
    pub(super) fn initialize(
        &mut self,
        vault: Pubkey,
        program: Pubkey,

        supported_token_mint: Pubkey,

        receipt_token_mint: Pubkey,
        receipt_token_program: Pubkey,
        receipt_token_decimals: u8,

        receipt_token_operation_reserved_amount: u64,
    ) -> Result<()> {
        let receipt_token_pricing_source = match program {
            JITO_VAULT_PROGRAM_ID => Ok(TokenPricingSource::JitoRestakingVault { address: vault }),
            _ => {
                err!(ErrorCode::FundRestakingNotSupportedVaultError)
            }
        }?;

        self.vault = vault;
        self.program = program;
        self.supported_token_mint = supported_token_mint;

        self.receipt_token_mint = receipt_token_mint;
        self.receipt_token_program = receipt_token_program;
        self.receipt_token_decimals = receipt_token_decimals;

        self.one_receipt_token_as_sol = 0;
        receipt_token_pricing_source.serialize_as_pod(&mut self.receipt_token_pricing_source);
        self.receipt_token_operation_reserved_amount = receipt_token_operation_reserved_amount;
        self.receipt_token_operation_receivable_amount = 0;

        self.sol_allocation_weight = 0;
        self.sol_allocation_capacity_amount = 0;

        self.num_operators = 0;

        Ok(())
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

    pub(super) fn add_operator(&mut self, operator_key: &Pubkey) -> Result<()> {
        if self
            .operators
            .iter()
            .take(self.num_operators as usize)
            .any(|op| op.operator == *operator_key)
        {
            err!(ErrorCode::FundRestakingVaultOperatorAlreadyRegisteredError)?
        }

        require_gt!(
            FUND_ACCOUNT_MAX_RESTAKING_VAULT_OPERATORS,
            self.num_operators as usize,
            ErrorCode::FundExceededMaxRestakingVaultOperatorsError
        );

        let mut operator = RestakingVaultOperator::zeroed();
        operator.initialize(*operator_key);
        self.operators[self.num_operators as usize] = operator;
        self.num_operators += 1;

        Ok(())
    }

    pub(super) fn get_operator(&self, operator: &Pubkey) -> Result<&RestakingVaultOperator> {
        self.operators
            .iter()
            .take(self.num_operators as usize)
            .find(|op| op.operator == *operator)
            .ok_or_else(|| error!(ErrorCode::FundRestakingVaultOperatorNotFoundError))
    }

    pub(super) fn get_operator_mut(
        &mut self,
        operator: &Pubkey,
    ) -> Result<&mut RestakingVaultOperator> {
        self.operators
            .iter_mut()
            .take(self.num_operators as usize)
            .find(|op| op.operator == *operator)
            .ok_or_else(|| error!(ErrorCode::FundRestakingVaultOperatorNotFoundError))
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Zeroable, Pod, Debug)]
#[repr(C)]
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
    pub(super) fn initialize(&mut self, operator: Pubkey) {
        self.operator = operator;
        self.supported_token_allocation_capacity_amount = 0;
        self.supported_token_redelegation_amount = 0;
        self.supported_token_allocation_weight = 0;
        self.supported_token_delegated_amount = 0;
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
            ErrorCode::FundInvalidConfigurationUpdateError
        );

        self.supported_token_redelegation_amount = token_amount;

        Ok(())
    }
}
