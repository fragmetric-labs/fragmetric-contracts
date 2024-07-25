import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";

export const deposit_sol = describe("deposit_sol", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.Restaking as Program<Restaking>;

    const admin = (program.provider as anchor.AnchorProvider).wallet;
    const payer = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user1.json")));
    const user = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user2.json")));
    console.log(`Payer key: ${payer.publicKey}`);
    console.log(`User key: ${user.publicKey}`);

    // for depositor provider
    // const userProvider = new anchor.AnchorProvider(program.provider.connection, new anchor.Wallet(user)); // and setProvider when needed

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

        // airdrop some SOL to the user
        airdropSignature = await program.provider.connection.requestAirdrop(
            user.publicKey,
            1 * anchor.web3.LAMPORTS_PER_SOL // 1 SOL
        );

        // confirm the transaction
        await program.provider.connection.confirmTransaction(airdropSignature);

        // check the balance
        const adminBal = await program.provider.connection.getBalance(admin.publicKey);
        console.log(`Admin SOL balance: ${adminBal}`);
        const payerBal = await program.provider.connection.getBalance(payer.publicKey);
        console.log(`Payer SOL balance: ${payerBal}`);
        const userBal = await program.provider.connection.getBalance(user.publicKey);
        console.log(`User SOL balance: ${userBal}`);
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
    })

    before("Initialize Fund", async () => {
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
        await program.methods
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
    })

    it("Deposit SOL!", async () => {
        let amount = new anchor.BN(1_000);

        const fundBal_bef = await program.provider.connection.getBalance(fund_pda);
        console.log(`fund balance before:`, fundBal_bef);

        try {
            const tx = await program.methods
                .fundDepositSol({
                    v1: {
                        0: {
                            amount: amount,
                        }
                    }
                })
                .accounts({
                    user: user.publicKey,
                    // depositor: provider.wallet.publicKey,
                    receiptTokenMint: receiptTokenMint,
                    // receiptTokenAccount: provider.wallet.publicKey,
                    tokenProgram: TOKEN_2022_PROGRAM_ID,
                })
                .signers([user])
                .rpc();
            console.log("DepositSOL transaction signature", tx);

            const fundBal_aft = await program.provider.connection.getBalance(fund_pda);
            console.log(`fund balance after:`, fundBal_aft);
            console.log(`balance difference:`, fundBal_aft - fundBal_bef);

            // check associated token account
            const associatedToken = await spl.getAssociatedTokenAddress(
                receiptTokenMint,
                user.publicKey,
                false,
                TOKEN_2022_PROGRAM_ID,
            );
            console.log(`associatedToken address:`, associatedToken);

            // check the sol_amount_in has accumulated
            let totalSolAmtInFund = (await program.account.fund.fetch(fund_pda)).data.v1[0].solAmountIn;
            console.log(`total sol amount in Fund:`, totalSolAmtInFund.toString());
            expect(totalSolAmtInFund.toString()).to.eq(amount.toString());
        } catch (err) {
            console.log("DepositSOL err:", err);
            throw Error(err);
        }
    });
});
