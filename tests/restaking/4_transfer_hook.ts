import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as utils from "../utils/utils";

export const transfer_hook = describe("transfer_hook", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.Restaking as Program<Restaking>;

    const user1 = anchor.web3.Keypair.generate();
    const user2 = anchor.web3.Keypair.generate();

    const admin = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user1.json")));
    const mintOwner = admin; // same as admin
    const decimals = 9;

    const receiptTokenMint = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("./fragsolMint.json")));
    const [extraAccountMetaList, ] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("extra-account-metas"), receiptTokenMint.publicKey.toBuffer()],
        program.programId,
    );
    console.log(`extraAccountMetaList address: ${extraAccountMetaList}`);

    let user1ReceiptTokenAccount = spl.getAssociatedTokenAddressSync(
        receiptTokenMint.publicKey,
        user1.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
    );
    let user2ReceiptTokenAccount = spl.getAssociatedTokenAddressSync(
        receiptTokenMint.publicKey,
        user2.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
    );
    console.log(`user1 receipt token account: ${user1ReceiptTokenAccount}, user2 receipt token account: ${user2ReceiptTokenAccount}`);

    before("Sol airdrop", async () => {
        await utils.requestAirdrop(provider, user1, 10);
        await utils.requestAirdrop(provider, user2, 10);
    });

    it("Create token accounts", async () => {
        const tx = new anchor.web3.Transaction().add(
            spl.createAssociatedTokenAccountInstruction(
                admin.publicKey,
                user1ReceiptTokenAccount,
                user1.publicKey,
                receiptTokenMint.publicKey,
                TOKEN_2022_PROGRAM_ID,
            ),
            spl.createAssociatedTokenAccountInstruction(
                admin.publicKey,
                user2ReceiptTokenAccount,
                user2.publicKey,
                receiptTokenMint.publicKey,
                TOKEN_2022_PROGRAM_ID,
            ),
        );
        const txSig = await anchor.web3.sendAndConfirmTransaction(
            provider.connection,
            tx,
            [admin],
        );
        console.log(`Create token accounts tx sig: ${txSig}`);
    });

    it("Mint tokens to user1 token account", async () => {
        const amount = 10 * 10 ** decimals; // 10개
    
        const txSig = await program.methods
            .tokenMintReceiptTokenForTest(new anchor.BN(amount))
            .accounts({
                payer: admin.publicKey,
                receiptTokenAccountOwner: user1.publicKey,
                receiptTokenMint: receiptTokenMint.publicKey,
            })
            .signers([admin])
            .rpc();
        console.log(`mint receipt token to user1 tx sig: ${txSig}`);
    });

    it("Create ExtraAccountMetaList Account", async () => {
        const initializeExtraAccountMetaListIx = await program.methods
            .tokenInitializeExtraAccountMetaList()
            .accounts({
                payer: mintOwner.publicKey,
                mint: receiptTokenMint.publicKey,
            })
            .instruction();
        const tx = new anchor.web3.Transaction().add(initializeExtraAccountMetaListIx);
        const txSig = await anchor.web3.sendAndConfirmTransaction(
            provider.connection,
            tx,
            [mintOwner],
        );
        console.log(`initializeExtraAccountMetaList tx sig: ${txSig}`);
    });

    it("Transfer Hook with Extra Account Meta", async () => {
        const amountToTransfer = 1 * 10 ** decimals;
    
        const transferHookIx = await spl.createTransferCheckedWithTransferHookInstruction(
            provider.connection,
            user1ReceiptTokenAccount,
            receiptTokenMint.publicKey,
            user2ReceiptTokenAccount,
            user1.publicKey,
            BigInt(amountToTransfer.toString()),
            decimals,
            [],
            undefined,
            spl.TOKEN_2022_PROGRAM_ID,
        );
        // console.log(`transferHookIx:`, transferHookIx);
        const tx = new anchor.web3.Transaction().add(transferHookIx);
        const txSig = await anchor.web3.sendAndConfirmTransaction(
            provider.connection,
            tx,
            [user1],
        );
        console.log(`transfer hook tx sig: ${txSig}`);
    });
});
