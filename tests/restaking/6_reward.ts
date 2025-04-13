import {BN, IdlAccounts, web3} from "@coral-xyz/anchor";
import {expect} from "chai";
import {step} from "mocha-steps";
import {getLogger} from "../../tools/lib";
import {Restaking} from '../../target/types/restaking';
import {restakingPlayground} from "../restaking";
import { RestakingPlayground } from "../../tools/restaking/playground";


const {logger} = getLogger('reward');

const rewardPoolTypes = [
    "base",
    "bonus",
];

function printUserRewardAccount(alias: string, account: IdlAccounts<Restaking>['userRewardAccount']) {
    for (let i = 0; i < rewardPoolTypes.length; i++) {
        const pool = rewardPoolTypes[i] == "base" ? account.baseUserRewardPool : account.bonusUserRewardPool;
        logger.debug(`[slot=${pool.updatedSlot.toString()}] ${alias}-pool#${pool.rewardPoolId}: allocated=${pool.tokenAllocatedAmount.totalAmount.toNumber().toLocaleString()}, contribution=${pool.contribution.toNumber().toLocaleString()}`);
        for (let j = 0; j < pool.numRewardSettlements; j++) {
            const settle = pool.rewardSettlements1[j];
            logger.debug(`> ${alias}-pool#${pool.rewardPoolId}-reward#${settle.rewardId}: settled-slot=${settle.lastSettledSlot.toNumber().toLocaleString()}, settled-amount=${settle.totalSettledAmount.toNumber().toLocaleString()}, settled-contribution=${settle.totalSettledContribution.toNumber().toLocaleString()}`);
        }
    }
}

describe("reward", async function () {
    const restaking = await restakingPlayground as RestakingPlayground;
    const userA = restaking.keychain.getKeypair('MOCK_USER9');
    const userB = restaking.keychain.getKeypair('MOCK_USER10');
    const PRICING_DIFF_ERROR_MODIFIER = 1_000;

    step("try airdrop SOL to mock accounts", async function () {
        await Promise.all([
            restaking.tryAirdrop(userA.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(1_000)),
            restaking.tryAirdrop(userB.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(1_000)),
        ]);

        await restaking.sleep(1);
    });

    step("rewards are settled based on the contribution proportion", async function () {
        const a1 = await restaking.runUserDepositSOL(userA, new BN(100 * (10 ** restaking.fragSOLDecimals)), null);

        expect(a1.fragSOLUserReward.baseUserRewardPool.contribution.toNumber()).eq(0);
        expect(a1.fragSOLUserReward.bonusUserRewardPool.contribution.toNumber()).eq(0);
        printUserRewardAccount('A', a1.fragSOLUserReward);

        await restaking.sleep(1);
        const b2 = await restaking.runUserDepositSOL(userB, new BN(200 * (10 ** restaking.fragSOLDecimals)), null);
        printUserRewardAccount('B', b2.fragSOLUserReward);

        await restaking.sleep(1);
        const [a3, b3] = await Promise.all([
            restaking.runUserDepositSOL(userA, new BN(300 * (10 ** restaking.fragSOLDecimals)), null),
            restaking.runUserUpdateRewardPools(userB),
        ]);
        printUserRewardAccount('A', a3.fragSOLUserReward);
        printUserRewardAccount('B', b3.fragSOLUserReward);
        expect(a3.fragSOLUserReward.baseUserRewardPool.contribution.divn(PRICING_DIFF_ERROR_MODIFIER).toString(), 'A contrib = 100(2slot) + 300(0slot)')
            .eq(b3.fragSOLUserReward.baseUserRewardPool.contribution.divn(PRICING_DIFF_ERROR_MODIFIER).toString(), 'B contrib = 200(1slot)');
        expect(a3.fragSOLUserReward.bonusUserRewardPool.contribution.divn(PRICING_DIFF_ERROR_MODIFIER).toString(), 'a')
            .eq(b3.fragSOLUserReward.bonusUserRewardPool.contribution.divn(PRICING_DIFF_ERROR_MODIFIER).toString(), 'b');

        await restaking.sleep(1);
        const [a4, b4] = await Promise.all([
            restaking.runUserUpdateRewardPools(userA),
            restaking.runUserUpdateRewardPools(userB),
        ]);
        printUserRewardAccount('A', a4.fragSOLUserReward);
        printUserRewardAccount('B', b4.fragSOLUserReward);
        expect(a4.fragSOLUserReward.bonusUserRewardPool.contribution.divn(PRICING_DIFF_ERROR_MODIFIER).mul(new BN(2)).toString(), 'A contrib = 100(3slot) + 300(1slot)') // 600
            .eq(b4.fragSOLUserReward.bonusUserRewardPool.contribution.divn(PRICING_DIFF_ERROR_MODIFIER).mul(new BN(3)).toString(), 'B contrib = 200(2slot)'); // 400

        // drop fPoint in approximately(time flies) 1:1 ratio to total contribution; contribution(11) has 2 + 5 more decimals than fPoint(4)
        const r4 = await restaking.runOperatorUpdateRewardPools();
        const s5Amount = r4.fragSOLReward.bonusRewardPool.contribution.divn(PRICING_DIFF_ERROR_MODIFIER).div(new BN(10 ** (2 + 5)));
        const s5 = await restaking.runFundManagerSettleReward({
            poolName: 'bonus',
            rewardName: 'fPoint',
            amount: s5Amount,
        });
        const r4Settle = s5.fragSOLReward.bonusRewardPool.rewardSettlements1[0];
        const r4Block = r4Settle.settlementBlocks[r4Settle.settlementBlocksTail - 1];
        expect(r4Block.amount.toString()).eq(s5Amount.toString(), 'c');

        await restaking.sleep(1);
        const [a6, b6] = await Promise.all([
            restaking.runUserUpdateRewardPools(userA),
            restaking.runUserUpdateRewardPools(userB),
        ]);
        printUserRewardAccount('A', a6.fragSOLUserReward);
        printUserRewardAccount('B', b6.fragSOLUserReward);

        const aSettledAmountDelta = a6.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledAmount
            .sub(a3.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledAmount);
        const aSettledContribDelta = a6.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledContribution
            .sub(a3.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledContribution);
        const bSettledAmountDelta = b6.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledAmount
            .sub(b3.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledAmount);
        const bSettledContribDelta = b6.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledContribution
            .sub(b3.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledContribution);
        // aSettle.totalSettledAmount/bSettle.totalSettledAmount = aSettle.totalSettledContribution/bSettle.totalSettledContribution

        expect((aSettledAmountDelta.toNumber() / bSettledAmountDelta.toNumber()).toPrecision(4))
            .eq((aSettledContribDelta.toNumber() / bSettledContribDelta.toNumber()).toPrecision(4), 'd');
    });

    step("rewards can be settled with custom contribution accrual rate enabled", async function () {
        // starts with A: 400, B: 200
        const b1 = await restaking.runUserDepositSOL(userB, new BN(200 * (10 ** restaking.fragSOLDecimals)), {
            user: userB.publicKey,
            walletProvider: 'STIMPACK',
            contributionAccrualRate: 150,
            expiredAt: new BN(Math.floor(Date.now() / 1000)),
        });
        // now A: 400, B: 200 + 200(x1.5)

        // flush contributions of all pools by settling zero rewards.
        await Promise.all([
            restaking.runFundManagerSettleReward({
                poolName: 'base',
                rewardName: 'fPoint',
                amount: new BN(0),
            }),
            restaking.runFundManagerSettleReward({
                poolName: 'bonus',
                rewardName: 'fPoint',
                amount: new BN(0),
            }),
        ]);

        await restaking.sleep(1);
        const [a3, b3] = await Promise.all([
            restaking.runUserUpdateRewardPools(userA),
            restaking.runUserUpdateRewardPools(userB),
        ]);
        printUserRewardAccount('A', a3.fragSOLUserReward);
        printUserRewardAccount('B', b3.fragSOLUserReward);

        // drop fPoint in approximately(time flies) 2:1 ratio to total contribution; contribution(11) has 2 + 5 more decimals than fPoint(4)
        const r3 = await restaking.runOperatorUpdateRewardPools();
        const s4Amount = r3.fragSOLReward.bonusRewardPool.contribution.mul(new BN(2)).div(new BN(10 ** (2 + 5)));
        await Promise.all([
            restaking.runFundManagerSettleReward({
                poolName: 'base',
                rewardName: 'fPoint',
                amount: s4Amount,
            }),
            restaking.runFundManagerSettleReward({
                poolName: 'bonus',
                rewardName: 'fPoint',
                amount: s4Amount,
            }),
        ]);
        const r5 = await restaking.runOperatorUpdateRewardPools();
        const base5 = r5.fragSOLReward.baseRewardPool;
        const bonus5 = r5.fragSOLReward.bonusRewardPool;

        expect(base5.updatedSlot.toString()).eq(bonus5.updatedSlot.toString(), 'a');
        expect(base5.tokenAllocatedAmount.totalAmount.toString()).eq(bonus5.tokenAllocatedAmount.totalAmount.toString(), 'b');
        expect(base5.contribution.toNumber()).lt(bonus5.contribution.toNumber(), 'c');
        expect(base5.rewardSettlements1[0].settlementBlocksLastRewardPoolContribution.toNumber(), 'd')
            .lt(bonus5.rewardSettlements1[0].settlementBlocksLastRewardPoolContribution.toNumber(), 'e');

        // now check users' settlements
        const [a6, b6] = await Promise.all([
            restaking.runUserUpdateRewardPools(userA),
            restaking.runUserUpdateRewardPools(userB),
        ]);

        // new base pool settled amounts are same; A: 400, B: 400 => A:B = 1:1
        expect(a6.fragSOLUserReward.baseUserRewardPool.rewardSettlements1[0].totalSettledAmount.divn(PRICING_DIFF_ERROR_MODIFIER).toNumber(), 'a6 base settled')
            .eq(b6.fragSOLUserReward.baseUserRewardPool.rewardSettlements1[0].totalSettledAmount.divn(PRICING_DIFF_ERROR_MODIFIER).toNumber(), 'b6 base settled');
        expect(a6.fragSOLUserReward.baseUserRewardPool.tokenAllocatedAmount.totalAmount.divn(PRICING_DIFF_ERROR_MODIFIER).toNumber(), 'b6 base allocated')
            .eq(b6.fragSOLUserReward.baseUserRewardPool.tokenAllocatedAmount.totalAmount.divn(PRICING_DIFF_ERROR_MODIFIER).toNumber(), 'b6 base allocated');

        // added bonus pool settled amounts are different; A: 400, B: 200+200(x1.5) => A:B = 4:5
        const a6BonusSettledAmountDelta = a6.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledAmount
            .sub(a3.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledAmount);
        const b6BonusSettledAmountDelta = b6.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledAmount
            .sub(b3.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledAmount);
        const a6BonusSettledContribDelta = a6.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledContribution
            .sub(a3.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledContribution);
        const b6BonusSettledContribDelta = b6.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledContribution
            .sub(b3.fragSOLUserReward.bonusUserRewardPool.rewardSettlements1[0].totalSettledContribution);

        expect((a6BonusSettledAmountDelta.toNumber() / b6BonusSettledAmountDelta.toNumber()).toPrecision(4), 'a6_amount / b6_amount')
            .eq((a6BonusSettledContribDelta.toNumber() / b6BonusSettledContribDelta.toNumber()).toPrecision(4), 'a6_contrib / b6_contrib');

        expect(a6BonusSettledContribDelta.divn(PRICING_DIFF_ERROR_MODIFIER).mul(new BN(5)).toString(), 'a6_contrib x 5')
            .eq(b6BonusSettledContribDelta.divn(PRICING_DIFF_ERROR_MODIFIER).mul(new BN(4)).toString(), 'b6_contrib x 4');
    });

    step("user can claim rewards", async () => {
        await restaking.runFundManagerSettleReward({poolName: "base", rewardName: "SWTCH", amount: new BN(0)});
        await restaking.sleep(1);
        await restaking.runFundManagerSettleReward({poolName: "base", rewardName: "SWTCH", amount: new BN(5)});
        await restaking.sleep(1);
        await restaking.runFundManagerSettleReward({poolName: "base", rewardName: "SWTCH", amount: new BN(0)});

        await restaking.tryAirdropRewardToken(restaking.wallet.publicKey, "SWTCH", new BN(5));
        await expect(restaking.runFundManagerUpdateReward({source: restaking.wallet, rewardName: "SWTCH", newRewardMint: restaking.distributingRewardsMetadata.find(r => r.name == "SWTCH").mint, claimable: true, transferAmount: new BN(5)})).rejectedWith('RewardAlreadyExistingRewardError');
        await restaking.runFundManagerUpdateReward({source: restaking.wallet, rewardName: "SWTCH", newRewardMint: null, claimable: true, transferAmount: new BN(5)});
        await restaking.sleep(1);

        const res = await restaking.runUserClaimReward(userA, {poolName: "base", rewardName: "SWTCH"});
        expect(res.event.userClaimedReward.claimedRewardTokenAmount.toNumber()).eq(2);
        expect(res.event.userClaimedReward.totalClaimedRewardTokenAmount.toNumber()).eq(2);

        await restaking.tryAirdropRewardToken(restaking.wallet.publicKey, "SWTCH", new BN(5));
        await restaking.runFundManagerSettleReward({source: restaking.wallet, poolName: "base", rewardName: "SWTCH", amount: new BN(5)});

        const res1 = await restaking.runUserClaimReward(userA, {poolName: "base", rewardName: "SWTCH"});
        expect(res1.event.userClaimedReward.claimedRewardTokenAmount.toNumber()).eq(2);
        expect(res1.event.userClaimedReward.totalClaimedRewardTokenAmount.toNumber()).eq(4);
    })
});
