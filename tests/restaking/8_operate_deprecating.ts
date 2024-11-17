import {BN} from "@coral-xyz/anchor";
import {expect} from "chai";
import {step} from "mocha-steps";
import {restakingPlayground} from "../restaking";
import {getLogger} from '../../tools/lib';

const {logger, LOG_PAD_SMALL, LOG_PAD_LARGE} = getLogger("restaking");

module.exports = (i: number) => describe(`operate#${i}`, async () => {
    const restaking = await restakingPlayground;

    step("fund operation: staking, normalize, restaking", async function () {
        const fragSOLFund0 = await restaking.getFragSOLFundAccount();
        const fragSOLFundReserveAccountBalance0 = await restaking.getFragSOLFundReserveAccountBalance();
        const nSOLPool0 = await restaking.getNSOLTokenPoolAccount();
        const nSOLMint0 = await restaking.getNSOLTokenMint();
        const fragSOLFundNSOLBalance0 = await restaking.getFragSOLFundNSOLAccountBalance();
        const jitoVaultNSOLBalance0 = await restaking.getFragSOLJitoVaultNSOLAccountBalance();
        logger.info(`before: fundSupportedTokens=${fragSOLFund0.supportedTokens.map(v => v.operationReservedAmount.toString()).join(', ')}, `
            +`fundSolOperationReservedAmount=${fragSOLFund0.solOperationReservedAmount}, fundReservedAmount=${fragSOLFundReserveAccountBalance0}, `
            +`nSOLSupportedTokens=${nSOLPool0.supportedTokens.map(v => v.lockedAmount.toString()).join(', ')}, nSOLOperationReservedAmount=?, nSOLSupply=${nSOLMint0.supply.toString()}, `
            +`fragSOLFundNSOL=${fragSOLFundNSOLBalance0.toString()}, jitoVaultNSOL=${jitoVaultNSOLBalance0.toString()}`
        );

        // TODO: currently staking sol to hard-coded LST, like localnet: jitoSOL
        const jitoSolSupportedTokenAccount = restaking.knownAddress.fragSOLSupportedTokenAccount("jitoSOL");
        const jitoSolBalance0 = await restaking.connection.getTokenAccountBalance(jitoSolSupportedTokenAccount, "confirmed");
        expect(fragSOLFund0.supportedTokens.some(s => s.operationReservedAmount.toString() == jitoSolBalance0.value.amount.toString())).eq(true, 'supported ATA balance should be equal');

        const {
            fragSOLFundReserveAccountBalance: fragSOLFundReserveAccountBalance1,
            fragSOLFundNSOLAccountBalance: fragSOLFundNSOLBalance1,
            fragSOLJitoVaultNSOLAccountBalance: jitoVaultNSOLBalance1,
            fragSOLFund: fragSOLFund1,
            nSOLTokenPool: nSOLTokenPool1,
        } = await restaking.runOperatorDeprecatingRun(restaking.keychain.getKeypair('ADMIN'));
        const nSOLJitoSOLBalance1 = await restaking.getNSOLSupportedTokenLockAccountBalance('jitoSOL');
        const nSOLMint1 = await restaking.getNSOLTokenMint();
        logger.info(
            `after: fundSupportedTokens=${fragSOLFund1.supportedTokens.map(v => v.operationReservedAmount.toString()).join(', ')}, fundSolOperationReservedAmount=${fragSOLFund1.solOperationReservedAmount}, fundReservedAmount=${fragSOLFundReserveAccountBalance1}, `+
            +`nSOLSupportedTokens=${nSOLTokenPool1.supportedTokens.map(v => v.lockedAmount.toString()).join(', ')}, nSOLSupply=${nSOLMint1.supply.toString()}, `
            +`fragSOLFundNSOL=${fragSOLFundNSOLBalance1.toString()}, jitoVaultNSOL=${jitoVaultNSOLBalance1.toString()}`
        );

        expect(fragSOLFundReserveAccountBalance1.toString()).eq('0', 'operation reserved should be zero after operation');
        const fee = (x: bigint) => x - x * BigInt(999) / BigInt(1000);
        const nSOLMintedAmount = nSOLMint1.supply - nSOLMint0.supply;
        const restakedNSOLAmount = nSOLMintedAmount + BigInt(fragSOLFundNSOLBalance0.toNumber());
        const unstakingFeeAmount = fee(restakedNSOLAmount);
        const unstakedNSOLAmount = restakedNSOLAmount - unstakingFeeAmount;
        expect(fragSOLFundNSOLBalance1.toString()).eq(unstakedNSOLAmount.toString(), 'unstaked amount should be in fund');
        expect(jitoVaultNSOLBalance1.sub(jitoVaultNSOLBalance0).toString()).eq(unstakingFeeAmount.toString(), 'unstaking fee 0.1% should be in jito vault');

        const stakedSOLAmount = fragSOLFund0.solOperationReservedAmount.sub(fragSOLFund1.solOperationReservedAmount);
        expect(stakedSOLAmount.gt(new BN(0))).eq(true, 'executed amount should be greater than zero');

        const jitoSolBalance1 = await restaking.connection.getTokenAccountBalance(jitoSolSupportedTokenAccount, "confirmed");
        expect(fragSOLFund1.supportedTokens.some(s => s.operationReservedAmount.toString() == jitoSolBalance1.value.amount.toString())).eq(true, 'blabla3');
        const mintedLSTAmount = new BN(jitoSolBalance1.value.amount).sub(new BN(jitoSolBalance0.value.amount));
        expect(nSOLJitoSOLBalance1.gt(new BN(0))).eq(true, "nSOL's supported tokens should be greater than zero");
        // expect(mintedLSTAmount.gt(stakedSOLAmount.div(new BN(2)))).eq(true, 'minted supported tokens should be not too less than staked sol amount');

        expect(fragSOLFund1.solOperationReservedAmount.toString()).eq('0', "fund account's solOperationReservedAmount should be 0");
    });
});
