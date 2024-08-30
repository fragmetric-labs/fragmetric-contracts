import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { EventParser, Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import * as chai from 'chai';
import chaiAsPromised from "chai-as-promised";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as utils from "../utils/utils";
import * as ed25519 from "ed25519";
import * as restaking from "./1_initialize";

chai.use(chaiAsPromised);

export const deposit_token = describe("deposit_token", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.Restaking as Program<Restaking>;

    const admin = (program.provider as anchor.AnchorProvider).wallet as anchor.Wallet;
    const payer = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user1.json")));
    const user = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user2.json")));
    console.log(`Payer(user1.json) key: ${payer.publicKey}`);
    console.log(`User(user2.json) key: ${user.publicKey}`);

    let userToken1Account: spl.Account;
    let userBSOLTokenAccount: spl.Account;
    let userMSOLTokenAccount: spl.Account;
    let userJitoSOLTokenAccount: spl.Account;
    let userInfTokenAccount: spl.Account;

    let amount = new anchor.BN(1_000_000_000 * 10); // 10
    const decimals = 9;

    const eventParser = new EventParser(program.programId, program.coder);

    // Localnet only
    before("Sol airdrop to user", async function () {
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

    // Devnet only
    before("Create and mint to user token1 account", async () => {
        if (utils.isDevnet(program.provider.connection)) {
            // create depositor's token account
            userToken1Account = await spl.getOrCreateAssociatedTokenAccount(
                program.provider.connection,
                user,
                restaking.tokenMint1,
                user.publicKey,
                false,
                undefined,
                undefined,
                TOKEN_2022_PROGRAM_ID,
            );
            console.log(`User Token1 Account:`, userToken1Account.address);
    
            // mint some tokens to depositor
            await spl.mintToChecked(
                program.provider.connection,
                payer, // payer를 depositor로 설정하면 missing signature 에러남
                restaking.tokenMint1,
                userToken1Account.address,
                payer.publicKey,
                amount.toNumber(),
                9,
                undefined,
                undefined,
                TOKEN_2022_PROGRAM_ID,
            );
            const depositorToken1Bal = await getTokenBalance(program.provider.connection, userToken1Account.address);
            console.log(`User Token1 balance:`, depositorToken1Bal);
            console.log("======= Create and mint to user token1 account =======");
        }
    });

    // Localnet only
    before("Mint mainnet mint tokens to user token account for localnet", async () => {
        if (utils.isLocalnet(program.provider.connection)) {
            // create user's bSOL token account
            userBSOLTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
                program.provider.connection,
                payer,
                restaking.bSOLMint.address,
                user.publicKey,
                false,
                undefined,
                undefined,
            );
            userMSOLTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
                program.provider.connection,
                payer,
                restaking.mSOLMint.address,
                user.publicKey,
                false,
                undefined,
                undefined,
            );
            userJitoSOLTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
                program.provider.connection,
                payer,
                restaking.jitoSOLMint.address,
                user.publicKey,
                false,
                undefined,
                undefined,
            );
            userInfTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
                program.provider.connection,
                payer,
                restaking.infMint.address,
                user.publicKey,
                false,
                undefined,
                undefined,
            );
            console.log(`user bSOL token account    = ${userBSOLTokenAccount.address}`);
            console.log(`user mSOL token account    = ${userMSOLTokenAccount.address}`);
            console.log(`user jitoSOL token account = ${userJitoSOLTokenAccount.address}`);
            console.log(`user INF token account     = ${userInfTokenAccount.address}`);
    
            // mint tokens to user
            const mintTokensTx = new anchor.web3.Transaction().add(
                spl.createMintToCheckedInstruction(
                    restaking.bSOLMint.address,
                    userBSOLTokenAccount.address,
                    payer.publicKey,
                    amount.toNumber(),
                    decimals,
                    [],
                ),
                spl.createMintToCheckedInstruction(
                    restaking.mSOLMint.address,
                    userMSOLTokenAccount.address,
                    payer.publicKey,
                    amount.toNumber(),
                    decimals,
                    [],
                ),
                spl.createMintToCheckedInstruction(
                    restaking.jitoSOLMint.address,
                    userJitoSOLTokenAccount.address,
                    payer.publicKey,
                    amount.toNumber(),
                    decimals,
                    [],
                ),
                spl.createMintToCheckedInstruction(
                    restaking.infMint.address,
                    userInfTokenAccount.address,
                    payer.publicKey,
                    amount.toNumber(),
                    decimals,
                    [],
                ),
            );
            await anchor.web3.sendAndConfirmTransaction(
                program.provider.connection,
                mintTokensTx,
                [payer],
            );
    
            console.log("======= Mint mainnet mint tokens to user token account for localnet =======");
        }
    });

    // Devnet only
    it("Deposit tokenMint1", async function () {
        if (!utils.isDevnet(program.provider.connection)) {
            this.skip();
        }

        const tokenAmountIn_bef = (await program.account.fund.fetch(restaking.fund_pda)).supportedTokens[0].operationReservedAmount.toNumber();
        console.log(`token balance before deposit: ${tokenAmountIn_bef}`);

        await program.methods
            .fundDepositToken(amount, null)
            .accounts({
                user: user.publicKey,
                supportedTokenMint: restaking.tokenMint1,
                userSupportedTokenAccount: userToken1Account.address,
            })
            .signers([user])
            .rpc({ commitment: "confirmed" });

        // check if token's amount_in increased correctly
        const tokenAmountIn_aft = (await program.account.fund.fetch(restaking.fund_pda)).supportedTokens[0].operationReservedAmount.toNumber();
        console.log(`token balance after deposit: ${tokenAmountIn_aft}`);
        expect(tokenAmountIn_aft - tokenAmountIn_bef).to.equal(amount.toNumber());
    });

    // Localnet only
    it("Deposit bSOL, mSOL, JitoSOL, INF with no metadata", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        let amount = new anchor.BN(1_000_000_000);

        let txSig = await program.methods
            .fundDepositToken(amount, null)
            .accounts({
                user: user.publicKey,
                supportedTokenMint: restaking.bSOLMint.address,
                userSupportedTokenAccount: userBSOLTokenAccount.address,
                // instructionSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
                depositTokenProgram: spl.TOKEN_PROGRAM_ID,
            })
            .signers([user])
            .rpc({commitment: "confirmed"});
        // parse event
        let committedTx = await program.provider.connection.getParsedTransaction(txSig, "confirmed");
        // console.log(`committedTx:`, committedTx);
        let events = eventParser.parseLogs(committedTx.meta.logMessages);
        for (const event of events) {
            expect(event.data.walletProvider).to.be.null;
            expect(event.data.contributionAccrualRate).to.be.null;
        }
    
        txSig = await program.methods
            .fundDepositToken(amount, null)
            .accounts({
                user: user.publicKey,
                supportedTokenMint: restaking.mSOLMint.address,
                userSupportedTokenAccount: userMSOLTokenAccount.address,
                // instructionSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
                depositTokenProgram: spl.TOKEN_PROGRAM_ID,
            })
            .signers([user])
            .rpc({ commitment: "confirmed" });
        // parse event
        committedTx = await program.provider.connection.getParsedTransaction(txSig, "confirmed");
        // console.log(`committedTx:`, committedTx);
        events = eventParser.parseLogs(committedTx.meta.logMessages);
        for (const event of events) {
            expect(event.data.walletProvider).to.be.null;
            expect(event.data.contributionAccrualRate).to.be.null;
        }
    
        // check the price of tokens
        let fundData = await program.account.fund.fetch(restaking.fund_pda);
        console.log(`bSOL price     = ${fundData.supportedTokens[0].price}`);
        console.log(`mSOL price     = ${fundData.supportedTokens[1].price}`);

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
    });

    // Devnet only
    it("Fail when exceeding token1 token cap!", async function () {
        if (!utils.isDevnet(program.provider.connection)) {
            this.skip();
        }

        const tokenCap1 = new anchor.BN(1_000_000_000 * 1000);
        amount = tokenCap1.sub(amount).add(new anchor.BN(1_000)); // exceeding amount

        // first mint token to depositor
        await spl.mintToChecked(
            program.provider.connection,
            payer,
            restaking.tokenMint1,
            userToken1Account.address,
            payer.publicKey,
            amount.toNumber(),
            9,
            undefined,
            undefined,
            TOKEN_2022_PROGRAM_ID,
        );

        const tokenAmountIn_bef = (await program.account.fund.fetch(restaking.fund_pda)).supportedTokens[0].operationReservedAmount;
        console.log(`fund token balance before deposit: ${tokenAmountIn_bef}`);

        expect(
            program.methods
                .fundDepositToken(amount, null)
                .accounts({
                    user: user.publicKey,
                    supportedTokenMint: restaking.tokenMint1,
                    userSupportedTokenAccount: userToken1Account.address,
                })
                .signers([user])
                .rpc()
          ).to.eventually.throw('ExceedsTokenCap');

        // check if token's amount_in increased correctly
        const tokenAmountIn_aft = (await program.account.fund.fetch(restaking.fund_pda)).supportedTokens[0].operationReservedAmount;
        console.log(`fund token balance after deposit: ${tokenAmountIn_aft}`);
        expect(tokenAmountIn_bef.toNumber()).to.equal(tokenAmountIn_aft.toNumber());
    });

    // Localnet only
    it("Fail when exceeding bSOL token cap!", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        const tokenCap = new anchor.BN(1_000_000_000 * 1000);
        amount = tokenCap.sub(amount).add(new anchor.BN(1)); // exceeding amount

        // first mint token to depositor
        await spl.mintToChecked(
            program.provider.connection,
            payer,
            restaking.bSOLMint.address,
            userBSOLTokenAccount.address,
            payer.publicKey,
            amount.toNumber(),
            9,
            undefined,
            undefined,
        );

        const tokenAmountIn_bef = (await program.account.fund.fetch(restaking.fund_pda)).supportedTokens[0].operationReservedAmount;
        console.log(`fund token balance before deposit: ${tokenAmountIn_bef}`);

        expect(
            program.methods
                .fundDepositToken(amount, null)
                .accounts({
                    user: user.publicKey,
                    supportedTokenMint: restaking.bSOLMint.address,
                    userSupportedTokenAccount: userBSOLTokenAccount.address,
                })
                .signers([user])
                .rpc()
          ).to.eventually.throw('ExceedsTokenCap');

        // check if token's amount_in increased correctly
        const tokenAmountIn_aft = (await program.account.fund.fetch(restaking.fund_pda)).supportedTokens[0].operationReservedAmount;
        console.log(`fund token balance after deposit: ${tokenAmountIn_aft}`);
        expect(tokenAmountIn_bef.toNumber()).to.equal(tokenAmountIn_aft.toNumber());
    });

    // Localnet only
    it("Deposit bSOL, mSOL, JitoSOL, INF with metadata - should pass signature verification", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        let amount = new anchor.BN(1_000_000_000);

        // first mint token to depositor
        await spl.mintToChecked(
            program.provider.connection,
            payer,
            restaking.bSOLMint.address,
            userBSOLTokenAccount.address,
            payer.publicKey,
            amount.toNumber(),
            9,
            undefined,
            undefined,
        );
        const userBSOLBal = await getTokenBalance(program.provider.connection, userBSOLTokenAccount.address);
        console.log(`user bSOL balance:`, userBSOLBal);

        const fundBSOLBal_bef = (await program.account.fund.fetch(restaking.fund_pda)).supportedTokens[0].operationReservedAmount.toNumber();
        console.log(`fund bSOL balance before deposit:`, fundBSOLBal_bef);

        const payload = {
            walletProvider: "backpack",
            contributionAccrualRate: 1.3,
        };
        const programBorshCoder = new anchor.BorshCoder(program.idl);
        let metadataType = program.idl.types.find(v => v.name == "metadata");
        let encodedData = programBorshCoder.types.encode(metadataType.name, payload);
        let decodedData = programBorshCoder.types.decode(metadataType.name, encodedData);
        expect(decodedData.walletProvider).to.equal(payload.walletProvider);
        expect(decodedData.contributionAccrualRate.toPrecision(2)).to.equal(payload.contributionAccrualRate.toString());

        const signature = ed25519.Sign(encodedData, Buffer.from(admin.payer.secretKey));
        const tx = new anchor.web3.Transaction().add(
            anchor.web3.Ed25519Program.createInstructionWithPublicKey({
                publicKey: admin.publicKey.toBytes(),
                message: encodedData,
                signature: signature,
            }),
            await program.methods
                .fundDepositToken(
                    amount,
                    payload,
                )
                .accounts({
                    user: user.publicKey,
                    supportedTokenMint: restaking.bSOLMint.address,
                    userSupportedTokenAccount: userBSOLTokenAccount.address,
                    // instructionSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
                    depositTokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([user])
                .instruction()
        );
        const depositTokenSig = await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            tx,
            [user],
            { commitment: "confirmed" },
        );

        const fundBSOLBal_aft = (await program.account.fund.fetch(restaking.fund_pda)).supportedTokens[0].operationReservedAmount.toNumber();
        console.log(`fund bSOL balance after deposit:`, fundBSOLBal_aft);
        expect(fundBSOLBal_aft - fundBSOLBal_bef).to.equal(amount.toNumber());

        // parse event
        const committedTx = await program.provider.connection.getParsedTransaction(depositTokenSig, "confirmed");
        // console.log(`committedTx:`, committedTx);
        const events = eventParser.parseLogs(committedTx.meta.logMessages);
        for (const event of events) {
            expect(decodedData.walletProvider).to.equal(payload.walletProvider);
            expect(decodedData.contributionAccrualRate.toPrecision(2)).to.equal(payload.contributionAccrualRate.toString());
            expect(event.data.walletProvider).to.equal(payload.walletProvider);
            expect(event.data.contributionAccrualRate.toPrecision(2)).to.equal(payload.contributionAccrualRate.toString());
            console.log(`Wallet provider: ${event.data.walletProvider}`);
            console.log(`contribution accrual rate: ${event.data.contributionAccrualRate}`);
        }

        // check the price of tokens
        let fundData = await program.account.fund.fetch(restaking.fund_pda);
        console.log(`bSOL price     = ${fundData.supportedTokens[0].price}`);
        console.log(`mSOL price     = ${fundData.supportedTokens[1].price}`);

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
    });
});

const getTokenBalance = async (
    connection: anchor.web3.Connection,
    tokenAccount: anchor.web3.PublicKey,
) => {
    const info = await connection.getTokenAccountBalance(tokenAccount);
    if (info.value.uiAmount == null) throw new Error("No balance found");
    return info.value.uiAmount;
}
