import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as fs from "fs";

describe("fund_initialize", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.Restaking as Program<Restaking>;

    const admin = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user1.json")));
    console.log(`admin key: ${admin.publicKey}`);

    const receipt_token_name = "fragSOL";
    const [receipt_token_mint_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from(receipt_token_name)],
        program.programId
    );
    const [fund_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("fund"), receipt_token_mint_pda.toBuffer()],
        program.programId
    );

    // const providerKeypair = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../../id.json")));

    let tokenMint1;
    let tokenMint2;

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

        // create token mint accounts
        tokenMint1 = await spl.createMint(
            provider.connection,
            admin,
            admin.publicKey,
            admin.publicKey,
            9,
            undefined,
            undefined,
            TOKEN_2022_PROGRAM_ID
        );
        tokenMint2 = await spl.createMint(
            provider.connection,
            admin,
            admin.publicKey,
            admin.publicKey,
            9,
            undefined,
            undefined,
            TOKEN_2022_PROGRAM_ID
        );
        console.log(`tokenMint1 written:`, tokenMint1);
        console.log("tokenMint2 written:", tokenMint2);

        fs.writeFileSync("./tests/restaking/tokenMint1", JSON.stringify(tokenMint1));
        fs.writeFileSync("./tests/restaking/tokenMint2", JSON.stringify(tokenMint2));
    });

    it("Is initialized!", async () => {
        const default_protocol_fee_rate = 10;
        const tokenCap1 = new anchor.BN(1_000_000_000 * 1000);
        const tokenCap2 = new anchor.BN(1_000_000_000 * 2000);

        const tokens = [
            {
                address: tokenMint1,
                tokenCap: tokenCap1,
                tokenAmountIn: new anchor.BN(0),
            },
            {
                address: tokenMint2,
                tokenCap: tokenCap2,
                tokenAmountIn: new anchor.BN(0),
            }
        ];

        // const [receipt_token_lock_account_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
        //     [Buffer.from("receipt_lock"), receipt_token_mint_pda.toBuffer()],
        //     program.programId
        // );

        // spl.createMint(
        //     provider.connection,
        //     admin,
        //     admin.publicKey,
        //     admin.publicKey,
        //     9,
        // )

        console.log(`fund_pda: ${fund_pda}, receipt_token_mint_pda: ${receipt_token_mint_pda}`);
        console.log(TOKEN_2022_PROGRAM_ID, anchor.web3.SystemProgram.programId);

        const tx = await program.methods
            .fundInitialize(
                receipt_token_name,
                default_protocol_fee_rate,
                tokens,
            )
            .accounts({
                admin: admin.publicKey,
                // fund: fund_pda,
                // receiptTokenMint: receipt_token_mint_pda,
                // receiptTokenLockAccount: receipt_token_lock_account_pda,
                tokenProgram: TOKEN_2022_PROGRAM_ID,
                // systemProgram: anchor.web3.SystemProgram.programId,
            })
            .signers([admin])
            .rpc();
        console.log("Initialize transaction signature", tx);

        // check fund initialized correctly
        const tokensInitialized = (await program.account.fund.fetch(fund_pda)).whitelistedTokens;
        console.log(`tokenInitialized:`, tokensInitialized);

        expect(tokensInitialized[0].address.toString()).to.eq(tokenMint1.toString());
        expect(tokensInitialized[0].tokenCap.toNumber()).to.eq(tokenCap1.toNumber());
        expect(tokensInitialized[0].tokenAmountIn.toNumber()).to.eq(0);

        expect(tokensInitialized[1].address.toString()).to.eq(tokenMint2.toString());
        expect(tokensInitialized[1].tokenCap.toNumber()).to.equal(tokenCap2.toNumber());
        expect(tokensInitialized[1].tokenAmountIn.toNumber()).to.eq(0);
    });
});
