use anchor_lang::prelude::*;

use crate::fund::*;
use crate::error::ErrorCode;

impl Fund {
    pub fn initialize(
        &mut self,
        admin: Pubkey,
        default_protocol_fee_rate: u16,
        receipt_token_mint: Pubkey,
        tokens: Vec<TokenInfo>,
    ) -> Result<()> {
        self.admin = admin;
        self.update_default_protocol_fee_rate(default_protocol_fee_rate)?;
        self.receipt_token_mint = receipt_token_mint;
        self.set_tokens(tokens)?;
        // self.receipt_token_lock_account = receipt_token_lock_account;
        self.sol_amount_in = 0;

        Ok(())
    }

    pub fn update_default_protocol_fee_rate(
        &mut self,
        default_protocol_fee_rate: u16,
    ) -> Result<()> {
        // max protocol fee rate (상수) 넘어서지 못하게 하는 제약조건 필요?
        self.default_protocol_fee_rate = default_protocol_fee_rate;

        Ok(())
    }

    pub fn set_tokens(&mut self, tokens: Vec<TokenInfo>) -> Result<()> {
        // check if there's no duplicated token address
        self.tokens = tokens;

        Ok(())
    }

    pub fn add_whitelisted_token(
        &mut self,
        token: Pubkey,
        token_cap: u64,
    ) -> Result<()> {
        self.check_if_token_exists(&token)?;

        self.tokens.push(
            TokenInfo {
                address: token,
                token_cap: token_cap,
                token_amount_in: 0
            }
        );

        Ok(())
    }

    fn check_if_token_exists(&self, token: &Pubkey) -> Result<()> {
        for check in self.tokens.iter().map(|info| &info.address) {
            if check == token {
                return Err(ErrorCode::AlreadyExistingToken)?;
            }
        }
        Ok(())
    }
}

#[test]
fn test_initialize() {
    let admin = Pubkey::new_unique();
    let default_protocol_fee_rate = 100;
    let receipt_token_mint = Pubkey::new_unique();

    let mut fund = Fund {
        admin: Pubkey::default(),
        default_protocol_fee_rate: 0,
        receipt_token_mint: Pubkey::default(),
        tokens: vec![],
        // receipt_token_lock_account: Pubkey::default(),
        sol_amount_in: 0,
    };

    let token1 = TokenInfo {
        address: Pubkey::new_unique(),
        token_cap: 1_000_000_000 * 1000,
        token_amount_in: 1_000_000_000,
    };
    let token2 = TokenInfo {
        address: Pubkey::new_unique(),
        token_cap: 1_000_000_000 * 2000,
        token_amount_in: 2_000_000_000,
    };

    let tokens = vec![token1, token2];

    let result = fund.initialize(
        admin,
        default_protocol_fee_rate,
        receipt_token_mint,
        tokens,
    );

    assert!(result.is_ok());
    assert_eq!(fund.admin, admin);
    assert_eq!(fund.default_protocol_fee_rate, default_protocol_fee_rate);
    assert_eq!(fund.receipt_token_mint, receipt_token_mint);
    msg!("fund tokens: {:?}", fund.tokens);
}

#[test]
fn test_add_whitelisted_token() {
    let mut fund = Fund {
        admin: Pubkey::default(),
        default_protocol_fee_rate: 0,
        receipt_token_mint: Pubkey::default(),
        tokens: vec![],
        sol_amount_in: 0
    };

    let token1 = TokenInfo {
        address: Pubkey::new_unique(),
        token_cap: 1_000_000_000 * 1000,
        token_amount_in: 1_000_000_000,
    };
    let token2 = TokenInfo {
        address: Pubkey::new_unique(),
        token_cap: 1_000_000_000 * 2000,
        token_amount_in: 2_000_000_000,
    };
    let token3 = token1.clone();
    let tokens = vec![token1, token2];

    fund.set_tokens(tokens).unwrap();

    fund.add_whitelisted_token(token3.address, token3.token_cap).unwrap();
}

#[test]
fn test_sort() {
    let whitelisted_tokens = vec![Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique()];
    msg!("whitelisted_tokens: {:?}", whitelisted_tokens);

    let mut sorted_tokens: Vec<_> = whitelisted_tokens.into_iter().collect();
    sorted_tokens.sort();
    msg!("sorted_tokens: {:?}", sorted_tokens);
}
