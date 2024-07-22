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

    const depositor = anchor.web3.Keypair.generate();
    console.log(`depositor key: ${depositor.publicKey}`);

    // for depositor provider
    const depositorProvider = new anchor.AnchorProvider(provider.connection, new anchor.Wallet(depositor)); // and setProvider when needed

    const lst1 = anchor.web3.Keypair.generate();
    const lst2 = anchor.web3.Keypair.generate();

    const receipt_token_name = "fragSOL";
    const [receipt_token_mint_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from(receipt_token_name)],
        program.programId
    );
    const [fund_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("fund"), receipt_token_mint_pda.toBuffer()],
        // [Buffer.from("fund")],
        program.programId
    );

    before("Sol airdrop", async () => {
        // airdrop depositor
        let airdropSignature = await provider.connection.requestAirdrop(
            depositor.publicKey,
            1 * anchor.web3.LAMPORTS_PER_SOL // 1 SOL
        );
        await provider.connection.confirmTransaction(airdropSignature);
        const depositorBal = await provider.connection.getBalance(depositor.publicKey);
        console.log(`depositor SOL balance: ${depositorBal}`);
        console.log(`depositor key: ${depositor.publicKey}`);
    });

    it("Deposit SOL!", async () => {
        let amount = new anchor.BN(1_000);

        const fundBal_bef = await provider.connection.getBalance(fund_pda);
        console.log(`fund balance before:`, fundBal_bef);

        try {
            const tx = await program.methods
                .depositSol(
                    amount,
                )
                .accounts({
                    depositor: depositor.publicKey,
                    // depositor: provider.wallet.publicKey,
                    receiptTokenMint: receipt_token_mint_pda,
                    // receiptTokenAccount: provider.wallet.publicKey,
                    tokenProgram: TOKEN_2022_PROGRAM_ID,
                })
                .signers([depositor])
                .rpc();
            console.log("DepositSOL transaction signature", tx);

            const fundBal_aft = await provider.connection.getBalance(fund_pda);
            console.log(`fund balance after:`, fundBal_aft);
            console.log(`balance difference:`, fundBal_aft - fundBal_bef);

            // check associated token account
            const associatedToken = await spl.getAssociatedTokenAddress(
                receipt_token_mint_pda,
                depositor.publicKey,
                false,
                TOKEN_2022_PROGRAM_ID,
            );
            console.log(`associatedToken address:`, associatedToken);

            // check the sol_amount_in has accumulated
            let totalSolAmtInFund = (await program.account.fund.fetch(fund_pda)).solAmountIn;
            console.log(`total sol amount in Fund:`, totalSolAmtInFund.toString());
            expect(totalSolAmtInFund.toString()).to.eq(amount.toString());
        } catch (err) {
            console.log("DepositSOL err:", err);
            throw Error(err);
        }
    });
});
