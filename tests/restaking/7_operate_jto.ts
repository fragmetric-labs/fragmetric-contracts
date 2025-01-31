import {BN, web3} from '@coral-xyz/anchor';
import {expect} from "chai";
import {step} from "mocha-steps";
import {RestakingPlayground} from "../../tools/restaking/jto_playground";
import * as anchor from "@coral-xyz/anchor";
import {getLogger} from "../../tools/lib";

export const restakingPlayground = RestakingPlayground.create('local', {
    provider: anchor.AnchorProvider.env(),
});

const {logger, LOG_PAD_SMALL, LOG_PAD_LARGE} = getLogger("restaking");

module.exports = (i: number) => describe(`operate#TODO${i}`, async () => {
    const restaking = await restakingPlayground;
    const user1 = restaking.keychain.getKeypair('MOCK_USER1');

    step("try airdrop SOL to authorized wallets", async function () {
        await Promise.all([
            restaking.tryAirdrop(restaking.keychain.getPublicKey('ADMIN'), new BN(web3.LAMPORTS_PER_SOL).muln(100)),
            restaking.tryAirdrop(restaking.keychain.getPublicKey('FUND_MANAGER'), new BN(web3.LAMPORTS_PER_SOL).muln(100)),
        ]);

        await restaking.sleep(1); // ...block hash not found?
    });

    // step("create known address lookup table", async function () {
    //     await restaking.getOrCreateKnownAddressLookupTable();
    // });

    step("create fragJTO token mint with extensions", async function () {
        const res0 = await restaking.runAdminInitializeFragJTOTokenMint();
        expect(res0.fragJTOMint.address.toString()).eq(restaking.knownAddress.fragJTOTokenMint.toString());
        expect(res0.fragJTOMint.mintAuthority.toString()).eq(restaking.keychain.getKeypair('ADMIN').publicKey.toString()); // shall be transferred to a PDA below
        expect(res0.fragJTOMint.freezeAuthority).null;
    });

    step("update fragJTO token metadata", async function () {
        await restaking.runAdminUpdateTokenMetadata();
    });

    step("initialize fund accounts", async () => {
        const {fragJTOMint, fragJTOFundAccount} = await restaking.runAdminInitializeOrUpdateFundAccount();

        expect(fragJTOMint.mintAuthority.toString()).eq(restaking.knownAddress.fragJTOFund.toString());
        expect(fragJTOFundAccount.dataVersion).gt(1);
    })

    step("initialize reward accounts", async () => {
        const {fragJTORewardAccount} = await restaking.runAdminInitializeOrUpdateRewardAccount();

        expect(fragJTORewardAccount.dataVersion).eq(parseInt(restaking.getConstant('rewardAccountCurrentVersion')));
    })

    step("initialize fragJTO extra account meta list", async () => {
        await restaking.runAdminInitializeFragJTOExtraAccountMetaList();
        const { fragJTOExtraAccountMetasAccount } = await restaking.runAdminUpdateFragJTOExtraAccountMetaList();

        expect(fragJTOExtraAccountMetasAccount.length).eq(8);
    })

    step("initialize reward pools and rewards", async function () {
        const res0 = await restaking.runFundManagerInitializeRewardPools();

        expect(res0.fragJTOReward.dataVersion).eq(parseInt(restaking.getConstant('rewardAccountCurrentVersion')));

        expect(res0.fragJTOReward.numRewards).eq(Object.values(restaking.rewardsMetadata).length);
        let i = 0;
        for (const v of Object.values(restaking.rewardsMetadata)) {
            const reward = res0.fragJTOReward.rewards1[i++];
            expect(restaking.binToString(reward.name)).eq(v.name.toString());
            expect(restaking.binToString(reward.description)).eq(v.description.toString());
        }

        expect(res0.fragJTOReward.numRewardPools).eq(Object.values(restaking.rewardPoolsMetadata).length);
        i = 0;
        for (const v of Object.values(restaking.rewardPoolsMetadata)) {
            const pool = res0.fragJTOReward.rewardPools1[i++];
            expect(restaking.binToString(pool.name)).eq(v.name.toString());
        }
    });

    step("settle fPoint reward (zeroing)", async () => {
        await new Promise(resolve => setTimeout(resolve, 1000)); // wait for few slot elapsed
        const res0 = await restaking.runFundManagerSettleReward({
            poolName: 'bonus',
            rewardName: 'fPoint',
            amount: new BN(0),
        });
        expect(res0.fragJTOReward.rewardPools1[res0.rewardPool.id].numRewardSettlements).eq(1);
        expect(res0.fragJTOReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].rewardId).eq(res0.reward.id);
        expect(res0.fragJTOReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].rewardPoolId).eq(res0.rewardPool.id);
        expect(res0.fragJTOReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].numSettlementBlocks).eq(1);
        expect(res0.fragJTOReward.rewardPools1[res0.rewardPool.id].rewardSettlements1[0].settledAmount.toNumber()).eq(0);
    });

    step("initialize fund supported tokens", async function () {
        const res0 = await restaking.runFundManagerInitializeFundSupportedTokens();

        expect(res0.fragJTOFund.numSupportedTokens).eq(Object.values(restaking.supportedTokenMetadata).length);
        let i = 0;
        for (const [symbol, v] of Object.entries(restaking.supportedTokenMetadata)) {
            const supported = res0.fragJTOFund.supportedTokens[i++];
            expect(supported.mint.toString()).eq(v.mint.toString());
            expect(supported.program.toString()).eq(v.program.toString());
            // expect(supported.oneTokenAsSol.toNumber()).greaterThan(web3.LAMPORTS_PER_SOL).lessThan(2 * web3.LAMPORTS_PER_SOL);
            expect(supported.token.operationReservedAmount.toNumber()).eq(0);
        }
    });


    step("initialize fund jito restaking vault", async () => {
        // await Promise.all(Object.values(restaking.restakingVaultMetadata).map(v => restaking.runAdminSetSecondaryAdminForJitoVault(v.vault)));
        const {fragJTOFundAccount} = await restaking.runFundManagerInitializeFundJitoRestakingVault();

        expect(fragJTOFundAccount.numRestakingVaults).eq(1);
        let i = 0;
        for (const [symbol, v] of Object.entries(restaking.restakingVaultMetadata)) {
            const vault = fragJTOFundAccount.restakingVaults[i++];
            expect(vault.vault.toString()).eq(v.vault.toString());
            expect(vault.program.toString()).eq(v.program.toString());
            // expect(vault.supportedTokenMint.toString()).eq(v.VSTMint.toString());
            // expect(vault.receiptTokenMint.toString()).eq(v.VRTMint.toString());
        }
    });

    step("initialize fund, supported tokens, restaking vaults strategy", async () => {
        await restaking.runFundManagerUpdateFundConfigurations();
    });

    step("deposit sol and tokens & request withdraw for most of them", async () => {
        await Promise.all([
            restaking.tryAirdrop(user1.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(200)),
            restaking.tryAirdropSupportedTokens(user1.publicKey, new BN(web3.LAMPORTS_PER_SOL).muln(200)),
        ]);
        await restaking.runUserDepositSupportedToken(user1, 'JTO', new BN(web3.LAMPORTS_PER_SOL).muln(100));
        await restaking.runUserDepositSupportedToken(user1, 'JTO', new BN(web3.LAMPORTS_PER_SOL).muln(50));
        await restaking.runUserDepositSupportedToken(user1, 'JTO', new BN(web3.LAMPORTS_PER_SOL).muln(50));
        const res1 = await restaking.getUserFragJTOFundAccount(user1.publicKey);

        let quarter = res1.receiptTokenAmount.divn(4);
        await restaking.runUserRequestWithdrawal(user1, quarter, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'));
        await restaking.runUserRequestWithdrawal(user1, quarter, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'));
        await restaking.runOperatorFundCommands();
        logger.info('waiting... (1 epoch = 64 slots)');
        await restaking.sleepUntil(192);

        await restaking.runUserWithdraw(user1, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'), new BN(1));
        await restaking.runUserWithdraw(user1, restaking.getConstantAsPublicKey('mainnetJtoMintAddress'), new BN(2));
        const res2 = await restaking.getUserFragJTOFundAccount(user1.publicKey);
        await restaking.runUserRequestWithdrawal(user1, res2.receiptTokenAmount);
        await restaking.runOperatorFundCommands(); // here a unrestaking request made

        logger.info('waiting...');
        await restaking.sleepUntil(320); // wait for more than one epoch
        await restaking.runOperatorFundCommands(); // the unrestaking request should be claimable on this cycle
        await restaking.runOperatorFundCommands(); // one more cycle to denormalize and unstake tokens

        logger.info('waiting...');
        await restaking.sleepUntil(440); // wait for more than one epoch
        await restaking.runOperatorFundCommands(); // one more cycle to claim unstaked tokens and proceed the last withdrawal batch
        await restaking.runUserWithdraw(user1, null, new BN(3));
    });
});
