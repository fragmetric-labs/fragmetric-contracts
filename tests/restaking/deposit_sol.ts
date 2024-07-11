import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";

describe("deposit_sol", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.Restaking as Program<Restaking>;

    const admin = anchor.web3.Keypair.generate();
    const depositor = anchor.web3.Keypair.generate();

    const lst1 = anchor.web3.Keypair.generate();
    const lst2 = anchor.web3.Keypair.generate();

    before("Sol airdrop", async () => {
        // airdrop some SOL to the signer
        let airdropSignature = await provider.connection.requestAirdrop(
            admin.publicKey,
            1 * anchor.web3.LAMPORTS_PER_SOL // 1 SOL
        );

        // confirm the transaction
        await provider.connection.confirmTransaction(airdropSignature);

        // check the balance
        const adminBal = await provider.connection.getBalance(admin.publicKey);
        console.log(`admin SOL balance: ${adminBal}`);

        // airdrop depositor
        airdropSignature = await provider.connection.requestAirdrop(
            depositor.publicKey,
            1 * anchor.web3.LAMPORTS_PER_SOL // 1 SOL
        );
        await provider.connection.confirmTransaction(airdropSignature);
        const depositorBal = await provider.connection.getBalance(depositor.publicKey);
        console.log(`depositor SOL balance: ${depositorBal}`);
    });

    it("Is initialized!", async () => {
        const receipt_token_name = "fragSOL";
        const default_protocol_fee_rate = 10;
        const whitelisted_tokens = [lst1.publicKey, lst2.publicKey];
        const lst_caps = [new anchor.BN(1000), new anchor.BN(2000)];

        const [fund_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("fund")],
            program.programId
        );
        const [receipt_token_mint_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from(receipt_token_name)],
            program.programId
        );
        const [receipt_token_lock_account_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("receipt_lock"), receipt_token_mint_pda.toBuffer()],
            program.programId
        );

        // spl.createMint(
        //     provider.connection,
        //     admin,
        //     admin.publicKey,
        //     admin.publicKey,
        //     9,
        // )

        console.log(fund_pda, receipt_token_mint_pda, receipt_token_lock_account_pda);
        console.log(TOKEN_2022_PROGRAM_ID, anchor.web3.SystemProgram.programId);

        const tx = await program.methods
            .initialize(
                receipt_token_name,
                default_protocol_fee_rate,
                whitelisted_tokens,
                lst_caps,
            )
            .accounts({
                admin: admin.publicKey,
                // fund: fund_pda,
                // receiptTokenMint: receipt_token_mint_pda,
                // receiptTokenLockAccount: receipt_token_lock_account_pda,
                // tokenProgram: TOKEN_2022_PROGRAM_ID,
                // systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([admin])
            .rpc();
        console.log("Initialize transaction signature", tx);
    });

    // it("Deposit SOL!", async () => {
    //     let amount = new anchor.BN(1_000);

    //     const tx = await program.methods
    //         .depositSol(
    //             amount,
    //         )
    //         .accounts({ depositor: depositor.publicKey })
    // });
});
