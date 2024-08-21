use anchor_lang::{prelude::*, solana_program::instruction::Instruction, system_program};
use anchor_spl::{
    associated_token::{self, get_associated_token_address_with_program_id},
    token_interface::spl_token_2022,
};

use restaking::{common::PDASignerSeeds, constants::FRAGSOL_MINT_ADDRESS, fund::*};
use solana_program_test::{tokio, ProgramTest, ProgramTestContext};
use solana_sdk::{account::Account, signature::Keypair, signer::Signer, transaction::Transaction};

#[tokio::test]
async fn test_deposit_sol() {
    let SetUpTest {
        validator,
        user,
        user_receipt,
        fund_token_authority,
        receipt_token_mint,
        receipt_token_account,
        fund,
    } = SetUpTest::new();

    let mut context = validator.start_with_context().await;
    let amount: u64 = 1_000;

    let deposit_sol_ix = Instruction {
        program_id: restaking::ID,
        accounts: restaking::accounts::FundDepositSOL {
            user: user.pubkey(),
            user_receipt,
            fund,
            fund_token_authority,
            receipt_token_mint,
            receipt_token_account,
            token_pricing_source_0: Pubkey::new_unique(),
            token_pricing_source_1: Pubkey::new_unique(),
            token_program: spl_token_2022::ID,
            associated_token_program: associated_token::ID,
            instruction_sysvar: None,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: restaking::instruction::FundDepositSol {
            amount,
            metadata: None,
        }
        .try_to_vec()
        .unwrap(),
    };

    let deposit_sol_tx = Transaction::new_signed_with_payer(
        &[deposit_sol_ix],
        Some(&user.pubkey()),
        &[&user],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(deposit_sol_tx)
        .await
        .unwrap();

    let mut _fund: restaking::fund::Fund = load_and_deserialize(context, fund).await;

    msg!(
        "fund sol_withdrawal_fee_rate: {}",
        _fund.withdrawal_status.sol_withdrawal_fee_rate
    );
}

pub struct SetUpTest {
    pub validator: ProgramTest,
    pub user: Keypair,
    pub user_receipt: Pubkey,
    pub fund_token_authority: Pubkey,
    pub receipt_token_mint: Pubkey,
    // pub receipt_token_lock_account: Pubkey,
    pub receipt_token_account: Pubkey,
    pub fund: Pubkey,
}

impl Default for SetUpTest {
    fn default() -> Self {
        // let mut validator = ProgramTest::new("restaking", restaking::ID, processor!(restaking::entry));
        let mut validator = ProgramTest::new("restaking", restaking::ID, None);
        // let mut validator = ProgramTest::default();
        // validator.add_program("restaking", restaking::ID, None);

        let user = Keypair::new();
        validator.add_account(
            user.pubkey(),
            Account {
                lamports: 1_000_000_000,
                ..Account::default()
            },
        );

        let (receipt_token_mint_pda, _) =
            Pubkey::find_program_address(&[b"fragSOL"], &restaking::ID);
        let (user_receipt_pda, _) = Pubkey::find_program_address(
            &[UserReceipt::SEED, FRAGSOL_MINT_ADDRESS.as_ref()],
            &restaking::ID,
        );
        let (fund_pda, _) = Pubkey::find_program_address(
            &[Fund::SEED, receipt_token_mint_pda.as_ref()],
            &restaking::ID,
        );
        let (fund_token_authority_pda, _) = Pubkey::find_program_address(
            &[FundTokenAuthority::SEED, FRAGSOL_MINT_ADDRESS.as_ref()],
            &restaking::ID,
        );
        // let (receipt_token_lock_account_pda, _) = Pubkey::find_program_address(&[b"receipt_lock", receipt_token_mint_pda.as_ref()], &restaking::ID);

        msg!("receipt_token_mint_pda: {}", receipt_token_mint_pda);
        msg!("fund_pda: {}", fund_pda);
        // msg!("receipt_token_lock_account_pda: {}", receipt_token_lock_account_pda);

        let receipt_token_account = get_associated_token_address_with_program_id(
            &user.pubkey(),
            &receipt_token_mint_pda,
            &associated_token::ID,
        );
        msg!("receipt_token_account: {}", receipt_token_account);

        Self {
            validator,
            user,
            user_receipt: user_receipt_pda,
            receipt_token_mint: receipt_token_mint_pda,
            fund_token_authority: fund_token_authority_pda,
            // receipt_token_lock_account: receipt_token_lock_account_pda,
            receipt_token_account,
            fund: fund_pda,
        }
    }
}

impl SetUpTest {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn create_initialize_tx() {}

pub async fn load_and_deserialize<T: AccountDeserialize>(
    mut ctx: ProgramTestContext,
    address: Pubkey,
) -> T {
    let account = ctx
        .banks_client
        .get_account(address)
        .await
        .unwrap() // unwraps the Result into an Option<Account>
        .unwrap(); // unwraps the Option<Account> into an Account

    T::try_deserialize(&mut account.data.as_slice()).unwrap()
}
