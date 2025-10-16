use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::{token_2022, token_interface};

use crate::modules::fund::{DepositMetadata, FundAccount, FundService, UserFundAccount};
use crate::modules::reward::{RewardAccount, RewardService, UserRewardAccount};
use crate::utils::{AccountInfoExt, AsAccountInfo, PDASeeds};
use crate::{errors, events};

use super::FundWithdrawalBatchAccount;

pub struct UserFundWithdrawService<'a, 'info> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    receipt_token_program: &'a Program<'info, Token2022>,
    fund_account: &'a mut AccountLoader<'info, FundAccount>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,

    user: &'a Signer<'info>,
    user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
    user_fund_account: &'a mut Account<'info, UserFundAccount>,
    user_reward_account: &'a mut UncheckedAccount<'info>,

    _current_slot: u64,
    current_timestamp: i64,
}

impl Drop for UserFundWithdrawService<'_, '_> {
    fn drop(&mut self) {
        self.fund_account.exit(&crate::ID).unwrap();
        self.user_fund_account.exit(&crate::ID).unwrap();
    }
}

impl<'a, 'info> UserFundWithdrawService<'a, 'info> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        receipt_token_program: &'a Program<'info, Token2022>,
        fund_account: &'a mut AccountLoader<'info, FundAccount>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
        user: &'a Signer<'info>,
        user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
        user_fund_account: &'a mut Account<'info, UserFundAccount>,
        user_reward_account: &'a mut UncheckedAccount<'info>,
    ) -> Result<Self> {
        let clock = Clock::get()?;
        Ok(Self {
            receipt_token_mint,
            receipt_token_program,
            fund_account,
            reward_account,
            user,
            user_receipt_token_account,
            user_fund_account,
            user_reward_account,
            _current_slot: clock.slot,
            current_timestamp: clock.unix_timestamp,
        })
    }

    pub fn process_request_withdrawal(
        &mut self,
        receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
        supported_token_mint: Option<Pubkey>,
        pricing_sources: &'info [AccountInfo<'info>],
        receipt_token_amount: u64,
    ) -> Result<events::UserRequestedWithdrawalFromFund> {
        // validate user receipt token account balance
        require_gte!(self.user_receipt_token_account.amount, receipt_token_amount);
        require_gt!(receipt_token_amount, 0);

        // update fund value before processing request
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .new_pricing_service(pricing_sources, true)?;

        // create a user withdrawal request
        let withdrawal_request = self.fund_account.load_mut()?.create_withdrawal_request(
            supported_token_mint,
            receipt_token_amount,
            self.current_timestamp,
        )?;

        // requested receipt_token_amount can be reduced based on the status of the underlying asset.
        require_gte!(
            receipt_token_amount,
            withdrawal_request.receipt_token_amount
        );
        let receipt_token_amount = withdrawal_request.receipt_token_amount;
        let batch_id = withdrawal_request.batch_id;
        let request_id = withdrawal_request.request_id;

        self.user_fund_account
            .push_withdrawal_request(withdrawal_request)?;

        // lock requested user receipt token amount
        // first, burn user receipt token (use burn/mint instead of transfer to avoid circular CPI through transfer hook)
        token_2022::burn(
            CpiContext::new(
                self.receipt_token_program.to_account_info(),
                token_2022::Burn {
                    mint: self.receipt_token_mint.to_account_info(),
                    from: self.user_receipt_token_account.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            receipt_token_amount,
        )?;

        // then, mint receipt token to fund's lock account
        token_2022::mint_to(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                token_2022::MintTo {
                    mint: self.receipt_token_mint.to_account_info(),
                    to: receipt_token_lock_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[self.fund_account.load()?.get_seeds().as_ref()],
            ),
            receipt_token_amount,
        )?;

        self.fund_account
            .load_mut()?
            .reload_receipt_token_supply(self.receipt_token_mint)?;

        self.user_fund_account
            .reload_receipt_token_amount(self.user_receipt_token_account)?;

        let user_reward_account_option = self
            .user_reward_account
            .as_account_info()
            .parse_optional_account_loader::<UserRewardAccount>()?;

        // reduce user's reward accrual rate
        let updated_user_reward_accounts =
            RewardService::new(self.receipt_token_mint, self.reward_account)?
                .update_reward_pools_token_allocation(
                    user_reward_account_option.as_ref(),
                    None,
                    receipt_token_amount,
                    None,
                )?;

        // log withdrawal request event
        Ok(events::UserRequestedWithdrawalFromFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: self.fund_account.key(),
            supported_token_mint,
            updated_user_reward_accounts,

            user: self.user.key(),
            user_receipt_token_account: self.user_receipt_token_account.key(),
            user_fund_account: self.user_fund_account.key(),

            batch_id,
            request_id,
            requested_receipt_token_amount: receipt_token_amount,
        })
    }

    pub fn process_cancel_withdrawal_request(
        &mut self,
        receipt_token_lock_account: &mut InterfaceAccount<'info, TokenAccount>,
        pricing_sources: &'info [AccountInfo<'info>],
        request_id: u64,
        supported_token_mint: Option<Pubkey>,
    ) -> Result<events::UserCanceledWithdrawalRequestFromFund> {
        // clear pending amount from both user fund account and global fund account
        let withdrawal_request = self
            .user_fund_account
            .pop_withdrawal_request(request_id, supported_token_mint)?;
        let receipt_token_amount = withdrawal_request.receipt_token_amount;
        self.fund_account
            .load_mut()?
            .cancel_withdrawal_request(&withdrawal_request)?;

        // update fund value after processing request
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .new_pricing_service(pricing_sources, true)?;

        // unlock requested user receipt token amount
        // first, burn locked receipt token (use burn/mint instead of transfer to avoid circular CPI through transfer hook)
        token_2022::burn(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                token_2022::Burn {
                    mint: self.receipt_token_mint.to_account_info(),
                    from: receipt_token_lock_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[self.fund_account.load()?.get_seeds().as_ref()],
            ),
            receipt_token_amount,
        )?;

        // then, mint receipt token to user's receipt token account
        token_2022::mint_to(
            CpiContext::new_with_signer(
                self.receipt_token_program.to_account_info(),
                token_2022::MintTo {
                    mint: self.receipt_token_mint.to_account_info(),
                    to: self.user_receipt_token_account.to_account_info(),
                    authority: self.fund_account.to_account_info(),
                },
                &[self.fund_account.load()?.get_seeds().as_ref()],
            ),
            receipt_token_amount,
        )?;

        // receipt_token_lock_account.reload()?;

        self.fund_account
            .load_mut()?
            .reload_receipt_token_supply(self.receipt_token_mint)?;

        self.user_fund_account
            .reload_receipt_token_amount(self.user_receipt_token_account)?;

        // increase user's reward accrual rate
        let user_reward_account_option = self
            .user_reward_account
            .as_account_info()
            .parse_optional_account_loader::<UserRewardAccount>()?;
        let updated_user_reward_accounts =
            RewardService::new(self.receipt_token_mint, self.reward_account)?
                .update_reward_pools_token_allocation(
                    None,
                    user_reward_account_option.as_ref(),
                    receipt_token_amount,
                    None,
                )?;

        // log withdrawal request canceled event
        Ok(events::UserCanceledWithdrawalRequestFromFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: self.fund_account.key(),
            supported_token_mint: withdrawal_request.supported_token_mint,
            updated_user_reward_accounts,

            user: self.user.key(),
            user_receipt_token_account: self.user_receipt_token_account.key(),
            user_fund_account: self.user_fund_account.key(),

            batch_id: withdrawal_request.batch_id,
            request_id: withdrawal_request.request_id,
            requested_receipt_token_amount: receipt_token_amount,
        })
    }

    fn process_withdraw(
        &mut self,
        system_program: &Program<'info, System>,

        // for supported token
        supported_token_program: Option<&Interface<'info, TokenInterface>>,
        supported_token_mint: Option<&InterfaceAccount<'info, Mint>>,
        fund_supported_token_reserve_account: Option<&InterfaceAccount<'info, TokenAccount>>,
        user_supported_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,

        // for SOL
        fund_reserve_account: &SystemAccount<'info>,

        fund_treasury_account: &SystemAccount<'info>,
        fund_withdrawal_batch_account: &mut Account<'info, FundWithdrawalBatchAccount>,
        request_id: u64,
    ) -> Result<events::UserWithdrewFromFund> {
        // calculate asset amounts and mark withdrawal request as claimed withdrawal fee is already paid.
        let supported_token_mint_key = supported_token_mint.map(|mint| mint.key());
        let withdrawal_request = self
            .user_fund_account
            .pop_withdrawal_request(request_id, supported_token_mint_key)?;

        let (asset_user_amount, asset_fee_amount, receipt_token_amount) =
            fund_withdrawal_batch_account.settle_withdrawal_request(&withdrawal_request)?;
        self.fund_account
            .load_mut()?
            .get_asset_state_mut(supported_token_mint_key)?
            .withdrawal_user_reserved_amount -= asset_user_amount;
        let mut transferring_asset_user_amount = asset_user_amount;

        // transfer micro remainder to the last user to maintain exact accounting
        if fund_withdrawal_batch_account.is_settled() {
            let remaining_asset_amount =
                fund_withdrawal_batch_account.get_remaining_asset_amount_after_settled();
            if remaining_asset_amount > 0 {
                let mut fund_account = self.fund_account.load_mut()?;
                let asset_state = fund_account.get_asset_state_mut(supported_token_mint_key)?;
                asset_state.withdrawal_user_reserved_amount -= remaining_asset_amount;
                transferring_asset_user_amount += remaining_asset_amount;
            }
        }

        // transfer either SOL or token to user account
        {
            let fund_account = self.fund_account.load()?;
            match supported_token_mint {
                Some(supported_token_mint) => {
                    token_interface::transfer_checked(
                        CpiContext::new_with_signer(
                            supported_token_program.unwrap().to_account_info(),
                            token_interface::TransferChecked {
                                from: fund_supported_token_reserve_account
                                    .unwrap()
                                    .to_account_info(),
                                to: user_supported_token_account.unwrap().to_account_info(),
                                mint: supported_token_mint.to_account_info(),
                                authority: fund_reserve_account.to_account_info(),
                            },
                            &[&self.fund_account.load()?.get_reserve_account_seeds()],
                        ),
                        transferring_asset_user_amount,
                        supported_token_mint.decimals,
                    )?;
                }
                None => {
                    anchor_lang::system_program::transfer(
                        CpiContext::new_with_signer(
                            system_program.to_account_info(),
                            anchor_lang::system_program::Transfer {
                                from: fund_reserve_account.to_account_info(),
                                to: self.user.to_account_info(),
                            },
                            &[&fund_account.get_reserve_account_seeds()],
                        ),
                        transferring_asset_user_amount,
                    )?;
                }
            };
        }

        // close the ticket to collect rent
        if fund_withdrawal_batch_account.is_settled() {
            fund_withdrawal_batch_account.close(fund_treasury_account.to_account_info())?;
        }

        let fund_account = self.fund_account.load()?;
        Ok(events::UserWithdrewFromFund {
            receipt_token_mint: fund_account.receipt_token_mint,
            fund_account: self.fund_account.key(),
            supported_token_mint: supported_token_mint_key,

            user: self.user.key(),
            user_receipt_token_account: self.user_receipt_token_account.key(),
            user_fund_account: self.user_fund_account.key(),
            user_supported_token_account: user_supported_token_account
                .map(|token_account| token_account.key()),

            fund_withdrawal_batch_account: fund_withdrawal_batch_account.key(),
            batch_id: withdrawal_request.batch_id,
            request_id: withdrawal_request.request_id,
            burnt_receipt_token_amount: receipt_token_amount,
            returned_receipt_token_amount: 0,
            withdrawn_amount: asset_user_amount,
            deducted_fee_amount: asset_fee_amount,
        })
    }

    pub fn process_withdraw_sol(
        &mut self,
        system_program: &Program<'info, System>,
        fund_withdrawal_batch_account: &mut Account<'info, FundWithdrawalBatchAccount>,
        fund_reserve_account: &SystemAccount<'info>,
        fund_treasury_account: &SystemAccount<'info>,
        request_id: u64,
    ) -> Result<events::UserWithdrewFromFund> {
        self.process_withdraw(
            system_program,
            None,
            None,
            None,
            None,
            fund_reserve_account,
            fund_treasury_account,
            fund_withdrawal_batch_account,
            request_id,
        )
    }

    pub fn process_withdraw_supported_token(
        &mut self,
        system_program: &Program<'info, System>,
        supported_token_program: &Interface<'info, TokenInterface>,
        supported_token_mint: &InterfaceAccount<'info, Mint>,
        fund_supported_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,
        user_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        fund_withdrawal_batch_account: &mut Account<'info, FundWithdrawalBatchAccount>,
        fund_reserve_account: &SystemAccount<'info>,
        fund_treasury_account: &SystemAccount<'info>,
        request_id: u64,
    ) -> Result<events::UserWithdrewFromFund> {
        self.process_withdraw(
            system_program,
            Some(supported_token_program),
            Some(supported_token_mint),
            Some(fund_supported_token_reserve_account),
            Some(user_supported_token_account),
            fund_reserve_account,
            fund_treasury_account,
            fund_withdrawal_batch_account,
            request_id,
        )
    }
}
