import {
  address,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
} from '@solana/kit';
import { LiteSVM } from 'litesvm';
import { describe, expect, test } from 'vitest';
import { RuntimeContext } from './context';
import { web3Compat } from './litesvm.web3js.node';

describe('all runtimes provide required rpc methods', async () => {
  const rpcUrl = process.env.SOLANA_RPC_DEVNET
    ? process.env.SOLANA_RPC_DEVNET
    : 'https://api.devnet.solana.com';
  const rpcSubscriptionsUrl = rpcUrl
    .replace('https://', 'wss://')
    .replace('http://', 'ws://');
  const solanaCtx = new RuntimeContext({
    rpc: createSolanaRpc(rpcUrl),
    rpcSubscriptions: createSolanaRpcSubscriptions(rpcSubscriptionsUrl),
    cluster: 'devnet',
  });

  const litesvm = LiteSVM.default().withBuiltins().withSysvars();
  const litesvmCtx = new RuntimeContext({
    type: 'litesvm',
    svm: litesvm,
  });

  const devnetAccountAddress = address(
    '4YHmpuyY54Bsj61qNxYGgtQy8xhacfnhdZ6W92rqB64w'
  );
  const builtinAccountAddress = address('11111111111111111111111111111111');
  const emptyAccountAddress = address('111111111111111111111111111111xx');

  test('getAccountInfo', async () => {
    const devnetAccount = await solanaCtx.rpc
      .getAccountInfo(devnetAccountAddress, {
        encoding: 'base64',
      })
      .send();
    expect(devnetAccount.value).not.null;

    // clone it to LiteSVM
    litesvm.setAccount(
      web3Compat.toPublicKey(devnetAccountAddress),
      web3Compat.toLegacyAccountInfoBytes(devnetAccount.value!)
    );
    const litesvmAccounts = await litesvmCtx.rpc
      .getAccountInfo(devnetAccountAddress, { encoding: 'base64' })
      .send();
    expect(litesvmAccounts.value).not.null;

    expect(litesvmAccounts.value).toMatchObject(devnetAccount.value!);
  });

  test('getAccounts', async () => {
    const devnetAccounts = await solanaCtx.rpc
      .getMultipleAccounts([builtinAccountAddress, emptyAccountAddress], {
        encoding: 'base64',
      })
      .send();
    expect(devnetAccounts.value).not.empty;

    const litesvmAccounts = await litesvmCtx.rpc
      .getMultipleAccounts([builtinAccountAddress, emptyAccountAddress], {
        encoding: 'base64',
      })
      .send();
    expect(litesvmAccounts).not.empty;

    expect(litesvmAccounts.value).toEqual(devnetAccounts.value);
  });

  test('loadMultipleAccounts, loadAccount with RuntimeContext', async () => {
    const litesvmAccounts = await litesvmCtx.fetchMultipleAccounts([
      builtinAccountAddress,
      emptyAccountAddress,
    ]);
    const devnetAccounts = await solanaCtx.fetchMultipleAccounts([
      builtinAccountAddress,
      emptyAccountAddress,
    ]);

    expect(litesvmAccounts).toEqual(devnetAccounts);
    expect(litesvmAccounts).toMatchInlineSnapshot(`
      [
        {
          "address": "11111111111111111111111111111111",
          "data": Uint8Array [
            115,
            121,
            115,
            116,
            101,
            109,
            95,
            112,
            114,
            111,
            103,
            114,
            97,
            109,
          ],
          "executable": true,
          "lamports": 1n,
          "programAddress": "NativeLoader1111111111111111111111111111111",
          "space": 14n,
        },
        null,
      ]
    `);
    expect(devnetAccounts).toMatchInlineSnapshot(`
      [
        {
          "address": "11111111111111111111111111111111",
          "data": Uint8Array [
            115,
            121,
            115,
            116,
            101,
            109,
            95,
            112,
            114,
            111,
            103,
            114,
            97,
            109,
          ],
          "executable": true,
          "lamports": 1n,
          "programAddress": "NativeLoader1111111111111111111111111111111",
          "space": 14n,
        },
        null,
      ]
    `);
  });

  test('loadAccount is utilizing cache', async () => {
    await expect(
      litesvmCtx.fetchAccount(emptyAccountAddress, true)
    ).resolves.toBeNull();

    // clone builtin account to empty account address
    const builtinAccount = await litesvmCtx.rpc
      .getAccountInfo(builtinAccountAddress, {
        encoding: 'base64',
      })
      .send();
    expect(builtinAccount.value).not.toBeNull();
    litesvm.setAccount(
      web3Compat.toPublicKey(emptyAccountAddress),
      web3Compat.toLegacyAccountInfoBytes(builtinAccount.value!)
    );

    // but it is still empty due to cache
    await expect(
      litesvmCtx.fetchAccount(emptyAccountAddress)
    ).resolves.toBeNull();

    // now not empty with force noCache
    await expect(
      litesvmCtx.fetchAccount(emptyAccountAddress, true)
    ).resolves.toMatchObject({
      lamports: builtinAccount!.value!.lamports,
    });

    // after overriding account data manually
    litesvm.setAccount(web3Compat.toPublicKey(emptyAccountAddress), {
      ...web3Compat.toLegacyAccountInfoBytes(builtinAccount.value!),
      lamports: 12345,
    });

    // still has old data
    await expect(
      litesvmCtx.fetchAccount(emptyAccountAddress)
    ).resolves.not.toMatchObject({
      lamports: 12345n,
    });

    // with invalidation
    litesvmCtx.invalidateAccount(emptyAccountAddress);

    // now has fresh data
    await expect(
      litesvmCtx.fetchMultipleAccounts([
        emptyAccountAddress,
        emptyAccountAddress,
      ])
    ).resolves.toMatchObject([
      {
        lamports: 12345n,
      },
      {
        lamports: 12345n,
      },
    ]);
  });
});
