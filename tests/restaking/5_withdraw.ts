import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as utils from "../utils/utils";
import * as restaking from "./1_fund_initialize";

chai.use(chaiAsPromised);

export const withdraw = describe("withdraw", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.Restaking as Program<Restaking>;

  const admin = (program.provider as anchor.AnchorProvider).wallet as anchor.Wallet;
  const payer = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user1.json")));
  const user = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../user2.json")));
  console.log(`Payer(user1.json) key: ${payer.publicKey}`);
  console.log(`User(user2.json) key: ${user.publicKey}`);

  let userReceiptTokenAccount: anchor.web3.PublicKey;

  const amount = 1_000_000_000;

  // Localnet only
  before("Sol airdrop to user", async function () {
    if (utils.isLocalnet(program.provider.connection)) {
      await utils.requestAirdrop(program.provider, user, 10);

      // check the balance
      const adminBal = await program.provider.connection.getBalance(admin.publicKey);
      console.log(`Admin SOL balance: ${adminBal}`);
      const payerBal = await program.provider.connection.getBalance(payer.publicKey);
      console.log(`Payer SOL balance: ${payerBal}`);
      const userBal = await program.provider.connection.getBalance(user.publicKey);
      console.log(`User SOL balance: ${userBal}`);
      console.log("======= Sol airdrop to user =======");
    }
  });

  before("Prepare program accounts", async () => {
    userReceiptTokenAccount = spl.getAssociatedTokenAddressSync(
      restaking.receiptTokenMint.publicKey,
      user.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );
    console.log(`user receipt token account = ${userReceiptTokenAccount}`);
    console.log("======= Prepare program accounts =======");
  });

  before("Deposit SOL to mint receipt token", async () => {
    let amount = new anchor.BN(1_000_000_000 * 5);
    await program.methods
      .fundDepositSol(amount, null)
      .accounts({
        user: user.publicKey,
        // instructionSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        pricingSource0: restaking.bSOLStakePoolPublicKey,
        pricingSource1: restaking.mSOLStakePoolPublicKey,
        pricingSource2: restaking.jitoSOLStakePoolPublicKey,
      })
      .signers([user])
      .rpc({ commitment: "confirmed" });

    const userReceiptTokenBalance = (await spl.getAccount(
      program.provider.connection,
      userReceiptTokenAccount,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    )).amount;
    console.log(`user receipt token balance: ${userReceiptTokenBalance}`);
    console.log("======= Deposit SOL to mint receipt token =======");
  })

  it("Request withdrawal", async () => {
    const balanceBefore = (
      await spl.getAccount(
        program.provider.connection,
        userReceiptTokenAccount,
        undefined,
        TOKEN_2022_PROGRAM_ID
      )
    ).amount;

    for (let i = 0; i < 3; i++) {
      await program.methods
        .fundRequestWithdrawal(new anchor.BN(amount))
        .accounts({
          user: user.publicKey,
        })
        .signers([user])
        .rpc();
    }

    const pendingBatchWithdrawal = (await program.account.fund.fetch(restaking.fund_pda))
      .withdrawalStatus.pendingBatchWithdrawal;
    expect(pendingBatchWithdrawal.numWithdrawalRequests.toNumber()).to.equal(3);

    const balanceAfter = (
      await spl.getAccount(
        program.provider.connection,
        userReceiptTokenAccount,
        undefined,
        TOKEN_2022_PROGRAM_ID
      )
    ).amount;
    expect(balanceBefore - balanceAfter).to.equal(BigInt(3 * amount));
  });

  it("Cancel withdrawal request", async () => {
    const balanceBefore = (
      await spl.getAccount(
        program.provider.connection,
        userReceiptTokenAccount,
        undefined,
        TOKEN_2022_PROGRAM_ID
      )
    ).amount;

    await program.methods
      .fundCancelWithdrawalRequest(new anchor.BN(2))
      .accounts({
        user: user.publicKey,
      })
      .signers([user])
      .rpc();

    const pendingBatchWithdrawal = (await program.account.fund.fetch(restaking.fund_pda))
      .withdrawalStatus.pendingBatchWithdrawal;
    expect(pendingBatchWithdrawal.numWithdrawalRequests.toNumber()).to.equal(2);

    const balanceAfter = (
      await spl.getAccount(
        program.provider.connection,
        userReceiptTokenAccount,
        undefined,
        TOKEN_2022_PROGRAM_ID
      )
    ).amount;
    expect(balanceAfter - balanceBefore).to.equal(BigInt(amount));

    expect(
      program.methods
        .fundCancelWithdrawalRequest(new anchor.BN(2))
        .accounts({
          user: user.publicKey,
        })
        .signers([user])
        .rpc()
    ).to.eventually.throw("FundWithdrawalRequestNotFound");
  });

  it("Process all withdrawals", async () => {
    await program.methods.operatorRun().accounts({
        pricingSource0: restaking.bSOLStakePoolPublicKey,
        pricingSource1: restaking.mSOLStakePoolPublicKey,
        pricingSource2: restaking.jitoSOLStakePoolPublicKey,
    }).signers([]).rpc();

    const fund = await program.account.fund.fetch(restaking.fund_pda)
    const withdrawalStatus = fund.withdrawalStatus;
    expect(withdrawalStatus.lastCompletedBatchId.toNumber()).to.equal(1);

    const reservedFund = withdrawalStatus.reservedFund;
    expect(reservedFund.numCompletedWithdrawalRequests.toNumber()).to.equal(2);
    expect(reservedFund.solRemaining.toNumber()).to.equal(2 * amount);
  });

  it("Withdraw sol", async () => {
    const sol_withdraw_fee_rate = 10;
    const fee = (amount * sol_withdraw_fee_rate) / 10000;
    const balanceBefore = await program.provider.connection.getBalance(
      user.publicKey
    );

    await program.methods
      .fundWithdraw(new anchor.BN(3))
      .accounts({
        user: user.publicKey,
        pricingSource0: restaking.bSOLStakePoolPublicKey,
        pricingSource1: restaking.mSOLStakePoolPublicKey,
        pricingSource2: restaking.jitoSOLStakePoolPublicKey,
      })
      .signers([user])
      .rpc();

    const balanceAfter = await program.provider.connection.getBalance(
      user.publicKey
    );
    expect(balanceAfter - balanceBefore).to.equal(amount - fee);

    const reservedFund = (await program.account.fund.fetch(restaking.fund_pda))
      .withdrawalStatus.reservedFund;
    expect(reservedFund.solRemaining.toNumber()).to.equal(amount + fee);
  });

  it("Block withdrawal", async () => {
    await program.methods
      .fundUpdateWithdrawalEnabledFlag(false)
      .accounts({ fund: restaking.fund_pda })
      .rpc();

    expect(
      program.methods
        .fundRequestWithdrawal(new anchor.BN(amount))
        .accounts({
          user: user.publicKey,
        })
        .signers([user])
        .rpc()
    ).to.eventually.throw("FundWithdrawalDisabled");

    expect(
      program.methods
        .fundWithdraw(new anchor.BN(1))
        .accounts({
          user: user.publicKey,
          // instructionSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          pricingSource0: restaking.bSOLStakePoolPublicKey,
          pricingSource1: restaking.mSOLStakePoolPublicKey,
          pricingSource2: restaking.jitoSOLStakePoolPublicKey,
        })
        .signers([user])
        .rpc()
    ).to.eventually.throw("FundWithdrawalDisabled");
  });
});
