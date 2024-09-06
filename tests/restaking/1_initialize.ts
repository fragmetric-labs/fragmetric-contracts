import * as anchor from "@coral-xyz/anchor";
import * as spl from "@solana/spl-token";
import { expect } from "chai";
import {RestakingPlayground} from "../../tools/restaking/playground";

export const initialize = describe("Initialize program accounts", async function() {
    const playground = await RestakingPlayground.local(anchor.AnchorProvider.env());

    it("May airdrop SOL to mock accounts", async () => {
        await playground.tryAirdropToMockAccounts();
    });

    it("Should create fragSOL token mint with Transfer Hook extension", async function() {
        const { fragSOLMint } = await playground.runInitializeFragSOLTokenMint();
        expect(fragSOLMint.address.toString()).to.equal(playground.knownAddress.fragSOLTokenMint.toString());
        expect(fragSOLMint.mintAuthority.toString()).to.equal(playground.keychain.getKeypair('ADMIN').publicKey.toString()); // shall be transferred to a PDA below
        expect(fragSOLMint.freezeAuthority).to.null;
    });

    it("Should mock supported token mints in localnet", async function() {
        if (!playground.isMaybeLocalnet) {
            return this.skip();
        }
        const tokenMint_bSOL = await spl.getMint(
            playground.connection,
            playground.supportedTokenMetadata.bSOL.mint,
        );
        const tokenMint_mSOL = await spl.getMint(
            playground.connection,
            playground.supportedTokenMetadata.mSOL.mint,
        );
        const tokenMint_jitoSOL = await spl.getMint(
            playground.connection,
            playground.supportedTokenMetadata.jitoSOL.mint,
        );

        expect(tokenMint_bSOL.mintAuthority.toString()).to.equal(playground.keychain.getKeypair('MOCK_ALL_MINT_AUTHORITY').publicKey.toString());
        expect(tokenMint_mSOL.mintAuthority.toString()).to.equal(playground.keychain.getKeypair('MOCK_ALL_MINT_AUTHORITY').publicKey.toString());
        expect(tokenMint_jitoSOL.mintAuthority.toString()).to.equal(playground.keychain.getKeypair('MOCK_ALL_MINT_AUTHORITY').publicKey.toString());
    });

    it("Should initialize fund, reward accounts and transfer token mint authority to PDA", async function() {
        const { fragSOLMint, fragSOLFundAccount, fragSOLRewardAccount } = await playground.runInitializeFragSOLFundAndRewardAccounts();

        expect(fragSOLFundAccount.dataVersion).to.gt(0);
        expect(fragSOLRewardAccount.dataVersion).to.equal(parseInt(playground.getConstant('rewardAccountCurrentVersion')));
        expect(fragSOLMint.mintAuthority.toString()).to.equal(playground.knownAddress.fragSOLTokenMintAuthority.toString());
    });

    it("Should initialize fund and supported tokens configuration", async function() {
        const { fragSOLFund } = await playground.runInitializeFragSOLFundConfiguration();

        expect(fragSOLFund.supportedTokens.length).eq(Object.values(playground.supportedTokenMetadata).length);
        let i = 0;
        for (const v of Object.values(playground.supportedTokenMetadata)) {
            const supported = fragSOLFund.supportedTokens[i++];
            expect(supported.mint.toString()).to.eq(v.mint.toString());
            expect(supported.program.toString()).to.eq(v.program.toString());
            expect(supported.price.toNumber()).to.greaterThan(0);
            expect(supported.operationReservedAmount.toNumber()).to.eq(0);
        }
    });

    it("Should initialize reward pools and rewards", async function() {
        const { fragSOLReward } = await playground.runInitializeFragSOLRewardConfiguration();

        expect(fragSOLReward.dataVersion).to.eq(parseInt(playground.getConstant('rewardAccountCurrentVersion')));

        expect(fragSOLReward.numRewards).eq(Object.values(playground.rewardsMetadata).length);
        let i = 0;
        for (const v of Object.values(playground.rewardsMetadata)) {
            const reward = fragSOLReward.rewards1[i++];
            expect(playground.binToString(reward.name)).to.eq(v.name.toString());
            expect(playground.binToString(reward.description)).to.eq(v.description.toString());
        }

        expect(fragSOLReward.numRewardPools).eq(Object.values(playground.rewardPoolsMetadata).length);
        i = 0;
        for (const v of Object.values(playground.rewardPoolsMetadata)) {
            const pool = fragSOLReward.rewardPools1[i++];
            expect(playground.binToString(pool.name)).to.eq(v.name.toString());
        }
    });

    it("Should settle fPoint reward (zeroing)", async () => {
        await new Promise(resolve => setTimeout(resolve, 1000)); // wait for few slot elapsed
        const { fragSOLReward, reward, rewardPool } = await playground.runSettleFragSOLReward({
            poolName: 'bonus',
            rewardName: 'fPoint',
            amount: new anchor.BN(0),
        });
        expect(fragSOLReward.rewardPools1[rewardPool.id].numRewardSettlements).eq(1);
        expect(fragSOLReward.rewardPools1[rewardPool.id].rewardSettlements1[0].rewardId).eq(reward.id);
        expect(fragSOLReward.rewardPools1[rewardPool.id].rewardSettlements1[0].rewardPoolId).eq(rewardPool.id);
        expect(fragSOLReward.rewardPools1[rewardPool.id].rewardSettlements1[0].numSettlementBlocks).eq(1);
        expect(fragSOLReward.rewardPools1[rewardPool.id].rewardSettlements1[0].settledAmount.toNumber()).eq(0);
    });
});
