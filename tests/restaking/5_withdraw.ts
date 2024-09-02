import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { Restaking } from "../../target/types/restaking";
import { before } from "mocha";
import * as utils from "../utils";
import * as restaking from "./1_initialize";
import {fragSOLTokenLockAddress, adminKeypair, wallet, stakePoolAccounts, fundManagerKeypair} from "./1_initialize";

chai.use(chaiAsPromised);

export const withdraw = describe("withdraw", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.Restaking as Program<Restaking>;

  const user = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(require("../mocks/user2.json")));
  console.log(`User(user2.json) key: ${user.publicKey}`);

  let userFundAccount: anchor.web3.PublicKey;

  const amount = 1_000_000_000;

  // Localnet only
  before("Sol airdrop to user", async function () {
    if (utils.isLocalnet(program.provider.connection)) {
      await utils.requestAirdrop(program.provider, user, 10);

      // check the balance
      const userBal = await program.provider.connection.getBalance(user.publicKey);
      console.log(`User SOL balance: ${userBal}`);
      console.log("======= Sol airdrop to user =======");
    }
  });

  before("Prepare program accounts", async () => {
    userFundAccount = spl.getAssociatedTokenAddressSync(
      restaking.fragSOLTokenMintKeypair.publicKey,
      user.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
    );
    console.log(`user receipt token account = ${userFundAccount}`);
    console.log("======= Prepare program accounts =======");
  });

  before("Deposit SOL to mint receipt token", async () => {
    let amount = new anchor.BN(1_000_000_000 * 5);

    const depositSolTx = new anchor.web3.Transaction().add(
      await program.methods
        .userUpdateAccountsIfNeeded()
        .accounts({
          user: user.publicKey,
        })
        .instruction(),
      await program.methods
        .userDepositSol(amount, null)
        .accounts({
          user: user.publicKey,
        })
        .remainingAccounts(stakePoolAccounts)
        .instruction(),
    );
    await anchor.web3.sendAndConfirmTransaction(
      program.provider.connection,
      depositSolTx,
      [user],
      { commitment: "confirmed" },
    );

    const userReceiptTokenBalance = (await spl.getAccount(
      program.provider.connection,
      userFundAccount,
      undefined,
      TOKEN_2022_PROGRAM_ID,
    )).amount;
    console.log(`user receipt token balance: ${userReceiptTokenBalance}`);
    console.log("======= Deposit SOL to mint receipt token =======");
  })

  it("Request withdrawal", async () => {
    const lockAccountBalanceBefore = (await spl.getAccount(
        program.provider.connection,
        fragSOLTokenLockAddress,
        undefined,
        TOKEN_2022_PROGRAM_ID,
    )).amount;

    const balanceBefore = (
      await spl.getAccount(
        program.provider.connection,
        userFundAccount,
        undefined,
        TOKEN_2022_PROGRAM_ID
      )
    ).amount;

    for (let i = 0; i < 3; i++) {
      await program.methods
        .userRequestWithdrawal(new anchor.BN(amount))
        .accounts({
          user: user.publicKey,
        })
        .signers([user])
        .rpc();
    }

    const pendingBatchWithdrawal = (await program.account.fundAccount.fetch(restaking.fragSOLFundAddress))
      .withdrawalStatus.pendingBatchWithdrawal;
    expect(pendingBatchWithdrawal.numWithdrawalRequests.toNumber()).to.equal(3);

    const balanceAfter = (
      await spl.getAccount(
        program.provider.connection,
        userFundAccount,
        undefined,
        TOKEN_2022_PROGRAM_ID
      )
    ).amount;
    expect(balanceBefore - balanceAfter).to.equal(BigInt(3 * amount));

    const lockAccountBalanceAfter = (await spl.getAccount(
        program.provider.connection,
        fragSOLTokenLockAddress,
        undefined,
        TOKEN_2022_PROGRAM_ID,
    )).amount;
    expect(lockAccountBalanceAfter - lockAccountBalanceBefore).to.equal(BigInt(3 * amount));
  });

  it("Cancel withdrawal request", async () => {
    const lockAccountBalanceBefore = (await spl.getAccount(
        program.provider.connection,
        fragSOLTokenLockAddress,
        undefined,
        TOKEN_2022_PROGRAM_ID,
    )).amount;

    const balanceBefore = (
      await spl.getAccount(
        program.provider.connection,
        userFundAccount,
        undefined,
        TOKEN_2022_PROGRAM_ID
      )
    ).amount;

    await program.methods
      .userCancelWithdrawalRequest(new anchor.BN(2))
      .accounts({
        user: user.publicKey,
      })
      .signers([user])
      .rpc();

    const pendingBatchWithdrawal = (await program.account.fundAccount.fetch(restaking.fragSOLFundAddress))
      .withdrawalStatus.pendingBatchWithdrawal;
    expect(pendingBatchWithdrawal.numWithdrawalRequests.toNumber()).to.equal(2);

    const balanceAfter = (
      await spl.getAccount(
        program.provider.connection,
        userFundAccount,
        undefined,
        TOKEN_2022_PROGRAM_ID
      )
    ).amount;
    expect(balanceAfter - balanceBefore).to.equal(BigInt(amount));

    expect(
      program.methods
        .userCancelWithdrawalRequest(new anchor.BN(2))
        .accounts({
          user: user.publicKey,
        })
        .signers([user])
        .rpc()
    ).to.eventually.throw("FundWithdrawalRequestNotFound");

    const lockAccountBalanceAfter = (await spl.getAccount(
        program.provider.connection,
        fragSOLTokenLockAddress,
        undefined,
        TOKEN_2022_PROGRAM_ID,
    )).amount;
    expect(lockAccountBalanceBefore - lockAccountBalanceAfter).to.equal(BigInt(amount));
  });

  it("Process all withdrawals", async () => {
    await program.methods.operatorProcessFundWithdrawalJob(
        false,
    )
        .remainingAccounts(stakePoolAccounts)
        .accounts({}).signers([]).rpc(); // should succeed

    expect(program.methods.operatorProcessFundWithdrawalJob(
        true, // forced, just after
    )
        .remainingAccounts(stakePoolAccounts)
        .accounts({}).signers([]).rpc()).eventually.throw('OperatorJobUnmetThresholdError');

    await program.methods.operatorProcessFundWithdrawalJob(
        true, // forced by admin?
    )
        .remainingAccounts(stakePoolAccounts)
        .accounts({
          operator: adminKeypair.publicKey,
        }).signers([adminKeypair]).rpc(); // should succeed

    const fund = await program.account.fundAccount.fetch(restaking.fragSOLFundAddress)
    const withdrawalStatus = fund.withdrawalStatus;
    expect(withdrawalStatus.lastCompletedBatchId.toNumber()).to.equal(2);

    const reservedFund = withdrawalStatus.reservedFund;
    expect(reservedFund.numCompletedWithdrawalRequests.toNumber()).to.equal(2);
    expect(reservedFund.solRemaining.toNumber()).to.equal(2 * amount);

    const lockAccountBalanceAfter = (await spl.getAccount(
        program.provider.connection,
        fragSOLTokenLockAddress,
        undefined,
        TOKEN_2022_PROGRAM_ID,
    )).amount;
    expect(lockAccountBalanceAfter).to.equal(BigInt(0));
  });

  it("Withdraw sol", async () => {
    const sol_withdraw_fee_rate = 10;
    const fee = (amount * sol_withdraw_fee_rate) / 10000;
    const balanceBefore = await program.provider.connection.getBalance(
      user.publicKey
    );

    await program.methods
      .userWithdraw(new anchor.BN(3))
      .accounts({
        user: user.publicKey,
      })
      .remainingAccounts(stakePoolAccounts)
      .signers([user])
      .rpc();

    const balanceAfter = await program.provider.connection.getBalance(
      user.publicKey
    );
    expect(balanceAfter - balanceBefore).to.equal(amount - fee);

    const reservedFund = (await program.account.fundAccount.fetch(restaking.fragSOLFundAddress))
      .withdrawalStatus.reservedFund;
    expect(reservedFund.solRemaining.toNumber()).to.equal(amount + fee);
  });

  it("Block withdrawal", async () => {
    await program.methods
      .fundManagerUpdateWithdrawalEnabledFlag(false)
      .signers([fundManagerKeypair])
      .rpc();

    expect(
      program.methods
        .userRequestWithdrawal(new anchor.BN(amount))
        .accounts({
          user: user.publicKey,
        })
        .remainingAccounts(stakePoolAccounts)
        .signers([user])
        .rpc()
    ).to.eventually.throw("FundWithdrawalDisabled");

    expect(
      program.methods
        .userRequestWithdrawal(new anchor.BN(1))
        .accounts({
          user: user.publicKey,
        })
        .remainingAccounts(stakePoolAccounts)
        .signers([user])
        .rpc()
    ).to.eventually.throw("FundWithdrawalDisabled");

    await program.methods
        .fundManagerUpdateWithdrawalEnabledFlag(true)
        .signers([fundManagerKeypair])
        .rpc();

    await program.methods
        .userRequestWithdrawal(new anchor.BN(1))
        .accounts({
          user: user.publicKey,
        })
        .remainingAccounts(stakePoolAccounts)
        .signers([user])
        .rpc()
  });
});
