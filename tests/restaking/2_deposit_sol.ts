import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { EventParser, Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as utils from "../utils";
import * as restaking from "./1_initialize";
import * as ed25519 from "ed25519";
import {adminKeypair, fundManagerKeypair, stakePoolAccounts, wallet} from './1_initialize';

export const deposit_sol = describe("deposit_sol", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.Restaking as Program<Restaking>;

    const payer = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../mocks/user1.json")));
    const user = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../mocks/user2.json")));
    console.log(`Payer(user1.json) key: ${payer.publicKey}`);
    console.log(`User(user2.json) key: ${user.publicKey}`);

    const eventParser = new EventParser(program.programId, program.coder);

    // Localnet only
    before("Sol airdrop to user", async () => {
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

    it("Zeroing fPoint reward", async () => {
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .fundManagerSettleReward(0, 0, new anchor.BN(0))
                        .accounts({
                            rewardTokenMint: program.programId,
                            rewardTokenProgram: program.programId,
                        })
                        .instruction(),
                ]),
            ),
            [wallet.payer, fundManagerKeypair],
        );
    });

    it("Update price", async () => {
        const updatePriceTxSig = await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .operatorUpdatePrices()
                        .accounts({ operator: wallet.publicKey })
                        .remainingAccounts(stakePoolAccounts)
                        .instruction(),
                ]),
            ),
            [wallet.payer],
        );

        // parse event
        await new Promise(resolve => setTimeout(resolve, 1000));
        const committedTx = await program.provider.connection.getParsedTransaction(updatePriceTxSig, "confirmed");
        // console.log(`committedTx:`, committedTx);
        const events = eventParser.parseLogs(committedTx.meta.logMessages);
        for (const event of events) {
            expect(event.data.fundAccount.receiptTokenPrice.toNumber()).to.be.equal(1_000_000_000);
            console.log(`Receipt token price: ${event.data.fundAccount.receiptTokenPrice}`);
        }
    });

    // Localnet test
    it("Deposit SOL with no metadata", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        let amount = new anchor.BN(1_000_000_000);

        const fundBal_bef = await program.provider.connection.getBalance(restaking.fragSOLFundAddress);
        console.log(`fund balance before deposit:`, fundBal_bef);

        const depositSolTxSig = await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .userUpdateAccountsIfNeeded()
                        .accounts({
                            user: user.publicKey,
                        })
                        .instruction(),
                    program.methods
                        .userDepositSol(amount, null)
                        .accounts({
                            user: user.publicKey,
                        })
                        .remainingAccounts(stakePoolAccounts)
                        .instruction(),
                ]),
            ),
            [user],
            { commitment: "confirmed" },
        );

        // const depositSolTx = await program.methods
        //     .fundDepositSol(amount, null)
        //     .accounts({
        //         user: user.publicKey,
        //     })
        //     .signers([user])
        //     .rpc({ commitment: "confirmed" });

        const fundBal_aft = await program.provider.connection.getBalance(restaking.fragSOLFundAddress);
        console.log(`fund balance after deposit:`, fundBal_aft);
        expect(fundBal_aft - fundBal_bef).to.equal(amount.toNumber());

        // check the solOperationReservedAmount has accumulated
        let fundData = await program.account.fundAccount.fetch(restaking.fragSOLFundAddress);
        console.log(`total sol operation reserved amount in Fund:`, fundData.solOperationReservedAmount.toNumber());
        expect(fundData.solOperationReservedAmount.toNumber()).to.eq(amount.toNumber());

        // check the price of tokens
        console.log(`bSOL price     = ${fundData.supportedTokens[0].price}`);
        console.log(`mSOL price     = ${fundData.supportedTokens[1].price}`);

        // check receipt token balance of user
        const userReceiptTokenAccountAddress = spl.getAssociatedTokenAddressSync(
            restaking.fragSOLTokenMintKeypair.publicKey,
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
        console.log(`user associatedToken address:`, userReceiptTokenAccountAddress.toBase58());
        console.log(`receipt token balance = ${userReceiptTokenAccount.amount}`);

        // parse event
        const committedTx = await program.provider.connection.getParsedTransaction(depositSolTxSig, "confirmed");
        const events = eventParser.parseLogs(committedTx.meta.logMessages);
        const rewardEvent = events.next().value as anchor.Event;
        expect(rewardEvent.data.updates.length).to.equal(1);
        expect(rewardEvent.data.updates[0].updatedUserRewardPools.length).to.equal(2);

        const depositEvent = events.next().value as anchor.Event;
        expect(depositEvent.data.walletProvider).to.be.null;
        expect(depositEvent.data.contributionAccrualRate).to.be.null;
        expect(depositEvent.data.userFundAccount.receiptTokenAmount.toString()).to.be.equal(
            userReceiptTokenAccount.amount.toString()
        )
        console.log(`Wallet provider: ${depositEvent.data.walletProvider}`);
        console.log(`contribution accrual rate: ${depositEvent.data.contributionAccrualRate}`);
        console.log(`receipt token balance:`, depositEvent.data.userFundAccount.receiptTokenAmount.toNumber());
    });

    // Localnet test
    it("Deposit SOL with metadata - should pass signature verification", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        let amount = new anchor.BN(1_000_000_000);

        const fundBal_bef = await program.provider.connection.getBalance(restaking.fragSOLFundAddress);
        console.log(`fund balance before deposit:`, fundBal_bef);

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
            anchor.web3.Ed25519Program.createInstructionWithPublicKey({
                publicKey: adminKeypair.publicKey.toBytes(),
                message: encodedData,
                signature: signature,
            }),
            await program.methods
                .userDepositSol(
                    amount,
                    payload,
                )
                .accounts({
                    user: user.publicKey,
                    // instructionSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
                })
                .remainingAccounts(stakePoolAccounts)
                .signers([user])
                .instruction()
        );
        const depositSolTx = await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            tx,
            [user],
            { commitment: "confirmed" },
        );

        const fundBal_aft = await program.provider.connection.getBalance(restaking.fragSOLFundAddress);
        console.log(`fund balance after deposit:`, fundBal_aft);
        expect(fundBal_aft - fundBal_bef).to.equal(amount.toNumber());

        // check the sol_amount_in has accumulated
        let fundData = await program.account.fundAccount.fetch(restaking.fragSOLFundAddress);
        console.log(`total sol operation reserved amount in Fund:`, fundData.solOperationReservedAmount.toNumber());
        expect(fundData.solOperationReservedAmount.toNumber()).to.eq(2 * amount.toNumber());

        // check the price of tokens
        console.log(`bSOL price     = ${fundData.supportedTokens[0].price}`);
        console.log(`mSOL price     = ${fundData.supportedTokens[1].price}`);

        // check receipt token balance of user
        const userReceiptTokenAccountAddress = spl.getAssociatedTokenAddressSync(
            restaking.fragSOLTokenMintKeypair.publicKey,
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
        console.log(`user associatedToken address:`, userReceiptTokenAccountAddress.toBase58());
        console.log(`receipt token balance = ${userReceiptTokenAccount.amount}`);

        // parse event
        const committedTx = await program.provider.connection.getParsedTransaction(depositSolTx, "confirmed");
        const events = eventParser.parseLogs(committedTx.meta.logMessages);
        const rewardEvent = events.next().value as anchor.Event;
        expect(rewardEvent.data.updates.length).to.equal(1);
        expect(rewardEvent.data.updates[0].updatedUserRewardPools.length).to.equal(2);

        const depositEvent = events.next().value as anchor.Event;
        expect(depositEvent.data.walletProvider).to.equal(payload.walletProvider);
        expect(depositEvent.data.contributionAccrualRate.toPrecision(2)).to.equal(payload.contributionAccrualRate.toString());
        expect(depositEvent.data.userFundAccount.receiptTokenAmount.toString()).to.be.equal(
            userReceiptTokenAccount.amount.toString()
        )
        console.log(`Wallet provider: ${depositEvent.data.walletProvider}`);
        console.log(`contribution accrual rate: ${depositEvent.data.contributionAccrualRate}`);
        console.log(`receipt token balance:`, depositEvent.data.userFundAccount.receiptTokenAmount.toNumber());
    });
});
