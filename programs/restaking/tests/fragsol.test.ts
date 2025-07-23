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
    validator.airdrop(restaking.knownAddresses.fundManager, 100_000_000_000n),
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
            "unrestakingAmountAsSupportedToken": 0n,
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
              },
              {
                "operator": "29rxXT5zbTR1ctiooHtb1Sa1TD4odzhQHsrLz3D78G5w",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
              },
              {
                "operator": "LKFpfXtBkH5b7D9mo8dPcjCLZCZpmLQC9ELkbkyVdah",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 92n,
              },
              {
                "operator": "GZxp4e2Tm3Pw9GyAaxuF6odT3XkRM96jpZkp3nxhoK4Y",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 2n,
              },
              {
                "operator": "CA8PaNSoFWzvbCJ2oK3QxBEutgyHSTT5omEptpj8YHPY",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 3n,
              },
              {
                "operator": "7yofWXChEHkPTSnyFdKx2Smq5iWVbGB4P1dkdC6zHWYR",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 1n,
              },
              {
                "operator": "BFEsrxFPsBcY2hR5kgyfKnpwgEc8wYQdngvRukLQXwG2",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 4n,
              },
              {
                "operator": "2sHNuid4rus4sK2EmndLeZcPNKkgzuEoc8Vro3PH2qop",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
              },
              {
                "operator": "5TGRFaLy3eF93pSNiPamCgvZUN3gzdYcs7jA3iCAsd1L",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 92n,
              },
              {
                "operator": "EkroMQiZJfphVd9iPvR4zMCHasTW72Uh1mFYkTxtQuY6",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
              },
              {
                "operator": "574DmorRvpaYrSrBRUwAjG7bBmrZYiTW3Fc8mvQatFqo",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 92n,
              },
              {
                "operator": "C6AF8qGCo2dL815ziRCmfdbFeL5xbRLuSTSZzTGBH68y",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
              },
              {
                "operator": "6AxtdRGAaiAyqcwxVBHsH3xtqCbQuffaiE4epT4koTxk",
                "tokenAllocationCapacityAmount": 18446744073709551615n,
                "tokenAllocationWeight": 0n,
              },
            ],
            "distributingRewardTokens": [],
            "pricingSource": {
              "__kind": "JitoRestakingVault",
              "address": "HR1ANmDHjaEhknvsTaK48M5xZtbBiwNdXM5NTiWhAb4S",
            },
            "rewardCommissionRateBps": 0,
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
            "unrestakingAmountAsSupportedToken": 0n,
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

  /** 5. operation cycle **/
  test('run operation cycles through multiple epoches to test cash-in/out flows including (un)stake/(un)restake', async () => {
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

  test(`there could be remaining lamports in uninitialized fund withdrawal stake accounts, due to jito tip`, async () => {
    // Airdrop 1 SOL to each jitoSOL withdrawal stake accounts
    await Promise.all([
      validator.airdrop(
        'AW3FiXBG6DzDFS9sg8LFYUyUXVWh4SBgXo3vaAgD2uDb',
        1_000_000_000n
      ),
      validator.airdrop(
        'AkV1gsoGeGdFuWrReeE21kcVgMJMUEe33jvuxQtQBQyQ',
        1_000_000_000n
      ),
      validator.airdrop(
        'BM2DCJ34zhLx5AYwzBE3W6VfEBWiTuyXzhrEPHdTKzdU',
        1_000_000_000n
      ),
      validator.airdrop(
        'G3X7uH2fhzyomtVqanJ1ZuhHi5wwdG8qxzqjt9RzLbXH',
        1_000_000_000n
      ),
      validator.airdrop(
        'DpP5TGf5fGbN2cCnJzaXrFKHzhSuef2vgb2FPTNCdmU4',
        1_000_000_000n
      ),
    ]);

    const userReceiptTokenBalanceBefore = await user1
      .resolve(true)
      .then((res) => res!.receiptTokenAmount);

    await validator.airdropToken(
      user1.address!,
      'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
      200_000_000_000_000n
    );

    await user1.deposit.execute(
      {
        assetMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        assetAmount: 200_000_000_000_000n,
      },
      { signers: [signer1] }
    );

    const mintedReceiptTokenAmount = await user1
      .resolve(true)
      .then((res) => res!.receiptTokenAmount - userReceiptTokenBalanceBefore);

    await user1.requestWithdrawal.execute(
      {
        assetMint: null,
        receiptTokenAmount: mintedReceiptTokenAmount,
      },
      { signers: [signer1] }
    );

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'EnqueueWithdrawalBatch',
      operator: restaking.knownAddresses.fundManager,
    });

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'UnstakeLST',
      operator: restaking.knownAddresses.fundManager,
    });
  });
});
