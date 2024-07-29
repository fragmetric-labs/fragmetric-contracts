import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";

const requestAirdrop = async (provider: anchor.Provider, user: anchor.web3.Keypair, amount: number) => {
    let airdropSignature = await provider.connection.requestAirdrop(
       user.publicKey,
       amount * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSignature);
    const userBal = await provider.connection.getBalance(user.publicKey);
    console.log(`user ${user.publicKey} SOL balance: ${userBal}`);
}

export {
    requestAirdrop
};
