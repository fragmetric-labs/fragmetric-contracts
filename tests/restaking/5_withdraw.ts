import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as utils from "../utils/utils";

export const withdraw = describe("withdraw", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.Restaking as Program<Restaking>;

    const user = anchor.web3.Keypair.generate();
    const admin = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user1.json")));
    const decimals = 9;

    const receiptTokenMint = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("./fragsolMint.json")));

    const [fund_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("fund"), receiptTokenMint.publicKey.toBuffer()],
        program.programId
    );

    let userReceiptTokenAccount = spl.getAssociatedTokenAddressSync(
        receiptTokenMint.publicKey,
        user.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
    );

    before("Sol airdrop", async () => {
        await utils.requestAirdrop(provider, user, 10);
    })

    before("Create token accounts", async () => {
        const tx = new anchor.web3.Transaction().add(
            spl.createAssociatedTokenAccountInstruction(
                admin.publicKey,
                userReceiptTokenAccount,
                user.publicKey,
                receiptTokenMint.publicKey,
                TOKEN_2022_PROGRAM_ID,
            ),
        );
        const txSig = await anchor.web3.sendAndConfirmTransaction(
            provider.connection,
            tx,
            [admin],
        );
    })

    before("Mint tokens to user1", async () => {
        const amount = 10 * 10 ** decimals;

        const txSig = await program.methods
            .tokenMintReceiptTokenForTest(new anchor.BN(amount))
            .accounts({
                payer: admin.publicKey,
                receiptTokenAccountOwner: user.publicKey,
            })
            .signers([admin])
            .rpc();
    })

    before("Create ExtraAccountMetaList Account", async () => {
        await program.methods
            .tokenInitializeExtraAccountMetaList()
            .accounts({
                payer: admin.publicKey,
            })
            .signers([admin])
            .rpc();
    })

    it("Request withdrawal", async () => {
        const amount = 1 * 10 ** decimals;

        console.log("User receipt token account:", userReceiptTokenAccount);
        console.log("User:", user.publicKey);

        await program.methods
            .fundRequestWithdrawal({
                v1: {
                    0: {
                        receiptTokenAmount: new anchor.BN(amount),
                    }
                }
            })
            .accounts({
                user: user.publicKey,
            })
            .signers([user]).rpc();
        
        const pendingBatch = (await program.account.fund.fetch(fund_pda)).data.v2[0].pendingWithdrawals;
        expect(pendingBatch.numWithdrawalRequests).to.equal(new anchor.BN(1));
    })
})