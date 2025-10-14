import { MAX_U64 } from '@fragmetric-labs/sdk';
import { getAddressDecoder, KeyPairSigner } from '@solana/kit';
import { afterAll, beforeEach, describe, expect, test } from 'vitest';
import { RestakingUserAccountContext } from '../../../clients/js/fragmetric-sdk/dist/programs/restaking/user';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeFragX } from './fragx.unit.init';

describe('restaking.fragX unit test', async () => {
  const testSuiteCtx = await createTestSuiteContext();

  type FragXTestContext = Awaited<ReturnType<typeof initializeFragX>>;

  let testCtx: FragXTestContext;
  let validator: FragXTestContext['validator'];
  let feePayer: FragXTestContext['feePayer'];
  let restaking: FragXTestContext['restaking'];
  let initializationTasks: FragXTestContext['initializationTasks'];
  let ctx: FragXTestContext['ctx'];

  let signer1: KeyPairSigner, signer2: KeyPairSigner, signer3: KeyPairSigner;
  let user1: RestakingUserAccountContext,
    user2: RestakingUserAccountContext,
    user3: RestakingUserAccountContext;

  let index = 0;

  beforeEach(async () => {
    testCtx = await initializeFragX(testSuiteCtx, index++);
    await testCtx.initializationTasks;

    ({ validator, feePayer, restaking, initializationTasks, ctx } = testCtx);

    [signer1, signer2, signer3] = await Promise.all([
      validator
        .newSigner('fragBTCTestSigner1', 100_000_000_000n)
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
      validator
        .newSigner('fragBTCTestSigner2', 100_000_000_000n)
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
    user1 = ctx.user(signer1);
    user2 = ctx.user(signer2);
    user3 = ctx.user(signer3);
  });
  afterAll(() => testCtx.validator.quit());

  /** configuration **/
  test('pricing source addresses field in fund account updates correctly', async () => {
    await expectMasked(ctx.fund.updatePrices.execute(null)).resolves
      .toMatchInlineSnapshot(`
      {
        "args": null,
        "events": {
          "operatorUpdatedFundPrices": {
            "fundAccount": "5nsRAb7faaGoVkMovx4eSkHk3fcsBnHRhLZxzPawVk87",
            "receiptTokenMint": "5TdWCgeGM4J9equWEF426F3eYLtuNcRUnMK2YSRRJCBD",
          },
          "unknown": [],
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);

    // 1) pricing_source_addresses field of fund account has correct data
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
    // 3-1) add heliusSol as supported token
    await ctx.fund.addSupportedToken.execute({
      mint: 'he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A',
      pricingSource: {
        __kind: 'SanctumSingleValidatorSPLStakePool',
        address: '3wK2g8ZdzAH8FJ7PKr2RcvGh7V9VYson5hrVsJM5Lmws',
      },
    });

    await ctx.normalizedTokenPool.addSupportedToken.execute({
      mint: 'he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A',
      pricingSource: {
        __kind: 'SanctumSingleValidatorSPLStakePool',
        address: '3wK2g8ZdzAH8FJ7PKr2RcvGh7V9VYson5hrVsJM5Lmws',
      },
    });
    fundAccount = await ctx.fund.resolveAccount(true);
    normalizedTokenPool = await ctx.normalizedTokenPool.resolveAddress(true);
    expect(getPricingSourcesManually()).toEqual(getPricingSourcesByField());
    expect(fundAccount!.data.numPricingSourceAddresses - 1).toEqual(
      prevNumPricingSourceAddresses
    );

    // 3-2) remove heliusSol from supported tokens
    await ctx.normalizedTokenPool.removeSupportedToken.execute({
      mint: 'he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A',
    });

    await ctx.fund.removeSupportedToken.execute({
      mint: 'he1iusmfkpAdwvxLNGV8Y1iSbj4rUy6yMhEA3fotn9A',
    });

    fundAccount = await ctx.fund.resolveAccount(true);
    normalizedTokenPool = await ctx.normalizedTokenPool.resolveAddress(true);
    expect(getPricingSourcesManually()).toEqual(getPricingSourcesByField());
    expect(fundAccount!.data.numPricingSourceAddresses).toEqual(
      prevNumPricingSourceAddresses
    );
  });

  test('remove supported tokens', async () => {
    await expect(
      ctx.fund.addSupportedToken.execute({
        mint: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
        pricingSource: {
          __kind: 'SanctumSingleValidatorSPLStakePool',
          address: 'ArAQfbzsdotoKB5jJcZa3ajQrrPcWr2YQoDAEAiFxJAC',
        },
      })
    ).resolves.not.toThrow();
    await expect(
      ctx.normalizedTokenPool.addSupportedToken.execute({
        mint: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
        pricingSource: {
          __kind: 'SanctumSingleValidatorSPLStakePool',
          address: 'ArAQfbzsdotoKB5jJcZa3ajQrrPcWr2YQoDAEAiFxJAC',
        },
      })
    ).resolves.not.toThrow();

    await expect(
      ctx.fund.addSupportedToken.execute({
        mint: 'strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA',
        pricingSource: {
          __kind: 'PeggedToken',
          address: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
        },
      })
    ).resolves.not.toThrow();
    await expect(
      ctx.normalizedTokenPool.addSupportedToken.execute({
        mint: 'strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA',
        pricingSource: {
          __kind: 'PeggedToken',
          address: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
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
            "address": "ArAQfbzsdotoKB5jJcZa3ajQrrPcWr2YQoDAEAiFxJAC",
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
        "receiptTokenMint": "FWWLbJYnQ5wV6kbK4gtcYcqu16uY34Ft1k7TUF8wQgVF",
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
        ],
        "wrappedTokenMint": "2zKRCiz4J66VdGvJQ4GQwM5MwsSaHeBjsjF7StjWTbXN",
      }
    `);

    // start remove
    // failed because used by other pegged token
    await expect(
      ctx.normalizedTokenPool.removeSupportedToken.execute({
        mint: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
      })
    ).rejects.toThrow();
    await expect(
      ctx.fund.removeSupportedToken.execute({
        mint: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
      })
    ).rejects.toThrow();

    // failed because used by ntp
    await expect(
      ctx.fund.removeSupportedToken.execute({
        mint: 'strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA',
      })
    ).rejects.toThrow();

    // success
    await expect(
      ctx.normalizedTokenPool.removeSupportedToken.execute({
        mint: 'strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA',
      })
    ).resolves.not.toThrow();
    await expect(
      ctx.fund.removeSupportedToken.execute({
        mint: 'strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA',
      })
    ).resolves.not.toThrow();
    await expect(
      ctx.normalizedTokenPool.removeSupportedToken.execute({
        mint: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
      })
    ).resolves.not.toThrow();
    await expect(
      ctx.fund.removeSupportedToken.execute({
        mint: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
      })
    ).resolves.not.toThrow();
  });

  test('remove token swap strategy', async () => {
    const fund_1 = await ctx.fund.resolve(true);
    expect(fund_1.tokenSwapStrategies).toHaveLength(1);

    await expect(
      ctx.fund.removeTokenSwapStrategy.execute({
        fromTokenMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL', // invalid from token mint
        toTokenMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
        swapSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: 'G2FiE1yn9N9ZJx5e1E2LxxMnHvb1H3hCuHLPfKJ98smA',
        },
      })
    ).rejects.toThrowError('Transaction simulation failed'); // fund: token swap strategy not found.
    await expect(
      ctx.fund.removeTokenSwapStrategy.execute({
        fromTokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        toTokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn', // invalid to token mint
        swapSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: 'G2FiE1yn9N9ZJx5e1E2LxxMnHvb1H3hCuHLPfKJ98smA',
        },
      })
    ).rejects.toThrowError('Transaction simulation failed'); // fund: token swap strategy validation failed.
    await expect(
      ctx.fund.removeTokenSwapStrategy.execute({
        fromTokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        toTokenMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
        swapSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL', // invalid swap source
        },
      })
    ).rejects.toThrowError('Transaction simulation failed'); // fund: token swap strategy validation failed.

    await expect(
      ctx.fund.removeTokenSwapStrategy.execute({
        fromTokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        toTokenMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
        swapSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: 'G2FiE1yn9N9ZJx5e1E2LxxMnHvb1H3hCuHLPfKJ98smA',
        },
      })
    ).resolves.not.toThrow();

    const fund_2 = await ctx.fund.resolve(true);
    expect(fund_2.tokenSwapStrategies).toHaveLength(0);
  });

  /** deposit */
  test('user can update reward pools and sync with global reward account anytime', async () => {
    await expectMasked(
      user1.deposit.execute(
        {
          assetMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
          assetAmount: 10_000_000_000n,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchInlineSnapshot(`
        {
          "args": {
            "applyPresetComputeUnitLimit": true,
            "assetAmount": 10000000000n,
            "assetMint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "metadata": null,
          },
          "events": {
            "unknown": [],
            "userCreatedOrUpdatedFundAccount": {
              "created": true,
              "receiptTokenAmount": 0n,
              "receiptTokenMint": "7sUUWN6ZvR3X2iUWjHP5KTNmQizKQEz6iKFytoSBcDDA",
              "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
              "userFundAccount": "GL9FuX9vkMASixFaXf3wrSr8ev8rbkJMVtjpdcLkUX6b",
            },
            "userCreatedOrUpdatedRewardAccount": {
              "created": true,
              "receiptTokenAmount": 0n,
              "receiptTokenMint": "7sUUWN6ZvR3X2iUWjHP5KTNmQizKQEz6iKFytoSBcDDA",
              "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
              "userRewardAccount": "6ZawY1XehsdncSNZPEdbcvfAACjNjR96jhL3KzWj6u1M",
            },
            "userDepositedToFund": {
              "contributionAccrualRate": {
                "__option": "None",
              },
              "depositedAmount": 10000000000n,
              "fundAccount": "4vAHVvvtgWMcZaxgtoT1K63p8ne2tGY4bM5YieFzuUep",
              "mintedReceiptTokenAmount": 10000000000n,
              "receiptTokenMint": "7sUUWN6ZvR3X2iUWjHP5KTNmQizKQEz6iKFytoSBcDDA",
              "supportedTokenMint": {
                "__option": "Some",
                "value": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
              },
              "updatedUserRewardAccounts": [
                "6ZawY1XehsdncSNZPEdbcvfAACjNjR96jhL3KzWj6u1M",
              ],
              "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
              "userFundAccount": "GL9FuX9vkMASixFaXf3wrSr8ev8rbkJMVtjpdcLkUX6b",
              "userReceiptTokenAccount": "FSemEnh52ptxSMV29rZxRGqontN2kxuHfeiGFRj133D7",
              "userSupportedTokenAccount": {
                "__option": "Some",
                "value": "8eFrzT4rLHeh6ikBBkTE76dtziRJWRuhcmatPx3B1EdG",
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

    const user1Reward_1 = await user1.reward.resolve(true);

    // user1 updateRewardPools
    await user1.reward.updatePools.execute(null);

    const user1Reward_2 = await user1.reward.resolve(true);

    const elapsedSlots =
      user1Reward_2?.basePool.updatedSlot! -
      user1Reward_1?.basePool.updatedSlot!;
    const increasedContribution =
      user1Reward_2?.basePool.contribution! -
      user1Reward_1?.basePool.contribution!;
    expect(increasedContribution, 't4_1').toEqual(
      BigInt(elapsedSlots) *
        user1Reward_2?.basePool.tokenAllocatedAmount.records[0].amount! *
        BigInt(
          user1Reward_2?.basePool.tokenAllocatedAmount.records[0]
            .contributionAccrualRate! * 100
        )
    );
    expect(
      user1Reward_2?.basePool.settlements[0].settledAmount,
      't4_2'
    ).toEqual(0n);
  });

  test('user can deposit large amount of token', async () => {
    await validator.airdropToken(
      signer1.address,
      'FRAGME9aN7qzxkHPmVP22tDhG87srsR9pr5SY9XdRd9R',
      1_000_000_000_000_000_000n
    );
    await expectMasked(
      user1.deposit.execute(
        {
          assetMint: 'FRAGME9aN7qzxkHPmVP22tDhG87srsR9pr5SY9XdRd9R',
          assetAmount: 1_000_000_000_000_000_000n,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchInlineSnapshot(`
        {
          "args": {
            "applyPresetComputeUnitLimit": true,
            "assetAmount": 1000000000000000000n,
            "assetMint": "FRAGME9aN7qzxkHPmVP22tDhG87srsR9pr5SY9XdRd9R",
            "metadata": null,
          },
          "events": {
            "unknown": [],
            "userCreatedOrUpdatedFundAccount": {
              "created": true,
              "receiptTokenAmount": 0n,
              "receiptTokenMint": "9Cn6WLm4dEgRE7G297J7usijc65zNBavZRA9TP78z6Sm",
              "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
              "userFundAccount": "oStWx5CihjrWuTQgiq4p6NfNsixZFGMApV4t5q5yKaW",
            },
            "userCreatedOrUpdatedRewardAccount": {
              "created": true,
              "receiptTokenAmount": 0n,
              "receiptTokenMint": "9Cn6WLm4dEgRE7G297J7usijc65zNBavZRA9TP78z6Sm",
              "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
              "userRewardAccount": "8qRoEDzaGn7JEcTLySRxyJS2X69rTDVGo5nNsWauuTy3",
            },
            "userDepositedToFund": {
              "contributionAccrualRate": {
                "__option": "None",
              },
              "depositedAmount": 1000000000000000000n,
              "fundAccount": "E4sjVVyt7h9qtCnzcEpCAXXArXXyLBEar49bEBsCdR5z",
              "mintedReceiptTokenAmount": 1000000000000000000n,
              "receiptTokenMint": "9Cn6WLm4dEgRE7G297J7usijc65zNBavZRA9TP78z6Sm",
              "supportedTokenMint": {
                "__option": "Some",
                "value": "FRAGME9aN7qzxkHPmVP22tDhG87srsR9pr5SY9XdRd9R",
              },
              "updatedUserRewardAccounts": [
                "8qRoEDzaGn7JEcTLySRxyJS2X69rTDVGo5nNsWauuTy3",
              ],
              "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
              "userFundAccount": "oStWx5CihjrWuTQgiq4p6NfNsixZFGMApV4t5q5yKaW",
              "userReceiptTokenAccount": "9SxSvTrCJau5kEuycKPNe9ythdhGCYDtf4hnSsMThBAu",
              "userSupportedTokenAccount": {
                "__option": "Some",
                "value": "9hXqo1QhxTqinenGB7M7iFYPPb59sg5Kr6Uvnq3c7ZNM",
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
  });

  /** jupsol & sanctum-multi-validator test **/
  test('new supported token with new pricing source deposits & withdraws without any issue', async () => {
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

    const user3ReceiptToken = await user3.receiptToken.resolve(true);

    // 1-2) user request withdraw total fragSOL
    const executionResult = await user3.requestWithdrawal.execute(
      {
        receiptTokenAmount: user3ReceiptToken?.amount!,
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
              "fundAccount": "56WijXFKfRs6dKtNrqqH2g5A9T1bDHsTLDPREKb5qNWe",
              "nextSequence": 0,
              "numOperated": 2n,
              "receiptTokenMint": "JCzYfo5HVZ9g1NYVqkBrXsv8SeXbTGGfuBEXTcGRCsuz",
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
                          "allocatedTokenAmount": 90000000000n,
                          "tokenMint": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
                        },
                      ],
                      "withdrawSol": true,
                      "withdrawStakeItems": [
                        {
                          "fundStakeAccount": "7uTFEJqfo9W9tdE5XxT3bXaquscNn7VczARgZDpjHJVx",
                          "fundStakeAccountIndex": 0,
                          "validatorStakeAccount": "EmutJdbKJ55hUyth15bar8ZxDCchR44udAXWYg9eLLDL",
                        },
                        {
                          "fundStakeAccount": "HR5rusEqLBp48oYHpE9p75EgsnQCGeFtkfNau2Ukgixk",
                          "fundStakeAccountIndex": 1,
                          "validatorStakeAccount": "Cwx3iMVjmJWTG5156eMGyNRQhBrGiyvnUnjqXVxXYEmL",
                        },
                        {
                          "fundStakeAccount": "HguWoruDGQ2wy6AaQwJhkfLi4qRJHT6hJCJQCoHyNjMS",
                          "fundStakeAccountIndex": 2,
                          "validatorStakeAccount": "AjQ5c1GCQkJcg6uukAYhjxY2wSKfX3Lb27FeXUdh8xe4",
                        },
                      ],
                    },
                  },
                ],
              },
              "fundAccount": "56WijXFKfRs6dKtNrqqH2g5A9T1bDHsTLDPREKb5qNWe",
              "nextSequence": 0,
              "numOperated": 6n,
              "receiptTokenMint": "JCzYfo5HVZ9g1NYVqkBrXsv8SeXbTGGfuBEXTcGRCsuz",
              "result": {
                "__option": "Some",
                "value": {
                  "__kind": "UnstakeLST",
                  "fields": [
                    {
                      "burntTokenAmount": 90000000000n,
                      "deductedSolFeeAmount": 100134473n,
                      "operationReceivableSolAmount": 100134473645n,
                      "operationReservedSolAmount": 0n,
                      "operationReservedTokenAmount": 0n,
                      "tokenMint": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
                      "totalUnstakingSolAmount": 100034339172n,
                      "unstakedSolAmount": 0n,
                      "unstakingSolAmount": 100034339172n,
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
              "fundAccount": "56WijXFKfRs6dKtNrqqH2g5A9T1bDHsTLDPREKb5qNWe",
              "nextSequence": 0,
              "numOperated": 10n,
              "receiptTokenMint": "JCzYfo5HVZ9g1NYVqkBrXsv8SeXbTGGfuBEXTcGRCsuz",
              "result": {
                "__option": "Some",
                "value": {
                  "__kind": "ClaimUnstakedSOL",
                  "fields": [
                    {
                      "claimedSolAmount": 100034339172n,
                      "offsetAssetReceivables": [
                        {
                          "assetAmount": 100034339172n,
                          "assetTokenMint": {
                            "__option": "None",
                          },
                        },
                      ],
                      "offsetSolReceivableAmount": 100034339172n,
                      "operationReceivableSolAmount": 100134473n,
                      "operationReservedSolAmount": 100034339172n,
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
              "fundAccount": "56WijXFKfRs6dKtNrqqH2g5A9T1bDHsTLDPREKb5qNWe",
              "nextSequence": 0,
              "numOperated": 13n,
              "receiptTokenMint": "JCzYfo5HVZ9g1NYVqkBrXsv8SeXbTGGfuBEXTcGRCsuz",
              "result": {
                "__option": "Some",
                "value": {
                  "__kind": "ProcessWithdrawalBatch",
                  "fields": [
                    {
                      "assetTokenMint": {
                        "__option": "None",
                      },
                      "deductedAssetFeeAmount": 200268947n,
                      "offsetAssetReceivables": [
                        {
                          "assetAmount": 100134473n,
                          "assetTokenMint": {
                            "__option": "None",
                          },
                        },
                      ],
                      "processedBatchAccounts": [
                        "FKaB11zNxf4EgMkEWrRh8doqRBh7g5tXzsFB4CadgHV6",
                      ],
                      "processedReceiptTokenAmount": 90000000000n,
                      "requestedReceiptTokenAmount": 90000000000n,
                      "requiredAssetAmount": 0n,
                      "reservedAssetUserAmount": 99934204698n,
                      "transferredAssetRevenueAmount": 102417354n,
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
            "applyPresetComputeUnitLimit": true,
            "assetMint": null,
            "requestId": 1n,
          },
          "events": {
            "unknown": [],
            "userWithdrewFromFund": {
              "batchId": 1n,
              "burntReceiptTokenAmount": 90000000000n,
              "deductedFeeAmount": 200268947n,
              "fundAccount": "56WijXFKfRs6dKtNrqqH2g5A9T1bDHsTLDPREKb5qNWe",
              "fundWithdrawalBatchAccount": "FKaB11zNxf4EgMkEWrRh8doqRBh7g5tXzsFB4CadgHV6",
              "receiptTokenMint": "JCzYfo5HVZ9g1NYVqkBrXsv8SeXbTGGfuBEXTcGRCsuz",
              "requestId": 1n,
              "returnedReceiptTokenAmount": 0n,
              "supportedTokenMint": {
                "__option": "None",
              },
              "user": "FZPz1bd26HAMxSRQ5uM69wnW5ATws2ZYyp9B47Lrv6Yj",
              "userFundAccount": "9rfku8yqmz5tda1Q8QBDhMyYPC21sryA1Ztr9xQmMZwD",
              "userReceiptTokenAccount": "ATAwHi6iKjSjC4yrCuUmhpU2rsVxJJ1ocHDY6jGj5DXu",
              "userSupportedTokenAccount": {
                "__option": "None",
              },
              "withdrawnAmount": 99934204698n,
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
    const amountBefore = await ctx.fund
      .resolveAccount(true)
      .then(
        (fund) =>
          fund!.data.supportedTokens.find(
            (token) =>
              token.mint == 'jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v'
          )!.token.operationReservedAmount
      );
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'StakeSOL',
      operator: restaking.knownAddresses.fundManager,
    });
    const amountAfter = await ctx.fund
      .resolveAccount(true)
      .then(
        (fund) =>
          fund!.data.supportedTokens.find(
            (token) =>
              token.mint == 'jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v'
          )!.token.operationReservedAmount
      );

    expect(amountAfter).toBeGreaterThan(amountBefore);
  });

  /** operation **/
  test('operation disabled', async () => {
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

  /** delegate */
  test('delegate reward account from user2 to user1', async () => {
    // for delegate, do deposit first to create reward account
    await expectMasked(
      user2.deposit.execute(
        {
          assetMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
          assetAmount: 10_000_000_000n,
        },
        { signers: [signer2] }
      )
    ).resolves.toMatchInlineSnapshot(`
        {
          "args": {
            "applyPresetComputeUnitLimit": true,
            "assetAmount": 10000000000n,
            "assetMint": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "metadata": null,
          },
          "events": {
            "unknown": [],
            "userCreatedOrUpdatedFundAccount": {
              "created": true,
              "receiptTokenAmount": 0n,
              "receiptTokenMint": "Gmx4No1Fki926g1jKoWS2k3JJdGnQuR3L1RNMr7PcQVD",
              "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
              "userFundAccount": "HosQKDxFmKd7BForhgWWYz68V5ZQnR4QBZVmhCfTmn28",
            },
            "userCreatedOrUpdatedRewardAccount": {
              "created": true,
              "receiptTokenAmount": 0n,
              "receiptTokenMint": "Gmx4No1Fki926g1jKoWS2k3JJdGnQuR3L1RNMr7PcQVD",
              "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
              "userRewardAccount": "91mNEHtBj9vnqfrZNDp55di5HPmC6D8aPBNsm3mKtnW",
            },
            "userDepositedToFund": {
              "contributionAccrualRate": {
                "__option": "None",
              },
              "depositedAmount": 10000000000n,
              "fundAccount": "2ejCogYUjKXZhNPWJKG53gcVRSASMqW8Di7E9qD7VWeu",
              "mintedReceiptTokenAmount": 10000000000n,
              "receiptTokenMint": "Gmx4No1Fki926g1jKoWS2k3JJdGnQuR3L1RNMr7PcQVD",
              "supportedTokenMint": {
                "__option": "Some",
                "value": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
              },
              "updatedUserRewardAccounts": [
                "91mNEHtBj9vnqfrZNDp55di5HPmC6D8aPBNsm3mKtnW",
              ],
              "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
              "userFundAccount": "HosQKDxFmKd7BForhgWWYz68V5ZQnR4QBZVmhCfTmn28",
              "userReceiptTokenAccount": "4D3xhDuDanWSDzXTksVjdCrgfPpoE9G5VWYdmVrA6ih2",
              "userSupportedTokenAccount": {
                "__option": "Some",
                "value": "C2cyUgbkih42h9z6zb5gxsadnowTgtLJfK4K2s5xX1as",
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

    const user2DelegateRes = await user2.reward.delegate.execute(
      { newDelegate: signer1.address },
      { signers: [signer2] }
    );
    await expectMasked(user2DelegateRes).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "delegate": null,
          "newDelegate": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
        },
        "events": {
          "unknown": [],
          "userDelegatedRewardAccount": {
            "delegate": {
              "__option": "Some",
              "value": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            },
            "receiptTokenMint": "Gmx4No1Fki926g1jKoWS2k3JJdGnQuR3L1RNMr7PcQVD",
            "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
            "userRewardAccount": "91mNEHtBj9vnqfrZNDp55di5HPmC6D8aPBNsm3mKtnW",
          },
        },
        "signature": "MASKED(signature)",
        "slot": "MASKED(/[.*S|s]lots?$/)",
        "succeeded": true,
      }
    `);
    await expect(
      user2.reward.resolve(true).then((res) => res?.delegate)
    ).resolves.toEqual(signer1.address);

    await expect(
      user2.reward.delegate.execute(
        { delegate: signer1.address, newDelegate: signer2.address },
        { signers: [signer1] }
      )
    ).resolves.not.toThrow();

    // fails to delegate
    await expect(
      user2.reward.delegate.execute(
        { delegate: signer1.address, newDelegate: signer2.address },
        { signers: [signer1] }
      )
    ).rejects.toThrowError('Transaction simulation failed'); // reward: user reward account authority must be either user or delegate
  });

  /** pricing source validation - supported token */
  test('fails if trying to add wrong pricing source when adding supported token', async () => {
    // stake pool pricing source
    await expect(
      ctx.fund.addSupportedToken.execute({
        mint: 'roxDFxTFHufJBFy3PgzZcgz6kwkQNPZpi9RfpcAv4bu',
        pricingSource: {
          __kind: 'SPLStakePool',
          address: 'BuMRVW5uUQqJmguCk4toGh7DB3CcJt6dk64JiUMdYS22',
        },
      })
    ).rejects.toThrowError('Transaction simulation failed'); // key not match error

    // orca liquidity pool pricing source
    await expect(
      ctx.fund.addSupportedToken.execute({
        mint: 'roxDFxTFHufJBFy3PgzZcgz6kwkQNPZpi9RfpcAv4bu',
        pricingSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
        },
      })
    ).rejects.toThrowError('Transaction simulation failed'); // AccountOwnedByWrongProgram

    await expect(
      ctx.fund.addSupportedToken.execute({
        mint: 'roxDFxTFHufJBFy3PgzZcgz6kwkQNPZpi9RfpcAv4bu',
        pricingSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: 'DxD41srN8Xk9QfYjdNXF9tTnP6qQxeF2bZF8s1eN62Pe', // inf/sol orca pool, example for not matching pool account
        },
      })
    ).rejects.toThrowError('Transaction simulation failed'); // key not match error
  });

  /** reward token mint validation at add compounding/distributing reward token */
  test('fails to add compounding/distributing reward token if provided reward token mint is not Mint account', async () => {
    const fund = await ctx.fund.resolveAccount(true);

    // fails because rewardTokenMint is not mint account (but system account)
    await expect(
      ctx.fund.addRestakingVaultCompoundingReward.execute({
        vault: fund?.data.restakingVaults[0].vault!,
        rewardTokenMint: '11111111111111111111111111111111',
      })
    ).rejects.toThrowError(); // Error Code: AccountOwnedByWrongProgram. Error Number: 3007. Error Message: The given account is owned by a different program than expected.

    // fails because rewardTokenMint is not mint account (but token account)
    await expect(
      ctx.fund.addRestakingVaultDistributingReward.execute({
        vault: fund?.data.restakingVaults[0].vault!,
        rewardTokenMint: ctx.fund.reserve.supportedTokens.children[0]
          .address as string,
      })
    ).rejects.toThrowError(); // Error Code: InvalidAccountData. Error Number: 17179869184. Error Message: An account's data contents was invalid.
  });

  test('token should be pegged to non-pegging token', async () => {
    await ctx.fund.addSupportedToken.execute({
      mint: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
      pricingSource: {
        __kind: 'SanctumSingleValidatorSPLStakePool',
        address: 'ArAQfbzsdotoKB5jJcZa3ajQrrPcWr2YQoDAEAiFxJAC',
      },
    });

    await ctx.fund.addSupportedToken.execute({
      mint: 'vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7',
      pricingSource: {
        __kind: 'PeggedToken',
        address: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
      },
    });

    // cannot pegg to pegging token
    await expect(
      ctx.fund.addSupportedToken.execute({
        mint: 'strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA',
        pricingSource: {
          __kind: 'PeggedToken',
          address: 'vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7',
        },
      })
    ).rejects.toThrowError();

    await expect(
      ctx.fund.addSupportedToken.execute({
        mint: 'strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA',
        pricingSource: {
          __kind: 'PeggedToken',
          address: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
        },
      })
    ).resolves.not.toThrow();

    // restore previous status
    await ctx.fund.removeSupportedToken.execute({
      mint: 'strng7mqqc1MBJJV6vMzYbEqnwVGvKKGKedeCvtktWA',
    });
    await ctx.fund.removeSupportedToken.execute({
      mint: 'vSoLxydx6akxyMD9XEcPvGYNGq6Nn66oqVb3UkGkei7',
    });
    await ctx.fund.removeSupportedToken.execute({
      mint: 'BonK1YhkXEGLZzwtcvRTip3gAL9nCeQD7ppZBLXhtTs',
    });
  });

  test.skip('user reward pool update fails when there are too many settlement blocks to synchronize', async () => {
    const MAX_REWARD_NUM = 16;
    const MAX_SETTLEMENT_BLOCK_NUM = 64;

    let rewardAccount = await ctx.reward.resolveAccount(true);
    const numOfAvailableReward =
      MAX_REWARD_NUM - rewardAccount!.data.numRewards;

    // add 16 rewards
    for (let i = 0; i < numOfAvailableReward; i++) {
      const reward = await validator.getSigner('mock reward' + i);
      await ctx.reward.addReward.execute({
        mint: reward.address,
        decimals: 9,
        name: 'mock reward' + i,
        description: 'mock reward for test',
      });
    }

    // user1 deposits sol to accumulate contribution
    await user1.resolveAddress(true);
    await validator.airdrop(user1.address!, 1_234_567_890_123n);
    await user1.deposit.execute(
      {
        assetAmount: 1_234_567_890_123n,
      },
      { signers: [signer1] }
    );

    /*
     * repeatedly call partial update ix to resolve DOS
     * - settle 1024 blocks to global reward pool
     * - user1 try to deposit and transaction exceeds maximum CU limit
     * - repeatedly settle 192 blocks to user reward pool (64 + 32 blocks to check boundary - kind of reward changes every 64 blocks)
     * - user1 succeeds to deposit without any error
     */

    // settle 16 * 64 blocks
    rewardAccount = await ctx.reward.resolveAccount(true);
    for (let i = 0; i < MAX_REWARD_NUM; i++) {
      const rewardAddress = rewardAccount!.data.rewards1[i].mint;
      for (let j = 0; j < MAX_SETTLEMENT_BLOCK_NUM; j++) {
        await ctx.reward.settleReward.execute({
          mint: rewardAddress,
          amount: 287_123_456_789_012_345n,
          isBonus: i == 0, // only settle fPoint in bonus pool
        });

        await validator.skipSlots(123_456_789n);
      }
    }

    // executing deposit instruction exceeds 1,400,000 CU
    await expect(
      user1.deposit.execute(
        {
          assetAmount: 1_000_000_000n,
        },
        { signers: [signer1] }
      )
    ).rejects.toThrowError();

    // repeatedly update 192 blocks
    await user1.reward.updatePools.executeChained({ numBlocksToSettle: 192 });

    // deposit succeeds after partial pool update
    await user1.deposit.executeChained(
      {
        assetAmount: 1_000_000_000n,
      },
      { signers: [signer1] }
    );
  });

  test('during unstake lst command execution, program should first withdraw stake from stake account whose vote account is preferred withdraw validator', async () => {
    /*
     * modified dynosol spl-stake-pool set 5th most lamport owned stake account's vote account as preferred validator
     * stake account1 : J9k2Epx9iftqxWrZJYE3AGYXhX6rp62zLxhcz6qmPn4Y, 25,423,208,013,321 active stake lamports
     * stake account2 : Cc7ge7LdGQoWS2u3YjJPD9czLaZFhMf42Tn4auCDpVHX, 25,401,409,690,850 active stake lamports
     * stake account3 : 6HjNQfjHErw3QPoYg4sg48R6t8ZpE3F48QhcLajc4uGQ, 20,423,570,527,132 active stake lamports
     * stake account4 : ENKZgg8HQYxpXya2WcetByHRQ9Vqg7zDSfxtJuG1JwFh, 20,423,509,088,968 active stake lamports
     * stake account5 : FtTUgU1ZG5QMXiCog1p7jVpb5PWK4vQGwd3qa2HKJfwk, 7,816,341,028,418 active stake lamports <== this stake account's vote account is set as preferred withdraw validator
     */
    await ctx.fund.addSupportedToken.execute({
      mint: 'DYNoyS3x5qgbccZg7RPXagm4xQzfnm5iwd9o8pMyJtdE',
      pricingSource: {
        __kind: 'SPLStakePool',
        address: 'DpooSqZRL3qCmiq82YyB4zWmLfH3iEqx2gy8f2B6zjru',
      },
    });

    await ctx.fund.updateAssetStrategy.execute({
      tokenMint: 'DYNoyS3x5qgbccZg7RPXagm4xQzfnm5iwd9o8pMyJtdE',
      tokenDepositable: true,
      solAllocationWeight: 1n,
      tokenAccumulatedDepositCapacityAmount: MAX_U64,
    });

    await validator.airdropToken(
      signer1.address,
      'DYNoyS3x5qgbccZg7RPXagm4xQzfnm5iwd9o8pMyJtdE',
      10_000_000_000_000n
    );

    await user1.deposit.execute(
      {
        assetMint: 'DYNoyS3x5qgbccZg7RPXagm4xQzfnm5iwd9o8pMyJtdE',
        assetAmount: 10_000_000_000_000n,
      },
      { signers: [signer1] }
    );

    const user1ReceiptToken1 = await user1.receiptToken.resolve(true);

    const res = await user1.requestWithdrawal.execute(
      {
        receiptTokenAmount: user1ReceiptToken1?.amount!,
      },
      { signers: [signer1] }
    );
    const requestId = res.events!.userRequestedWithdrawalFromFund!.requestId;

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'EnqueueWithdrawalBatch',
      operator: restaking.knownAddresses.fundManager,
    });

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
                        "allocatedTokenAmount": 10000000000000n,
                        "tokenMint": "DYNoyS3x5qgbccZg7RPXagm4xQzfnm5iwd9o8pMyJtdE",
                      },
                    ],
                    "withdrawSol": true,
                    "withdrawStakeItems": [
                      {
                        "fundStakeAccount": "Arz6vs8cC6N46nYFR2utcrvh7ZyoZNkCyJ86A121Eo9t",
                        "fundStakeAccountIndex": 0,
                        "validatorStakeAccount": "FtTUgU1ZG5QMXiCog1p7jVpb5PWK4vQGwd3qa2HKJfwk",
                      },
                      {
                        "fundStakeAccount": "8FjqGGNUyEEayyT7sLcKxr8Yr78BUP2aJSGGDfWCNdDb",
                        "fundStakeAccountIndex": 1,
                        "validatorStakeAccount": "J9k2Epx9iftqxWrZJYE3AGYXhX6rp62zLxhcz6qmPn4Y",
                      },
                      {
                        "fundStakeAccount": "5Ezv3ytgPTJv2bnFtNvV9wNmomYCwjtimwioq8kndu8x",
                        "fundStakeAccountIndex": 2,
                        "validatorStakeAccount": "Cc7ge7LdGQoWS2u3YjJPD9czLaZFhMf42Tn4auCDpVHX",
                      },
                      {
                        "fundStakeAccount": "6jN5rCxvwKTM8FYiVVFtnzJjSt19CkML6gDDhSjrd34A",
                        "fundStakeAccountIndex": 3,
                        "validatorStakeAccount": "6HjNQfjHErw3QPoYg4sg48R6t8ZpE3F48QhcLajc4uGQ",
                      },
                      {
                        "fundStakeAccount": "DYekvpKbCRD86pwavvFADBXJWeEbmvdBLhD24SXzLsVE",
                        "fundStakeAccountIndex": 4,
                        "validatorStakeAccount": "ENKZgg8HQYxpXya2WcetByHRQ9Vqg7zDSfxtJuG1JwFh",
                      },
                    ],
                  },
                },
              ],
            },
            "fundAccount": "7GyrFmKHXzKQupjFHVmCLdSqJwTFKf1BRAJux7BMgPpC",
            "nextSequence": 0,
            "numOperated": 6n,
            "receiptTokenMint": "9BRhJGuAZpefyy54pLu6N1dofCCp5XDjqTHXN1CBSPaT",
            "result": {
              "__option": "Some",
              "value": {
                "__kind": "UnstakeLST",
                "fields": [
                  {
                    "burntTokenAmount": 10000000000000n,
                    "deductedSolFeeAmount": 10164109114n,
                    "operationReceivableSolAmount": 9774422301958n,
                    "operationReservedSolAmount": 389686810808n,
                    "operationReservedTokenAmount": 0n,
                    "tokenMint": "DYNoyS3x5qgbccZg7RPXagm4xQzfnm5iwd9o8pMyJtdE",
                    "totalUnstakingSolAmount": 9764258192844n,
                    "unstakedSolAmount": 389686810808n,
                    "unstakingSolAmount": 9764258192844n,
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

  /** verify impact on global reward account by the state changes of the user_reward_accounts */

  // 1. deposit
  test('reward impact on deposit', async () => {
    const depositAmount = 10_000_000_000n; // 10 sol

    // first deposit
    await user1.deposit.execute(
      {
        assetAmount: depositAmount,
      },
      { signers: [signer1] }
    );

    const reward_1 = await ctx.reward.resolve(true);
    const user1Reward_1 = await user1.reward.resolve(true);

    // operator updates reward pools
    await ctx.reward.updatePools.execute({});

    const reward_2 = await ctx.reward.resolve(true);
    const user1Reward_2 = await user1.reward.resolve(true);

    let rewardElapsedSlots =
      reward_2!.basePool.updatedSlot - reward_1!.basePool.updatedSlot;
    let rewardTotalContributionAccrualRate =
      reward_1!.basePool.tokenAllocatedAmount.records.reduce(
        (sum, cur) =>
          sum + cur.amount * BigInt(cur.contributionAccrualRate * 100),
        0n
      );

    expect(reward_2!.basePool.contribution).toEqual(
      rewardElapsedSlots * rewardTotalContributionAccrualRate
    );
    expect(reward_2!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      depositAmount
    );
    expect(reward_2!.basePool.tokenAllocatedAmount.records[0].amount).toEqual(
      depositAmount
    );

    expect(user1Reward_2!.basePool.contribution).toEqual(0n);
    expect(user1Reward_2!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      depositAmount
    );
    expect(
      user1Reward_2!.basePool.tokenAllocatedAmount.records[0].amount
    ).toEqual(depositAmount);

    // second deposit
    await user1.deposit.execute(
      {
        assetAmount: depositAmount,
      },
      { signers: [signer1] }
    );

    const reward_3 = await ctx.reward.resolve(true);
    const user1Reward_3 = await user1.reward.resolve(true);

    rewardElapsedSlots =
      reward_3!.basePool.updatedSlot - reward_2!.basePool.updatedSlot;
    rewardTotalContributionAccrualRate =
      reward_2!.basePool.tokenAllocatedAmount.records.reduce(
        (sum, cur) =>
          sum + cur.amount * BigInt(cur.contributionAccrualRate * 100),
        0n
      );

    expect(reward_3!.basePool.contribution).toEqual(
      reward_2!.basePool.contribution +
        rewardElapsedSlots * rewardTotalContributionAccrualRate
    );
    expect(reward_3!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      depositAmount * 2n
    );
    expect(reward_3!.basePool.tokenAllocatedAmount.records[0].amount).toEqual(
      depositAmount * 2n
    );

    let user1RewardElapsedSlots =
      user1Reward_3!.basePool.updatedSlot - user1Reward_1!.basePool.updatedSlot;
    let user1RewardTotalContributionAccrualRate =
      user1Reward_1!.basePool.tokenAllocatedAmount.records.reduce(
        (sum, cur) =>
          sum + cur.amount * BigInt(cur.contributionAccrualRate * 100),
        0n
      );

    expect(user1Reward_3!.basePool.contribution).toEqual(
      user1RewardElapsedSlots * user1RewardTotalContributionAccrualRate
    );
    expect(user1Reward_3!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      depositAmount * 2n
    );
    expect(
      user1Reward_3!.basePool.tokenAllocatedAmount.records[0].amount
    ).toEqual(depositAmount * 2n);
  });

  // 2. transfer
  // 2.a. user1 (reward_account o) -> user2 (reward_account o)
  // 2.b. user1 (reward_account o) -> user2 (reward_account x)
  // 2.c. user1 (reward_account x) -> user2 (reward_account o)
  // 2.d. user1 (reward_account x) -> user2 (reward_account x)

  // 2.a. user1 (reward_account o) -> user2 (reward_account o)
  test('reward impact on transfer, user1 (reward_account o) -> user2 (reward_account o)', async () => {
    const depositAmount = 1_000_000_000n; // 1 sol

    // user1 creates user_reward_account
    await user1.deposit.execute(
      { assetAmount: depositAmount },
      { signers: [signer1] }
    );

    // user2 creates user_reward_account
    await user2.deposit.execute(
      { assetAmount: depositAmount },
      { signers: [signer2] }
    );

    const reward_1 = await ctx.reward.resolve(true);
    const user1Reward_1 = await user1.reward.resolve(true);
    const user2Reward_1 = await user2.reward.resolve(true);

    expect(user1Reward_1!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      depositAmount
    );
    expect(user2Reward_1!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      depositAmount
    );
    expect(reward_1!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      depositAmount * 2n
    );

    const user1_1 = await user1.resolve(true);
    const user2_1 = await user2.resolve(true);

    // user1 -> user2
    await user1.transfer.execute(
      {
        receiptTokenAmount: user1_1!.receiptTokenAmount,
        recipient: user2.address!,
      },
      { signers: [signer1] }
    );

    const reward_2 = await ctx.reward.resolve(true);
    const user1Reward_2 = await user1.reward.resolve(true);
    const user2Reward_2 = await user2.reward.resolve(true);

    // user1 reward's tokenAllocatedAmount moved to user2 reward
    expect(user1Reward_2!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      0n
    );
    expect(user2Reward_2!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      depositAmount * 2n
    );

    // global reward's tokenAllocatedAmount remains same
    expect(reward_2!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      reward_1!.basePool.tokenAllocatedAmount.totalAmount
    );

    // because both user1 and user2 have user_reward_account, so global reward's contribution is sum of these two's
    expect(reward_2!.basePool.contribution).toEqual(
      user1Reward_2!.basePool.contribution +
        user2Reward_2!.basePool.contribution
    );
  });

  // 2.b. user1 (reward_account o) -> user2 (reward_account x)
  test('reward impact on transfer, user1 (reward_account o) -> user2 (reward_account x)', async () => {
    const depositAmount = 1_000_000_000n; // 1 sol

    // user1 creates user_reward_account
    await user1.deposit.execute(
      { assetAmount: depositAmount },
      { signers: [signer1] }
    );

    const reward_1 = await ctx.reward.resolve(true);
    const user1Reward_1 = await user1.reward.resolve(true);

    expect(user1Reward_1!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      depositAmount
    );
    expect(reward_1!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      depositAmount
    );

    const user1_1 = await user1.resolve(true);
    const user2_1 = await user2.resolve(true);

    // user2 doesn't create user_reward_account
    // user1 -> user2
    await user1.transfer.execute(
      {
        receiptTokenAmount: user1_1!.receiptTokenAmount,
        recipient: user2.address!,
      },
      { signers: [signer1] }
    );

    const reward_2 = await ctx.reward.resolve(true);
    const user1Reward_2 = await user1.reward.resolve(true);

    // user1 reward's tokenAllocatedAmount moved away
    expect(user1Reward_2!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      0n
    );

    // global reward's tokenAllocatedAmount moved away too
    expect(reward_2!.basePool.tokenAllocatedAmount.totalAmount).toEqual(0n);

    // because user2 reward account doesn't exist, global reward's contribution is equal to user1 reward's contribution
    expect(reward_2!.basePool.contribution).toEqual(
      user1Reward_2!.basePool.contribution
    );
  });

  // 2.c. user1 (reward_account x) -> user2 (reward_account o)
  test('reward impact on transfer, user1 (reward_account x) -> user2 (reward_account o)', async () => {
    const depositAmount = 1_000_000_000n; // 1 sol

    // user2 creates user_reward_account
    await user2.deposit.execute(
      { assetAmount: depositAmount },
      { signers: [signer2] }
    );

    const reward_1 = await ctx.reward.resolve(true);
    const user2Reward_1 = await user2.reward.resolve(true);

    const user1_1 = await user1.resolve(true);
    const user2_1 = await user2.resolve(true);

    // user1 doesn't create user_reward_account
    // user2 just transfers receipt token to user1 to airdrop receipt token to user1
    await user2.transfer.execute(
      {
        receiptTokenAmount: user2_1!.receiptTokenAmount,
        recipient: user1.address!,
      },
      { signers: [signer2] }
    );

    const user1_2 = await user1.resolve(true);
    const user2_2 = await user2.resolve(true);

    // now user1 (reward_account x) -> user2 (reward_account o)
    await user1.transfer.execute(
      {
        receiptTokenAmount: user1_2!.receiptTokenAmount,
        recipient: user2.address!,
      },
      { signers: [signer1] }
    );

    const reward_2 = await ctx.reward.resolve(true);
    const user2Reward_2 = await user2.reward.resolve(true);

    // user2 reward's tokenAllocatedAmount doesn't change because it's returned back from user1
    expect(user2Reward_2!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      user2Reward_1!.basePool.tokenAllocatedAmount.totalAmount
    );

    // global reward's tokenAllocatedAmount doesn't change,
    // because user1 reward account doesn't exist,
    // so user1's action doesn't give an impact on the global reward.
    expect(reward_2!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      reward_1!.basePool.tokenAllocatedAmount.totalAmount
    );

    // global reward's contribution is equal to user2 reward's contribution,
    // because user1's action doesn't give an impact on the global reward.
    expect(reward_2!.basePool.contribution).toEqual(
      user2Reward_2!.basePool.contribution
    );
  });

  test('reward impact on transfer, user1 (reward_account x) -> user2 (reward_account x)', async () => {
    const depositAmount = 1_000_000_000n; // 1 sol

    // user3 creates user_reward_account
    await user3.deposit.execute(
      { assetAmount: depositAmount },
      { signers: [signer3] }
    );

    const reward_1 = await ctx.reward.resolve(true);
    const user3_1 = await user3.resolve(true);
    const user1_1 = await user1.resolve(true);

    // user3 transfers receipt token to user1
    await user3.transfer.execute(
      {
        receiptTokenAmount: user3_1!.receiptTokenAmount,
        recipient: user1.address!,
      },
      { signers: [signer3] }
    );

    const reward_2 = await ctx.reward.resolve(true);

    const user1_2 = await user1.resolve(true);
    const user2_1 = await user2.resolve(true);

    // now user1 (reward_accoount x) -> user2 (reward_account x)
    await user1.transfer.execute(
      {
        receiptTokenAmount: user1_2!.receiptTokenAmount,
        recipient: user2.address!,
      },
      { signers: [signer1] }
    );

    const reward_3 = await ctx.reward.resolve(true);

    // global reward's tokenAllocatedAmount moved away
    expect(reward_3!.basePool.tokenAllocatedAmount.totalAmount).toEqual(0n);
    expect(reward_3!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      reward_2!.basePool.tokenAllocatedAmount.totalAmount
    );

    // global reward's contribution accumulation has been stopped after user3 transferred his receipt token amount to user1.
    let rewardElapsedSlots =
      reward_2!.basePool.updatedSlot - reward_1!.basePool.updatedSlot;
    let rewardTotalContributionAccrualRate =
      reward_1!.basePool.tokenAllocatedAmount.records.reduce(
        (sum, cur) =>
          sum + cur.amount * BigInt(cur.contributionAccrualRate * 100),
        0n
      );

    expect(reward_3!.basePool.contribution).toEqual(
      rewardElapsedSlots * rewardTotalContributionAccrualRate
    );
  });

  // 3. request withdrawal
  test('reward impact on request withdrawal', async () => {
    // user1 deposits first
    const depositAmount = 1_000_000_000n; // 1 sol

    await user1.deposit.execute(
      { assetAmount: depositAmount },
      { signers: [signer1] }
    );

    const reward_1 = await ctx.reward.resolve(true);
    const user1Fund_1 = await user1.fund.resolve(true);
    const user1Reward_1 = await user1.reward.resolve(true);
    const user1_1 = await user1.resolve(true);

    expect(user1Reward_1!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      user1_1!.receiptTokenAmount
    );
    expect(reward_1!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      user1_1!.receiptTokenAmount
    );

    // user1 requests withdrawal
    await user1.requestWithdrawal.execute(
      { receiptTokenAmount: user1_1!.receiptTokenAmount },
      { signers: [signer1] }
    );

    const reward_2 = await ctx.reward.resolve(true);
    const user1Fund_2 = await user1.fund.resolve(true);
    const user1Reward_2 = await user1.reward.resolve(true);
    const user1_2 = await user1.resolve(true);

    // withdrawal requested receipt token amount is written at the user's fund_account
    expect(user1Fund_2!.withdrawalRequests[0].receiptTokenAmount).toEqual(
      user1_1!.receiptTokenAmount
    );

    expect(user1Reward_2!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      0n
    );
    expect(user1Reward_2!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      user1Reward_1!.basePool.tokenAllocatedAmount.totalAmount -
        user1_1!.receiptTokenAmount
    );

    expect(reward_2!.basePool.tokenAllocatedAmount.totalAmount).toEqual(0n);
    expect(reward_2!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      reward_1!.basePool.tokenAllocatedAmount.totalAmount -
        user1_1!.receiptTokenAmount
    );

    expect(user1_2!.receiptTokenAmount).toEqual(0n);

    // operator updates reward pools
    await ctx.reward.updatePools.execute({});

    // user1 updates reward pools
    await user1.reward.updatePools.execute({});

    const reward_3 = await ctx.reward.resolve(true);
    const user1Reward_3 = await user1.reward.resolve(true);

    // global reward's contribution has been stopped increasing after user1 requested withdrawal
    expect(reward_3!.basePool.contribution).toEqual(
      reward_2!.basePool.contribution
    );

    // user1 reward's contribution has been stopped increasing after user1 requested withdrawal
    expect(user1Reward_3!.basePool.contribution).toEqual(
      user1Reward_2!.basePool.contribution
    );
  });

  // 4. cancel withdrawal request
  test('reward impact on cancel withdrawal request', async () => {
    // user1 deposits first
    const depositAmount = 1_000_000_000n; // 1 sol

    await user1.deposit.execute(
      { assetAmount: depositAmount },
      { signers: [signer1] }
    );

    const reward_1 = await ctx.reward.resolve(true);
    const user1Fund_1 = await user1.fund.resolve(true);
    const user1Reward_1 = await user1.reward.resolve(true);
    const user1_1 = await user1.resolve(true);

    // user1 requests withdrawal
    await user1.requestWithdrawal.execute(
      { receiptTokenAmount: user1_1!.receiptTokenAmount },
      { signers: [signer1] }
    );

    const reward_2 = await ctx.reward.resolve(true);
    const user1Fund_2 = await user1.fund.resolve(true);
    const user1Reward_2 = await user1.reward.resolve(true);
    const user1_2 = await user1.resolve(true);

    // operator updates reward pools
    await ctx.reward.updatePools.execute({});

    const reward_3 = await ctx.reward.resolve(true);

    // global reward's contribution doesn't change because user1 requested withdrawal
    expect(reward_3!.basePool.contribution).toEqual(
      reward_2!.basePool.contribution
    );

    // user1 cancels withdrawal request
    await user1.cancelWithdrawalRequest.execute(
      { requestId: user1Fund_2!.withdrawalRequests[0].requestId },
      { signers: [signer1] }
    );

    const reward_4 = await ctx.reward.resolve(true);
    const user1Fund_3 = await user1.fund.resolve(true);
    const user1Reward_3 = await user1.reward.resolve(true);
    const user1_3 = await user1.resolve(true);

    expect(user1Reward_3!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      user1_1!.receiptTokenAmount
    );

    expect(reward_4!.basePool.tokenAllocatedAmount.totalAmount).toEqual(
      user1Reward_3!.basePool.tokenAllocatedAmount.totalAmount
    );

    expect(user1_3!.receiptTokenAmount).toEqual(user1_1!.receiptTokenAmount);

    // operator updates reward pools
    await ctx.reward.updatePools.execute({});

    // user1 updates reward pools
    await user1.reward.updatePools.execute({});

    const reward_5 = await ctx.reward.resolve(true);
    const user1Reward_4 = await user1.reward.resolve(true);

    // global reward's contribution would increase again
    expect(reward_5!.basePool.contribution).toBeGreaterThan(
      reward_4!.basePool.contribution
    );

    // user1 reward's contribution would increase again
    expect(user1Reward_4!.basePool.contribution).toBeGreaterThan(
      user1Reward_3!.basePool.contribution
    );
  });

  // 5. withdraw
  test('reward impact on withdraw', async () => {
    const depositAmount = 1_000_000_000n; // 1 sol

    // user1 deposits first
    await user1.deposit.execute(
      { assetAmount: depositAmount },
      { signers: [signer1] }
    );

    const reward_1 = await ctx.reward.resolve(true);
    const user1_1 = await user1.resolve(true);

    // user1 requests withdrawal
    await user1.requestWithdrawal.execute(
      { receiptTokenAmount: user1_1!.receiptTokenAmount },
      { signers: [signer1] }
    );

    const reward_2 = await ctx.reward.resolve(true);
    const user1Fund_2 = await user1.fund.resolve(true);

    // operator runs withdrawal batch commands
    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'EnqueueWithdrawalBatch',
      operator: restaking.knownAddresses.fundManager,
    });

    await ctx.fund.runCommand.executeChained({
      forceResetCommand: 'ProcessWithdrawalBatch',
      operator: restaking.knownAddresses.fundManager,
    });

    // user1 withdraws
    await user1.withdraw.execute(
      { requestId: user1Fund_2!.withdrawalRequests[0].requestId },
      { signers: [signer1] }
    );

    const reward_3 = await ctx.reward.resolve(true);

    // global reward's contribution counting has been stopped after user1 requested withdrawal
    expect(reward_3!.basePool.contribution).toEqual(
      reward_2!.basePool.contribution
    );
  });

  // 6. wrap
  test('reward impact on wrap', async () => {
    const depositAmount = 1_000_000_000n; // 1 sol

    // user1 deposits first
    await user1.deposit.execute(
      { assetAmount: depositAmount },
      { signers: [signer1] }
    );

    const reward_1 = await ctx.reward.resolve(true);
    const user1Reward_1 = await user1.reward.resolve(true);
    const user1_1 = await user1.resolve(true);
    const fund_1 = await ctx.fund.resolveAccount(true);

    // user1 wraps receipt token
    await user1.wrap.execute(
      { receiptTokenAmount: user1_1!.receiptTokenAmount },
      { signers: [signer1] }
    );

    const reward_2 = await ctx.reward.resolve(true);
    const user1Reward_2 = await user1.reward.resolve(true);
    const user1_2 = await user1.resolve(true);
    const fund_2 = await ctx.fund.resolveAccount(true);

    expect(fund_2!.data.wrappedToken.supply).toEqual(
      fund_1!.data.wrappedToken.supply + user1_1!.receiptTokenAmount
    );
    expect(fund_2!.data.wrappedToken.retainedAmount).toEqual(
      fund_2!.data.wrappedToken.supply
    );

    expect(user1_2!.wrappedTokenAmount).toEqual(user1_1!.receiptTokenAmount);
    expect(user1_2!.receiptTokenAmount).toEqual(0n);

    // operator updates reward pools
    await ctx.reward.updatePools.execute({});

    // user1 updates reward pools
    await user1.reward.updatePools.execute({});

    const reward_3 = await ctx.reward.resolve(true);
    const user1Reward_3 = await user1.reward.resolve(true);

    // global reward's contribution keeps increasing because fund wrap account reward account's contribution increases
    expect(reward_3!.basePool.contribution).toBeGreaterThan(
      reward_2!.basePool.contribution
    );

    // user1 reward's contribution has been stopped increasing after user wrapped his receipt token
    expect(user1Reward_3!.basePool.contribution).toEqual(
      user1Reward_2!.basePool.contribution
    );
  });

  // 7. unwrap
  test('reward impact on unwrap', async () => {
    const depositAmount = 1_000_000_000n; // 1 sol

    // user1 deposits first
    await user1.deposit.execute(
      { assetAmount: depositAmount },
      { signers: [signer1] }
    );

    const reward_1 = await ctx.reward.resolve(true);
    const user1Reward_1 = await user1.reward.resolve(true);
    const user1_1 = await user1.resolve(true);

    // user1 wraps receipt token
    await user1.wrap.execute(
      { receiptTokenAmount: user1_1!.receiptTokenAmount },
      { signers: [signer1] }
    );

    const reward_2 = await ctx.reward.resolve(true);
    const user1Reward_2 = await user1.reward.resolve(true);
    const user1_2 = await user1.resolve(true);
    const fund_2 = await ctx.fund.resolveAccount(true);

    // user1 unwraps receipt token
    await user1.unwrap.execute(
      { wrappedTokenAmount: user1_2!.wrappedTokenAmount },
      { signers: [signer1] }
    );

    const reward_3 = await ctx.reward.resolve(true);
    const user1Reward_3 = await user1.reward.resolve(true);
    const user1_3 = await user1.resolve(true);
    const fund_3 = await ctx.fund.resolveAccount(true);

    expect(fund_3!.data.wrappedToken.supply).toEqual(0n);
    expect(fund_3!.data.wrappedToken.retainedAmount).toEqual(
      fund_3!.data.wrappedToken.supply
    );

    expect(user1_3!.wrappedTokenAmount).toEqual(0n);
    expect(user1_3!.receiptTokenAmount).toEqual(user1_1!.receiptTokenAmount);

    // operator updates reward pools
    await ctx.reward.updatePools.execute({});

    // user1 updates reward pools
    await user1.reward.updatePools.execute({});

    const reward_4 = await ctx.reward.resolve(true);
    const user1Reward_4 = await user1.reward.resolve(true);

    // global reward's contribution keeps increasing because user1's reward contribution increases
    expect(reward_4!.basePool.contribution).toBeGreaterThan(
      reward_3!.basePool.contribution
    );

    // user1 reward's contribution starts to increase again after user1 unwrapped his wrapped token
    expect(user1Reward_4!.basePool.contribution).toBeGreaterThan(
      user1Reward_3!.basePool.contribution
    );
  });
});
