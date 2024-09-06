import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Restaking } from "../../target/types/restaking";
import * as fs from "fs";
import { Buffer } from "buffer";
import * as bs58 from "bs58";

const requestAirdrop = async (provider: anchor.Provider, user: anchor.web3.Keypair, amount: number) => {
    let airdropSignature = await provider.connection.requestAirdrop(
       user.publicKey,
       amount * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSignature);
    const userBal = await provider.connection.getBalance(user.publicKey);
    console.log(`user ${user.publicKey} SOL balance: ${userBal}`);
}

const skipSlots = async (program: anchor.Program<Restaking>, signer: anchor.web3.Keypair, skip: number) => {
    let currentSlot = await program.provider.connection.getSlot();
    console.log(`BEFORE skip slots, current slot: ${currentSlot}`);

    for (let i = 0; i < skip; i++) {
        await program.methods
            .emptyIx()
            .accounts({})
            .signers([signer])
            .rpc();
    }

    currentSlot = await program.provider.connection.getSlot();
    console.log(`AFTER skip slots, current slot: ${currentSlot}`);
}

function isLocalnet(conn: anchor.web3.Connection): boolean {
    return conn.rpcEndpoint == "http://0.0.0.0:8899";
}

function isDevnet(conn: anchor.web3.Connection): boolean {
    return conn.rpcEndpoint == anchor.web3.clusterApiUrl("devnet");
}

function isMainnetBeta(conn: anchor.web3.Connection): boolean {
    return conn.rpcEndpoint == anchor.web3.clusterApiUrl("mainnet-beta");
}

export {
    requestAirdrop,
    skipSlots,
    isLocalnet,
    isDevnet,
    isMainnetBeta as isMainnet,
};
