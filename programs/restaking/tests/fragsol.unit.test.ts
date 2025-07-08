import { getAddressDecoder } from '@solana/kit';
import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeFragSOL } from './fragsol.unit.init';

describe('restaking.fragSOL unit test', async () => {
  const testCtx = initializeFragSOL(await createTestSuiteContext());

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  const { validator, feePayer, restaking, initializationTasks } = testCtx;
  const ctx = restaking.fragSOL;

  const [signer1, signer2, signer3] = await Promise.all([
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
  const user1 = ctx.user(signer1);
  const user2 = ctx.user(signer2);
  const user3 = ctx.user(signer3);

  /** configuration **/
  test('pricing source addresses field in fund account updates correctly', async () => {
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

  test('remove token swap strategy', async () => {
    const fund_1 = await ctx.fund.resolve(true);
    expect(fund_1.tokenSwapStrategies).toHaveLength(1);

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
    ).rejects.toThrowError('Transaction simulation failed'); // fund: token swap strategy not found.
    await expect(
      ctx.fund.removeTokenSwapStrategy.execute({
        fromTokenMint: 'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn',
        toTokenMint: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
        swapSource: {
          __kind: 'OrcaDEXLiquidityPool',
          address: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL', // invalid swap source
        },
      })
    ).rejects.toThrowError('Transaction simulation failed'); // fund: token swap strategy validation error.

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
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            "userFundAccount": "8FYBsBTMsvx8a9UoDtD8DtV2815GxGbFNEwdR4mPLfQk",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            "userRewardAccount": "DWgJYd9xnUMH6eeMt6mymDLSzNCda7RWnxC6rNthZHWS",
          },
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 10000000000n,
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "mintedReceiptTokenAmount": 10000000000n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            },
            "updatedUserRewardAccounts": [
              "DWgJYd9xnUMH6eeMt6mymDLSzNCda7RWnxC6rNthZHWS",
            ],
            "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            "userFundAccount": "8FYBsBTMsvx8a9UoDtD8DtV2815GxGbFNEwdR4mPLfQk",
            "userReceiptTokenAccount": "GQAbtLfueaLzuxAq8aLDMu2gGfDiUpPrmdyuc3SaSEUV",
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
    expect(increasedContribution, 't7_1').toEqual(
      elapsedSlots *
        user1Reward_2?.basePool.tokenAllocatedAmount.records[0].amount! *
        BigInt(
          user1Reward_2?.basePool.tokenAllocatedAmount.records[0]
            .contributionAccrualRate!
        ) *
        100n
    );
    expect(
      user1Reward_2?.basePool.settlements[0].settledAmount,
      't7_2'
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
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 1000000000000000000n,
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "mintedReceiptTokenAmount": 830540552187657575n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "FRAGME9aN7qzxkHPmVP22tDhG87srsR9pr5SY9XdRd9R",
            },
            "updatedUserRewardAccounts": [
              "DWgJYd9xnUMH6eeMt6mymDLSzNCda7RWnxC6rNthZHWS",
            ],
            "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            "userFundAccount": "8FYBsBTMsvx8a9UoDtD8DtV2815GxGbFNEwdR4mPLfQk",
            "userReceiptTokenAccount": "GQAbtLfueaLzuxAq8aLDMu2gGfDiUpPrmdyuc3SaSEUV",
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
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "nextSequence": 0,
            "numOperated": 2n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "result": {
              "__option": "Some",
              "value": {
                "__kind": "EnqueueWithdrawalBatch",
                "fields": [
                  {
                    "enqueuedReceiptTokenAmount": 83048479175n,
                    "totalQueuedReceiptTokenAmount": 83048479175n,
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
                        "fundStakeAccount": "FLhN5pBMowDVsUdHgeGvRmXUU33rQsYyDEpfQrVLnme4",
                        "fundStakeAccountIndex": 0,
                        "validatorStakeAccount": "EmutJdbKJ55hUyth15bar8ZxDCchR44udAXWYg9eLLDL",
                      },
                      {
                        "fundStakeAccount": "w9kjgBJeTeTnoLEPLXo4Zi9wtj3F2FMwnHb7UrCtPF4",
                        "fundStakeAccountIndex": 1,
                        "validatorStakeAccount": "Cwx3iMVjmJWTG5156eMGyNRQhBrGiyvnUnjqXVxXYEmL",
                      },
                      {
                        "fundStakeAccount": "53ysYB98VupmR7XPKm5mf86qYt6nK6edgWJKEPQtuk7X",
                        "fundStakeAccountIndex": 2,
                        "validatorStakeAccount": "AjQ5c1GCQkJcg6uukAYhjxY2wSKfX3Lb27FeXUdh8xe4",
                      },
                    ],
                  },
                },
              ],
            },
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "nextSequence": 0,
            "numOperated": 6n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
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
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "nextSequence": 0,
            "numOperated": 10n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "result": {
              "__option": "Some",
              "value": {
                "__kind": "ClaimUnstakedSOL",
                "fields": [
                  {
                    "claimedSolAmount": 100034339172n,
                    "offsettedAssetReceivables": [
                      {
                        "assetAmount": 100034339172n,
                        "assetTokenMint": {
                          "__option": "None",
                        },
                      },
                    ],
                    "offsettedSolReceivableAmount": 100034339172n,
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
                    "receiptTokenAmount": 83048479175n,
                  },
                },
              ],
            },
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "nextSequence": 0,
            "numOperated": 13n,
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
                    "deductedAssetFeeAmount": 200268946n,
                    "offsettedAssetReceivables": [
                      {
                        "assetAmount": 100134473n,
                        "assetTokenMint": {
                          "__option": "None",
                        },
                      },
                    ],
                    "processedBatchAccounts": [
                      "BSAf3XCVkGJthdmprrf9faibAhcDW8m67DJuHk15tkM7",
                    ],
                    "processedReceiptTokenAmount": 83048479175n,
                    "requestedReceiptTokenAmount": 83048479175n,
                    "requiredAssetAmount": 0n,
                    "reservedAssetUserAmount": 99934204699n,
                    "transferredAssetRevenueAmount": 102417353n,
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
            "burntReceiptTokenAmount": 83048479175n,
            "deductedFeeAmount": 200268947n,
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "fundWithdrawalBatchAccount": "BSAf3XCVkGJthdmprrf9faibAhcDW8m67DJuHk15tkM7",
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "requestId": 1n,
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
            "withdrawnAmount": 99934204699n,
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
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
            "userFundAccount": "C4oLiF2syWC13edFZTs8dJrQPCZRUy68847i65ABtw26",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
            "userRewardAccount": "5XfY8Akkz4pShk2LWEkqXJsQyTB2DqwwDCnRisiG4qKT",
          },
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 10000000000n,
            "fundAccount": "7xraTDZ4QWgvgJ5SCZp4hyJN2XEfyGRySQjdG49iZfU8",
            "mintedReceiptTokenAmount": 10000000000n,
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            },
            "updatedUserRewardAccounts": [
              "5XfY8Akkz4pShk2LWEkqXJsQyTB2DqwwDCnRisiG4qKT",
            ],
            "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
            "userFundAccount": "C4oLiF2syWC13edFZTs8dJrQPCZRUy68847i65ABtw26",
            "userReceiptTokenAccount": "6cbZmnm1m76g7Lt1eMTNz2EXbNG8kirE17XnvtPpq2Uc",
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
            "receiptTokenMint": "Cs29UiPhAkM2v8fZW7qCJ1UjhF1UAhgrsKj61yGGYizD",
            "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
            "userRewardAccount": "5XfY8Akkz4pShk2LWEkqXJsQyTB2DqwwDCnRisiG4qKT",
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

    await user2.reward.delegate.execute(
      { delegate: signer1.address, newDelegate: signer2.address },
      { signers: [signer1] }
    );

    // fails to delegate
    await expect(
      user2.reward.delegate.execute(
        { delegate: signer1.address, newDelegate: signer2.address },
        { signers: [signer1] }
      )
    ).rejects.toThrowError('Transaction simulation failed'); // reward: user reward account authority must be either user or delegate
  });

  /** pricing source validation - supported token */
  test(`fails if trying to add wrong pricing source when adding supported token`, async () => {
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
});
