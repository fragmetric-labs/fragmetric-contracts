use super::*;
use crate::errors::ErrorCode;
use crate::modules::pricing::TokenPricingSource;
use crate::utils::PDASeeds;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

#[constant]
/// ## Version History
/// * v1: Initial Version
/// * v2: Change reserve fund structure
/// * v3: Remove `sol_fee_income_reserved_amount` field
/// * v4: Add `one_receipt_token_as_sol`, `receipt_token_decimals` fields
/// * v5: Add `operation` field
pub const FUND_ACCOUNT_CURRENT_VERSION: u16 = 5;

const MAX_SUPPORTED_TOKENS: usize = 16;

#[account]
#[derive(InitSpace)]
pub struct FundAccount {
    data_version: u16,
    bump: u8,
    pub receipt_token_mint: Pubkey,
    #[max_len(MAX_SUPPORTED_TOKENS)]
    pub(super) supported_tokens: Vec<SupportedTokenInfo>,
    pub(super) sol_capacity_amount: u64,
    pub(super) sol_accumulated_deposit_amount: u64,
    // TODO v0.3/operation: visibility
    pub(in crate::modules) sol_operation_reserved_amount: u64,
    // TODO v0.3/operation: visibility
    pub(in crate::modules) withdrawal: WithdrawalState,
    pub(super) one_receipt_token_as_sol: u64,
    receipt_token_decimals: u8,
    pub(super) operation: OperationState,
    _reserved: [u8; 256], // TODO: is it okay to just reduce to arbitrary number ... in wip?, was _reserved: [u8; 1271],
}

impl PDASeeds<2> for FundAccount {
    const SEED: &'static [u8] = b"fund";

    fn get_seeds(&self) -> [&[u8]; 2] {
        [Self::SEED, self.receipt_token_mint.as_ref()]
    }

    fn get_bump_ref(&self) -> &u8 {
        &self.bump
    }
}

impl FundAccount {
    pub const RESERVE_SEED: &'static [u8] = b"fund_reserve";
    pub const TREASURY_SEED: &'static [u8] = b"fund_treasury";

    pub(super) fn initialize(
        &mut self,
        bump: u8,
        receipt_token_mint: Pubkey,
        receipt_token_decimals: u8,
    ) {
        if self.data_version == 0 {
            self.bump = bump;
            self.receipt_token_mint = receipt_token_mint;
            self.withdrawal.initialize(self.data_version);
            self.data_version = 1;
        }

        if self.data_version == 1 {
            self.withdrawal.initialize(self.data_version);
            self.data_version = 2;
        }

        if self.data_version == 2 {
            self.withdrawal.initialize(self.data_version);
            self.data_version = 3;
        }

        if self.data_version == 3 {
            self.one_receipt_token_as_sol = 0;
            self.receipt_token_decimals = receipt_token_decimals;
            self.data_version = 4;
        }

        if self.data_version == 4 {
            self.operation.initialize(self.data_version);
            self.data_version = 5;
        }
    }

    #[inline(always)]
    pub(super) fn update_if_needed(&mut self, receipt_token_mint: &InterfaceAccount<Mint>) {
        self.initialize(
            self.bump,
            receipt_token_mint.key(),
            receipt_token_mint.decimals,
        );
    }

    #[inline(always)]
    pub fn is_latest_version(&self) -> bool {
        self.data_version == FUND_ACCOUNT_CURRENT_VERSION
    }

    pub(super) fn get_receipt_token_decimals(&self) -> u8 {
        self.receipt_token_decimals
    }

    pub(super) fn get_supported_token(&self, token: Pubkey) -> Result<&SupportedTokenInfo> {
        self.supported_tokens
            .iter()
            .find(|info| info.mint == token)
            .ok_or_else(|| error!(ErrorCode::FundNotSupportedTokenError))
    }

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn get_supported_token_mut(
        &mut self,
        token: Pubkey,
    ) -> Result<&mut SupportedTokenInfo> {
        self.supported_tokens
            .iter_mut()
            .find(|info| info.mint == token)
            .ok_or_else(|| error!(ErrorCode::FundNotSupportedTokenError))
    }

    pub(super) fn set_sol_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        require_gte!(
            capacity_amount,
            self.sol_accumulated_deposit_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.sol_capacity_amount = capacity_amount;

        Ok(())
    }

    pub(super) fn add_supported_token(
        &mut self,
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
    ) -> Result<()> {
        if self.supported_tokens.iter().any(|info| info.mint == mint) {
            err!(ErrorCode::FundAlreadySupportedTokenError)?
        }

        require_gt!(
            MAX_SUPPORTED_TOKENS,
            self.supported_tokens.len(),
            ErrorCode::FundExceededMaxSupportedTokensError
        );

        let token_info =
            SupportedTokenInfo::new(mint, program, decimals, capacity_amount, pricing_source);
        self.supported_tokens.push(token_info);

        Ok(())
    }

    pub(super) fn deposit_sol(&mut self, amount: u64) -> Result<()> {
        let new_sol_accumulated_deposit_amount = self
            .sol_accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        require_gte!(
            self.sol_capacity_amount,
            new_sol_accumulated_deposit_amount,
            ErrorCode::FundExceededSOLCapacityAmountError
        );

        self.sol_accumulated_deposit_amount = new_sol_accumulated_deposit_amount;
        self.sol_operation_reserved_amount = self
            .sol_operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}

#[derive(Clone, InitSpace, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct SupportedTokenInfo {
    mint: Pubkey,
    program: Pubkey,
    decimals: u8,
    capacity_amount: u64,
    accumulated_deposit_amount: u64,
    operation_reserved_amount: u64,
    pub(super) one_token_as_sol: u64,
    pricing_source: TokenPricingSource,
    operating_amount: u64,
    _reserved: [u8; 120],
}

impl SupportedTokenInfo {
    fn new(
        mint: Pubkey,
        program: Pubkey,
        decimals: u8,
        capacity_amount: u64,
        pricing_source: TokenPricingSource,
    ) -> Self {
        Self {
            mint,
            program,
            decimals,
            capacity_amount,
            accumulated_deposit_amount: 0,
            operation_reserved_amount: 0,
            one_token_as_sol: 0,
            pricing_source,
            operating_amount: 0,
            _reserved: [0; 120],
        }
    }

    pub(in crate::modules) fn get_mint(&self) -> Pubkey {
        self.mint
    }

    pub(in crate::modules) fn get_operation_reserved_amount(&self) -> u64 {
        self.operation_reserved_amount
    }

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn set_operation_reserved_amount(&mut self, amount: u64) {
        self.operation_reserved_amount = amount;
    }

    pub(in crate::modules) fn get_operating_amount(&self) -> u64 {
        self.operating_amount
    }

    // TODO v0.3/operation: visibility
    pub(in crate::modules) fn set_operating_amount(&mut self, amount: u64) {
        self.operating_amount = amount;
    }

    pub(super) fn get_decimals(&self) -> u8 {
        self.decimals
    }

    #[inline(always)]
    pub(super) fn get_pricing_source(&self) -> TokenPricingSource {
        self.pricing_source.clone()
    }

    pub(super) fn set_capacity_amount(&mut self, capacity_amount: u64) -> Result<()> {
        require_gte!(
            capacity_amount,
            self.accumulated_deposit_amount,
            ErrorCode::FundInvalidUpdateError
        );

        self.capacity_amount = capacity_amount;

        Ok(())
    }

    pub(super) fn deposit_token(&mut self, amount: u64) -> Result<()> {
        let new_accumulated_deposit_amount = self
            .accumulated_deposit_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        require_gte!(
            self.capacity_amount,
            new_accumulated_deposit_amount,
            ErrorCode::FundExceededTokenCapacityAmountError
        );

        self.accumulated_deposit_amount = new_accumulated_deposit_amount;
        self.operation_reserved_amount = self
            .operation_reserved_amount
            .checked_add(amount)
            .ok_or_else(|| error!(ErrorCode::CalculationArithmeticException))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::pricing::TokenPricingSource;

    fn create_uninitialized_fund_account() -> FundAccount {
        let buffer = [0u8; 8 + FundAccount::INIT_SPACE];
        FundAccount::try_deserialize_unchecked(&mut &buffer[..]).unwrap()
    }

    #[test]
    fn test_initialize_update_fund_account() {
        let mut fund = create_uninitialized_fund_account();
        fund.initialize(0, Pubkey::new_unique(), 9);

        assert_eq!(fund.sol_capacity_amount, 0);
        assert_eq!(fund.withdrawal.get_sol_withdrawal_fee_rate_as_f32(), 0.);
        assert!(fund.withdrawal.get_withdrawal_enabled_flag());
        assert_eq!(fund.withdrawal.get_batch_processing_threshold_amount(), 0);
        assert_eq!(fund.withdrawal.get_batch_processing_threshold_duration(), 0);

        fund.sol_accumulated_deposit_amount = 1_000_000_000_000;
        fund.set_sol_capacity_amount(0).unwrap_err();

        let new_amount = 10;
        let new_duration = 10;
        fund.withdrawal
            .set_batch_processing_threshold(Some(new_amount), None);
        assert_eq!(
            fund.withdrawal.get_batch_processing_threshold_amount(),
            new_amount
        );
        assert_eq!(fund.withdrawal.get_batch_processing_threshold_duration(), 0);

        fund.withdrawal
            .set_batch_processing_threshold(None, Some(new_duration));
        assert_eq!(
            fund.withdrawal.get_batch_processing_threshold_amount(),
            new_amount
        );
        assert_eq!(
            fund.withdrawal.get_batch_processing_threshold_duration(),
            new_duration
        );
    }

    #[test]
    fn test_update_token() {
        let mut fund = create_uninitialized_fund_account();
        fund.initialize(0, Pubkey::new_unique(), 9);

        let token1 = Pubkey::new_unique();
        let token2 = Pubkey::new_unique();

        fund.add_supported_token(
            token1,
            Pubkey::default(),
            9,
            1_000_000_000,
            TokenPricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap();
        fund.add_supported_token(
            token2,
            Pubkey::default(),
            9,
            1_000_000_000,
            TokenPricingSource::MarinadeStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap();
        fund.add_supported_token(
            token1,
            Pubkey::default(),
            9,
            1_000_000_000,
            TokenPricingSource::MarinadeStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap_err();
        assert_eq!(fund.supported_tokens.len(), 2);
        assert_eq!(fund.supported_tokens[0].capacity_amount, 1_000_000_000);

        fund.supported_tokens[0].accumulated_deposit_amount = 1_000_000_000;
        fund.get_supported_token_mut(token1)
            .unwrap()
            .set_capacity_amount(0)
            .unwrap_err();
    }

    #[test]
    fn test_deposit_sol() {
        let mut fund = create_uninitialized_fund_account();
        fund.initialize(0, Pubkey::new_unique(), 9);
        fund.set_sol_capacity_amount(100_000).unwrap();

        assert_eq!(fund.sol_operation_reserved_amount, 0);
        assert_eq!(fund.sol_accumulated_deposit_amount, 0);

        fund.deposit_sol(100_000).unwrap();
        assert_eq!(fund.sol_operation_reserved_amount, 100_000);
        assert_eq!(fund.sol_accumulated_deposit_amount, 100_000);

        fund.deposit_sol(100_000).unwrap_err();
    }

    #[test]
    fn test_deposit_token() {
        let mut fund = create_uninitialized_fund_account();
        fund.initialize(0, Pubkey::new_unique(), 9);

        fund.add_supported_token(
            Pubkey::new_unique(),
            Pubkey::default(),
            9,
            1_000,
            TokenPricingSource::SPLStakePool {
                address: Pubkey::new_unique(),
            },
        )
        .unwrap();

        assert_eq!(fund.supported_tokens[0].operation_reserved_amount, 0);
        assert_eq!(fund.supported_tokens[0].accumulated_deposit_amount, 0);

        fund.supported_tokens[0].deposit_token(1_000).unwrap();
        assert_eq!(fund.supported_tokens[0].operation_reserved_amount, 1_000);
        assert_eq!(fund.supported_tokens[0].accumulated_deposit_amount, 1_000);

        fund.supported_tokens[0].deposit_token(1_000).unwrap_err();
    }
}
