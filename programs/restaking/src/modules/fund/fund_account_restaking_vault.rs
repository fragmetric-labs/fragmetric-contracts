use anchor_lang::prelude::*;
use bytemuck::Zeroable;

use crate::errors::ErrorCode;
use crate::modules::pricing::{TokenPricingSource, TokenPricingSourcePod};

pub const FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS: usize = 30;
pub const FUND_ACCOUNT_MAX_RESTAKING_VAULT_COMPOUNDING_REWARD_TOKENS: usize = 10;
pub const FUND_ACCOUNT_MAX_RESTAKING_VAULT_DISTRIBUTING_REWARD_TOKENS: usize = 30;

#[zero_copy]
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
    num_delegations: u8,
    delegations: [RestakingVaultDelegation; FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS],

    pub reward_commission_rate_bps: u16,

    /// auto-compounding
    _padding3: [u8; 5],
    num_compounding_reward_tokens: u8,
    compounding_reward_token_mints:
        [Pubkey; FUND_ACCOUNT_MAX_RESTAKING_VAULT_COMPOUNDING_REWARD_TOKENS],

    /// reward to distribute
    _padding4: [u8; 7],
    num_distributing_reward_tokens: u8,
    // distributing_reward_tokens: [
    //     {
    //         mint: Pubkey,
    //         threshold_min_amount: u64,
    //         threshold_max_amount: u64,
    //         threshold_interval_seconds: u64,
    //         last_settled_at: u64
    //     }; FUND_ACCOUNT_MAX_RESTAKING_VAULT_DISTRIBUTING_REWARD_TOKENS
    // ],
    distributing_reward_token_mints:
        [Pubkey; FUND_ACCOUNT_MAX_RESTAKING_VAULT_DISTRIBUTING_REWARD_TOKENS],

    // _reserved: [u8; 1336],
    _reserved: [u8; 2296],
}

impl RestakingVault {
    #[deny(clippy::wildcard_enum_match_arm)]
    pub fn initialize(
        &mut self,
        vault: Pubkey,
        program: Pubkey,

        supported_token_mint: Pubkey,

        receipt_token_mint: Pubkey,
        receipt_token_program: Pubkey,
        receipt_token_decimals: u8,
        receipt_token_pricing_source: TokenPricingSource,

        receipt_token_operation_reserved_amount: u64,
    ) -> Result<()> {
        match receipt_token_pricing_source {
            TokenPricingSource::JitoRestakingVault { .. }
            | TokenPricingSource::SolvBTCVault { .. } => {}
            // otherwise fails
            TokenPricingSource::SPLStakePool { .. }
            | TokenPricingSource::MarinadeStakePool { .. }
            | TokenPricingSource::FragmetricNormalizedTokenPool { .. }
            | TokenPricingSource::FragmetricRestakingFund { .. }
            | TokenPricingSource::OrcaDEXLiquidityPool { .. }
            | TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. }
            | TokenPricingSource::PeggedToken { .. } => {
                err!(ErrorCode::FundRestakingNotSupportedVaultError)?
            }
            #[cfg(all(test, not(feature = "idl-build")))]
            TokenPricingSource::Mock { .. } => {
                err!(ErrorCode::FundRestakingNotSupportedVaultError)?
            }
        }

        *self = Zeroable::zeroed();

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

        self.num_delegations = 0;

        Ok(())
    }

    pub fn set_sol_allocation_strategy(
        &mut self,
        weight: u64,
        sol_capacity_amount: u64,
    ) -> Result<()> {
        self.sol_allocation_weight = weight;
        self.sol_allocation_capacity_amount = sol_capacity_amount;

        Ok(())
    }

    pub fn add_compounding_reward_token(
        &mut self,
        compounding_reward_token_mint: Pubkey,
    ) -> Result<()> {
        if self
            .get_compounding_reward_tokens_iter()
            .any(|reward_token| *reward_token == compounding_reward_token_mint)
        {
            err!(ErrorCode::FundRestakingVaultCompoundingRewardTokenAlreadyRegisteredError)?
        }

        if self
            .get_distributing_reward_tokens_iter()
            .any(|reward_token| *reward_token == compounding_reward_token_mint)
        {
            err!(ErrorCode::FundRestakingVaultDistributingRewardTokenAlreadyRegisteredError)?
        }

        require_gt!(
            FUND_ACCOUNT_MAX_RESTAKING_VAULT_COMPOUNDING_REWARD_TOKENS,
            self.num_compounding_reward_tokens as usize,
            ErrorCode::FundExceededMaxRestakingVaultCompoundingRewardTokensError
        );

        self.compounding_reward_token_mints[self.num_compounding_reward_tokens as usize] =
            compounding_reward_token_mint;
        self.num_compounding_reward_tokens += 1;

        Ok(())
    }

    pub fn remove_compounding_reward_token(
        &mut self,
        compounding_reward_token_mint: Pubkey,
    ) -> Result<()> {
        let matched_idx = self
            .get_compounding_reward_tokens_iter()
            .position(|reward_token| *reward_token == compounding_reward_token_mint)
            .ok_or(ErrorCode::FundRestakingVaultCompoundingRewardTokenNotRegisteredError)?;

        self.num_compounding_reward_tokens -= 1;
        self.compounding_reward_token_mints
            .swap(matched_idx, self.num_compounding_reward_tokens as usize);
        self.compounding_reward_token_mints[self.num_compounding_reward_tokens as usize] =
            Pubkey::default();

        Ok(())
    }

    pub fn get_compounding_reward_tokens_iter(&self) -> impl Iterator<Item = &Pubkey> {
        self.compounding_reward_token_mints[..self.num_compounding_reward_tokens as usize].iter()
    }

    pub fn add_distributing_reward_token(
        &mut self,
        distributing_reward_token_mint: Pubkey,
    ) -> Result<()> {
        if self
            .get_distributing_reward_tokens_iter()
            .any(|reward_token| *reward_token == distributing_reward_token_mint)
        {
            err!(ErrorCode::FundRestakingVaultDistributingRewardTokenAlreadyRegisteredError)?
        }

        if self
            .get_compounding_reward_tokens_iter()
            .any(|reward_token| *reward_token == distributing_reward_token_mint)
        {
            err!(ErrorCode::FundRestakingVaultCompoundingRewardTokenAlreadyRegisteredError)?
        }

        require_gt!(
            FUND_ACCOUNT_MAX_RESTAKING_VAULT_DISTRIBUTING_REWARD_TOKENS,
            self.num_distributing_reward_tokens as usize,
            ErrorCode::FundExceededMaxRestakingVaultDistributingRewardTokensError,
        );

        self.distributing_reward_token_mints[self.num_distributing_reward_tokens as usize] =
            distributing_reward_token_mint;
        self.num_distributing_reward_tokens += 1;

        Ok(())
    }

    pub fn remove_distributing_reward_token(
        &mut self,
        distributing_reward_token_mint: Pubkey,
    ) -> Result<()> {
        let matched_idx = self
            .get_distributing_reward_tokens_iter()
            .position(|reward_token| *reward_token == distributing_reward_token_mint)
            .ok_or(ErrorCode::FundRestakingVaultDistributingRewardTokenNotRegisteredError)?;

        self.num_distributing_reward_tokens -= 1;
        self.distributing_reward_token_mints
            .swap(matched_idx, self.num_distributing_reward_tokens as usize);
        self.distributing_reward_token_mints[self.num_distributing_reward_tokens as usize] =
            Pubkey::default();

        Ok(())
    }

    pub fn get_distributing_reward_tokens_iter(&self) -> impl Iterator<Item = &Pubkey> {
        self.distributing_reward_token_mints[..self.num_distributing_reward_tokens as usize].iter()
    }

    pub fn add_delegation(
        &mut self,
        operator: Pubkey,
        index: Option<u8>,
        delegated_amount: u64,
        undelegating_amount: u64,
    ) -> Result<()> {
        if let Some(index) = index {
            require_eq!(self.num_delegations, index);
        }

        if self
            .get_delegations_iter()
            .any(|delegation| delegation.operator == operator)
        {
            err!(ErrorCode::FundRestakingVaultOperatorAlreadyRegisteredError)?
        }

        require_gt!(
            FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS,
            self.num_delegations as usize,
            ErrorCode::FundExceededMaxRestakingVaultDelegationsError
        );

        self.delegations[self.num_delegations as usize].initialize(
            operator,
            delegated_amount,
            undelegating_amount,
        );
        self.num_delegations += 1;

        Ok(())
    }

    pub fn get_delegations_iter(&self) -> impl Iterator<Item = &RestakingVaultDelegation> {
        self.delegations[..self.num_delegations as usize].iter()
    }

    pub fn get_delegations_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut RestakingVaultDelegation> {
        self.delegations[..self.num_delegations as usize].iter_mut()
    }

    pub fn get_delegation_mut(
        &mut self,
        operator: &Pubkey,
    ) -> Result<&mut RestakingVaultDelegation> {
        self.get_delegations_iter_mut()
            .find(|op| op.operator == *operator)
            .ok_or_else(|| error!(ErrorCode::FundRestakingVaultOperatorNotFoundError))
    }

    pub fn get_delegation_by_index(&self, index: usize) -> Result<&RestakingVaultDelegation> {
        self.delegations[..self.num_delegations as usize]
            .get(index)
            .ok_or_else(|| error!(ErrorCode::FundRestakingVaultOperatorNotFoundError))
    }
}

#[zero_copy]
#[repr(C)]
pub(super) struct RestakingVaultDelegation {
    pub operator: Pubkey,

    /// configuration: used for delegation strategy.
    pub supported_token_allocation_weight: u64,
    pub supported_token_allocation_capacity_amount: u64,

    /// informative field; these values shall be synced from remote state periodically.
    pub supported_token_delegated_amount: u64,
    pub supported_token_undelegating_amount: u64,

    /// configuration: the amount requested to be undelegated as soon as possible regardless of current state, this value should be decreased by each undelegation requested amount.
    pub supported_token_redelegating_amount: u64,

    _reserved: [u8; 24],
}

impl RestakingVaultDelegation {
    fn initialize(&mut self, operator: Pubkey, delegated_amount: u64, undelegating_amount: u64) {
        *self = Zeroable::zeroed();

        self.operator = operator;
        self.supported_token_delegated_amount = delegated_amount;
        self.supported_token_undelegating_amount = undelegating_amount;
    }

    pub fn set_supported_token_allocation_strategy(
        &mut self,
        weight: u64,
        supported_token_capacity_amount: u64,
    ) -> Result<()> {
        self.supported_token_allocation_weight = weight;
        self.supported_token_allocation_capacity_amount = supported_token_capacity_amount;

        Ok(())
    }

    pub fn set_supported_token_redelegating_amount(&mut self, token_amount: u64) -> Result<()> {
        require_gte!(
            token_amount,
            self.supported_token_delegated_amount,
            ErrorCode::FundInvalidConfigurationUpdateError
        );

        self.supported_token_redelegating_amount = token_amount;

        Ok(())
    }
}
