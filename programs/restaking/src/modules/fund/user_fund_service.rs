use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::{token_2022, token_interface};
use std::cell::RefMut;

use crate::errors::ErrorCode;
use crate::events;
use crate::modules::fund::{DepositMetadata, FundAccount, FundService, UserFundAccount};
use crate::modules::reward::{RewardAccount, RewardService, UserRewardAccount};
use crate::utils::PDASeeds;

use super::FundWithdrawalBatchAccount;

pub struct UserFundService<'info: 'a, 'a> {
    receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
    receipt_token_program: &'a Program<'info, Token2022>,
    fund_account: &'a mut AccountLoader<'info, FundAccount>,
    reward_account: &'a mut AccountLoader<'info, RewardAccount>,

    user: &'a Signer<'info>,
    user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
    user_fund_account: &'a mut Account<'info, UserFundAccount>,
    user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,

    _current_slot: u64,
    current_timestamp: i64,
}

impl Drop for UserFundService<'_, '_> {
    fn drop(&mut self) {
        self.fund_account.exit(&crate::ID).unwrap();
        self.user_fund_account.exit(&crate::ID).unwrap();
    }
}

impl<'info, 'a> UserFundService<'info, 'a> {
    pub fn new(
        receipt_token_mint: &'a mut InterfaceAccount<'info, Mint>,
        receipt_token_program: &'a Program<'info, Token2022>,
        fund_account: &'a mut AccountLoader<'info, FundAccount>,
        reward_account: &'a mut AccountLoader<'info, RewardAccount>,
        user: &'a Signer<'info>,
        user_receipt_token_account: &'a mut InterfaceAccount<'info, TokenAccount>,
        user_fund_account: &'a mut Account<'info, UserFundAccount>,
        user_reward_account: &'a mut AccountLoader<'info, UserRewardAccount>,
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

    fn process_deposit(
        &mut self,
        // for SOL
        system_program: Option<&Program<'info, System>>,
        fund_reserve_account: Option<&SystemAccount<'info>>,

        // for supported tokens
        supported_token_program: Option<&Interface<'info, TokenInterface>>,
        supported_token_mint: Option<&InterfaceAccount<'info, Mint>>,
        fund_supported_token_reserve_account: Option<&InterfaceAccount<'info, TokenAccount>>,
        user_supported_token_account: Option<&InterfaceAccount<'info, TokenAccount>>,

        instructions_sysvar: &AccountInfo,
        pricing_sources: &'info [AccountInfo<'info>],

        asset_amount: u64,
        metadata: Option<DepositMetadata>,
        metadata_signer_key: &Pubkey,
    ) -> Result<events::UserDepositedToFund> {
        let supported_token_mint_key = supported_token_mint.map(|mint| mint.key());

        // validate user asset balance
        match supported_token_mint_key {
            Some(..) => {
                require_gte!(user_supported_token_account.unwrap().amount, asset_amount);
            }
            None => {
                require_gte!(self.user.lamports(), asset_amount);
            }
        }

        // validate deposit metadata
        let (wallet_provider, contribution_accrual_rate) = metadata
            .map(|metadata| {
                metadata.verify(
                    instructions_sysvar,
                    metadata_signer_key,
                    self.user.key,
                    self.current_timestamp,
                )
            })
            .transpose()?
            .unzip();

        // mint receipt token
        let mut pricing_service = FundService::new(self.receipt_token_mint, self.fund_account)?
            .new_pricing_service(pricing_sources)?;

        let receipt_token_mint_amount = if self.receipt_token_mint.supply == 0 {
            // receipt_token_mint_amount will be equal to asset_amount at the initial minting, so like either 1SOL = 1RECEIPT-TOKEN or 1SUPPORTED-TOKEN = 1RECEIPT-TOKEN.
            asset_amount
        } else {
            pricing_service.get_asset_amount_as_token(
                supported_token_mint_key.as_ref(),
                asset_amount,
                &self.receipt_token_mint.key(),
            )?
        };

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
            receipt_token_mint_amount,
        )?;

        self.fund_account
            .load_mut()?
            .reload_receipt_token_supply(self.receipt_token_mint)?;

        self.user_fund_account
            .reload_receipt_token_amount(self.user_receipt_token_account)?;

        // increase user's reward accrual rate
        let event = RewardService::new(self.receipt_token_mint, self.reward_account)?
            .update_reward_pools_token_allocation(
                None,
                Some(self.user_reward_account),
                receipt_token_mint_amount,
                contribution_accrual_rate,
            )?;

        // transfer user asset to the fund
        let deposited_amount = self
            .fund_account
            .load_mut()?
            .deposit(supported_token_mint_key, asset_amount)?;
        assert_eq!(asset_amount, deposited_amount);

        match supported_token_mint {
            Some(supported_token_mint) => {
                token_interface::transfer_checked(
                    CpiContext::new(
                        supported_token_program.unwrap().to_account_info(),
                        token_interface::TransferChecked {
                            from: user_supported_token_account.unwrap().to_account_info(),
                            to: fund_supported_token_reserve_account
                                .unwrap()
                                .to_account_info(),
                            mint: supported_token_mint.to_account_info(),
                            authority: self.user.to_account_info(),
                        },
                    ),
                    deposited_amount,
                    supported_token_mint.decimals,
                )?;
            }
            None => {
                anchor_lang::system_program::transfer(
                    CpiContext::new(
                        system_program.unwrap().to_account_info(),
                        anchor_lang::system_program::Transfer {
                            from: self.user.to_account_info(),
                            to: fund_reserve_account.unwrap().to_account_info(),
                        },
                    ),
                    deposited_amount,
                )?;
            }
        }

        // update asset value again
        FundService::new(self.receipt_token_mint, self.fund_account)?
            .update_asset_values(&mut pricing_service)?;

        Ok(events::UserDepositedToFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: self.fund_account.key(),
            supported_token_mint: supported_token_mint_key,
            updated_user_reward_accounts: event.updated_user_reward_accounts,

            user: self.user.key(),
            user_receipt_token_account: self.user_receipt_token_account.key(),
            user_fund_account: self.user_fund_account.key(),
            user_supported_token_account: user_supported_token_account
                .map(|token_account| token_account.key()),

            wallet_provider,
            contribution_accrual_rate,
            deposited_amount,
            minted_receipt_token_amount: receipt_token_mint_amount,
        })
    }

    pub fn process_deposit_sol(
        &mut self,
        system_program: &Program<'info, System>,
        fund_reserve_account: &SystemAccount<'info>,
        instructions_sysvar: &AccountInfo,
        pricing_sources: &'info [AccountInfo<'info>],
        sol_amount: u64,
        metadata: Option<DepositMetadata>,
        metadata_signer_key: &Pubkey,
    ) -> Result<events::UserDepositedToFund> {
        self.process_deposit(
            Some(system_program),
            Some(fund_reserve_account),
            None,
            None,
            None,
            None,
            instructions_sysvar,
            pricing_sources,
            sol_amount,
            metadata,
            metadata_signer_key,
        )
    }

    pub fn process_deposit_supported_token(
        &mut self,
        supported_token_program: &Interface<'info, TokenInterface>,
        supported_token_mint: &InterfaceAccount<'info, Mint>,
        fund_supported_token_reserve_account: &InterfaceAccount<'info, TokenAccount>,
        user_supported_token_account: &InterfaceAccount<'info, TokenAccount>,
        instructions_sysvar: &AccountInfo,
        pricing_sources: &'info [AccountInfo<'info>],
        supported_token_amount: u64,
        metadata: Option<DepositMetadata>,
        metadata_signer_key: &Pubkey,
    ) -> Result<events::UserDepositedToFund> {
        self.process_deposit(
            None,
            None,
            Some(supported_token_program),
            Some(supported_token_mint),
            Some(fund_supported_token_reserve_account),
            Some(user_supported_token_account),
            instructions_sysvar,
            pricing_sources,
            supported_token_amount,
            metadata,
            metadata_signer_key,
        )
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
            .new_pricing_service(pricing_sources)?;

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

        // receipt_token_lock_account.reload()?;

        self.fund_account
            .load_mut()?
            .reload_receipt_token_supply(self.receipt_token_mint)?;

        self.user_fund_account
            .reload_receipt_token_amount(self.user_receipt_token_account)?;

        // reduce user's reward accrual rate
        let event1 = RewardService::new(self.receipt_token_mint, self.reward_account)?
            .update_reward_pools_token_allocation(
                Some(self.user_reward_account),
                None,
                receipt_token_amount,
                None,
            )?;

        // log withdrawal request event
        Ok(events::UserRequestedWithdrawalFromFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: self.fund_account.key(),
            supported_token_mint,
            updated_user_reward_accounts: event1.updated_user_reward_accounts,

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
            .new_pricing_service(pricing_sources)?;

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
        let event1 = RewardService::new(self.receipt_token_mint, self.reward_account)?
            .update_reward_pools_token_allocation(
                None,
                Some(self.user_reward_account),
                receipt_token_amount,
                None,
            )?;

        // log withdrawal request canceled event
        Ok(events::UserCanceledWithdrawalRequestFromFund {
            receipt_token_mint: self.receipt_token_mint.key(),
            fund_account: self.fund_account.key(),
            supported_token_mint: withdrawal_request.supported_token_mint,
            updated_user_reward_accounts: event1.updated_user_reward_accounts,

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
                        asset_user_amount,
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
                        asset_user_amount,
                    )?;
                }
            };
        }

        // after all requests are settled
        if fund_withdrawal_batch_account.is_settled() {
            // move small remains to operation reserved
            let remaining_asset_amount =
                fund_withdrawal_batch_account.get_remaining_asset_amount_after_settled();
            {
                let mut fund_account = self.fund_account.load_mut()?;
                let asset_state = fund_account.get_asset_state_mut(supported_token_mint_key)?;
                asset_state.withdrawal_user_reserved_amount -= remaining_asset_amount;
                asset_state.operation_reserved_amount += remaining_asset_amount;
            }

            // close the ticket to collect rent
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
            returned_receipt_token_amount: 0, // TODO/v0.4: returned_receipt_token_amount? if fund is absolutely lack of the certain asset
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
