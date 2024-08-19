import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { EventParser, Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as fs from "fs";
import * as utils from "../utils/utils";
import * as ed25519 from "ed25519";

export const deposit_sol = describe("deposit_sol", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.Restaking as Program<Restaking>;

    const admin = (program.provider as anchor.AnchorProvider).wallet;
    const payer = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user1.json")));
    const user = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user2.json")));
    console.log(`Payer key: ${payer.publicKey}`);
    console.log(`User key: ${user.publicKey}`);

    const adminKeypair = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../../id.json")));

    // for depositor provider
    // const userProvider = new anchor.AnchorProvider(program.provider.connection, new anchor.Wallet(user)); // and setProvider when needed

    let receiptTokenMint: anchor.web3.Keypair;
    let tokenMint1: anchor.web3.PublicKey;
    let tokenMint2: anchor.web3.PublicKey;
    let fund_pda: anchor.web3.PublicKey;
    let fund_token_authority_pda: anchor.web3.PublicKey;

    const eventParser = new EventParser(program.programId, program.coder);

    before("Sol airdrop", async () => {
        await utils.requestAirdrop(program.provider, payer, 10);
        await utils.requestAirdrop(program.provider, user, 10);

        // check the balance
        const adminBal = await program.provider.connection.getBalance(admin.publicKey);
        console.log(`Admin SOL balance: ${adminBal}`);
        const payerBal = await program.provider.connection.getBalance(payer.publicKey);
        console.log(`Payer SOL balance: ${payerBal}`);
        const userBal = await program.provider.connection.getBalance(user.publicKey);
        console.log(`User SOL balance: ${userBal}`);
    });

    before("Prepare accounts", async () => {
        receiptTokenMint = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("./fragsolMint.json")));
        tokenMint1 = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/tokenMint1", {encoding: "utf8"}).replace(/"/g, ''));
        tokenMint2 = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/tokenMint2", {encoding: "utf8"}).replace(/"/g, ''));
        console.log(`tokenMint1: ${tokenMint1}, tokenMint2: ${tokenMint2}`);

        [fund_pda] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("fund"), receiptTokenMint.publicKey.toBuffer()],
            program.programId
        );
        [fund_token_authority_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("fund_token_authority"), receiptTokenMint.publicKey.toBuffer()],
            program.programId,
        );

        const receiptTokenMintAccount = (await spl.getMint(program.provider.connection, receiptTokenMint.publicKey, undefined, TOKEN_2022_PROGRAM_ID));
        console.log("Fund =", fund_pda);
        console.log("Fund Token Authority =", fund_token_authority_pda);
        console.log("Receipt Token Mint =", receiptTokenMintAccount.address);
        console.log("It's authority =", receiptTokenMintAccount.mintAuthority);
        console.log("It's freeze authority = ", receiptTokenMintAccount.freezeAuthority);
    });

    it("Deposit SOL with no metadata", async () => {
        let amount = new anchor.BN(1_000_000_000);

        const fundBal_bef = await program.provider.connection.getBalance(fund_pda);
        console.log(`fund balance before:`, fundBal_bef);

        const tx = await program.methods
            .fundDepositSol({
                v1: {
                    0: {
                        amount: amount,
                    }
                }
            }, null)
            .accounts({
                user: user.publicKey,
            })
            .signers([user])
            .rpc({ commitment: "confirmed" });
        console.log("DepositSOL transaction signature", tx);

        const fundBal_aft = await program.provider.connection.getBalance(fund_pda);
        console.log(`fund balance after:`, fundBal_aft);
        console.log(`balance difference:`, fundBal_aft - fundBal_bef);

        // check associated token account
        const associatedToken = await spl.getAssociatedTokenAddress(
            receiptTokenMint.publicKey,
            user.publicKey,
            false,
            TOKEN_2022_PROGRAM_ID,
        );
        console.log(`associatedToken address:`, associatedToken);

        // check the sol_amount_in has accumulated
        let fundData = (await program.account.fund.fetch(fund_pda)).data.v2[0];

        console.log(`total sol amount in Fund:`, fundData.solAmountIn.toString());
        expect(fundData.solAmountIn.toString()).to.eq(amount.toString());

        // parse event
        const committedTx = await program.provider.connection.getParsedTransaction(tx, "confirmed");
        console.log(`committedTx:`, committedTx);
        const events = eventParser.parseLogs(committedTx.meta.logMessages);
        for (const event of events) {
            console.log(`FundDepositSOL event:`, event);
        }
    });

    it("Deposit SOL with metadata - should pass signature verification", async () => {
        let amount = new anchor.BN(1_000_000_000);

        const fundBal_bef = await program.provider.connection.getBalance(fund_pda);
        console.log(`fund balance before:`, fundBal_bef);

        const payload = {
            walletProvider: "backpack",
            fpointAccrualRateMultiplier: 1.3,
        };
        const programBorshCoder = new anchor.BorshCoder(program.idl);
        let encodedData = programBorshCoder.types.encode(program.idl.types[21].name, payload);
        console.log(`encodedData:`, encodedData);
        let decodedData = programBorshCoder.types.decode(program.idl.types[21].name, encodedData);
        console.log(`decodedData:`, decodedData);
        const signature = ed25519.Sign(encodedData, Buffer.from(adminKeypair.secretKey));

        const tx = new anchor.web3.Transaction().add(
            anchor.web3.Ed25519Program.createInstructionWithPublicKey({
                publicKey: admin.publicKey.toBytes(),
                message: encodedData,
                signature: signature,
            }),
            await program.methods
                .fundDepositSol(
                    {v1: {0: {amount: amount}}},
                    payload,
                )
                .accounts({
                    user: user.publicKey,
                    instructionSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
                })
                .signers([user])
                .instruction()
        );
        const txSig = await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            tx,
            [user],
            { commitment: "confirmed" },
        );
        console.log("DepositSOL transaction signature", txSig);

        const fundBal_aft = await program.provider.connection.getBalance(fund_pda);
        console.log(`fund balance after:`, fundBal_aft);
        console.log(`balance difference:`, fundBal_aft - fundBal_bef);

        let fundData = (await program.account.fund.fetch(fund_pda)).data.v2[0];
        console.log(`total sol amount in Fund:`, fundData.solAmountIn.toString());

        // parse event
        const committedTx = await program.provider.connection.getParsedTransaction(txSig, "confirmed");
        console.log(`committedTx:`, committedTx);
        const events = eventParser.parseLogs(committedTx.meta.logMessages);
        for (const event of events) {
            console.log(`FundDepositSOL event:`, event);
        }
    });
});
