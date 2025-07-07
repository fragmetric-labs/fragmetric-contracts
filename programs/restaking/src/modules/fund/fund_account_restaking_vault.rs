use anchor_lang::prelude::*;
use bytemuck::Zeroable;

use crate::errors::ErrorCode;
use crate::modules::pricing::{TokenPricingSource, TokenPricingSourcePod};

pub const FUND_ACCOUNT_MAX_RESTAKING_VAULT_DELEGATIONS: usize = 30;
pub const FUND_ACCOUNT_MAX_RESTAKING_VAULT_COMPOUNDING_REWARD_TOKENS: usize = 4;
pub const FUND_ACCOUNT_MAX_RESTAKING_VAULT_DISTRIBUTING_REWARD_TOKENS: usize = 30;

#[zero_copy]
#[repr(C, packed(8))]
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
    compounding_reward_tokens:
        [RewardToken; FUND_ACCOUNT_MAX_RESTAKING_VAULT_COMPOUNDING_REWARD_TOKENS],

    /// reward to distribute
    _padding4: [u8; 7],
    num_distributing_reward_tokens: u8,
    distributing_reward_tokens:
        [RewardToken; FUND_ACCOUNT_MAX_RESTAKING_VAULT_DISTRIBUTING_REWARD_TOKENS],

    pub supported_token_compounded_amount: i128,
    pub supported_token_to_receipt_token_exchange_ratio: TokenExchangeRatio,
    pub supported_token_to_receipt_token_exchange_ratio_updated_timestamp: i64,

    _padding5: [u8; 32],
    /// Expected amount of vst by unrestaking vrt.
    /// This field is updated when the vault uses vst as expected receivable amount after unrestaking process is completed.
    /// It does NOT include unrestaking amount as vrt.
    pub pending_supported_token_unrestaking_amount: u64,

    _reserved: [u8; 776],
}

#[zero_copy]
#[repr(C)]
pub(super) struct TokenExchangeRatio {
    pub numerator: u64,
    pub denominator: u64,
}

impl RestakingVault {
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
            | TokenPricingSource::SolvBTCVault { .. }
            | TokenPricingSource::VirtualVault { .. } => {}
            // otherwise fails
            TokenPricingSource::SPLStakePool { .. }
            | TokenPricingSource::MarinadeStakePool { .. }
            | TokenPricingSource::FragmetricNormalizedTokenPool { .. }
            | TokenPricingSource::FragmetricRestakingFund { .. }
            | TokenPricingSource::OrcaDEXLiquidityPool { .. }
            | TokenPricingSource::SanctumSingleValidatorSPLStakePool { .. }
            | TokenPricingSource::SanctumMultiValidatorSPLStakePool { .. }
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
    ) -> Result<&mut Self> {
        self.sol_allocation_weight = weight;
        self.sol_allocation_capacity_amount = sol_capacity_amount;

        Ok(self)
    }

    pub fn set_reward_commission_rate_bps(
        &mut self,
        reward_commission_rate_bps: u16,
    ) -> Result<&mut Self> {
        // hard limit on reward commission rate to be less than or equal to 10%
        require_gte!(1000, reward_commission_rate_bps);

        self.reward_commission_rate_bps = reward_commission_rate_bps;

        Ok(self)
    }

    pub fn get_reward_commission_amount(&self, reward_token_amount: u64) -> Result<u64> {
        crate::utils::get_proportional_amount(
            reward_token_amount,
            self.reward_commission_rate_bps as u64,
            10_000,
        )
    }

    pub fn add_compounding_reward_token(
        &mut self,
        compounding_reward_token_mint: Pubkey,
    ) -> Result<()> {
        if self
            .get_compounding_reward_tokens_iter()
            .any(|reward_token| reward_token.mint == compounding_reward_token_mint)
        {
            err!(ErrorCode::FundRestakingVaultCompoundingRewardTokenAlreadyRegisteredError)?
        }

        if self
            .get_distributing_reward_tokens_iter()
            .any(|reward_token| reward_token.mint == compounding_reward_token_mint)
        {
            err!(ErrorCode::FundRestakingVaultDistributingRewardTokenAlreadyRegisteredError)?
        }

        require_gt!(
            FUND_ACCOUNT_MAX_RESTAKING_VAULT_COMPOUNDING_REWARD_TOKENS,
            self.num_compounding_reward_tokens as usize,
            ErrorCode::FundExceededMaxRestakingVaultCompoundingRewardTokensError
        );

        self.compounding_reward_tokens[self.num_compounding_reward_tokens as usize]
            .initialize(compounding_reward_token_mint);
        self.num_compounding_reward_tokens += 1;

        Ok(())
    }

    pub fn remove_compounding_reward_token(
        &mut self,
        compounding_reward_token_mint: &Pubkey,
    ) -> Result<()> {
        let matched_idx = self
            .get_compounding_reward_tokens_iter()
            .position(|reward_token| reward_token.mint == *compounding_reward_token_mint)
            .ok_or(ErrorCode::FundRestakingVaultCompoundingRewardTokenNotRegisteredError)?;

        self.num_compounding_reward_tokens -= 1;
        self.compounding_reward_tokens
            .swap(matched_idx, self.num_compounding_reward_tokens as usize);
        self.compounding_reward_tokens[self.num_compounding_reward_tokens as usize] =
            Zeroable::zeroed();

        Ok(())
    }

    pub fn get_compounding_reward_tokens_iter(&self) -> impl Iterator<Item = &RewardToken> {
        self.compounding_reward_tokens[..self.num_compounding_reward_tokens as usize].iter()
    }

    pub fn get_compounding_reward_tokens_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut RewardToken> {
        self.compounding_reward_tokens[..self.num_compounding_reward_tokens as usize].iter_mut()
    }

    pub fn get_compounding_reward_token(&self, mint: &Pubkey) -> Result<&RewardToken> {
        self.get_compounding_reward_tokens_iter()
            .find(|reward_token| reward_token.mint == *mint)
            .ok_or_else(|| {
                error!(ErrorCode::FundRestakingVaultCompoundingRewardTokenNotRegisteredError)
            })
    }

    pub fn get_compounding_reward_token_mut(&mut self, mint: &Pubkey) -> Result<&mut RewardToken> {
        self.get_compounding_reward_tokens_iter_mut()
            .find(|reward_token| reward_token.mint == *mint)
            .ok_or_else(|| {
                error!(ErrorCode::FundRestakingVaultCompoundingRewardTokenNotRegisteredError)
            })
    }

    pub fn add_distributing_reward_token(
        &mut self,
        distributing_reward_token_mint: Pubkey,
    ) -> Result<()> {
        if self
            .get_distributing_reward_tokens_iter()
            .any(|reward_token| reward_token.mint == distributing_reward_token_mint)
        {
            err!(ErrorCode::FundRestakingVaultDistributingRewardTokenAlreadyRegisteredError)?
        }

        if self
            .get_compounding_reward_tokens_iter()
            .any(|reward_token| reward_token.mint == distributing_reward_token_mint)
        {
            err!(ErrorCode::FundRestakingVaultCompoundingRewardTokenAlreadyRegisteredError)?
        }

        require_gt!(
            FUND_ACCOUNT_MAX_RESTAKING_VAULT_DISTRIBUTING_REWARD_TOKENS,
            self.num_distributing_reward_tokens as usize,
            ErrorCode::FundExceededMaxRestakingVaultDistributingRewardTokensError,
        );

        self.distributing_reward_tokens[self.num_distributing_reward_tokens as usize]
            .initialize(distributing_reward_token_mint);
        self.num_distributing_reward_tokens += 1;

        Ok(())
    }

    pub fn remove_distributing_reward_token(
        &mut self,
        distributing_reward_token_mint: &Pubkey,
    ) -> Result<()> {
        let matched_idx = self
            .get_distributing_reward_tokens_iter()
            .position(|reward_token| reward_token.mint == *distributing_reward_token_mint)
            .ok_or(ErrorCode::FundRestakingVaultDistributingRewardTokenNotRegisteredError)?;

        self.num_distributing_reward_tokens -= 1;
        self.distributing_reward_tokens
            .swap(matched_idx, self.num_distributing_reward_tokens as usize);
        self.distributing_reward_tokens[self.num_distributing_reward_tokens as usize] =
            Zeroable::zeroed();

        Ok(())
    }

    pub fn get_distributing_reward_tokens_iter(&self) -> impl Iterator<Item = &RewardToken> {
        self.distributing_reward_tokens[..self.num_distributing_reward_tokens as usize].iter()
    }

    pub fn get_distributing_reward_tokens_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut RewardToken> {
        self.distributing_reward_tokens[..self.num_distributing_reward_tokens as usize].iter_mut()
    }

    pub fn get_distributing_reward_token(&self, mint: &Pubkey) -> Result<&RewardToken> {
        self.get_distributing_reward_tokens_iter()
            .find(|reward_token| reward_token.mint == *mint)
            .ok_or_else(|| {
                error!(ErrorCode::FundRestakingVaultDistributingRewardTokenNotRegisteredError)
            })
    }

    pub fn get_distributing_reward_token_mut(&mut self, mint: &Pubkey) -> Result<&mut RewardToken> {
        self.get_distributing_reward_tokens_iter_mut()
            .find(|reward_token| reward_token.mint == *mint)
            .ok_or_else(|| {
                error!(ErrorCode::FundRestakingVaultDistributingRewardTokenNotRegisteredError)
            })
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

    pub fn update_supported_token_compounded_amount(
        &mut self,
        supported_token_amount_numerator: u64,
        receipt_token_amount_denominator: u64,
    ) -> Result<()> {
        let receipt_token_amount = self.receipt_token_operation_reserved_amount
            + self.receipt_token_operation_receivable_amount;

        // calculate supported token amount based on previous vault receipt token price
        let supported_token_amount_before = crate::utils::get_proportional_amount(
            receipt_token_amount,
            self.supported_token_to_receipt_token_exchange_ratio
                .numerator,
            self.supported_token_to_receipt_token_exchange_ratio
                .denominator,
        )?;

        // calculate supported token amount based on current vault receipt token price
        let supported_token_amount = crate::utils::get_proportional_amount(
            receipt_token_amount,
            supported_token_amount_numerator,
            receipt_token_amount_denominator,
        )?;

        self.supported_token_compounded_amount +=
            supported_token_amount as i128 - supported_token_amount_before as i128;

        Ok(())
    }

    pub fn update_supported_token_receipt_token_exchange_ratio(
        &mut self,
        supported_token_amount_numerator: u64,
        receipt_token_amount_denominator: u64,
    ) -> Result<()> {
        self.supported_token_to_receipt_token_exchange_ratio = TokenExchangeRatio {
            numerator: supported_token_amount_numerator,
            denominator: receipt_token_amount_denominator,
        };
        self.supported_token_to_receipt_token_exchange_ratio_updated_timestamp =
            Clock::get()?.unix_timestamp;

        Ok(())
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

    _reserved: [u8; 32],
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
}

#[zero_copy]
#[repr(C)]
pub(super) struct RewardToken {
    pub mint: Pubkey,
    pub harvest_threshold_min_amount: u64,
    pub harvest_threshold_max_amount: u64,
    pub harvest_threshold_interval_seconds: i64,
    pub last_harvested_at: i64,
    _reserved: [u8; 16],
}

impl RewardToken {
    fn initialize(&mut self, mint: Pubkey) {
        *self = Zeroable::zeroed();

        self.mint = mint;
        self.harvest_threshold_max_amount = u64::MAX;
    }

    pub fn update_harvest_threshold(
        &mut self,
        harvest_threshold_min_amount: u64,
        harvest_threshold_max_amount: u64,
        harvest_threshold_interval_seconds: i64,
    ) -> Result<()> {
        require_gte!(harvest_threshold_max_amount, harvest_threshold_min_amount);
        require_gte!(harvest_threshold_interval_seconds, 0);

        self.harvest_threshold_min_amount = harvest_threshold_min_amount;
        self.harvest_threshold_max_amount = harvest_threshold_max_amount;
        self.harvest_threshold_interval_seconds = harvest_threshold_interval_seconds;

        Ok(())
    }

    pub fn get_available_amount_to_harvest(&self, amount: u64, current_timestamp: i64) -> u64 {
        let available = current_timestamp
            >= self.last_harvested_at + self.harvest_threshold_interval_seconds
            && amount >= self.harvest_threshold_min_amount;

        available
            .then(|| amount.min(self.harvest_threshold_max_amount))
            .unwrap_or_default()
    }
}
