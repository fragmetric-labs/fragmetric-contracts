import * as anchor from "@coral-xyz/anchor";
import { Program, web3 } from "@coral-xyz/anchor";
import { expect } from "chai";
import { DepositProgram } from "../target/types/deposit_program";

describe("deposit-program", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DepositProgram as Program<DepositProgram>;

  const user1 = anchor.web3.Keypair.generate();

  it("Is initialized!", async () => {
    // airdrop some SOL to new user
    const airdropSignature = await provider.connection.requestAirdrop(
      user1.publicKey,
      5 * web3.LAMPORTS_PER_SOL // 5 SOL
    );

    // confirm the transaction
    await provider.connection.confirmTransaction(airdropSignature);

    // check the balance
    const user1Bal = await provider.connection.getBalance(user1.publicKey);
    console.log(`User1 SOL balance: ${user1Bal}`);

    const tx = await program.methods
      .initialize()
      .accounts({ userTokenAmount: user1.publicKey })
      .signers([user1])
      .rpc();
    console.log("Initialize transaction signature", tx);

    const account1 = await program.account.userTokenAmount.fetch(user1.publicKey);
    expect(account1.amount.toNumber()).to.equal(0);
  });

  it("Incremented the token amount", async () => {
    const data = {
      token: "LST1",
      amount: new anchor.BN(100)
    };

    const tx = await program.methods
      .increment(data)
      .accounts({ userTokenAmount: user1.publicKey, user: user1.publicKey })
      .signers([user1])
      .rpc();
    console.log("Increment transaction:", tx);

    const account1 = await program.account.userTokenAmount.fetch(user1.publicKey);
    console.log("Updated amount:", account1.amount.toNumber());
    expect(account1.amount.toNumber()).to.equal(100);
  });

  it("Decremented the token amount", async () => {
    const data = {
      token: "LST1",
      amount: new anchor.BN(30),
    };

    const tx = await program.methods
      .decrement(data)
      .accounts({ userTokenAmount: user1.publicKey, user: user1.publicKey })
      .signers([user1])
      .rpc();
    console.log("Decrement transaction:", tx);

    const account1 = await program.account.userTokenAmount.fetch(user1.publicKey);
    console.log("Updated amount:", account1.amount.toNumber());
    expect(account1.amount.toNumber()).to.equal(70);
  });

  it("Incremented again", async () => {
    const data = {
      token: "LST1",
      amount: new anchor.BN(50)
    };

    const tx = await program.methods
      .increment(data)
      .accounts({ userTokenAmount: user1.publicKey, user: user1.publicKey })
      .signers([user1])
      .rpc();
    console.log("Increment transaction:", tx);

    const account1 = await program.account.userTokenAmount.fetch(user1.publicKey);
    console.log("Updated amount:", account1.amount.toNumber());
    expect(account1.amount.toNumber()).to.equal(120);
  });
});
