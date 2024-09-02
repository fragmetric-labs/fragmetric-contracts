import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { EventParser, Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import * as chai from 'chai';
import chaiAsPromised from "chai-as-promised";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as utils from "../utils";
import * as ed25519 from "ed25519";
import {
    wallet,
    adminKeypair,
    tokenMint_bSOL,
    tokenMint_mSOL,
    tokenMint_INF,
    tokenMint_jitoSOL,
    fragSOLFundAddress,
    fragSOLTokenMintKeypair, tokenMintAuthorityKeypair_all, tokenMintAddress_bSOL, stakePoolAccounts,
} from "./1_initialize";

chai.use(chaiAsPromised);

export const deposit_token = describe("deposit_token", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.Restaking as Program<Restaking>;

    const payer = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../mocks/user1.json")));
    const user = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../mocks/user2.json")));
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

    before("Sol airdrop to user", async function () {
        if (utils.isLocalnet(program.provider.connection)) {
            await utils.requestAirdrop(program.provider, user, 10);

            // check the balance
            const adminBal = await program.provider.connection.getBalance(adminKeypair.publicKey);
            console.log(`Admin SOL balance: ${adminBal}`);
            const payerBal = await program.provider.connection.getBalance(payer.publicKey);
            console.log(`Payer SOL balance: ${payerBal}`);
            const userBal = await program.provider.connection.getBalance(user.publicKey);
            console.log(`User SOL balance: ${userBal}`);
            console.log("======= Sol airdrop to user =======");
        }
    });

    before("Mint mainnet mint tokens to user token account for localnet", async () => {
        if (utils.isLocalnet(program.provider.connection)) {
            // create user's bSOL token account
            [
                userBSOLTokenAccount,
                userMSOLTokenAccount,
                userJitoSOLTokenAccount,
                userInfTokenAccount,
            ] = await Promise.all([
                tokenMint_bSOL.address,
                tokenMint_mSOL.address,
                tokenMint_jitoSOL.address,
                tokenMint_INF.address,
            ].map(tokenMintAddress => spl.getOrCreateAssociatedTokenAccount(
                program.provider.connection,
                wallet.payer,
                tokenMintAddress,
                user.publicKey
            )));
            const tokenAccounts = [
                userBSOLTokenAccount,
                userMSOLTokenAccount,
                userJitoSOLTokenAccount,
                userInfTokenAccount,
            ];
            console.log(`user bSOL token account    = ${userBSOLTokenAccount.address}`);
            console.log(`user mSOL token account    = ${userMSOLTokenAccount.address}`);
            console.log(`user jitoSOL token account = ${userJitoSOLTokenAccount.address}`);
            console.log(`user INF token account     = ${userInfTokenAccount.address}`);

            // mint tokens to user
            await anchor.web3.sendAndConfirmTransaction(
                program.provider.connection,
                new anchor.web3.Transaction().add(
                    ...tokenAccounts.map(tokenAccount => spl.createMintToCheckedInstruction(
                        tokenAccount.mint,
                        tokenAccount.address,
                        tokenMintAuthorityKeypair_all.publicKey,
                        amount.toNumber(),
                        decimals,
                    )),
                ),
                [wallet.payer, tokenMintAuthorityKeypair_all],
            );
        }
    });

    it("Deposit bSOL, mSOL, JitoSOL, INF with no metadata", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        let amount = new anchor.BN(1_000_000_000);
        let txSig = await program.methods
            .userDepositSupportedToken(amount, null)
            .accounts({
                user: user.publicKey,
                supportedTokenMint: tokenMintAddress_bSOL,
                supportedTokenProgram: spl.TOKEN_PROGRAM_ID,
                userSupportedTokenAccount: userBSOLTokenAccount.address,
            })
            .remainingAccounts(stakePoolAccounts)
            .signers([user])
            .rpc({commitment: "confirmed"});

        // parse event
        let committedTx = await program.provider.connection.getParsedTransaction(txSig, "confirmed");
        let events = eventParser.parseLogs(committedTx.meta.logMessages);
        let rewardEvent = events.next().value as anchor.Event;
        expect(rewardEvent.data.updates.length).to.equal(1);
        expect(rewardEvent.data.updates[0].updatedUserRewardPools.length).to.equal(2);

        let depositEvent = events.next().value as anchor.Event;
        expect(depositEvent.data.walletProvider).to.be.null;
        expect(depositEvent.data.contributionAccrualRate).to.be.null;
    
        txSig = await program.methods
            .userDepositSupportedToken(amount, null)
            .accounts({
                user: user.publicKey,
                supportedTokenMint: tokenMint_mSOL.address,
                supportedTokenProgram: spl.TOKEN_PROGRAM_ID,
                userSupportedTokenAccount: userMSOLTokenAccount.address,
            })
            .remainingAccounts(stakePoolAccounts)
            .signers([user])
            .rpc({ commitment: "confirmed" });
        // parse event
        committedTx = await program.provider.connection.getParsedTransaction(txSig, "confirmed");
        events = eventParser.parseLogs(committedTx.meta.logMessages);
        rewardEvent = events.next().value as anchor.Event;
        expect(rewardEvent.data.updates.length).to.equal(1);
        expect(rewardEvent.data.updates[0].updatedUserRewardPools.length).to.equal(2);

        depositEvent = events.next().value as anchor.Event;
        expect(depositEvent.data.walletProvider).to.be.null;
        expect(depositEvent.data.contributionAccrualRate).to.be.null;
    
        // check the price of tokens
        let fundData = await program.account.fundAccount.fetch(fragSOLFundAddress);
        console.log(`bSOL price     = ${fundData.supportedTokens[0].price}`);
        console.log(`mSOL price     = ${fundData.supportedTokens[1].price}`);

        // check receipt token balance of user
        const userReceiptTokenAccountAddress = spl.getAssociatedTokenAddressSync(
            fragSOLTokenMintKeypair.publicKey,
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

    it("Fail when exceeding bSOL token cap!", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        const tokenCap = new anchor.BN(1_000_000_000 * 1000);
        amount = tokenCap.sub(amount).add(new anchor.BN(1)); // exceeding amount

        // first mint token to depositor
        await spl.mintToChecked(
            program.provider.connection,
            wallet.payer,
            tokenMint_bSOL.address,
            userBSOLTokenAccount.address,
            tokenMintAuthorityKeypair_all,
            amount.toNumber(),
            9,
            undefined,
            undefined,
        );

        const tokenAmountIn_bef = (await program.account.fundAccount.fetch(fragSOLFundAddress)).supportedTokens[0].operationReservedAmount;
        console.log(`fund token balance before deposit: ${tokenAmountIn_bef}`);

        expect(
            program.methods
                .userDepositSol(amount, null)
                .accounts({
                    user: user.publicKey,
                    supportedTokenProgram: spl.TOKEN_PROGRAM_ID,
                    supportedTokenMint: tokenMint_bSOL.address,
                    userSupportedTokenAccount: userBSOLTokenAccount.address,
                })
                .remainingAccounts(stakePoolAccounts)
                .signers([user])
                .rpc()
          ).to.eventually.throw('ExceedsTokenCap');

        // check if token's amount_in increased correctly
        const tokenAmountIn_aft = (await program.account.fundAccount.fetch(fragSOLFundAddress)).supportedTokens[0].operationReservedAmount;
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
            wallet.payer,
            tokenMint_bSOL.address,
            userBSOLTokenAccount.address,
            tokenMintAuthorityKeypair_all,
            amount.toNumber(),
            9,
            undefined,
            undefined,
        );
        const userBSOLBal = await getTokenBalance(program.provider.connection, userBSOLTokenAccount.address);
        console.log(`user bSOL balance:`, userBSOLBal);

        const fundBSOLBal_bef = (await program.account.fundAccount.fetch(fragSOLFundAddress)).supportedTokens[0].operationReservedAmount.toNumber();
        console.log(`fund bSOL balance before deposit:`, fundBSOLBal_bef);

        const payload = {
            walletProvider: "backpack",
            contributionAccrualRate: 1.3,
        };
        const programBorshCoder = new anchor.BorshCoder(program.idl);
        let depositMetadataType = program.idl.types.find(v => v.name == "depositMetadata");
        let encodedData = programBorshCoder.types.encode(depositMetadataType.name, payload);
        let decodedData = programBorshCoder.types.decode(depositMetadataType.name, encodedData);
        expect(decodedData.walletProvider).to.equal(payload.walletProvider);
        expect(decodedData.contributionAccrualRate.toPrecision(2)).to.equal(payload.contributionAccrualRate.toString());

        const signature = ed25519.Sign(encodedData, Buffer.from(adminKeypair.secretKey));
        const tx = new anchor.web3.Transaction().add(
            await program.methods
                .userUpdateAccountsIfNeeded()
                .accounts({
                    user: user.publicKey,
                })
                .instruction(),
            anchor.web3.Ed25519Program.createInstructionWithPublicKey({
                publicKey: adminKeypair.publicKey.toBytes(),
                message: encodedData,
                signature: signature,
            }),
            await program.methods
                .userDepositSupportedToken(
                    amount,
                    payload,
                )
                .accounts({
                    user: user.publicKey,
                    supportedTokenMint: tokenMint_bSOL.address,
                    supportedTokenProgram: spl.TOKEN_PROGRAM_ID,
                    userSupportedTokenAccount: userBSOLTokenAccount.address,
                    // instructionSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
                })
                .remainingAccounts(stakePoolAccounts)
                .signers([user])
                .instruction()
        );
        const depositTokenSig = await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            tx,
            [user],
            { commitment: "confirmed" },
        );

        const fundBSOLBal_aft = (await program.account.fundAccount.fetch(fragSOLFundAddress)).supportedTokens[0].operationReservedAmount.toNumber();
        console.log(`fund bSOL balance after deposit:`, fundBSOLBal_aft);
        expect(fundBSOLBal_aft - fundBSOLBal_bef).to.equal(amount.toNumber());

        // parse event
        const committedTx = await program.provider.connection.getParsedTransaction(depositTokenSig, "confirmed");
        const events = eventParser.parseLogs(committedTx.meta.logMessages);
        const rewardEvent = events.next().value as anchor.Event;
        expect(rewardEvent.data.updates.length).to.equal(1);
        expect(rewardEvent.data.updates[0].updatedUserRewardPools.length).to.equal(2);

        const depositEvent = events.next().value as anchor.Event;
        expect(depositEvent.data.walletProvider).to.equal(payload.walletProvider);
        expect(depositEvent.data.contributionAccrualRate.toPrecision(2)).to.equal(payload.contributionAccrualRate.toString());
        console.log(`Wallet provider: ${depositEvent.data.walletProvider}`);
        console.log(`contribution accrual rate: ${depositEvent.data.contributionAccrualRate}`);

        // check the price of tokens
        let fundData = await program.account.fundAccount.fetch(fragSOLFundAddress);
        console.log(`bSOL price     = ${fundData.supportedTokens[0].price}`);
        console.log(`mSOL price     = ${fundData.supportedTokens[1].price}`);

        // check receipt token balance of user
        const userReceiptTokenAccountAddress = spl.getAssociatedTokenAddressSync(
            fragSOLTokenMintKeypair.publicKey,
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
