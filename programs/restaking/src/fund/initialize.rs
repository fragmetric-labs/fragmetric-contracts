use anchor_lang::prelude::*;

use crate::structs::Fund;

impl Fund {
    pub fn initialize(
        &mut self,
        admin: Pubkey,
        default_protocol_fee_rate: u16,
        whitelisted_tokens: Vec<Pubkey>,
        lst_caps: Vec<u64>,
        receipt_token_mint: Pubkey,
        // receipt_token_lock_account: Pubkey,
        lsts_amount_in: Vec<u128>,
    ) -> Result<()> {
        self.admin = admin;
        self.update_default_protocol_fee_rate(default_protocol_fee_rate)?;
        self.update_whitelisted_tokens(whitelisted_tokens)?;
        self.update_lst_caps(lst_caps)?;
        self.receipt_token_mint = receipt_token_mint;
        // self.receipt_token_lock_account = receipt_token_lock_account;
        self.sol_amount_in = 0;
        self.update_lst_amount_in(lsts_amount_in)?;

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

    pub fn update_whitelisted_tokens(&mut self, whitelisted_tokens: Vec<Pubkey>) -> Result<()> {
        // check if there's no duplicated token address
        self.whitelisted_tokens = whitelisted_tokens;

        Ok(())
    }

    pub fn update_lst_caps(&mut self, lst_caps: Vec<u64>) -> Result<()> {
        // check if lst_caps length is same as whitelisted_tokens length
        self.lst_caps = lst_caps;

        Ok(())
    }

    pub fn update_lst_amount_in(&mut self, lsts_amount_in: Vec<u128>) -> Result<()> {
        // check length
        self.lsts_amount_in = lsts_amount_in;

        Ok(())
    }
}

#[test]
fn test_initialize() {
    let admin = Pubkey::new_unique();
    let token_mint_authority = Pubkey::new_unique();
    let default_protocol_fee_rate = 100;
    let whitelisted_tokens = vec![Pubkey::new_unique(), Pubkey::new_unique()];
    let lst_caps = vec![1_000_000_000 * 1000, 1_000_000_000 * 2000];
    let receipt_token_mint = Pubkey::new_unique();
    let lsts_amount_in = vec![1_000_000_000, 2_000_000_000];

    let mut fund = Fund {
        admin: Pubkey::default(),
        default_protocol_fee_rate: 0,
        whitelisted_tokens: vec![],
        lst_caps: vec![],
        receipt_token_mint: Pubkey::default(),
        // receipt_token_lock_account: Pubkey::default(),
        sol_amount_in: 0,
        lsts_amount_in: vec![],
    };

    let result = fund.initialize(
        admin,
        default_protocol_fee_rate,
        whitelisted_tokens.clone(),
        lst_caps.clone(),
        receipt_token_mint,
        lsts_amount_in,
    );

    assert!(result.is_ok());
    assert_eq!(fund.admin, admin);
    assert_eq!(fund.default_protocol_fee_rate, default_protocol_fee_rate);
    assert_eq!(fund.whitelisted_tokens, whitelisted_tokens);
    assert_eq!(fund.lst_caps, lst_caps);
    assert_eq!(fund.receipt_token_mint, receipt_token_mint);
}
