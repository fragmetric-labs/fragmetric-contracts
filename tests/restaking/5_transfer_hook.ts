import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as utils from "../utils";
import {wallet, adminKeypair, fragSOLTokenMintKeypair, stakePoolAccounts} from "./1_initialize";
import {expect} from "chai";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";

chai.use(chaiAsPromised);

describe("transfer_hook", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.Restaking as Program<Restaking>;

    const user2 = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../mocks/user2.json")));
    const user3 = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../mocks/user3.json")));
    console.log(`User2(user2.json) key: ${user2.publicKey}`);
    console.log(`User3(user3.json) key: ${user3.publicKey}`);

    let user2ReceiptTokenAccount: anchor.web3.PublicKey;
    let user3ReceiptTokenAccount: anchor.web3.PublicKey;
    let extraAccountMetaList: anchor.web3.PublicKey;

    // Localnet only
    before("Sol airdrop to user", async () => {
        if (utils.isLocalnet(program.provider.connection)) {
            await utils.requestAirdrop(program.provider, wallet.payer, 10);
            await utils.requestAirdrop(program.provider, user2, 10);
            await utils.requestAirdrop(program.provider, user3, 10);

            // check the balance
            const adminBal = await program.provider.connection.getBalance(wallet.publicKey);
            console.log(`Admin SOL balance: ${adminBal}`);
            const user2Bal = await program.provider.connection.getBalance(user2.publicKey);
            console.log(`User2 SOL balance: ${user2Bal}`);
            const user3Bal = await program.provider.connection.getBalance(user3.publicKey);
            console.log(`User3 SOL balance: ${user3Bal}`);
            console.log("======= Sol airdrop to user =======");
        }
    });

    before("Prepare program accounts", async () => {
        user2ReceiptTokenAccount = spl.getAssociatedTokenAddressSync(
            fragSOLTokenMintKeypair.publicKey,
            user2.publicKey,
            false,
            TOKEN_2022_PROGRAM_ID,
        );
        user3ReceiptTokenAccount = spl.getAssociatedTokenAddressSync(
            fragSOLTokenMintKeypair.publicKey,
            user3.publicKey,
            false,
            TOKEN_2022_PROGRAM_ID,
        );
        [extraAccountMetaList, ] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("extra-account-metas"), fragSOLTokenMintKeypair.publicKey.toBuffer()],
            program.programId,
        );
        console.log(`user2 receipt token account    = ${user2ReceiptTokenAccount}`);
        console.log(`user3 receipt token account    = ${user3ReceiptTokenAccount}`);
        console.log(`extraAccountMetaList address   = ${extraAccountMetaList}`);
        console.log("======= Prepare program accounts =======");
    })

    before("Deposit SOL to mint receipt token", async () => {
        let amount = new BN(1_000_000_000);
        
        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                await program.methods
                    .userUpdateAccountsIfNeeded()
                    .accounts({
                        user: user2.publicKey,
                    })
                    .instruction(),
                await program.methods
                    .userDepositSol(amount, null)
                    .accounts({
                        user: user2.publicKey,
                    })
                    .remainingAccounts(stakePoolAccounts)
                    .instruction(),
            ),
            [user2],
            { commitment: "confirmed" },
        );

        await spl.createAccount(
            program.provider.connection,
            wallet.payer,
            fragSOLTokenMintKeypair.publicKey,
            user3.publicKey,
            null,
            null,
            TOKEN_2022_PROGRAM_ID,
        );

        const user2ReceiptTokenBalance = (await spl.getAccount(
            program.provider.connection,
            user2ReceiptTokenAccount,
            undefined,
            TOKEN_2022_PROGRAM_ID,
        )).amount;

        const user3ReceiptTokenBalance = (await spl.getAccount(
            program.provider.connection,
            user3ReceiptTokenAccount,
            undefined,
            TOKEN_2022_PROGRAM_ID,
        )).amount;

        console.log(`user2 receipt token balance: ${user2ReceiptTokenBalance}`);
        console.log(`user3 receipt token balance: ${user3ReceiptTokenBalance}`);
        console.log("======= Deposit SOL to mint receipt token =======");
    });

    // it.skip("Create ExtraAccountMetaList Account", async () => {
    //     const tx = new anchor.web3.Transaction().add(
    //         await program.methods
    //             .tokenInitializeExtraAccountMetaList()
    //             .accounts({
    //                 payer: mintOwner.publicKey,
    //             })
    //             .instruction()
    //     );
    //     await anchor.web3.sendAndConfirmTransaction(
    //         program.provider.connection,
    //         tx,
    //         [mintOwner],
    //     );
    // });
    //
    // it.skip("Update ExtraAccountMetaList account", async () => {
    //     const tx = new anchor.web3.Transaction().add(
    //         await program.methods
    //             .tokenUpdateExtraAccountMetaList()
    //             .accounts({
    //                 payer: mintOwner.publicKey,
    //             })
    //             .instruction()
    //     );
    //     await anchor.web3.sendAndConfirmTransaction(
    //         program.provider.connection,
    //         tx,
    //         [mintOwner],
    //     );
    // });

    it("Transfer Hook with Extra Account Meta", async () => {
        const amountToTransfer = 1_000_000_000;
        const decimals = 9;
    
        const transferHookIx = await spl.createTransferCheckedWithTransferHookInstruction(
            program.provider.connection,
            user2ReceiptTokenAccount,
            fragSOLTokenMintKeypair.publicKey,
            user3ReceiptTokenAccount,
            user2.publicKey,
            BigInt(amountToTransfer.toString()),
            decimals,
            [],
            undefined,
            spl.TOKEN_2022_PROGRAM_ID,
        );
        const tx = new anchor.web3.Transaction().add(transferHookIx);
        expect(anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            tx,
            [user2],
        )).to.eventually.throws('TokenNotTransferableError');

        // console.log(`transfer hook tx sig: ${txSig}`);
    });
});
