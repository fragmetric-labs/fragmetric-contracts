import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
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
    isLocalnet,
    isDevnet,
    isMainnetBeta as isMainnet,
};
