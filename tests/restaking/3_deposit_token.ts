import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import * as chai from 'chai';
import chaiAsPromised from "chai-as-promised";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as fs from "fs";

chai.use(chaiAsPromised);

describe("deposit_token", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.Restaking as Program<Restaking>;

    const admin = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user1.json")));
    const depositor = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user2.json")));
    console.log(`depositor key: ${depositor.publicKey}`);

    const receipt_token_name = "fragSOL";
    const [receipt_token_mint_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from(receipt_token_name)],
        program.programId
    );
    const [fund_pda, ] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("fund"), receipt_token_mint_pda.toBuffer()],
        program.programId
    );

    let tokenMint1;
    let tokenMint2;

    let depositorMint1ATA;

    let amount = new anchor.BN(1_000_000); // 0.001

    before("Sol airdrop", async () => {
        // airdrop depositor
        let airdropSignature = await provider.connection.requestAirdrop(
            depositor.publicKey,
            1 * anchor.web3.LAMPORTS_PER_SOL // 1 SOL
        );
        await provider.connection.confirmTransaction(airdropSignature);
        const depositorBal = await provider.connection.getBalance(depositor.publicKey);
        console.log(`depositor SOL balance: ${depositorBal}`);
        console.log(`depositor key: ${depositor.publicKey}`);

        tokenMint1 = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/tokenMint1", {encoding: "utf8"}).replace(/"/g, ''));
        tokenMint2 = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/tokenMint2", {encoding: "utf8"}).replace(/"/g, ''));
        console.log(`tokenMint1: ${tokenMint1}, tokenMint2: ${tokenMint2}`);

        // create depositor's token account
        depositorMint1ATA = await spl.getOrCreateAssociatedTokenAccount(
            provider.connection,
            depositor,
            tokenMint1,
            depositor.publicKey,
            false,
            undefined,
            undefined,
            TOKEN_2022_PROGRAM_ID,
        );
        console.log(`depositor's mint1 ATA:`, depositorMint1ATA);

        // mint some tokens to depositor
        await spl.mintToChecked(
            provider.connection,
            admin, // payer를 depositor로 설정하면 missing signature 에러남
            tokenMint1,
            depositorMint1ATA.address,
            admin.publicKey,
            amount.toNumber(),
            9,
            undefined,
            undefined,
            TOKEN_2022_PROGRAM_ID,
        );

        const depositorToken1Bal = await getTokenBalance(provider.connection, depositorMint1ATA.address);
        console.log(`depositor token1 balance:`, depositorToken1Bal);
    });

    it("Deposit Token!", async () => {
        try {
            const tx = await program.methods
                .depositToken(
                    amount,
                )
                .accounts({
                    depositor: depositor.publicKey,
                    tokenMint: tokenMint1,
                    depositorTokenAccount: depositorMint1ATA.address,
                    receiptTokenMint: receipt_token_mint_pda,
                    tokenProgram: TOKEN_2022_PROGRAM_ID,
                })
                .signers([depositor])
                .rpc();
            console.log(`deposit token tx: ${tx}`);

            // check if token's amount_in increased correctly
            const tokensFromFund = (await program.account.fund.fetch(fund_pda)).tokens;
            console.log("tokensFromFund:", tokensFromFund);

            expect(tokensFromFund[0].tokenAmountIn.toNumber()).to.eq(amount.toNumber());
        } catch (err) {
            console.log("Deposit Token err:");
            throw new Error(err);
        }
    });

    it("Fail when exceeding token cap!", async () => {
        const tokenCap1 = new anchor.BN(1_000_000_000 * 1000);
        amount = tokenCap1.sub(amount).add(new anchor.BN(1_000)); // exceeding amount

        // first mint token to depositor
        await spl.mintToChecked(
            provider.connection,
            admin,
            tokenMint1,
            depositorMint1ATA.address,
            admin.publicKey,
            amount.toNumber(),
            9,
            undefined,
            undefined,
            TOKEN_2022_PROGRAM_ID,
        );
        const depositorToken1Bal = await getTokenBalance(provider.connection, depositorMint1ATA.address);
        console.log(`depositor token1 balance:`, depositorToken1Bal);

        expect(
            program.methods
                .depositToken(
                    amount,
                )
                .accounts({
                    depositor: depositor.publicKey,
                    tokenMint: tokenMint1,
                    depositorTokenAccount: depositorMint1ATA.address,
                    receiptTokenMint: receipt_token_mint_pda,
                    tokenProgram: TOKEN_2022_PROGRAM_ID,
                })
                .signers([depositor])
                .rpc()
          ).to.eventually.throw('ExceedsTokenCap');

        // check if token's amount_in increased correctly
        const tokensFromFund = (await program.account.fund.fetch(fund_pda)).tokens;
        console.log("tokensFromFund:", tokensFromFund);

        expect(tokensFromFund[0].tokenAmountIn.toNumber()).to.eq(new anchor.BN(1_000_000).toNumber());
    });
});

const getTokenBalance = async (connection, tokenAccount) => {
    const info = await connection.getTokenAccountBalance(tokenAccount);
    if (info.value.uiAmount == null) throw new Error("No balance found");
    console.log("Balance:", info.value.uiAmount);
    return info.value.uiAmount;
}
