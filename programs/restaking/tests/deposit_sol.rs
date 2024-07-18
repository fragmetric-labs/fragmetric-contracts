use anchor_lang::{prelude::*, solana_program::instruction::Instruction, system_program};
use anchor_spl::{
    associated_token::{
        self, get_associated_token_address_with_program_id,
        spl_associated_token_account::processor::process_instruction, AssociatedToken,
    },
    token_interface::spl_token_2022,
};
use restaking;
use solana_program_test::{processor, tokio, ProgramTest, ProgramTestContext};
use solana_sdk::{account::Account, signature::Keypair, signer::Signer, transaction::Transaction};

#[tokio::test]
async fn test_deposit_sol() {
    let SetUpTest {
        validator,
        depositor,
        receipt_token_mint,
        receipt_token_account,
        fund,
    } = SetUpTest::new();

    let mut context = validator.start_with_context().await;
    let amount: u64 = 1_000;

    // let deposit_sol_initialize_ix = Instruction {
    //     program_id: restaking::ID,
    //     accounts: restaking::accounts::InitializeDepositSOL { // Context struct type
    //         depositor: depositor.pubkey(),
    //         receipt_token_mint,
    //         receipt_token_account,
    //         token_program: spl_token_2022::ID,
    //         // associated_token_program: associated_token::ID,
    //         system_program: system_program::ID,
    //     }
    //     .to_account_metas(None),
    //     data: restaking::instruction::InitializeDepositSol {}.try_to_vec().unwrap(), // instruction name
    // };
    // let deposit_sol_initialize_tx = Transaction::new_signed_with_payer(
    //     &[deposit_sol_initialize_ix],
    //     Some(&depositor.pubkey()),
    //     &[&depositor],
    //     context.last_blockhash,
    // );

    // context
    //     .banks_client
    //     .process_transaction(deposit_sol_initialize_tx)
    //     .await
    //     .unwrap();

    let deposit_sol_ix = Instruction {
        program_id: restaking::ID,
        accounts: restaking::accounts::DepositSOL {
            depositor: depositor.pubkey(),
            fund,
            receipt_token_mint,
            receipt_token_account,
            token_program: spl_token_2022::ID,
            associated_token_program: associated_token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: restaking::instruction::DepositSol { amount }
            .try_to_vec()
            .unwrap(),
    };

    let deposit_sol_tx = Transaction::new_signed_with_payer(
        &[deposit_sol_ix],
        Some(&depositor.pubkey()),
        &[&depositor],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(deposit_sol_tx)
        .await
        .unwrap();

    let _fund: restaking::Fund = load_and_deserialize(context, fund).await;

    msg!("fund admin: {}", _fund.admin);
    msg!(
        "fund default_protocol_fee_rate: {}",
        _fund.default_protocol_fee_rate
    );
}

pub struct SetUpTest {
    pub validator: ProgramTest,
    pub depositor: Keypair,
    pub receipt_token_mint: Pubkey,
    // pub receipt_token_lock_account: Pubkey,
    pub receipt_token_account: Pubkey,
    pub fund: Pubkey,
}

impl SetUpTest {
    pub fn new() -> Self {
        // let mut validator = ProgramTest::new("restaking", restaking::ID, processor!(restaking::entry));
        let mut validator = ProgramTest::new("restaking", restaking::ID, None);
        // let mut validator = ProgramTest::default();
        // validator.add_program("restaking", restaking::ID, None);

        let depositor = Keypair::new();
        validator.add_account(
            depositor.pubkey(),
            Account {
                lamports: 1_000_000_000,
                ..Account::default()
            },
        );

        let (receipt_token_mint_pda, _) =
            Pubkey::find_program_address(&[b"fragSOL"], &restaking::ID);
        let (fund_pda, _) = Pubkey::find_program_address(
            &[b"fund", receipt_token_mint_pda.as_ref()],
            &restaking::ID,
        );
        // let (fund_pda, _) = Pubkey::find_program_address(&[b"fund"], &restaking::ID);
        // let (receipt_token_lock_account_pda, _) = Pubkey::find_program_address(&[b"receipt_lock", receipt_token_mint_pda.as_ref()], &restaking::ID);

        msg!("receipt_token_mint_pda: {}", receipt_token_mint_pda);
        msg!("fund_pda: {}", fund_pda);
        // msg!("receipt_token_lock_account_pda: {}", receipt_token_lock_account_pda);

        let receipt_token_account = get_associated_token_address_with_program_id(
            &depositor.pubkey(),
            &receipt_token_mint_pda,
            &associated_token::ID,
        );
        msg!("receipt_token_account: {}", receipt_token_account);

        Self {
            validator,
            depositor,
            receipt_token_mint: receipt_token_mint_pda,
            // receipt_token_lock_account: receipt_token_lock_account_pda,
            receipt_token_account,
            fund: fund_pda,
        }
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
