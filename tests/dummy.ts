import * as anchor from "@coral-xyz/anchor";
import { Program, web3 } from "@coral-xyz/anchor";
import * as chai from 'chai';
import chaiAsPromised from "chai-as-promised";
import { expect } from "chai";
import { Dummy } from "../target/types/dummy";

chai.use(chaiAsPromised);

describe("dummy", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Dummy as Program<Dummy>;

  const isLocal = provider.connection.rpcEndpoint.includes("127.0.0.1");
  const user1 = isLocal ? anchor.web3.Keypair.generate() : anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("./user1.json")));
  const user2 = isLocal ? anchor.web3.Keypair.generate() : anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("./user2.json")));

  const [userData1, userDataBump1] = anchor.web3.PublicKey.findProgramAddressSync([
        Buffer.from("user_token_amount"),
        user1.publicKey.toBuffer(),
      ],
      program.programId,
    );
  const [userData2, userDataBump2] = anchor.web3.PublicKey.findProgramAddressSync([
        Buffer.from("user_token_amount"),
        user2.publicKey.toBuffer(),
      ],
      program.programId,
    );

  console.log({
    isLocal,
    program: program.programId.toBase58(),
    user1: user1.publicKey.toBase58(),
    userData1: userData1.toBase58(),
    userDataBump1,
    user2: user2.publicKey.toBase58(),
    userData2: userData2.toBase58(),
    userDataBump2,
  })

  it("Is initialized!", async () => {
    if (isLocal) {
      // airdrop some SOL to new user
      const airdropSignature = await provider.connection.requestAirdrop(
        user1.publicKey,
        5 * web3.LAMPORTS_PER_SOL // 5 SOL
      );

      // confirm the transaction
      await provider.connection.confirmTransaction(airdropSignature);
    }

    // check the balance
    const user1Bal = await provider.connection.getBalance(user1.publicKey);
    console.log(`User1 SOL balance: ${user1Bal}`);

    const tx = await program.methods
      .initialize()
      .accounts({ user: user1.publicKey })
      .signers([user1])
      .rpc();
    console.log("Initialize transaction signature", tx);

    const account1 = await program.account.userTokenAmount.fetch(userData1);
    expect(account1.amount.toNumber()).to.equal(0);
  });

  it("Incremented the token amount", async () => {
    const data = {
      token: "LST1",
      amount: new anchor.BN(100)
    };

    const tx = await program.methods
      .increment(data)
      .accounts({ userTokenAmount: userData1, user: user1.publicKey })
      .signers([user1])
      .rpc();
    console.log("Increment transaction:", tx);

    const account1 = await program.account.userTokenAmount.fetch(userData1);
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
      .accounts({ userTokenAmount: userData1, user: user1.publicKey })
      .signers([user1])
      .rpc();
    console.log("Decrement transaction:", tx);

    const account1 = await program.account.userTokenAmount.fetch(userData1);
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
      .accounts({ userTokenAmount: userData1, user: user1.publicKey })
      .signers([user1])
      .rpc();
    console.log("Increment transaction:", tx);

    const account1 = await program.account.userTokenAmount.fetch(userData1);
    console.log("Updated amount:", account1.amount.toNumber());
    expect(account1.amount.toNumber()).to.equal(120);
  });

  it("Initialize user2", async () => {
    if (isLocal) {
      // airdrop some SOL to new user
      const airdropSignature = await provider.connection.requestAirdrop(
          user2.publicKey,
          4 * web3.LAMPORTS_PER_SOL // 4 SOL
      );

      // confirm the transaction
      await provider.connection.confirmTransaction(airdropSignature);
    }

    // check the balance
    const user2Bal = await provider.connection.getBalance(user2.publicKey);
    console.log(`User2 SOL balance: ${user2Bal}`);

    const tx = await program.methods
      .initialize()
      .accounts({ user: user2.publicKey })
      .signers([user2])
      .rpc();
    console.log("Initialize transaction signature", tx);

    const account2 = await program.account.userTokenAmount.fetch(userData2);
    expect(account2.amount.toNumber()).to.equal(0);
  });

  it("Incremented user2's token amount", async () => {
    const data = {
      token: "LST1",
      amount: new anchor.BN(50)
    };

    const tx = await program.methods
      .increment(data)
      .accounts({ userTokenAmount: userData2, user: user2.publicKey })
      .signers([user2])
      .rpc();
    console.log("Increment transaction:", tx);

    const account2 = await program.account.userTokenAmount.fetch(userData2);
    console.log("Updated amount:", account2.amount.toNumber());
    expect(account2.amount.toNumber()).to.equal(50);

    const account1 = await program.account.userTokenAmount.fetch(userData1);
    console.log("Check user1's amount is not changed:", account1.amount.toNumber());
    expect(account1.amount.toNumber()).to.equal(120);
  });

  it("can send versioned data", async () => {
    const data = {
      v1: {
        field1: 1234,
        field2: "hello",
      },
    };
    expect(
      program.methods
        .versionedMethod(data)
        .accounts({ userTokenAmount: userData2, user: user2.publicKey })
        .signers([user2])
        .rpc()
    ).to.eventually.throw('NotImplemented');
  });
});
