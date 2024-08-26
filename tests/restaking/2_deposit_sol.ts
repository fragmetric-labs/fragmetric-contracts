import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { EventParser, Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as utils from "../utils/utils";
import * as restaking from "./1_fund_initialize";
import * as ed25519 from "ed25519";

export const deposit_sol = describe("deposit_sol", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.Restaking as Program<Restaking>;

    const admin = (program.provider as anchor.AnchorProvider).wallet as anchor.Wallet;
    const payer = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user1.json")));
    const user = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user2.json")));
    console.log(`Payer(user1.json) key: ${payer.publicKey}`);
    console.log(`User(user2.json) key: ${user.publicKey}`);

    const eventParser = new EventParser(program.programId, program.coder);

    // Localnet only
    before("Sol airdrop to user", async () => {
        if (utils.isLocalnet(program.provider.connection)) {
            await utils.requestAirdrop(program.provider, user, 10);
    
            // check the balance
            const adminBal = await program.provider.connection.getBalance(admin.publicKey);
            console.log(`Admin SOL balance: ${adminBal}`);
            const payerBal = await program.provider.connection.getBalance(payer.publicKey);
            console.log(`Payer SOL balance: ${payerBal}`);
            const userBal = await program.provider.connection.getBalance(user.publicKey);
            console.log(`User SOL balance: ${userBal}`);
            console.log("======= Sol airdrop to user =======");
        }
    });

    it("Update price", async () => {
        const updatePriceTx = await program.methods
            .fundUpdatePrice()
            .accounts({
                payer: admin.publicKey,
            })
            .rpc({commitment: "confirmed"});

        // parse event
        const committedTx = await program.provider.connection.getParsedTransaction(updatePriceTx, "confirmed");
        // console.log(`committedTx:`, committedTx);
        const events = eventParser.parseLogs(committedTx.meta.logMessages);
        for (const event of events) {
            expect(event.data.receiptTokenPrice.toNumber()).to.be.equal(1_000_000_000);
            console.log(`Receipt token price: ${event.data.receiptTokenPrice}`);
        }
    });

    it("Deposit SOL with no metadata", async () => {
        let amount = new anchor.BN(1_000_000_000);

        const fundBal_bef = await program.provider.connection.getBalance(restaking.fund_pda);
        console.log(`fund balance before deposit:`, fundBal_bef);

        const depositSolTx = await program.methods
            .fundDepositSol(amount, null)
            .accounts({
                user: user.publicKey,
            })
            .signers([user])
            .rpc({ commitment: "confirmed" });

        const fundBal_aft = await program.provider.connection.getBalance(restaking.fund_pda);
        console.log(`fund balance after deposit:`, fundBal_aft);
        expect(fundBal_aft - fundBal_bef).to.equal(amount.toNumber());

        // check the sol_amount_in has accumulated
        let fundData = await program.account.fund.fetch(restaking.fund_pda);
        console.log(`total sol amount in Fund:`, fundData.solAmountIn.toNumber());
        expect(fundData.solAmountIn.toNumber()).to.eq(amount.toNumber());

        // check the price of tokens
        console.log(`bSOL price     = ${fundData.supportedTokens[0].tokenPrice}`);
        console.log(`mSOL price     = ${fundData.supportedTokens[1].tokenPrice}`);

        // check receipt token balance of user
        const userReceiptTokenAccountAddress = spl.getAssociatedTokenAddressSync(
            restaking.receiptTokenMint.publicKey,
            user.publicKey,
            false,
            TOKEN_2022_PROGRAM_ID,
        );
        const userReceiptTokenAccount = await spl.getAccount(
            program.provider.connection,
            userReceiptTokenAccountAddress,
            undefined,
            TOKEN_2022_PROGRAM_ID,
        );
        console.log(`user associatedToken address:`, userReceiptTokenAccountAddress);
        console.log(`receipt token balance = ${userReceiptTokenAccount.amount}`);

        // parse event
        const committedTx = await program.provider.connection.getParsedTransaction(depositSolTx, "confirmed");
        // console.log(`committedTx:`, committedTx);
        const events = eventParser.parseLogs(committedTx.meta.logMessages);
        for (const event of events) {
            expect(event.data.walletProvider).to.be.null;
            expect(event.data.fpointAccrualRateMultiplier).to.be.null;
            console.log(`Wallet provider: ${event.data.walletProvider}`);
            console.log(`fPoint accrual rate multiplier: ${event.data.fpointAccrualRateMultiplier}`);
        }
    });

    it("Deposit SOL with metadata - should pass signature verification", async () => {
        let amount = new anchor.BN(1_000_000_000);

        const fundBal_bef = await program.provider.connection.getBalance(restaking.fund_pda);
        console.log(`fund balance before deposit:`, fundBal_bef);

        const payload = {
            walletProvider: "backpack",
            fpointAccrualRateMultiplier: 1.3,
        };
        const programBorshCoder = new anchor.BorshCoder(program.idl);
        let encodedData = programBorshCoder.types.encode(program.idl.types[9].name, payload);
        let decodedData = programBorshCoder.types.decode(program.idl.types[9].name, encodedData);
        expect(decodedData.walletProvider).to.equal(payload.walletProvider);
        expect(decodedData.fpointAccrualRateMultiplier.toPrecision(2)).to.equal(payload.fpointAccrualRateMultiplier.toString());

        const signature = ed25519.Sign(encodedData, Buffer.from(admin.payer.secretKey));
        const tx = new anchor.web3.Transaction().add(
            anchor.web3.Ed25519Program.createInstructionWithPublicKey({
                publicKey: admin.publicKey.toBytes(),
                message: encodedData,
                signature: signature,
            }),
            await program.methods
                .fundDepositSol(
                    amount,
                    payload,
                )
                .accounts({
                    user: user.publicKey,
                    // instructionSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
                })
                .signers([user])
                .instruction()
        );
        const depositSolTx = await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            tx,
            [user],
            { commitment: "confirmed" },
        );

        const fundBal_aft = await program.provider.connection.getBalance(restaking.fund_pda);
        console.log(`fund balance after deposit:`, fundBal_aft);
        expect(fundBal_aft - fundBal_bef).to.equal(amount.toNumber());

        // check the sol_amount_in has accumulated
        let fundData = await program.account.fund.fetch(restaking.fund_pda);
        console.log(`total sol amount in Fund:`, fundData.solAmountIn.toNumber());
        expect(fundData.solAmountIn.toNumber()).to.eq(2 * amount.toNumber());

        // check the price of tokens
        console.log(`bSOL price     = ${fundData.supportedTokens[0].tokenPrice}`);
        console.log(`mSOL price     = ${fundData.supportedTokens[1].tokenPrice}`);

        // check receipt token balance of user
        const userReceiptTokenAccountAddress = spl.getAssociatedTokenAddressSync(
            restaking.receiptTokenMint.publicKey,
            user.publicKey,
            false,
            TOKEN_2022_PROGRAM_ID,
        );
        const userReceiptTokenAccount = await spl.getAccount(
            program.provider.connection,
            userReceiptTokenAccountAddress,
            undefined,
            TOKEN_2022_PROGRAM_ID,
        );
        console.log(`user associatedToken address:`, userReceiptTokenAccountAddress);
        console.log(`receipt token balance = ${userReceiptTokenAccount.amount}`);

        // parse event
        const committedTx = await program.provider.connection.getParsedTransaction(depositSolTx, "confirmed");
        // console.log(`committedTx:`, committedTx);
        const events = eventParser.parseLogs(committedTx.meta.logMessages);
        for (const event of events) {
            expect(decodedData.walletProvider).to.equal(payload.walletProvider);
            expect(decodedData.fpointAccrualRateMultiplier.toPrecision(2)).to.equal(payload.fpointAccrualRateMultiplier.toString());
            expect(event.data.walletProvider).to.equal(payload.walletProvider);
            expect(event.data.fpointAccrualRateMultiplier.toPrecision(2)).to.equal(payload.fpointAccrualRateMultiplier.toString());
            console.log(`Wallet provider: ${event.data.walletProvider}`);
            console.log(`fPoint accrual rate multiplier: ${event.data.fpointAccrualRateMultiplier}`);
        }
    });
});
