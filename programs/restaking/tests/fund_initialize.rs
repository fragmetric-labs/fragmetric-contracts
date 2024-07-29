use anchor_lang::{prelude::*, solana_program::instruction::Instruction, system_program};
use anchor_spl::associated_token::get_associated_token_address;
use solana_program_test::{tokio, ProgramTest, ProgramTestContext};
use solana_sdk::{account::Account, signature::Keypair, signer::Signer, transaction::Transaction};

#[tokio::test]
async fn test_fund_initialize() {
    let SetUpTest {
        validator,
        admin,
        receipt_token_mint,
        fund,
        fund_token_authority,
        receipt_token_lock_account,
    } = SetUpTest::new();

    let mut context = validator.start_with_context().await;

    let lst1 = Pubkey::new_unique();
    let lst2 = Pubkey::new_unique();

    let initialize_ix = Instruction {
        program_id: restaking::ID,
        accounts: restaking::accounts::FundInitialize {
            admin: admin.pubkey(),
            fund,
            receipt_token_mint,
            fund_token_authority,
            receipt_token_lock_account,
            token_program: anchor_spl::token_2022::ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: restaking::instruction::FundInitialize {
            request: restaking::fund::FundInitializeRequest::V1(
                restaking::fund::FundInitializeRequestV1 {
                    default_protocol_fee_rate: 10,
                    whitelisted_tokens: vec![
                        restaking::fund::TokenInfo {
                            address: lst1.key(),
                            token_cap: 1_000_000_000 * 1000,
                            token_amount_in: 1_000_000_000,
                        },
                        restaking::fund::TokenInfo {
                            address: lst2.key(),
                            token_cap: 1_000_000_000 * 2000,
                            token_amount_in: 2_000_000_000,
                        },
                    ],
                },
            ),
        }
        .try_to_vec()
        .unwrap(),
    };

    let initialize_tx = Transaction::new_signed_with_payer(
        &[initialize_ix],
        Some(&admin.pubkey()),
        &[&admin],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(initialize_tx)
        .await
        .unwrap();
}

pub struct SetUpTest {
    pub validator: ProgramTest,
    pub admin: Keypair,
    pub receipt_token_mint: Pubkey,
    pub fund: Pubkey,
    pub fund_token_authority: Pubkey,
    pub receipt_token_lock_account: Pubkey,
}

impl Default for SetUpTest {
    fn default() -> Self {
        // let mut validator = ProgramTest::new("restaking", restaking::ID, processor!(restaking::entry));
        let mut validator = ProgramTest::new("restaking", restaking::ID, None);
        // let mut validator = ProgramTest::default();
        // validator.add_program("restaking", restaking::ID, None);

        let admin = Keypair::new();
        validator.add_account(
            admin.pubkey(),
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
        let (fund_token_authority_pda, _) = Pubkey::find_program_address(
            &[b"fund_token_authority", receipt_token_mint_pda.as_ref()],
            &restaking::ID,
        );
        let receipt_token_lock_account =
            get_associated_token_address(&fund_token_authority_pda, &receipt_token_mint_pda);

        msg!("receipt_token_mint_pda: {}", receipt_token_mint_pda);
        msg!("fund_pda: {}", fund_pda);
        msg!("fund_token_authority_pda: {}", fund_token_authority_pda);
        msg!(
            "receipt_token_lock_account_pda: {}",
            receipt_token_lock_account
        );

        Self {
            validator,
            admin,
            receipt_token_mint: receipt_token_mint_pda,
            fund: fund_pda,
            fund_token_authority: fund_token_authority_pda,
            receipt_token_lock_account,
        }
    }
}
impl SetUpTest {
    pub fn new() -> Self {
        Self::default()
    }
}

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
