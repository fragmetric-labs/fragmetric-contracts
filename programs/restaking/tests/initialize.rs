use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{account::Account, signature::Keypair, signer::Signer};

#[tokio::test]
async fn test_initialize() {
    let SetUpTest {
        validator,
        admin,
        fund_pda,
    } = SetUpTest::new();

    let mut context = validator.start_with_context().await;

    // let init_ix = Instruction {
    //     program_id: restaking::ID,
    //     accounts: restaking::accounts::Initialize {
    //         admin: admin.pubkey(),
    //         token_mint_authority: admin.pubkey(),
    //         fund: fund_pda,
    //     }
    // }
}

/// Struct set up to hold the validator, an optional user account, and the fund PDA.
/// Use SetUpTest::new() to create a new instance.
pub struct SetUpTest {
    pub validator: ProgramTest,
    pub admin: Keypair,
    pub fund_pda: Pubkey,
}

/// Returns the validator, an optional funded user account, and the fund PDA.
impl SetUpTest {
    pub fn new() -> Self {
        // Both of these work

        // let mut validator = ProgramTest::default();
        // validator.add_program("restaking", restaking::ID, None);
        let mut validator = ProgramTest::new("restaking", restaking::ID, None);

        // create a new user and fund with 1 SOL
        // add the user to the validator / ledger
        let admin = Keypair::new();
        validator.add_account(
            admin.pubkey(),
            Account {
                lamports: 1_000_000_000,
                ..Account::default()
            },
        );

        // get the fund PDA -- uses the same seed we used in the program
        let (fund_pda, _) = Pubkey::find_program_address(&[b"fund"], &restaking::ID);

        Self {
            validator,
            admin,
            fund_pda,
        }
    }
}
