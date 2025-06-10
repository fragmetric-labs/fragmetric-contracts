import { getAddressDecoder } from '@solana/kit';
import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeFragSOL } from './fragsol.init';

describe('restaking.fragSOL test', async () => {
  const testCtx = initializeFragSOL(await createTestSuiteContext());

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  const { validator, feePayer, restaking, initializationTasks } = testCtx;
  const ctx = restaking.fragSOL;

  const [signer1, signer2, signer3] = await Promise.all([
    validator
      .newSigner('fragSOLDepositTestSigner1', 100_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
            100_000_000_000n
          ),
        ]);
        return signer;
      }),
    validator.newSigner('fragSOLDepositTestSigner2', 100_000_000_000n),
    validator
      .newSigner('fragSOLDepositTestSigner3', 100_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v',
            100_000_000_000n
          ),
        ]);
        return signer;
      }),
  ]);
  const user1 = ctx.user(signer1);
  const user2 = ctx.user(signer2);
  const user3 = ctx.user(signer3);

  /** 1. configuration **/
  test(`restaking.fragSOL initializationTasks snapshot`, async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test('restaking.fragSOL.resolve', async () => {
    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
        "__pricingSources": [
          {
            "address": "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb",
            "role": 0,
          },
          {
            "address": "8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC",
            "role": 0,
          },
          {
            "address": "Hr9pzexrBge3vgmBNRR8u42CNQgBXdHm4UkUN2DH4a7r",
            "role": 0,
          },
          {
            "address": "2aMLkB5p5gVvCwKkdSo5eZAL1WwhZbxezQr1wxiynRhq",
            "role": 0,
          },
          {
            "address": "8VpRhuxa7sUUepdY3kQiTmX9rS5vx4WgaXiAnXq4KCtr",
            "role": 0,
          },
          {
            "address": "stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi",
            "role": 0,
          },
          {
            "address": "9mhGNSPArRMHpLDMSmxAvuoizBqtBGqYdT8WGuqgxNdn",
            "role": 0,
          },
          {
            "address": "LUKAypUYCVCptMKuN7ug3NGyRFz6p3SvKLHEXudS56X",
            "role": 0,
          },
          {
            "address": "3wK2g8ZdzAH8FJ7PKr2RcvGh7V9VYson5hrVsJM5Lmws",
            "role": 0,
          },
          {
            "address": "BuMRVW5uUQqJmguCk4toGh7DB3CcJt6dk64JiUMdYS22",
            "role": 0,
          },
          {
            "address": "8iax3u8PEcP6VhBtLLG7QAoSrCp7fUbCJtmHPrqHxdas",
            "role": 0,
          },
          {
            "address": "ArAQfbzsdotoKB5jJcZa3ajQrrPcWr2YQoDAEAiFxJAC",
            "role": 0,
          },
          {
            "address": "Fu9BYC6tWBo1KMKaP3CFoKfRhqv9akmy3DuYwnCyWiyC",
            "role": 0,
          },
          {
            "address": "ECRqn7gaNASuvTyC5xfCUjehWZCSowMXstZiM5DNweyB",
            "role": 0,
          },
          {
            "address": "8Dv3hNYcEWEaa4qVx9BTN1Wfvtha1z8cWDUXb7KVACVe",
            "role": 0,
          },
          {
            "address": "GZDX5JYXDzCEDL3kybhjN7PSixL4ams3M2G4CvWmMmm5",
            "role": 0,
          },
          {
            "address": "HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S",
            "role": 0,
          },
          {
            "address": "GVqitNXDVx1PdG47PMNeNEoHSEnVNqybW7E8NckmSJ2R",
            "role": 0,
          },
        ],
        "depositResidualMicroReceiptTokenAmount": 0n,
        "metadata": null,
        "normalizedToken": {
          "mint": "4noNmx2RpxK4zdr68Fq1CYM5VhN4yjgGZEFyuB7t2pBX",
          "oneTokenAsSol": 0n,
          "operationReservedAmount": 0n,
          "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        },
        "oneReceiptTokenAsSOL": 0n,
        "receiptTokenDecimals": 9,
        "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
        "receiptTokenSupply": 0n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "CkXLPfDG3cDawtUvnztq99HdGoQWhJceBZxqKYL2TUrg",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": null,
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1000000000n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": null,
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1205735187n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1296734792n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "BNso1VUJnh4zcfpZa6986Ea66P6TCp59hvtNJ8b1X85",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1054746972n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "Bybit2vBJGhPF52GBdNaQfUJ6ZpThSgHBobjWZpLPb4B",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1084766427n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1112605262n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1220167026n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "Dso1bDeDjCQxTrWHqUUi63oBvV7Mdm6WaobLbQ7gnPQ",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1117005681n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "FRAGME9aN7qzxkHPmVP22tDhG87srsR9pr5SY9XdRd9R",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1001411968n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1095705782n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "roxDFxTFHufJBFy3PgzZcgz6kwkQNPZpi9RfpcAv4bu",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1209802749n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "sctmadV2fcLtrxjzYhTZzwAGjXUXKtYSBrrM36EtdcY",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1020253633n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1110727355n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1088470650n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "HUBsveNpjo5pWqNkH57QzxjQASdTVXcSK7bVKTSZtcSX",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1108013014n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "picobAEvs6w7QEknPce34wAE4gknZA9v5tTonnmHYdX",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1125085484n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1108758017n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q",
      }
    `);
  });

  test('restaking.fragSOL.fund.resolve', async () => {
    await expect(ctx.fund.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "assetStrategies": [
          {
            "solAccumulatedDepositAmount": 0n,
            "solAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "solDepositable": true,
            "solWithdrawable": true,
            "solWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "solWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "BNso1VUJnh4zcfpZa6986Ea66P6TCp59hvtNJ8b1X85",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "Bybit2vBJGhPF52GBdNaQfUJ6ZpThSgHBobjWZpLPb4B",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "Dso1bDeDjCQxTrWHqUUi63oBvV7Mdm6WaobLbQ7gnPQ",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "FRAGME9aN7qzxkHPmVP22tDhG87srsR9pr5SY9XdRd9R",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "roxDFxTFHufJBFy3PgzZcgz6kwkQNPZpi9RfpcAv4bu",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "sctmadV2fcLtrxjzYhTZzwAGjXUXKtYSBrrM36EtdcY",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "HUBsveNpjo5pWqNkH57QzxjQASdTVXcSK7bVKTSZtcSX",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "picobAEvs6w7QEknPce34wAE4gknZA9v5tTonnmHYdX",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
          {
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "tokenAccumulatedDepositAmount": 0n,
            "tokenAccumulatedDepositCapacityAmount": 18446744073709551615n,
            "tokenDepositable": true,
            "tokenMint": "strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA",
            "tokenRebalancingAmount": 0n,
            "tokenWithdrawable": false,
            "tokenWithdrawalNormalReserveMaxAmount": 18446744073709551615n,
            "tokenWithdrawalNormalReserveRateBps": 0,
          },
        ],
        "generalStrategy": {
          "depositEnabled": true,
          "donationEnabled": true,
          "operationEnabled": true,
          "transferEnabled": true,
          "withdrawalBatchThresholdSeconds": 1n,
          "withdrawalEnabled": true,
          "withdrawalFeeRateBps": 20,
        },
        "restakingVaultStrategies": [
          {
            "compoundingRewardTokens": [
              {
                "harvestThresholdIntervalSeconds": 0n,
                "harvestThresholdMaxAmount": 18446744073709551615n,
                "harvestThresholdMinAmount": 0n,
                "lastHarvestedAt": 0n,
                "mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
              },
            ],
            "delegations": [
              {
                "operator": "FzZ9EXmHv7ANCXijpALUBzCza6wYNprnsfaEHuoNx9sE",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 92n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "29rxXT5zbTR1ctiooHtb1Sa1TD4odzhQHsrLz3D78G5w",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "LKFpfXtBkH5b7D9mo8dPcjCLZCZpmLQC9ELkbkyVdah",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 92n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "GZxp4e2Tm3Pw9GyAaxuF6odT3XkRM96jpZkp3nxhoK4Y",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 2n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "CA8PaNSoFWzvbCJ2oK3QxBEutgyHSTT5omEptpj8YHPY",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 3n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "7yofWXChEHkPTSnyFdKx2Smq5iWVbGB4P1dkdC6zHWYR",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 1n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "BFEsrxFPsBcY2hR5kgyfKnpwgEc8wYQdngvRukLQXwG2",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 4n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "2sHNuid4rus4sK2EmndLeZcPNKkgzuEoc8Vro3PH2qop",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "5TGRFaLy3eF93pSNiPamCgvZUN3gzdYcs7jA3iCAsd1L",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 92n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "EkroMQiZJfphVd9iPvR4zMCHasTW72Uh1mFYkTxtQuY6",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "574DmorRvpaYrSrBRUwAjG7bBmrZYiTW3Fc8mvQatFqo",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 92n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "C6AF8qGCo2dL815ziRCmfdbFeL5xbRLuSTSZzTGBH68y",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
                "tokenRedelegatingAmount": 0n,
              },
              {
                "operator": "6AxtdRGAaiAyqcwxVBHsH3xtqCbQuffaiE4epT4koTxk",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
                "tokenRedelegatingAmount": 0n,
              },
            ],
            "distributingRewardTokens": [],
            "pricingSource": {
              "__kind": "JitoRestakingVault",
              "address": "HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S",
            },
            "solAllocationCapacityAmount": 18446744073709551615n,
            "solAllocationWeight": 1n,
            "vault": "HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S",
          },
        ],
        "tokenSwapStrategies": [],
      }
    `);
  });

  test(`restaking.fragSOL.reward.resolve`, async () => {
    await expectMasked(ctx.reward.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      {
        "basePool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "customContributionAccrualRateEnabled": false,
          "initialSlot": "MASKED(/[.*S|s]lots?$/)",
          "settlements": [
            {
              "blocks": [
                {
                  "amount": 0n,
                  "endingContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "endingSlot": "MASKED(/[.*S|s]lots?$/)",
                  "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "settledSlots": "MASKED(/[.*S|s]lots?$/)",
                  "startingContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "startingSlot": "MASKED(/[.*S|s]lots?$/)",
                  "userSettledAmount": 0n,
                  "userSettledContribution": "MASKED(/[.*C|c]ontribution?$/)",
                },
              ],
              "claimedAmount": 0n,
              "claimedAmountUpdatedSlot": "MASKED(/[.*S|s]lots?$/)",
              "remainingAmount": 0n,
              "reward": {
                "claimable": false,
                "decimals": 9,
                "description": "Switchboard Token",
                "id": 1,
                "mint": "FSWSBMV5EB7J8JdafNBLZpfSCLiFwpMCqod2RpkU4RNn",
                "name": "SWTCH",
                "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
              },
              "settledAmount": 0n,
              "settlementBlocksLastRewardPoolContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settlementBlocksLastSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [],
            "totalAmount": 0n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "bonusPool": {
          "contribution": "MASKED(/[.*C|c]ontribution?$/)",
          "customContributionAccrualRateEnabled": true,
          "initialSlot": "MASKED(/[.*S|s]lots?$/)",
          "settlements": [
            {
              "blocks": [
                {
                  "amount": 0n,
                  "endingContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "endingSlot": "MASKED(/[.*S|s]lots?$/)",
                  "settledContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "settledSlots": "MASKED(/[.*S|s]lots?$/)",
                  "startingContribution": "MASKED(/[.*C|c]ontribution?$/)",
                  "startingSlot": "MASKED(/[.*S|s]lots?$/)",
                  "userSettledAmount": 0n,
                  "userSettledContribution": "MASKED(/[.*C|c]ontribution?$/)",
                },
              ],
              "claimedAmount": 0n,
              "claimedAmountUpdatedSlot": "MASKED(/[.*S|s]lots?$/)",
              "remainingAmount": 0n,
              "reward": {
                "claimable": false,
                "decimals": 4,
                "description": "Airdrop point for fToken",
                "id": 0,
                "mint": "11111111111111111111111111111111",
                "name": "fPoint",
                "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
              },
              "settledAmount": 0n,
              "settlementBlocksLastRewardPoolContribution": "MASKED(/[.*C|c]ontribution?$/)",
              "settlementBlocksLastSlot": "MASKED(/[.*S|s]lots?$/)",
            },
          ],
          "tokenAllocatedAmount": {
            "records": [],
            "totalAmount": 0n,
          },
          "updatedSlot": "MASKED(/[.*S|s]lots?$/)",
        },
        "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
        "rewards": [
          {
            "claimable": false,
            "decimals": 4,
            "description": "Airdrop point for fToken",
            "id": 0,
            "mint": "11111111111111111111111111111111",
            "name": "fPoint",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          },
          {
            "claimable": false,
            "decimals": 9,
            "description": "Switchboard Token",
            "id": 1,
            "mint": "FSWSBMV5EB7J8JdafNBLZpfSCLiFwpMCqod2RpkU4RNn",
            "name": "SWTCH",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
          },
        ],
      }
    `);
  });

  test.skip('pricing source addresses field in fund account updates correctly', async () => {
    await expectMasked(ctx.fund.updatePrices.execute(null)).resolves
      .toMatchInlineSnapshot(`
      {
        "args": null,
        "events": {
          "operatorUpdatedFundPrices": {
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
          },
          "unknown": [],
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    // 1) pricing_sourece_addresses field of fund account has correct data
    let fundAccount = await ctx.fund.resolveAccount(true);
    let normalizedTokenPool =
      await ctx.normalizedTokenPool.resolveAddress(true);

    // get pricing sources from fund account manually
    const getPricingSourcesManually = () => {
      const data = fundAccount!.data;
      const supportedTokens = data.supportedTokens.slice(
        0,
        data.numSupportedTokens
      );
      const restakingVaults = data.restakingVaults.slice(
        0,
        data.numRestakingVaults
      );

      return supportedTokens
        .filter(
          (v) => v.pricingSource.discriminant != 8 // skip pegged token
        ) // skip pegged token
        .map((v) => v.pricingSource.address)
        .concat(restakingVaults.map((v) => v.receiptTokenPricingSource.address))
        .concat(normalizedTokenPool ? [normalizedTokenPool] : []);
    };

    // get pricing sources from fund account field (new feature)
    const getPricingSourcesByField = () => {
      const data = fundAccount!.data;
      return data.pricingSourceAddresses.slice(
        0,
        data.numPricingSourceAddresses
      );
    };
    expect(getPricingSourcesManually()).toEqual(getPricingSourcesByField());

    // 2) user can get pricing_source_addresses by parsing fund account
    // - num_pricing_source_addresses offset: 0x9000
    // - pricing_source_addresses offset: 0x9001
    const fetchedAccount = await ctx.runtime.fetchAccount(fundAccount!.address);
    const byteData = fetchedAccount!.data;

    const encodedNumPricingSourceAddresses = byteData.slice(0x9000, 0x9001);
    const numPricingSourceAddresses = Buffer.from(
      encodedNumPricingSourceAddresses
    ).readUInt8(0);
    expect(numPricingSourceAddresses).toEqual(
      fundAccount!.data.numPricingSourceAddresses
    );

    const ADDRESS_SIZE = 32;
    const MAX_PRICING_SOURCE_ADDRESSES = 33;
    const pricingSourceAddresses: string[] = [];
    const encodedPricingSourceAddresses = byteData.slice(
      0x9001,
      0x9001 + ADDRESS_SIZE * MAX_PRICING_SOURCE_ADDRESSES
    );

    for (
      let offset = 0;
      offset < ADDRESS_SIZE * MAX_PRICING_SOURCE_ADDRESSES;
      offset += ADDRESS_SIZE
    ) {
      const chunk = encodedPricingSourceAddresses.slice(
        offset,
        offset + ADDRESS_SIZE
      );
      const address = getAddressDecoder().decode(chunk);
      pricingSourceAddresses.push(address);
    }
    expect(pricingSourceAddresses).toEqual(
      fundAccount!.data.pricingSourceAddresses
    );

    const prevNumPricingSourceAddresses = numPricingSourceAddresses;

    // 3) pricing_source_addresses updates correctly after calling add_supported_token ix & remove_supported_token
    // 3-1) add bSol as supported token
    await ctx.fund.addSupportedToken.execute({
      mint: 'bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1',
      pricingSource: {
        __kind: 'SPLStakePool',
        address: 'stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi',
      },
    });

    await ctx.normalizedTokenPool.addSupportedToken.execute({
      mint: 'bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1',
      pricingSource: {
        __kind: 'SPLStakePool',
        address: 'stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi',
      },
    });
    fundAccount = await ctx.fund.resolveAccount(true);
    normalizedTokenPool = await ctx.normalizedTokenPool.resolveAddress(true);
    expect(getPricingSourcesManually()).toEqual(getPricingSourcesByField());
    expect(fundAccount!.data.numPricingSourceAddresses - 1).toEqual(
      prevNumPricingSourceAddresses
    );

    // 3-2) remove bSol from supported tokens
    await ctx.normalizedTokenPool.removeSupportedToken.execute({
      mint: 'bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1',
    });

    await ctx.fund.removeSupportedToken.execute({
      mint: 'bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1',
    });

    fundAccount = await ctx.fund.resolveAccount(true);
    normalizedTokenPool = await ctx.normalizedTokenPool.resolveAddress(true);
    expect(getPricingSourcesManually()).toEqual(getPricingSourcesByField());
    expect(fundAccount!.data.numPricingSourceAddresses).toEqual(
      prevNumPricingSourceAddresses
    );
  });

  test.skip('remove supported tokens', async () => {
    await expect(
      ctx.fund.addSupportedToken.execute({
        mint: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
        pricingSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: '4yp9YAXCJsKWMDZq2Q4j4amktvJGXBCpr3Lmv2cYBrb8',
        },
      })
    ).resolves.not.toThrow();
    await expect(
      ctx.normalizedTokenPool.addSupportedToken.execute({
        mint: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
        pricingSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: '4yp9YAXCJsKWMDZq2Q4j4amktvJGXBCpr3Lmv2cYBrb8',
        },
      })
    ).resolves.not.toThrow();

    await expect(
      ctx.fund.addSupportedToken.execute({
        mint: 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
        pricingSource: {
          __kind: 'PeggedToken',
          address: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
        },
      })
    ).resolves.not.toThrow();
    await expect(
      ctx.normalizedTokenPool.addSupportedToken.execute({
        mint: 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
        pricingSource: {
          __kind: 'PeggedToken',
          address: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
        },
      })
    ).resolves.not.toThrow();

    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
        "__pricingSources": [
          {
            "address": "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb",
            "role": 0,
          },
          {
            "address": "8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC",
            "role": 0,
          },
          {
            "address": "Hr9pzexrBge3vgmBNRR8u42CNQgBXdHm4UkUN2DH4a7r",
            "role": 0,
          },
          {
            "address": "2aMLkB5p5gVvCwKkdSo5eZAL1WwhZbxezQr1wxiynRhq",
            "role": 0,
          },
          {
            "address": "8VpRhuxa7sUUepdY3kQiTmX9rS5vx4WgaXiAnXq4KCtr",
            "role": 0,
          },
          {
            "address": "4yp9YAXCJsKWMDZq2Q4j4amktvJGXBCpr3Lmv2cYBrb8",
            "role": 0,
          },
          {
            "address": "HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S",
            "role": 0,
          },
          {
            "address": "GVqitNXDVx1PdG47PMNeNEoHSEnVNqybW7E8NckmSJ2R",
            "role": 0,
          },
        ],
        "depositResidualMicroReceiptTokenAmount": 0n,
        "metadata": null,
        "normalizedToken": {
          "mint": "4noNmx2RpxK4zdr68Fq1CYM5VhN4yjgGZEFyuB7t2pBX",
          "oneTokenAsSol": 0n,
          "operationReservedAmount": 0n,
          "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        },
        "oneReceiptTokenAsSOL": 0n,
        "receiptTokenDecimals": 9,
        "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
        "receiptTokenSupply": 0n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "CkXLPfDG3cDawtUvnztq99HdGoQWhJceBZxqKYL2TUrg",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": null,
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1000000000n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": null,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1160715954n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "Bybit2vBJGhPF52GBdNaQfUJ6ZpThSgHBobjWZpLPb4B",
            "oneTokenAsReceiptToken": 0n,
            "oneTokenAsSol": 1048076503n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q",
      }
    `);

    // start remove
    // failed because used by other pegged token
    await expect(
      ctx.normalizedTokenPool.removeSupportedToken.execute({
        mint: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
      })
    ).rejects.toThrow();
    await expect(
      ctx.fund.removeSupportedToken.execute({
        mint: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
      })
    ).rejects.toThrow();

    // failed because used by ntp
    await expect(
      ctx.fund.removeSupportedToken.execute({
        mint: 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
      })
    ).rejects.toThrow();

    // success
    await expect(
      ctx.normalizedTokenPool.removeSupportedToken.execute({
        mint: 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
      })
    ).resolves.not.toThrow();
    await expect(
      ctx.fund.removeSupportedToken.execute({
        mint: 'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
      })
    ).resolves.not.toThrow();
    await expect(
      ctx.normalizedTokenPool.removeSupportedToken.execute({
        mint: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
      })
    ).resolves.not.toThrow();
    await expect(
      ctx.fund.removeSupportedToken.execute({
        mint: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
      })
    ).resolves.not.toThrow();
  });

  /** 2. deposit **/
  test('user can deposit SOL', async () => {
    await expectMasked(
      user1.deposit.execute(
        { assetMint: null, assetAmount: 5_000_000_000n },
        { signers: [signer1] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 5000000000n,
          "assetMint": null,
          "metadata": null,
        },
        "events": {
          "unknown": [],
          "userCreatedOrUpdatedFundAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "user": "EhxcijcPKVdQ9zTSXGeLrgSEFJjbiNiC34j9prg3St29",
            "userFundAccount": "47srXvirv37rsKhrVxtz7JWGq4CE2Ao4vjFUvTNdvg92",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "user": "EhxcijcPKVdQ9zTSXGeLrgSEFJjbiNiC34j9prg3St29",
            "userRewardAccount": "9XZgibwtji6havXCPHKRoqpnR7MJUYgavQKCvDWALXGR",
          },
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 5000000000n,
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "mintedReceiptTokenAmount": 5000000000n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "supportedTokenMint": {
              "__option": "None",
            },
            "updatedUserRewardAccounts": [
              "9XZgibwtji6havXCPHKRoqpnR7MJUYgavQKCvDWALXGR",
            ],
            "user": "EhxcijcPKVdQ9zTSXGeLrgSEFJjbiNiC34j9prg3St29",
            "userFundAccount": "47srXvirv37rsKhrVxtz7JWGq4CE2Ao4vjFUvTNdvg92",
            "userReceiptTokenAccount": "BWfL432qksE6DpBEpRsuqaq4U6GdgPz1PGXKXNQkfr8M",
            "userSupportedTokenAccount": {
              "__option": "None",
            },
            "walletProvider": {
              "__option": "None",
            },
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    await expect(
      user1.receiptToken.resolve(true).then((res) => res?.amount)
    ).resolves.toEqual(5000000000n);

    await expect(user1.resolve()).resolves.toMatchInlineSnapshot(`
      {
        "lamports": 94962596960n,
        "maxWithdrawalRequests": 4,
        "receiptTokenAmount": 5000000000n,
        "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
        "supportedAssets": [
          {
            "amount": 94962596960n,
            "decimals": 9,
            "depositable": true,
            "mint": null,
            "program": null,
            "withdrawable": true,
          },
          {
            "amount": 100000000000n,
            "decimals": 9,
            "depositable": true,
            "mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "BNso1VUJnh4zcfpZa6986Ea66P6TCp59hvtNJ8b1X85",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "Bybit2vBJGhPF52GBdNaQfUJ6ZpThSgHBobjWZpLPb4B",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "Dso1bDeDjCQxTrWHqUUi63oBvV7Mdm6WaobLbQ7gnPQ",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "FRAGME9aN7qzxkHPmVP22tDhG87srsR9pr5SY9XdRd9R",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "roxDFxTFHufJBFy3PgzZcgz6kwkQNPZpi9RfpcAv4bu",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "sctmadV2fcLtrxjzYhTZzwAGjXUXKtYSBrrM36EtdcY",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "HUBsveNpjo5pWqNkH57QzxjQASdTVXcSK7bVKTSZtcSX",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "picobAEvs6w7QEknPce34wAE4gknZA9v5tTonnmHYdX",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
        ],
        "user": "EhxcijcPKVdQ9zTSXGeLrgSEFJjbiNiC34j9prg3St29",
        "withdrawalRequests": [],
        "wrappedTokenAmount": 0n,
        "wrappedTokenMint": "h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q",
      }
    `);
  });

  test('user can deposit token with SPLStakePool pricing source', async () => {
    await expectMasked(
      user1.deposit.execute(
        {
          assetMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
          assetAmount: 5_000_000_000n,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 5000000000n,
          "assetMint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
          "metadata": null,
        },
        "events": {
          "unknown": [],
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 5000000000n,
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "mintedReceiptTokenAmount": 6028675939n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            },
            "updatedUserRewardAccounts": [
              "9XZgibwtji6havXCPHKRoqpnR7MJUYgavQKCvDWALXGR",
            ],
            "user": "EhxcijcPKVdQ9zTSXGeLrgSEFJjbiNiC34j9prg3St29",
            "userFundAccount": "47srXvirv37rsKhrVxtz7JWGq4CE2Ao4vjFUvTNdvg92",
            "userReceiptTokenAccount": "BWfL432qksE6DpBEpRsuqaq4U6GdgPz1PGXKXNQkfr8M",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "4uGht5ZgiTn77KERTVtMm4WpxTeztWmpxgXhkNYBbXcQ",
            },
            "walletProvider": {
              "__option": "None",
            },
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    await expect(
      user1.receiptToken.resolve(true).then((res) => res?.amount)
    ).resolves.toBeGreaterThan(5000000000n * 2n);

    await expect(user1.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "lamports": 94962596960n,
        "maxWithdrawalRequests": 4,
        "receiptTokenAmount": 11028675939n,
        "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
        "supportedAssets": [
          {
            "amount": 94962596960n,
            "decimals": 9,
            "depositable": true,
            "mint": null,
            "program": null,
            "withdrawable": true,
          },
          {
            "amount": 95000000000n,
            "decimals": 9,
            "depositable": true,
            "mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "BNso1VUJnh4zcfpZa6986Ea66P6TCp59hvtNJ8b1X85",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "Bybit2vBJGhPF52GBdNaQfUJ6ZpThSgHBobjWZpLPb4B",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "Dso1bDeDjCQxTrWHqUUi63oBvV7Mdm6WaobLbQ7gnPQ",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "FRAGME9aN7qzxkHPmVP22tDhG87srsR9pr5SY9XdRd9R",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "roxDFxTFHufJBFy3PgzZcgz6kwkQNPZpi9RfpcAv4bu",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "sctmadV2fcLtrxjzYhTZzwAGjXUXKtYSBrrM36EtdcY",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "HUBsveNpjo5pWqNkH57QzxjQASdTVXcSK7bVKTSZtcSX",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "picobAEvs6w7QEknPce34wAE4gknZA9v5tTonnmHYdX",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
          {
            "amount": 0n,
            "decimals": 9,
            "depositable": true,
            "mint": "strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA",
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "withdrawable": false,
          },
        ],
        "user": "EhxcijcPKVdQ9zTSXGeLrgSEFJjbiNiC34j9prg3St29",
        "withdrawalRequests": [],
        "wrappedTokenAmount": 0n,
        "wrappedTokenMint": "h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q",
      }
    `);

    await expectMasked(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "__lookupTableAddress": "MASKED(__lookupTableAddress)",
        "__pricingSources": [
          {
            "address": "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb",
            "role": 0,
          },
          {
            "address": "8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC",
            "role": 0,
          },
          {
            "address": "Hr9pzexrBge3vgmBNRR8u42CNQgBXdHm4UkUN2DH4a7r",
            "role": 0,
          },
          {
            "address": "2aMLkB5p5gVvCwKkdSo5eZAL1WwhZbxezQr1wxiynRhq",
            "role": 0,
          },
          {
            "address": "8VpRhuxa7sUUepdY3kQiTmX9rS5vx4WgaXiAnXq4KCtr",
            "role": 0,
          },
          {
            "address": "stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi",
            "role": 0,
          },
          {
            "address": "9mhGNSPArRMHpLDMSmxAvuoizBqtBGqYdT8WGuqgxNdn",
            "role": 0,
          },
          {
            "address": "LUKAypUYCVCptMKuN7ug3NGyRFz6p3SvKLHEXudS56X",
            "role": 0,
          },
          {
            "address": "3wK2g8ZdzAH8FJ7PKr2RcvGh7V9VYson5hrVsJM5Lmws",
            "role": 0,
          },
          {
            "address": "BuMRVW5uUQqJmguCk4toGh7DB3CcJt6dk64JiUMdYS22",
            "role": 0,
          },
          {
            "address": "8iax3u8PEcP6VhBtLLG7QAoSrCp7fUbCJtmHPrqHxdas",
            "role": 0,
          },
          {
            "address": "ArAQfbzsdotoKB5jJcZa3ajQrrPcWr2YQoDAEAiFxJAC",
            "role": 0,
          },
          {
            "address": "Fu9BYC6tWBo1KMKaP3CFoKfRhqv9akmy3DuYwnCyWiyC",
            "role": 0,
          },
          {
            "address": "ECRqn7gaNASuvTyC5xfCUjehWZCSowMXstZiM5DNweyB",
            "role": 0,
          },
          {
            "address": "8Dv3hNYcEWEaa4qVx9BTN1Wfvtha1z8cWDUXb7KVACVe",
            "role": 0,
          },
          {
            "address": "GZDX5JYXDzCEDL3kybhjN7PSixL4ams3M2G4CvWmMmm5",
            "role": 0,
          },
          {
            "address": "HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S",
            "role": 0,
          },
          {
            "address": "GVqitNXDVx1PdG47PMNeNEoHSEnVNqybW7E8NckmSJ2R",
            "role": 0,
          },
        ],
        "depositResidualMicroReceiptTokenAmount": 727359n,
        "metadata": null,
        "normalizedToken": {
          "mint": "4noNmx2RpxK4zdr68Fq1CYM5VhN4yjgGZEFyuB7t2pBX",
          "oneTokenAsSol": 0n,
          "operationReservedAmount": 0n,
          "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        },
        "oneReceiptTokenAsSOL": 1000000000n,
        "receiptTokenDecimals": 9,
        "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
        "receiptTokenSupply": 11028675939n,
        "restakingVaultReceiptTokens": [
          {
            "mint": "CkXLPfDG3cDawtUvnztq99HdGoQWhJceBZxqKYL2TUrg",
            "oneReceiptTokenAsSol": 0n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "vault": "HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S",
          },
        ],
        "supportedAssets": [
          {
            "decimals": 9,
            "depositable": true,
            "mint": null,
            "oneTokenAsReceiptToken": 1000000000n,
            "oneTokenAsSol": 1000000000n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 5000000000n,
            "operationTotalAmount": 5000000000n,
            "program": null,
            "unstakingAmountAsSOL": 0n,
            "withdrawable": true,
            "withdrawableValueAsReceiptTokenAmount": 11028675939n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "oneTokenAsReceiptToken": 1205735187n,
            "oneTokenAsSol": 1205735187n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 5000000000n,
            "operationTotalAmount": 5000000000n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 6028675939n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
            "oneTokenAsReceiptToken": 1296734792n,
            "oneTokenAsSol": 1296734792n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "BNso1VUJnh4zcfpZa6986Ea66P6TCp59hvtNJ8b1X85",
            "oneTokenAsReceiptToken": 1054746972n,
            "oneTokenAsSol": 1054746972n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "Bybit2vBJGhPF52GBdNaQfUJ6ZpThSgHBobjWZpLPb4B",
            "oneTokenAsReceiptToken": 1084766427n,
            "oneTokenAsSol": 1084766427n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
            "oneTokenAsReceiptToken": 1112605262n,
            "oneTokenAsSol": 1112605262n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1",
            "oneTokenAsReceiptToken": 1220167026n,
            "oneTokenAsSol": 1220167026n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "Dso1bDeDjCQxTrWHqUUi63oBvV7Mdm6WaobLbQ7gnPQ",
            "oneTokenAsReceiptToken": 1117005680n,
            "oneTokenAsSol": 1117005681n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "FRAGME9aN7qzxkHPmVP22tDhG87srsR9pr5SY9XdRd9R",
            "oneTokenAsReceiptToken": 1001411968n,
            "oneTokenAsSol": 1001411968n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A",
            "oneTokenAsReceiptToken": 1095705782n,
            "oneTokenAsSol": 1095705782n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "roxDFxTFHufJBFy3PgzZcgz6kwkQNPZpi9RfpcAv4bu",
            "oneTokenAsReceiptToken": 1209802749n,
            "oneTokenAsSol": 1209802749n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "sctmadV2fcLtrxjzYhTZzwAGjXUXKtYSBrrM36EtdcY",
            "oneTokenAsReceiptToken": 1020253633n,
            "oneTokenAsSol": 1020253633n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs",
            "oneTokenAsReceiptToken": 1110727355n,
            "oneTokenAsSol": 1110727355n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7",
            "oneTokenAsReceiptToken": 1088470650n,
            "oneTokenAsSol": 1088470650n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "HUBsveNpjo5pWqNkH57QzxjQASdTVXcSK7bVKTSZtcSX",
            "oneTokenAsReceiptToken": 1108013014n,
            "oneTokenAsSol": 1108013014n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "picobAEvs6w7QEknPce34wAE4gknZA9v5tTonnmHYdX",
            "oneTokenAsReceiptToken": 1125085484n,
            "oneTokenAsSol": 1125085484n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
          {
            "decimals": 9,
            "depositable": true,
            "mint": "strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA",
            "oneTokenAsReceiptToken": 1108758017n,
            "oneTokenAsSol": 1108758017n,
            "operationReceivableAmount": 0n,
            "operationReservedAmount": 0n,
            "operationTotalAmount": 0n,
            "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "unstakingAmountAsSOL": 0n,
            "withdrawable": false,
            "withdrawableValueAsReceiptTokenAmount": 0n,
            "withdrawalLastBatchProcessedAt": "MASKED(/.*At?$/)",
            "withdrawalResidualMicroAssetAmount": 0n,
            "withdrawalUserReservedAmount": 0n,
          },
        ],
        "wrappedTokenMint": "h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q",
      }
    `);
  });

  /** 3. Withdraw **/
  test('user can withdraw receipt tokens as SOL', async () => {
    let {
      receiptTokenSupply: expectedReceiptTokenSupply,
      oneReceiptTokenAsSOL,
    } = await ctx.resolve(true).then((data) => data!);

    for (let i = 1; i <= 4; i++) {
      const receiptTokenAmount = 23_456_789n * BigInt(i);
      expectedReceiptTokenSupply -= receiptTokenAmount;

      await expect(
        user1.requestWithdrawal.execute(
          {
            assetMint: null,
            receiptTokenAmount: receiptTokenAmount,
          },
          { signers: [signer1] }
        )
      ).resolves.toMatchObject({
        events: {
          userRequestedWithdrawalFromFund: {
            supportedTokenMint: { __option: 'None' },
            requestedReceiptTokenAmount: receiptTokenAmount,
          },
        },
      });
    }
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'EnqueueWithdrawalBatch',
    });
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'ProcessWithdrawalBatch',
    });
    await expect(
      ctx.fund
        .resolveAccount(true)
        .then((account) => account?.data.sol.withdrawalLastProcessedBatchId)
    ).resolves.toEqual(1n);

    for (let i = 1; i <= 4; i++) {
      const res = await user1.withdraw.execute(
        {
          assetMint: null,
          requestId: BigInt(i),
        },
        { signers: [signer1] }
      );
      const evt = res.events!.userWithdrewFromFund!;
      expect(
        evt.burntReceiptTokenAmount,
        'burntReceiptTokenAmount = withdrawnAmount + deductedFeeAmount + [optional remainder]'
      ).toBeOneOf([
        evt.withdrawnAmount + evt.deductedFeeAmount,
        evt.withdrawnAmount + evt.deductedFeeAmount + 1n,
      ]);
    }

    await expect(
      ctx.resolve(true),
      'receipt token supply reduced as withdrawal reqs being processed but the price maintains'
    ).resolves.toMatchObject({
      receiptTokenSupply: expectedReceiptTokenSupply,
      oneReceiptTokenAsSOL: oneReceiptTokenAsSOL,
    });
  });

  /** 4. Wrapped Token Holder **/
  test('fund manager can add wrapped token holder', async () => {
    const fundWrap = ctx.fund.wrap;
    const fundWrapReward = ctx.fund.wrap.reward;

    await user3.deposit.execute(
      { assetMint: null, assetAmount: 20_000_000_000n },
      { signers: [signer3] }
    );
    await user3.wrap.execute(
      { receiptTokenAmount: 10_000_000_000n },
      { signers: [signer3] }
    );
    await expect(
      user3.receiptToken.resolve(true).then((res) => res?.amount)
    ).resolves.toBeOneOf([10_000_000_000n, 10_000_000_000n - 1n]);
    await expect(
      user3.wrappedToken.resolve(true).then((res) => res?.amount)
    ).resolves.toEqual(10_000_000_000n);
    await expect(
      fundWrap.resolve(true).then((res) => res!.retainedAmount)
    ).resolves.toEqual(10_000_000_000n);
    await expect(
      fundWrapReward
        .resolve(true)
        .then((res) => res!.basePool.tokenAllocatedAmount.totalAmount)
    ).resolves.toEqual(10_000_000_000n);

    // user3's wrapped token account as holder
    await expectMasked(
      ctx.fund.initializeWrappedTokenHolder.execute({
        wrappedTokenAccount: user3.wrappedToken.address!,
      })
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "wrappedTokenAccount": "Hi1AHmGBCSpWwM3LL1E6RzoTx7aRd9ZuWcHSNxdyRcF8",
        },
        "events": {
          "fundManagerUpdatedFund": {
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
          },
          "unknown": [],
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "user": "Hi1AHmGBCSpWwM3LL1E6RzoTx7aRd9ZuWcHSNxdyRcF8",
            "userRewardAccount": "F8gsBkeQCLmGyXk8CMhNNcJh8X6mpJGkXXTfwWqL35GH",
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    await expectMasked(fundWrap.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "holders": [
          {
            "amount": 0n,
            "tokenAccount": "Hi1AHmGBCSpWwM3LL1E6RzoTx7aRd9ZuWcHSNxdyRcF8",
          },
        ],
        "retainedAmount": 10000000000n,
        "wrappedAmount": 10000000000n,
        "wrappedToken": {
          "decimals": 9,
          "mint": "h7veGmqGWmFPe2vbsrKVNARvucfZ2WKCXUvJBmbJ86Q",
          "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        },
      }
    `);
    await expect(
      fundWrap.holders.resolve(true)?.then((res) => res.length)
    ).resolves.toEqual(1);
    expect(fundWrap.holders.children[0]!.address).toEqual(
      user3.wrappedToken.address
    );

    const holderReward = fundWrap.holders.children[0]!.reward;
    await expect(
      holderReward.resolve().then((res) => res!.delegate)
    ).resolves.toEqual(ctx.parent.knownAddresses.fundManager);
    await expect(
      holderReward
        .resolve()
        .then((res) => res!.basePool.tokenAllocatedAmount.totalAmount)
    ).resolves.toEqual(0n);
  });

  test('wrapped token holder amount is updated by operator', async () => {
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'Initialize',
    });

    await expect(
      ctx.fund.wrap.resolve(true).then((res) => res!.retainedAmount)
    ).resolves.toEqual(0n);
    await expect(
      ctx.fund.wrap.reward
        .resolve(true)
        .then((res) => res!.basePool.tokenAllocatedAmount.totalAmount)
    ).resolves.toEqual(0n);
    await expect(
      ctx.fund.wrap.holders.children[0]!.reward.resolve(true).then(
        (res) => res!.basePool.tokenAllocatedAmount.totalAmount
      )
    ).resolves.toEqual(10_000_000_000n);
  });

  test('wrapped token retained amount remains non-negative', async () => {
    const fundWrap = ctx.fund.wrap;
    const fundWrapReward = ctx.fund.wrap.reward;
    const holderReward = fundWrap.holders.children[0]!.reward;

    await user3.unwrap.execute(
      { wrappedTokenAmount: 5_000_000_000n },
      { signers: [signer3] }
    );
    await expect(
      user3.receiptToken.resolve(true).then((res) => res?.amount)
    ).resolves.toBeOneOf([15_000_000_000n, 15_000_000_000n - 1n]);
    await expect(
      user3.wrappedToken.resolve(true).then((res) => res?.amount)
    ).resolves.toEqual(5_000_000_000n);
    await expect(
      fundWrap.resolve(true).then((res) => res!.retainedAmount)
    ).resolves.toEqual(0n);
    await expect(
      fundWrapReward
        .resolve(true)
        .then((res) => res!.basePool.tokenAllocatedAmount.totalAmount)
    ).resolves.toEqual(0n);
    await expect(
      holderReward
        .resolve(true)
        .then((res) => res?.basePool.tokenAllocatedAmount.totalAmount)
    ).resolves.toEqual(10_000_000_000n);

    await user2.deposit.execute(
      { assetMint: null, assetAmount: 5_000_000_000n },
      { signers: [signer2] }
    );
    await user2.wrap.execute(
      { receiptTokenAmount: 5_000_000_000n },
      { signers: [signer2] }
    );
    await expect(
      fundWrap.resolve(true).then((res) => res!.retainedAmount)
    ).resolves.toEqual(0n);
    await expect(
      fundWrapReward
        .resolve(true)
        .then((res) => res!.basePool.tokenAllocatedAmount.totalAmount)
    ).resolves.toEqual(0n);
    await expect(
      holderReward
        .resolve(true)
        .then((res) => res?.basePool.tokenAllocatedAmount.totalAmount)
    ).resolves.toEqual(10_000_000_000n);

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'Initialize',
    });

    await expect(
      fundWrap.resolve(true).then((res) => res!.retainedAmount)
    ).resolves.toEqual(5_000_000_000n);
    await expect(
      fundWrapReward
        .resolve(true)
        .then((res) => res!.basePool.tokenAllocatedAmount.totalAmount)
    ).resolves.toEqual(5_000_000_000n);
    await expect(
      holderReward
        .resolve(true)
        .then((res) => res?.basePool.tokenAllocatedAmount.totalAmount)
    ).resolves.toEqual(5_000_000_000n);
  });

  /** Jupsol & sanctum-multi-validator test **/
  test.skip('new supported token with new pricing source deposits & withdraws without any issue', async () => {
    await validator.airdrop(
      restaking.knownAddresses.fundManager,
      100_000_000_000n
    );

    // 1) unstake test from jupSOL stake pool validators
    // 1-0) make jupSOL depositable & only weighted
    await ctx.fund.updateAssetStrategy.execute({
      tokenMint: 'jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v',
      tokenDepositable: true,
      solAllocationWeight: 1n,
    });
    await ctx.fund.updateAssetStrategy.execute({
      tokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
      solAllocationWeight: 0n,
    });

    // 1-1) user deposits 90 jupSOL
    await user3.deposit.execute(
      {
        assetMint: 'jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v',
        assetAmount: 90_000_000_000n,
      },
      { signers: [signer3] }
    );

    // 1-2) user request withdraw 90 fragSOL
    const executionResult = await user3.requestWithdrawal.execute(
      {
        receiptTokenAmount: 90_000_000_000n,
      },
      { signers: [signer3] }
    );
    const requestId =
      executionResult.events!.userRequestedWithdrawalFromFund!.requestId;

    // 1-3) run commands
    /*
    -> EnqueueWithdrawalRequest
    -> UnstakeLST(JupSol)
    -> (skip epoch: waiting for validator to unstake SOL)
    -> ClaimUnstakedSOL
    -> ProcessWithdrawalBatch
    */
    await expectMasked(
      ctx.fund.runCommand.executeChained({
        forceResetCommand: 'EnqueueWithdrawalBatch',
        operator: restaking.knownAddresses.fundManager,
      })
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "forceResetCommand": "EnqueueWithdrawalBatch",
          "operator": "5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx",
        },
        "events": {
          "operatorRanFundCommand": {
            "command": {
              "__kind": "EnqueueWithdrawalBatch",
              "fields": [
                {
                  "forced": true,
                  "state": {
                    "__kind": "New",
                  },
                },
              ],
            },
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "nextSequence": 0,
            "numOperated": 21n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "result": {
              "__option": "Some",
              "value": {
                "__kind": "EnqueueWithdrawalBatch",
                "fields": [
                  {
                    "enqueuedReceiptTokenAmount": 90000000000n,
                    "totalQueuedReceiptTokenAmount": 90000000000n,
                  },
                ],
              },
            },
          },
          "unknown": [],
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    // *** since there is not enough sol in jupSOL reserve stake account, validator needs to unstake sol ***
    await expectMasked(
      ctx.fund.runCommand.executeChained({
        forceResetCommand: 'UnstakeLST',
        operator: restaking.knownAddresses.fundManager,
      })
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "forceResetCommand": null,
          "operator": "5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx",
        },
        "events": {
          "operatorRanFundCommand": {
            "command": {
              "__kind": "UnstakeLST",
              "fields": [
                {
                  "state": {
                    "__kind": "Execute",
                    "items": [
                      {
                        "allocatedTokenAmount": 54345440002n,
                        "tokenMint": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
                      },
                    ],
                    "withdrawSol": true,
                    "withdrawStakeItems": [
                      {
                        "fundStakeAccount": "FLhN5pBMowDVsUdHgeGvRmXUU33rQsYyDEpfQrVLnme4",
                        "fundStakeAccountIndex": 0,
                        "validatorStakeAccount": "EmutJdbKJ55hUyth15bar8ZxDCchR44udAXWYg9eLLDL",
                      },
                      {
                        "fundStakeAccount": "w9kjgBJeTeTnoLEPLXo4Zi9wtj3F2FMwnHb7UrCtPF4",
                        "fundStakeAccountIndex": 1,
                        "validatorStakeAccount": "AjQ5c1GCQkJcg6uukAYhjxY2wSKfX3Lb27FeXUdh8xe4",
                      },
                      {
                        "fundStakeAccount": "53ysYB98VupmR7XPKm5mf86qYt6nK6edgWJKEPQtuk7X",
                        "fundStakeAccountIndex": 2,
                        "validatorStakeAccount": "Cwx3iMVjmJWTG5156eMGyNRQhBrGiyvnUnjqXVxXYEmL",
                      },
                    ],
                  },
                },
              ],
            },
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "nextSequence": 0,
            "numOperated": 25n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "result": {
              "__option": "Some",
              "value": {
                "__kind": "UnstakeLST",
                "fields": [
                  {
                    "burntTokenAmount": 54345440002n,
                    "deductedSolFeeAmount": 60234569n,
                    "operationReceivableSolAmount": 60233477252n,
                    "operationReservedSolAmount": 29766522766n,
                    "operationReservedTokenAmount": 35654559998n,
                    "tokenMint": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
                    "totalUnstakingSolAmount": 60173242683n,
                    "unstakedSolAmount": 1090656n,
                    "unstakingSolAmount": 60173242683n,
                  },
                ],
              },
            },
          },
          "unknown": [],
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);
    await validator.skipEpoch();

    await expectMasked(
      ctx.fund.runCommand.executeChained({
        forceResetCommand: 'ClaimUnstakedSOL',
        operator: restaking.knownAddresses.fundManager,
      })
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "forceResetCommand": null,
          "operator": "5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx",
        },
        "events": {
          "operatorRanFundCommand": {
            "command": {
              "__kind": "ClaimUnstakedSOL",
              "fields": [
                {
                  "state": {
                    "__kind": "Execute",
                    "claimableStakeAccountIndices": {
                      "0": 0,
                    },
                    "poolTokenMints": [
                      "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
                    ],
                  },
                },
              ],
            },
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "nextSequence": 0,
            "numOperated": 29n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "result": {
              "__option": "Some",
              "value": {
                "__kind": "ClaimUnstakedSOL",
                "fields": [
                  {
                    "claimedSolAmount": 60173242683n,
                    "offsettedAssetReceivables": [
                      {
                        "assetAmount": 60173242683n,
                        "assetTokenMint": {
                          "__option": "None",
                        },
                      },
                    ],
                    "offsettedSolReceivableAmount": 60173242683n,
                    "operationReceivableSolAmount": 60234569n,
                    "operationReservedSolAmount": 89939765449n,
                    "tokenMint": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
                    "totalUnstakingSolAmount": 0n,
                    "transferredSolRevenueAmount": 0n,
                  },
                ],
              },
            },
          },
          "unknown": [],
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    await expectMasked(
      ctx.fund.runCommand.executeChained({
        forceResetCommand: 'ProcessWithdrawalBatch',
        operator: restaking.knownAddresses.fundManager,
      })
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "forceResetCommand": null,
          "operator": "5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx",
        },
        "events": {
          "operatorRanFundCommand": {
            "command": {
              "__kind": "ProcessWithdrawalBatch",
              "fields": [
                {
                  "forced": true,
                  "state": {
                    "__kind": "Execute",
                    "assetTokenMint": {
                      "__option": "None",
                    },
                    "numProcessingBatches": 1,
                    "receiptTokenAmount": 90000000000n,
                  },
                },
              ],
            },
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "nextSequence": 0,
            "numOperated": 32n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "result": {
              "__option": "Some",
              "value": {
                "__kind": "ProcessWithdrawalBatch",
                "fields": [
                  {
                    "assetTokenMint": {
                      "__option": "None",
                    },
                    "deductedAssetFeeAmount": 180000000n,
                    "offsettedAssetReceivables": [
                      {
                        "assetAmount": 60234569n,
                        "assetTokenMint": {
                          "__option": "None",
                        },
                      },
                    ],
                    "processedReceiptTokenAmount": 90000000000n,
                    "requestedReceiptTokenAmount": 90000000000n,
                    "requiredAssetAmount": 0n,
                    "reservedAssetUserAmount": 89820000003n,
                    "transferredAssetRevenueAmount": 124890806n,
                    "withdrawalFeeRateBps": 20,
                  },
                ],
              },
            },
          },
          "unknown": [],
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    // 1-4) user withdraws sol
    await expectMasked(
      user3.withdraw.execute({ requestId: requestId }, { signers: [signer3] })
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "assetMint": null,
          "requestId": 5n,
        },
        "events": {
          "unknown": [],
          "userWithdrewFromFund": {
            "batchId": 2n,
            "burntReceiptTokenAmount": 90000000000n,
            "deductedFeeAmount": 180000000n,
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "fundWithdrawalBatchAccount": "J1cPTrKYvp3v1BvfeQ8iRnvTjkbygBZNfepV72uVpRf1",
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "requestId": 5n,
            "returnedReceiptTokenAmount": 0n,
            "supportedTokenMint": {
              "__option": "None",
            },
            "user": "FZPz1bd26HAMxSRQ5uM69wnW5ATws2ZYyp9B47Lrv6Yj",
            "userFundAccount": "4ZcNhSQEEKwJy1JqGCAc71nUnV1qyb8yGFNJhHdxLLev",
            "userReceiptTokenAccount": "HkfGcLacktLYF1bGksFDoN8PK9oFu8a6sseXE8riQzj9",
            "userSupportedTokenAccount": {
              "__option": "None",
            },
            "withdrawnAmount": 89820000003n,
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    // 2) stake test
    // 2-1) user deposits more sol to trigger staking
    await user3.deposit.execute(
      {
        assetAmount: 50_000_000_000n,
      },
      { signers: [signer3] }
    );

    // 2-2) run 'StakeSOL'command to stake SOL & get jupSOL
    await expectMasked(
      ctx.fund.runCommand.executeChained({
        forceResetCommand: 'StakeSOL',
        operator: restaking.knownAddresses.fundManager,
      })
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "forceResetCommand": null,
          "operator": "5FjrErTQ9P1ThYVdY9RamrPUCQGTMCcczUjH21iKzbwx",
        },
        "events": {
          "operatorRanFundCommand": {
            "command": {
              "__kind": "StakeSOL",
              "fields": [
                {
                  "state": {
                    "__kind": "Execute",
                    "items": [
                      {
                        "allocatedSolAmount": 50000000015n,
                        "tokenMint": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
                      },
                    ],
                  },
                },
              ],
            },
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "nextSequence": 0,
            "numOperated": 35n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "result": {
              "__option": "Some",
              "value": {
                "__kind": "StakeSOL",
                "fields": [
                  {
                    "deductedSolFeeAmount": 0n,
                    "mintedTokenAmount": 45111504825n,
                    "operationReceivableSolAmount": 0n,
                    "operationReservedSolAmount": 0n,
                    "operationReservedTokenAmount": 80766064823n,
                    "stakedSolAmount": 50000000015n,
                    "tokenMint": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
                  },
                ],
              },
            },
          },
          "unknown": [],
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);
  });

  /** 4. Operation **/
  test('run operation cycles through multiple epoches to test cach-in/out flows including (un)stake/(un)restake', async () => {
    await user1.resolveAddress(true);

    for (const mint of [
      null,
      'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
      'mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So',
      'BNso1VUJnh4zcfpZa6986Ea66P6TCp59hvtNJ8b1X85',
      'Bybit2vBJGhPF52GBdNaQfUJ6ZpThSgHBobjWZpLPb4B',
      'jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v',
      'bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1',
      'Dso1bDeDjCQxTrWHqUUi63oBvV7Mdm6WaobLbQ7gnPQ',
      'FRAGME9aN7qzxkHPmVP22tDhG87srsR9pr5SY9XdRd9R',
      'he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A',
      'roxDFxTFHufJBFy3PgzZcgz6kwkQNPZpi9RfpcAv4bu',
      'sctmadV2fcLtrxjzYhTZzwAGjXUXKtYSBrrM36EtdcY',
      'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
      'vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7',
      'HUBsveNpjo5pWqNkH57QzxjQASdTVXcSK7bVKTSZtcSX',
      'picobAEvs6w7QEknPce34wAE4gknZA9v5tTonnmHYdX',
      'strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA',
    ]) {
      if (mint) {
        await validator.airdropToken(user1.address!, mint, 100_000_000_000n);
      } else {
        await validator.airdrop(user1.address!, 100_000_000_000n);
      }
      await expect(
        user1.deposit.execute(
          {
            assetAmount: 100_000_000_000n,
            assetMint: mint,
          },
          {
            signers: [signer1],
          }
        )
      ).resolves.not.toThrow();
    }

    await expect(
      ctx.fund.runCommand.executeChained(null)
    ).resolves.not.toThrow();

    // all assets should be restaked fully (practically a coupe of SOL can be left)
    await expect(ctx.fund.reserve.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      {
        "address": "8fswMoFYJNM8pqnDDZM4yfLrLvWwgDCV47HWUgpjSbpG",
        "data": Uint8Array [],
        "executable": false,
        "lamports": 890889n,
        "programAddress": "11111111111111111111111111111111",
        "space": 0n,
      }
    `);
    await expect(ctx.fund.reserve.normalizedToken.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      {
        "amount": 0n,
        "closeAuthority": {
          "__option": "None",
        },
        "delegate": {
          "__option": "None",
        },
        "delegatedAmount": 0n,
        "isNative": {
          "__option": "None",
        },
        "mint": "4noNmx2RpxK4zdr68Fq1CYM5VhN4yjgGZEFyuB7t2pBX",
        "owner": "8fswMoFYJNM8pqnDDZM4yfLrLvWwgDCV47HWUgpjSbpG",
        "state": 1,
      }
    `);

    // now trigger cash-out flow
    await expect(
      user1.requestWithdrawal.execute(
        {
          assetMint: null,
          receiptTokenAmount: 123_456_000_000n,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchObject({
      events: {
        userRequestedWithdrawalFromFund: {
          supportedTokenMint: { __option: 'None' },
          requestedReceiptTokenAmount: 123_456_000_000n,
        },
      },
    });

    const withdrawalLastProcessedBatchId = await ctx.fund
      .resolveAccount(true)
      .then((account) => account!.data.sol.withdrawalLastProcessedBatchId);

    // to enqueue withdrawal batch and make an unrestake request from vaults
    await expect(
      ctx.fund.runCommand.executeChained(null)
    ).resolves.not.toThrow();

    await expect(validator.skipEpoch()).resolves.not.toThrow();
    await expect(validator.skipEpoch()).resolves.not.toThrow();

    // to claim unrestaked vst from vaults and make unstake requests from stake pools
    await expect(
      ctx.fund.runCommand.executeChained(null)
    ).resolves.not.toThrow();

    await expect(validator.skipEpoch()).resolves.not.toThrow();
    await expect(validator.skipEpoch()).resolves.not.toThrow();

    // to claim unstaked SOL from stake pools and process withdrawal batch
    await expect(
      ctx.fund.runCommand.executeChained(null)
    ).resolves.not.toThrow();

    await expect(
      ctx.fund
        .resolveAccount(true)
        .then((account) => account?.data.sol.withdrawalLastProcessedBatchId)
    ).resolves.toEqual(withdrawalLastProcessedBatchId + 1n);
    await expectMasked(ctx.fund.latestWithdrawalBatches.resolve(true)).resolves
      .toMatchInlineSnapshot(`
      [
        {
          "assetFeeAmount": 246912000n,
          "assetUserAmount": 123209088005n,
          "batchId": 2n,
          "claimedAssetUserAmount": 0n,
          "claimedReceiptTokenAmount": 0n,
          "numClaimedRequests": 0n,
          "numRequests": 1n,
          "processedAt": "MASKED(/.*At?$/)",
          "receiptTokenAmount": 123456000000n,
          "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
          "supportedTokenMint": {
            "__option": "None",
          },
          "supportedTokenProgram": {
            "__option": "None",
          },
        },
      ]
    `);
  });

  test('operation disable', async () => {
    await ctx.fund.updateGeneralStrategy.execute({
      operationEnabled: false,
    });

    await expect(ctx.fund.runCommand.executeChained(null)).rejects.toThrowError(
      'Transaction simulation failed'
    ); // fund: operation is disable
    await expect(
      ctx.fund.runCommand.executeChained({ forceResetCommand: 'Initialize' })
    ).rejects.toThrowError('Transaction simulation failed');

    await ctx.fund.updateGeneralStrategy.execute({
      operationEnabled: true,
    });

    await expect(
      ctx.fund.runCommand.executeChained(null)
    ).resolves.not.toThrow();
  });
});
