use anchor_lang::prelude::*;

use crate::fund::*;
use crate::error::ErrorCode;

impl Fund {
    pub fn update_token(
        &mut self,
        token: Pubkey,
        info: TokenInfo,
    ) -> Result<()> {
        for mapped_token in self.tokens.iter_mut() {
            if mapped_token.address == token {
                *mapped_token = info;
                return Ok(())
            }
        }
        err!(ErrorCode::NotExistingToken)
    }
}

#[test]
fn test_update_token() {
    let admin = Pubkey::new_unique();
    let default_protocol_fee_rate = 10;
    let receipt_token_mint = Pubkey::new_unique();

    let mut fund = Fund {
        admin: Pubkey::default(),
        default_protocol_fee_rate: 0,
        receipt_token_mint: Pubkey::default(),
        tokens: vec![],
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
    let mut token1_update = token1.clone();
    token1_update.token_cap = 1_000_000_000 * 3000;
    let tokens = vec![token1, token2];

    fund.initialize(admin, default_protocol_fee_rate, receipt_token_mint, tokens).unwrap();
    msg!("{:?}", fund.tokens.iter());

    fund.update_token(token1_update.address, token1_update).unwrap();
    msg!("{:?}", fund.tokens.iter());
}
