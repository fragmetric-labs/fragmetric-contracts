import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";

export const fund_initialize = describe("fund_initialize", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.Restaking as Program<Restaking>;

    const admin = (program.provider as anchor.AnchorProvider).wallet;
    const payer = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user1.json")));
    console.log(`Payer key: ${payer.publicKey}`);

    let receiptTokenMint: anchor.web3.PublicKey;
    let tokenMint1: anchor.web3.PublicKey;
    let tokenMint2: anchor.web3.PublicKey;
    let fund_pda: anchor.web3.PublicKey;
    let receipt_token_authority_pda: anchor.web3.PublicKey;

    before("Sol airdrop", async () => {
        // airdrop some SOL to the admin
        let airdropSignature = await program.provider.connection.requestAirdrop(
            admin.publicKey,
            1 * anchor.web3.LAMPORTS_PER_SOL // 1 SOL
        );

        // confirm the transaction
        await program.provider.connection.confirmTransaction(airdropSignature);

        // airdrop some SOL to the signer
        airdropSignature = await program.provider.connection.requestAirdrop(
            payer.publicKey,
            1 * anchor.web3.LAMPORTS_PER_SOL // 1 SOL
        );

        // confirm the transaction
        await program.provider.connection.confirmTransaction(airdropSignature);

        // check the balance
        const adminBal = await program.provider.connection.getBalance(admin.publicKey);
        console.log(`Admin SOL balance: ${adminBal}`);
        const payerBal = await program.provider.connection.getBalance(payer.publicKey);
        console.log(`Payer SOL balance: ${payerBal}`);
    });

    before("Create Mint", async () => {
        receiptTokenMint = await spl.createMint(
            program.provider.connection,
            payer,
            payer.publicKey,
            payer.publicKey,
            9,
            undefined,
            undefined,
            TOKEN_2022_PROGRAM_ID,
        );
        // create token mint accounts
        tokenMint1 = await spl.createMint(
            program.provider.connection,
            payer,
            payer.publicKey,
            payer.publicKey,
            9,
            undefined,
            undefined,
            TOKEN_2022_PROGRAM_ID
        );
        tokenMint2 = await spl.createMint(
            program.provider.connection,
            payer,
            payer.publicKey,
            payer.publicKey,
            9,
            undefined,
            undefined,
            TOKEN_2022_PROGRAM_ID
        );

        [fund_pda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("fund"), receiptTokenMint.toBuffer()],
            program.programId
        );
        [receipt_token_authority_pda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("receipt_token_authority"), receiptTokenMint.toBuffer()],
            program.programId,
        );

        const receiptTokenMintAccount = (await spl.getMint(program.provider.connection, receiptTokenMint, undefined, TOKEN_2022_PROGRAM_ID));
        console.log("Fund =", fund_pda);
        console.log("Receipt Token Authority =", receipt_token_authority_pda);
        console.log("Receipt Token Mint =", receiptTokenMintAccount.address);
        console.log("It's authority =", receiptTokenMintAccount.mintAuthority);
        console.log("It's freeze authority = ", receiptTokenMintAccount.freezeAuthority);
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

        const tx = await program.methods
            .fundInitialize({
                v1: {
                    0: {
                        defaultProtocolFeeRate: default_protocol_fee_rate,
                        whitelistedTokens: tokens,
                    }
                }
            })
            .accounts({
                receiptTokenMint: receiptTokenMint,
                // tokenProgram: TOKEN_2022_PROGRAM_ID,
            })
            .signers([])
            .rpc();
        console.log("Initialize transaction signature", tx);

        // check fund initialized correctly
        const tokensInitialized = (await program.account.fund.fetch(fund_pda)).data.v1[0].whitelistedTokens;

        expect(tokensInitialized[0].address.toString()).to.eq(tokenMint1.toString());
        expect(tokensInitialized[0].tokenCap.toNumber()).to.eq(tokenCap1.toNumber());
        expect(tokensInitialized[0].tokenAmountIn.toNumber()).to.eq(0);

        expect(tokensInitialized[1].address.toString()).to.eq(tokenMint2.toString());
        expect(tokensInitialized[1].tokenCap.toNumber()).to.equal(tokenCap2.toNumber());
        expect(tokensInitialized[1].tokenAmountIn.toNumber()).to.eq(0);
    });
});
