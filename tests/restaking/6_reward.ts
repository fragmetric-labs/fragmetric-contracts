import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import * as chai from "chai";
import chaiAsPromised from "chai-as-promised";
import { Restaking } from "../../target/types/restaking";
import { before, Context } from "mocha";
import * as utils from "../utils";
import * as restaking from "./1_initialize";
import { Restaking } from "../../target/types/restaking";
import { RestakingPlayground } from "../../tools/restaking/playground";

chai.use(chaiAsPromised);

const TOKEN_DENOMINATOR = 10**9;
const CONTRIBUTION_DENOMINATOR = 10**(9+2);

type RewardSettlement = {
    rewardId: number,
    rewardPoolId: number,
    remainingAmount: number,
    claimedAmount: number,
    claimedAmountUpdatedSlot: number,
    settledAmount: number,
    settlementBlocksLastRewardPoolContribution: number,
    settlementBlocksLastSlot: number,
    settlementBlocks: SettlementBlock[],
};
type SettlementBlock = {
    amount: number,
    startingRewardPoolContribution: number,
    startingSlot: number,
    endingRewardPoolContribution: number,
    endingSlot: number,
    userSettledAmount: number,
    userSettledContribution: number,
};
type UserRewardSettlement = {
    rewardId: number,
    settledAmount: number,
    settledContribution: number,
    settledSlot: number,
    claimedAmount: number,
};
enum RewardType {
    Point,
    Token,
    SOL,
};

export const reward = describe("Reward", async function () {
    const playground = await RestakingPlayground.local(anchor.AnchorProvider.env());

    it("May airdrop SOL to mock accounts", async () => {
        await playground.tryAirdropToMockAccounts();
    });

    it("user1 deposited to mint 100 fragSOL", async function () {
        const amount = new anchor.BN(1_000_000_000 * 100);
        const { fragSOLReward } = await playground.runUserDepositSOL(, amount, null);

        await checkRewardAccount(playground, playground.knownAddress.fragSOLReward);
        await checkUserRewardAccount(playground, "user1", user1RewardAddress);
    });

    let skipCounts = [9, 9, 19];
    for (let i = 0; i < 3; i++) {
        it(`[after ${skipCounts[i]+1} slots] just update after delay`, async function () {
            await playground.skipSlots(playground.wallet, skipCounts[i]);
        
            await anchor.web3.sendAndConfirmTransaction(
                program.provider.connection,
                new anchor.web3.Transaction().add(
                    ...await Promise.all([
                        program.methods
                            .adminUpdateRewardPools()
                            .accounts({
                                payer: restaking.adminKeypair.publicKey,
                            })
                            .signers([restaking.adminKeypair])
                            .instruction(),
                        program.methods
                            .userUpdateRewardPools()
                            .accounts({
                                user: user2.publicKey,
                            })
                            .signers([user2])
                            .instruction(),
                    ]),
                ),
                [restaking.adminKeypair, user2],
            );
    
            await checkRewardAccount(program, restaking.fragSOLRewardAddress);
            await checkUserRewardAccount(program, "user2", user2RewardAddress);
        });
    }

    it("user3 deposited to mint 200 fragSOL", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        let amount = new anchor.BN(1_000_000_000 * 200);

        const depositSOLTxSig = await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .userUpdateAccountsIfNeeded()
                        .accounts({
                            user: user3.publicKey,
                        })
                        .instruction(),
                    program.methods
                        .userDepositSol(amount, null)
                        .accounts({
                            user: user3.publicKey,
                        })
                        .remainingAccounts(restaking.stakePoolAccounts)
                        .instruction(),
                ]),
            ),
            [user3],
        );

        await checkRewardAccount(program, restaking.fragSOLRewardAddress);
        await checkUserRewardAccount(program, "user3", user3RewardAddress);
    });

    it(`[after ${9+1} slots] user2 deposited to mint 100 fragSOL`, async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        await utils.skipSlots(program, restaking.wallet.payer, 9);

        let amount = new anchor.BN(1_000_000_000 * 100);

        const depositSOLTxSig = await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .userUpdateAccountsIfNeeded()
                        .accounts({
                            user: user2.publicKey,
                        })
                        .instruction(),
                    program.methods
                        .userDepositSol(amount, null)
                        .accounts({
                            user: user2.publicKey,
                        })
                        .remainingAccounts(restaking.stakePoolAccounts)
                        .instruction(),
                ]),
            ),
            [user2],
        );

        await checkRewardAccount(program, restaking.fragSOLRewardAddress);
        await checkUserRewardAccount(program, "user2", user2RewardAddress);
    });

    it(`[after ${4+1} slots] just update to calculate fPoint drop amount`, async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        await utils.skipSlots(program, restaking.wallet.payer, 4);

        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .adminUpdateRewardPools()
                        .accounts({
                            payer: restaking.adminKeypair.publicKey,
                        })
                        .signers([restaking.adminKeypair])
                        .instruction(),
                    program.methods
                        .userUpdateRewardPools()
                        .accounts({
                            user: user2.publicKey,
                        })
                        .signers([user2])
                        .instruction(),
                    program.methods
                        .userUpdateRewardPools()
                        .accounts({
                            user: user3.publicKey,
                        })
                        .signers([user3])
                        .instruction(),
                ]),
            ),
            [restaking.adminKeypair, user2, user3],
        );

        await checkRewardAccount(program, restaking.fragSOLRewardAddress);
        await checkUserRewardAccount(program, "user2", user2RewardAddress);
        await checkUserRewardAccount(program, "user3", user3RewardAddress);
    });

    it("settle reward fPoint as 2:1 ratio based on contribution (case1. respect past contribution)", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        const rewardAccount = await program.account.rewardAccount.fetch(restaking.fragSOLRewardAddress);

        let dropAmount = new anchor.BN(rewardAccount.rewardPools[0].contribution.toNumber() * 2);
        console.log(`dropAmount: ${dropAmount.toNumber() / CONTRIBUTION_DENOMINATOR}`);

        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .fundManagerSettleReward(0, 0, dropAmount)
                        .accounts({
                            rewardTokenMint: null,
                            rewardTokenProgram: null,
                        })
                        .signers([restaking.adminKeypair])
                        .instruction(),
                ]),
            ),
            [restaking.fundManagerKeypair],
        );

        await checkRewardAccount(program, restaking.fragSOLRewardAddress);
        await checkRewardAccountSettlements(program, restaking.fragSOLRewardAddress);
    });

    for (let i = 0; i < 3; i++) {
        it(`[after ${4+1} slots] just update to check user rewards`, async function () {
            if (!utils.isLocalnet(program.provider.connection)) {
                this.skip();
            }

            // this is inside the loop because if this kind of variable declared outside of the loop, then it could be overwritten if it's reused at other test cases below and would not work what intended.
            // and also, it's inside the 'it' function because user rewardAddress is set at before function, and before function is set after the outside variables are set.
            let signers = [
                { name: "user2", value: user2, rewardAddress: user2RewardAddress, rewardType: RewardType.Point },
                { name: "user2", value: user2, rewardAddress: user2RewardAddress, rewardType: RewardType.Point },
                { name: "user3", value: user3, rewardAddress: user3RewardAddress, rewardType: RewardType.Point },
            ];
    
            await utils.skipSlots(program, restaking.wallet.payer, 4);
    
            await anchor.web3.sendAndConfirmTransaction(
                program.provider.connection,
                new anchor.web3.Transaction().add(
                    ...await Promise.all([
                        program.methods
                            .userUpdateRewardPools()
                            .accounts({
                                user: signers[i].value.publicKey,
                            })
                            .instruction(),
                    ]),
                ),
                [signers[i].value],
            );
    
            await checkUserRewardAccount(program, signers[i].name, signers[i].rewardAddress);
            await checkUserRewardAccountSettlements(program, signers[i].name, signers[i].rewardAddress, restaking.fragSOLRewardAddress);
        });
    }

    it(`[after ${9+1} slots] settle reward xToken as zero amount (case2. set zero amount to intentionally clear past contribution)`, async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        await utils.skipSlots(program, restaking.wallet.payer, 9);

        let dropAmount = new anchor.BN(0);

        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .fundManagerSettleReward(0, 1, dropAmount)
                        .accounts({
                            rewardTokenMint: null,
                            rewardTokenProgram: null,
                        })
                        .signers([restaking.adminKeypair])
                        .instruction(),
                ]),
            ),
            [restaking.fundManagerKeypair],
        );

        await checkRewardAccount(program, restaking.fragSOLRewardAddress);
        await checkRewardAccountSettlements(program, restaking.fragSOLRewardAddress);
    });

    it(`[after ${9+1} slots] just update to calculate fPoint drop amount`, async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        await utils.skipSlots(program, restaking.wallet.payer, 9);

        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .adminUpdateRewardPools()
                        .accounts({
                            payer: restaking.adminKeypair.publicKey,
                        })
                        .signers([restaking.adminKeypair])
                        .instruction(),
                    program.methods
                        .userUpdateRewardPools()
                        .accounts({
                            user: user2.publicKey,
                        })
                        .signers([user2])
                        .instruction(),
                    program.methods
                        .userUpdateRewardPools()
                        .accounts({
                            user: user3.publicKey,
                        })
                        .signers([user3])
                        .instruction(),
                ]),
            ),
            [restaking.adminKeypair, user2, user3],
        );

        await checkRewardAccount(program, restaking.fragSOLRewardAddress);
        await checkRewardAccountSettlements(program, restaking.fragSOLRewardAddress);
        await checkUserRewardAccount(program, "user2", user2RewardAddress);
        await checkUserRewardAccountSettlements(program, "user2", user2RewardAddress, restaking.fragSOLRewardAddress);
        await checkUserRewardAccount(program, "user3", user3RewardAddress);
        await checkUserRewardAccountSettlements(program, "user3", user3RewardAddress, restaking.fragSOLRewardAddress);
    });

    it("settle reward fPoint as 1:1 ratio based on contribution", async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        const rewardAccount = await program.account.rewardAccount.fetch(restaking.fragSOLRewardAddress);

        let dropAmount = (() => {
            const rewardPool = rewardAccount.rewardPools[0];
            return rewardPool.contribution.sub(rewardPool.rewardSettlements[0].settlementBlocksLastRewardPoolContribution);
        })();
        console.log(`dropAmount: ${dropAmount.toNumber() / CONTRIBUTION_DENOMINATOR}`);

        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .fundManagerSettleReward(0, 0, dropAmount)
                        .accounts({
                            rewardTokenMint: null,
                            rewardTokenProgram: null,
                        })
                        .signers([restaking.adminKeypair])
                        .instruction(),
                ]),
            ),
            [restaking.fundManagerKeypair],
        );

        await checkRewardAccount(program, restaking.fragSOLRewardAddress);
        await checkRewardAccountSettlements(program, restaking.fragSOLRewardAddress);
    });

    it(`[after ${9+1} slots] user2 deposited to mint 100 fragSOL`, async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        await utils.skipSlots(program, restaking.wallet.payer, 9);

        let amount = new anchor.BN(1_000_000_000 * 100);

        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .userUpdateAccountsIfNeeded()
                        .accounts({
                            user: user2.publicKey,
                        })
                        .instruction(),
                    program.methods
                        .userDepositSol(amount, null)
                        .accounts({
                            user: user2.publicKey,
                        })
                        .remainingAccounts(restaking.stakePoolAccounts)
                        .instruction(),
                ]),
            ),
            [user2],
        );

        await checkRewardAccount(program, restaking.fragSOLRewardAddress);
        await checkUserRewardAccount(program, "user2", user2RewardAddress);
        await checkUserRewardAccountSettlements(program, "user2", user2RewardAddress, restaking.fragSOLRewardAddress);
    });

    it(`[after ${9+1} slots] just update to check user rewards`, async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        await utils.skipSlots(program, restaking.wallet.payer, 9);

        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .adminUpdateRewardPools()
                        .accounts({
                            payer: restaking.adminKeypair.publicKey,
                        })
                        .signers([restaking.adminKeypair])
                        .instruction(),
                    program.methods
                        .userUpdateRewardPools()
                        .accounts({
                            user: user2.publicKey,
                        })
                        .signers([user2])
                        .instruction(),
                    program.methods
                        .userUpdateRewardPools()
                        .accounts({
                            user: user3.publicKey,
                        })
                        .signers([user3])
                        .instruction(),
                ]),
            ),
            [restaking.adminKeypair, user2, user3],
        );

        await checkRewardAccount(program, restaking.fragSOLRewardAddress);
        await checkRewardAccountSettlements(program, restaking.fragSOLRewardAddress);
        await checkUserRewardAccount(program, "user2", user2RewardAddress);
        await checkUserRewardAccountSettlements(program, "user2", user2RewardAddress, restaking.fragSOLRewardAddress);
        await checkUserRewardAccount(program, "user3", user3RewardAddress);
        await checkUserRewardAccountSettlements(program, "user3", user3RewardAddress, restaking.fragSOLRewardAddress);
    });

    it(`[after ${9+1} slots] settle reward a fPoint... to check clearing stale blocks`, async function () {
        if (!utils.isLocalnet(program.provider.connection)) {
            this.skip();
        }

        await utils.skipSlots(program, restaking.wallet.payer, 9);

        let user2RewardSettlementsBefore = await checkUserRewardAccountSettlements(program, "user2", user2RewardAddress, restaking.fragSOLRewardAddress);
        let user3RewardSettlementsBefore = await checkUserRewardAccountSettlements(program, "user3", user3RewardAddress, restaking.fragSOLRewardAddress);

        let dropAmount = new anchor.BN(1_000_000_000 * 1000);

        await anchor.web3.sendAndConfirmTransaction(
            program.provider.connection,
            new anchor.web3.Transaction().add(
                ...await Promise.all([
                    program.methods
                        .fundManagerSettleReward(0, 0, dropAmount)
                        .accounts({
                            rewardTokenMint: null,
                            rewardTokenProgram: null,
                        })
                        .signers([restaking.adminKeypair])
                        .instruction(),
                    program.methods
                        .userUpdateRewardPools()
                        .accounts({
                            user: user2.publicKey,
                        })
                        .signers([user2])
                        .instruction(),
                    program.methods
                        .userUpdateRewardPools()
                        .accounts({
                            user: user3.publicKey,
                        })
                        .signers([user3])
                        .instruction(),
                ]),
            ),
            [restaking.fundManagerKeypair, user2, user3],
        );

        await checkRewardAccount(program, restaking.fragSOLRewardAddress);
        await checkRewardAccountSettlements(program, restaking.fragSOLRewardAddress);

        await checkUserRewardAccount(program, "user2", user2RewardAddress);
        let user2RewardSettlementsAfter = await checkUserRewardAccountSettlements(program, "user2", user2RewardAddress, restaking.fragSOLRewardAddress);

        for (let i = 0; i < user2RewardSettlementsBefore.length; i++) {
            await checkUserRewardAccountSettlementsDifference("user2", i, user2RewardSettlementsBefore[i], user2RewardSettlementsAfter[i]);
        }

        await checkUserRewardAccount(program, "user3", user3RewardAddress);
        let user3RewardSettlementsAfter = await checkUserRewardAccountSettlements(program, "user3", user3RewardAddress, restaking.fragSOLRewardAddress);

        for (let i = 0; i < user3RewardSettlementsBefore.length; i++) {
            await checkUserRewardAccountSettlementsDifference("user3", i, user3RewardSettlementsBefore[i], user3RewardSettlementsAfter[i]);
        }
    });
});

const checkRewardAccount = async (playground: RestakingPlayground, rewardAccountAddress: anchor.web3.PublicKey) => {
    const rewardAccount = await playground.account.rewardAccount.fetch(rewardAccountAddress);

    for (let i = 0; i < rewardAccount.rewardPools1.length; i++) {
        const rewardPool = rewardAccount.rewardPools1[i];

        console.log(`[RewardAccount..RewardPool=${i}] after update: tokenAllocatedAmount=${rewardPool.tokenAllocatedAmount.totalAmount.toNumber() / TOKEN_DENOMINATOR}, contribution=${rewardPool.contribution.toNumber() / CONTRIBUTION_DENOMINATOR}, rewardPoolBitmap (0: custum contribution accrual rate enables, 1: is closed, 2: has holder)=${rewardPool.rewardPoolBitmap}, contributionUpdatedSlot=${rewardPool.updatedSlot}`);
    }
}

const checkRewardAccountSettlements = async (program: anchor.Program<Restaking>, rewardAccountAddress: anchor.web3.PublicKey) => {
    const rewardAccount = await program.account.rewardAccount.fetch(rewardAccountAddress);

    for (let i = 0; i < rewardAccount.rewardPools.length; i++) {
        const rewardPool = rewardAccount.rewardPools[i];

        for (let j = 0; j < rewardPool.rewardSettlements.length; j++) {
            const rewardSettlement = rewardPool.rewardSettlements[j];
            const rewardType = rewardAccount.rewards[rewardSettlement.rewardId].rewardType;

            let convertedRewardSettlementBlocks: SettlementBlock[] = [];
            for (let k = 0; k < rewardSettlement.settlementBlocks.length; k++) {
                const rewardSettlementBlock = rewardSettlement.settlementBlocks[k];

                let convertedRewardSettlementBlock: SettlementBlock = {
                    amount: rewardSettlementBlock.amount.toNumber() / (rewardType.point ? CONTRIBUTION_DENOMINATOR : TOKEN_DENOMINATOR),
                    startingRewardPoolContribution: rewardSettlementBlock.startingRewardPoolContribution.toNumber() / CONTRIBUTION_DENOMINATOR,
                    startingSlot: rewardSettlementBlock.startingSlot.toNumber(),
                    endingRewardPoolContribution: rewardSettlementBlock.endingRewardPoolContribution.toNumber() / CONTRIBUTION_DENOMINATOR,
                    endingSlot: rewardSettlementBlock.endingSlot.toNumber(),
                    userSettledAmount: rewardSettlementBlock.userSettledAmount.toNumber() / (rewardType.point ? CONTRIBUTION_DENOMINATOR : TOKEN_DENOMINATOR),
                    userSettledContribution: rewardSettlementBlock.userSettledContribution.toNumber() / CONTRIBUTION_DENOMINATOR,
                };

                // let convertedRewardSettlementBlock: SettlementBlock = Object.fromEntries(Object.entries(rewardSettlementBlock).map(b => [b[0], b[1].toNumber()])) as unknown as SettlementBlock;
                // console.log(`convertedRewardSettlementBlock:`, convertedRewardSettlementBlock);
                convertedRewardSettlementBlocks.push(convertedRewardSettlementBlock);
            }

            const convertedRewardSettlement: RewardSettlement = {
                rewardId: rewardSettlement.rewardId,
                rewardPoolId: rewardSettlement.rewardPoolId,
                remainingAmount: rewardSettlement.remainingAmount.toNumber() / TOKEN_DENOMINATOR,
                claimedAmount: rewardSettlement.claimedAmount.toNumber() / TOKEN_DENOMINATOR,
                claimedAmountUpdatedSlot: rewardSettlement.claimedAmountUpdatedSlot.toNumber(),
                settledAmount: rewardSettlement.settledAmount.toNumber() / (rewardType.point ? CONTRIBUTION_DENOMINATOR : TOKEN_DENOMINATOR),
                settlementBlocksLastRewardPoolContribution: rewardSettlement.settlementBlocksLastRewardPoolContribution.toNumber() / CONTRIBUTION_DENOMINATOR,
                settlementBlocksLastSlot: rewardSettlement.settlementBlocksLastSlot.toNumber(),
                settlementBlocks: convertedRewardSettlementBlocks,
            };

            console.log(`[RewardAccount..RewardPool=${i}..RewardSettlement=${j}] after settlement: rewardSettlement=`, convertedRewardSettlement);
        }
    }
}

const checkUserRewardAccount = async (playground: RestakingPlayground, userName: string, userRewardAccountAddress: anchor.web3.PublicKey) => {
    const userRewardAccount = await playground.account.userRewardAccount.fetch(userRewardAccountAddress);

    for (let i = 0; i < userRewardAccount.userRewardPools1.length; i++) {
        const userRewardPool = userRewardAccount.userRewardPools1[i];

        console.log(`[UserRewardAccount=${userName}..UserRewardPool=${i}] after update: tokenAllocatedAmount=${userRewardPool.tokenAllocatedAmount.totalAmount.toNumber() / TOKEN_DENOMINATOR}, contribution=${userRewardPool.contribution.toNumber() / CONTRIBUTION_DENOMINATOR}, updatedSlot=${userRewardPool.updatedSlot}`);
    }
}

const checkUserRewardAccountSettlements = async (program: anchor.Program<Restaking>, userName: string, userRewardAccountAddress: anchor.web3.PublicKey, rewardAccountAddress: anchor.web3.PublicKey) => {
    const userRewardAccount = await program.account.userRewardAccount.fetch(userRewardAccountAddress);
    const rewardAccount = await program.account.rewardAccount.fetch(rewardAccountAddress);

    let userRewardPoolSettlements = []; // [userRewardPools][userRewardSettlements]
    for (let i = 0; i < userRewardAccount.userRewardPools.length; i++) {
        const userRewardPool = userRewardAccount.userRewardPools[i];

        userRewardPoolSettlements.push([]);
        for (let j = 0; j < userRewardPool.rewardSettlements.length; j++) {
            const userRewardSettlement = userRewardPool.rewardSettlements[j];
            const rewardType = rewardAccount.rewards[userRewardSettlement.rewardId].rewardType;

            const convertedUserRewardSettlement: UserRewardSettlement = {
                rewardId: userRewardSettlement.rewardId,
                settledAmount: userRewardSettlement.settledAmount.toNumber() / (rewardType.point ? CONTRIBUTION_DENOMINATOR : TOKEN_DENOMINATOR),
                settledContribution: userRewardSettlement.settledContribution.toNumber() / CONTRIBUTION_DENOMINATOR,
                settledSlot: userRewardSettlement.settledSlot.toNumber(),
                claimedAmount: userRewardSettlement.claimedAmount.toNumber() / TOKEN_DENOMINATOR,
            };

            console.log(`[UserRewardAccount=${userName}..UserRewardPool=${i}..UserRewardSettlement=${j}] after settlement: userRewardSettlement=`, convertedUserRewardSettlement);

            userRewardPoolSettlements[i].push(convertedUserRewardSettlement);
        }
    }

    return userRewardPoolSettlements;
}

const checkUserRewardAccountSettlementsDifference = async (userName: string, userRewardPoolId: number, befores: UserRewardSettlement[], afters: UserRewardSettlement[]) => {
    for (let i = 0; i < befores.length; i++) {
        let before = befores[i];
        let after = afters[i];

        console.log(`[UserRewardAccount=${userName}..UserRewardPool=${userRewardPoolId}..UserRewardSettlement=${i}] after update (passed slots=${after.settledSlot - before.settledSlot}): settledAmount=${after.settledAmount} (${after.settledAmount >= before.settledAmount ? "+" : "-"}${after.settledAmount >= before.settledAmount ? after.settledAmount - before.settledAmount : before.settledAmount - after.settledAmount}), settledContribution=${after.settledContribution} (${after.settledContribution >= before.settledContribution ? "+" : "-"}${after.settledContribution >= before.settledContribution ? after.settledContribution - before.settledContribution : before.settledContribution - after.settledContribution})`);
    }
}
