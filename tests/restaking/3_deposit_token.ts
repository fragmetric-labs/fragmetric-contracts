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
import * as utils from "../utils/utils";

chai.use(chaiAsPromised);

export const deposit_token = describe("deposit_token", () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const program = anchor.workspace.Restaking as Program<Restaking>;

    const admin = (program.provider as anchor.AnchorProvider).wallet;
    const payer = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user1.json")));
    const user = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user2.json")));
    console.log(`User key: ${user.publicKey}`);

    let receiptTokenMint: anchor.web3.Keypair;

    let tokenMint1: anchor.web3.PublicKey;
    let tokenMint2: anchor.web3.PublicKey;

    let bSOLMintPublicKey: anchor.web3.PublicKey;
    let mSOLMintPublicKey: anchor.web3.PublicKey;
    let jitoSOLMintPublicKey: anchor.web3.PublicKey;
    let infMintPublicKey: anchor.web3.PublicKey;

    let bSOLMint: spl.Mint;
    let mSOLMint: spl.Mint;
    let jitoSOLMint: spl.Mint;
    let infMint: spl.Mint;

    let fund_pda: anchor.web3.PublicKey;
    let fund_token_authority_pda: anchor.web3.PublicKey;

    let userToken1Account: spl.Account;
    let userBSOLTokenAccount: spl.Account;
    let userMSOLTokenAccount: spl.Account;
    let userJitoSOLTokenAccount: spl.Account;
    let userInfTokenAccount: spl.Account;

    let amount = new anchor.BN(1_000_000_000 * 10); // 10
    const decimals = 9;

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

        bSOLMintPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/mainnet/addresses/bSOL_mint", {encoding: "utf8"}).replace(/"/g, ''));
        mSOLMintPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/mainnet/addresses/mSOL_mint", {encoding: "utf8"}).replace(/"/g, ''));
        jitoSOLMintPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/mainnet/addresses/JitoSOL_mint", {encoding: "utf8"}).replace(/"/g, ''));
        infMintPublicKey = new anchor.web3.PublicKey(fs.readFileSync("./tests/restaking/lsts/mainnet/addresses/INF_mint", {encoding: "utf8"}).replace(/"/g, ''));
        console.log(`bSOLMint: ${bSOLMintPublicKey}, mSOLMint: ${mSOLMintPublicKey}, jitoSOLMint: ${jitoSOLMintPublicKey}, infMint: ${infMintPublicKey}`);

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

    before("Prepare mainnet mint accounts", async () => {
        bSOLMint = await spl.getMint(
            program.provider.connection,
            bSOLMintPublicKey,
        );
        mSOLMint = await spl.getMint(
            program.provider.connection,
            mSOLMintPublicKey,
        );
        jitoSOLMint = await spl.getMint(
            program.provider.connection,
            jitoSOLMintPublicKey,
        );
        infMint = await spl.getMint(
            program.provider.connection,
            infMintPublicKey,
        );
    });

    // before("Create and Mint User Token Account", async () => {
    //     // create depositor's token account
    //     userToken1Account = await spl.getOrCreateAssociatedTokenAccount(
    //         program.provider.connection,
    //         user,
    //         tokenMint1,
    //         user.publicKey,
    //         false,
    //         undefined,
    //         undefined,
    //         TOKEN_2022_PROGRAM_ID,
    //     );
    //     console.log(`User Token1 Account:`, userToken1Account.address);

    //     // mint some tokens to depositor
    //     await spl.mintToChecked(
    //         program.provider.connection,
    //         payer, // payer를 depositor로 설정하면 missing signature 에러남
    //         tokenMint1,
    //         userToken1Account.address,
    //         payer.publicKey,
    //         amount.toNumber(),
    //         9,
    //         undefined,
    //         undefined,
    //         TOKEN_2022_PROGRAM_ID,
    //     );
    //     const depositorToken1Bal = await getTokenBalance(program.provider.connection, userToken1Account.address);
    //     console.log(`User Token1 balance:`, depositorToken1Bal);
    // });

    before("Mint mainnet mint tokens to the user token account", async () => {
        // create user's bSOL token account
        userBSOLTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
            program.provider.connection,
            payer,
            bSOLMint.address,
            user.publicKey,
            false,
            undefined,
            undefined,
        );
        userMSOLTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
            program.provider.connection,
            payer,
            mSOLMint.address,
            user.publicKey,
            false,
            undefined,
            undefined,
        );
        userJitoSOLTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
            program.provider.connection,
            payer,
            jitoSOLMint.address,
            user.publicKey,
            false,
            undefined,
            undefined,
        );
        userInfTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
            program.provider.connection,
            payer,
            infMint.address,
            user.publicKey,
            false,
            undefined,
            undefined,
        );
        console.log(`user bSOL token account: ${userBSOLTokenAccount.address}, mSOL token account: ${userMSOLTokenAccount.address}, JitoSOL token account: ${userJitoSOLTokenAccount.address}, INF token account: ${userInfTokenAccount.address}`);

        // mint tokens to user
        const mintTokensTx = new anchor.web3.Transaction().add(
            spl.createMintToCheckedInstruction(
                bSOLMint.address,
                userBSOLTokenAccount.address,
                payer.publicKey,
                amount.toNumber(),
                decimals,
                [],
            ),
            spl.createMintToCheckedInstruction(
                mSOLMint.address,
                userMSOLTokenAccount.address,
                payer.publicKey,
                amount.toNumber(),
                decimals,
                [],
            ),
            spl.createMintToCheckedInstruction(
                jitoSOLMint.address,
                userJitoSOLTokenAccount.address,
                payer.publicKey,
                amount.toNumber(),
                decimals,
                [],
            ),
            spl.createMintToCheckedInstruction(
                infMint.address,
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
    });

    it.skip("Deposit tokenMint1", async () => {
        try {
            const tx = await program.methods
                .fundDepositToken({
                    v1: {
                        0: {
                            amount: amount,
                        }
                    }
                })
                .accounts({
                    user: user.publicKey,
                    tokenMint: tokenMint1,
                    userTokenAccount: userToken1Account.address,
                })
                .signers([user])
                .rpc();
            console.log(`Deposit token tx: ${tx}`);

            // check if token's amount_in increased correctly
            const tokensFromFund = (await program.account.fund.fetch(fund_pda)).data.v2[0].whitelistedTokens[0];
            console.log("Tokens from fund:", tokensFromFund.tokenAmountIn);

            expect(tokensFromFund.tokenAmountIn.toNumber()).to.eq(amount.toNumber());
        } catch (err) {
            console.log("Deposit Token err:");
            throw new Error(err);
        }
    });

    it("Deposit bSOL, mSOL, JitoSOL, INF", async () => {
        const txs = new anchor.web3.Transaction().add(
            await program.methods
                .fundDepositToken({
                    v1: {
                        0: {
                            amount: amount,
                        }
                    }
                })
                .accounts({
                    user: user.publicKey,
                    tokenMint: bSOLMint.address,
                    userTokenAccount: userBSOLTokenAccount.address,
                    depositTokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([user])
                .instruction(),
            await program.methods
                .fundDepositToken({
                    v1: {
                        0: {
                            amount: amount,
                        }
                    }
                })
                .accounts({
                    user: user.publicKey,
                    tokenMint: mSOLMint.address,
                    userTokenAccount: userMSOLTokenAccount.address,
                    depositTokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([user])
                .instruction(),
            await program.methods
                .fundDepositToken({
                    v1: {
                        0: {
                            amount: amount,
                        }
                    }
                })
                .accounts({
                    user: user.publicKey,
                    tokenMint: jitoSOLMint.address,
                    userTokenAccount: userJitoSOLTokenAccount.address,
                    depositTokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([user])
                .instruction(),
            await program.methods
                .fundDepositToken({
                    v1: {
                        0: {
                            amount: amount,
                        }
                    }
                })
                .accounts({
                    user: user.publicKey,
                    tokenMint: infMint.address,
                    userTokenAccount: userInfTokenAccount.address,
                    depositTokenProgram: spl.TOKEN_PROGRAM_ID,
                })
                .signers([user])
                .instruction(),
        );
        const txSig = await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            txs,
            [user],
        );
        console.log(`user deposited tokens tx sig: ${txSig}`);
    });

    it.skip("Fail when exceeding token cap!", async () => {
        const tokenCap1 = new anchor.BN(1_000_000_000 * 1000);
        amount = tokenCap1.sub(amount).add(new anchor.BN(1_000)); // exceeding amount

        // first mint token to depositor
        await spl.mintToChecked(
            program.provider.connection,
            payer,
            tokenMint1,
            userToken1Account.address,
            payer.publicKey,
            amount.toNumber(),
            9,
            undefined,
            undefined,
            TOKEN_2022_PROGRAM_ID,
        );
        const userToken1Bal = await getTokenBalance(program.provider.connection, userToken1Account.address);
        console.log(`user token1 balance:`, userToken1Bal);

        expect(
            program.methods
                .fundDepositToken({
                    v1: {
                        0: {
                            amount: amount,
                        }
                    }
                })
                .accounts({
                    user: user.publicKey,
                    tokenMint: tokenMint1,
                    userTokenAccount: userToken1Account.address,
                })
                .signers([user])
                .rpc()
          ).to.eventually.throw('ExceedsTokenCap');

        // check if token's amount_in increased correctly
        const tokensFromFund = (await program.account.fund.fetch(fund_pda)).data.v2[0].whitelistedTokens;
        console.log("tokensFromFund:", tokensFromFund);

        expect(tokensFromFund[0].tokenAmountIn.toNumber()).to.eq(new anchor.BN(1_000_000).toNumber());
    });

    it("Fail when exceeding token cap!", async () => {
        const tokenCap1 = new anchor.BN(1_000_000_000 * 1000);
        amount = tokenCap1.sub(amount).add(new anchor.BN(990)); // exceeding amount

        // first mint token to depositor
        await spl.mintToChecked(
            program.provider.connection,
            payer,
            bSOLMint.address,
            userBSOLTokenAccount.address,
            payer.publicKey,
            amount.toNumber(),
            9,
            undefined,
            undefined,
        );
        const userBSOLBal = await getTokenBalance(program.provider.connection, userBSOLTokenAccount.address);
        console.log(`user bSOL balance:`, userBSOLBal);

        expect(
            program.methods
                .fundDepositToken({
                    v1: {
                        0: {
                            amount: amount,
                        }
                    }
                })
                .accounts({
                    user: user.publicKey,
                    tokenMint: bSOLMint.address,
                    userTokenAccount: userBSOLTokenAccount.address,
                })
                .signers([user])
                .rpc()
          ).to.eventually.throw('ExceedsTokenCap');

        // check if token's amount_in increased correctly
        const tokensFromFund = (await program.account.fund.fetch(fund_pda)).data.v2[0].whitelistedTokens;
        console.log("tokensFromFund:", tokensFromFund);

        expect(tokensFromFund[0].tokenAmountIn.toNumber()).to.eq(new anchor.BN(1_000_000_000 * 10).toNumber());
    });
});

const getTokenBalance = async (connection, tokenAccount) => {
    const info = await connection.getTokenAccountBalance(tokenAccount);
    if (info.value.uiAmount == null) throw new Error("No balance found");
    return info.value.uiAmount;
}
