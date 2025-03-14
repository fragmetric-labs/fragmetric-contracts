import * as web3 from '@solana/web3.js';
import { LiteSVM } from 'litesvm';
import { describe, expect, test } from 'vitest';
import { ProgramContext } from '../program';
import { RuntimeContext } from '../runtime';
import { BaseAccountContext } from './base';
import { FragmetricMetadataContext } from './metadata';
import { TokenAccountContext } from './token';

describe('AccountContext can be used to decode and transform raw account data', async () => {
  const litesvm = LiteSVM.default().withBuiltins().withSysvars();
  const program = new ProgramContext(
    new RuntimeContext({
      type: 'litesvm',
      svm: litesvm,
    })
  );

  test('BaseAccountContext', async () => {
    const ctx = new BaseAccountContext(
      program,
      '11111111111111111111111111111111'
    );
    expect(ctx.account).toBeUndefined();
    await ctx.resolveAccount();
    expect(ctx.account).toMatchInlineSnapshot(`
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
      }
    `);
    expect(ctx.account?.programAddress).toEqual(
      'NativeLoader1111111111111111111111111111111'
    );
  });

  test('TokenAccountContext', async () => {
    const dump = {
      pubkey: new web3.PublicKey(
        'B3iqP1N6xUAzHrJQCeUoZH1SrkADNxaLZfzGpzKhEZ3L'
      ),
      account: {
        lamports: 2108880,
        data: Uint8Array.from(
          Buffer.from(
            '1jQIm7aVczkUg33oUvvSTP8oTicipjSAaXZDynX3bJL1UnjbqR+6Wd3MvCUkaC6o45a2wgo7lKWjbkkuT3iIx7j7SQMU+QAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgcAAAAPAAEAAA==',
            'base64'
          )
        ),
        owner: new web3.PublicKey(
          'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb'
        ),
        executable: false,
        space: 175,
      },
    };
    litesvm.setAccount(dump.pubkey, dump.account);

    const ctx = TokenAccountContext.fromAssociatedTokenSeeds2022(program, () =>
      Promise.resolve({
        owner: 'HWdpqHAJ1U3hmFpqJg5tJVrjaCJ7PuzB6j1VQf5VDqgJ',
        mint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
      })
    );
    expect(ctx.account).toBeUndefined();
    expect(ctx.address).toBeUndefined();

    await ctx.resolveAddress();
    expect(ctx.address).toEqual(dump.pubkey.toString());
    expect(ctx.account).toBeUndefined();

    await ctx.resolveAccount();
    expect(ctx.account).not.toBeUndefined();
    expect(ctx.account).toMatchInlineSnapshot(`
      {
        "address": "B3iqP1N6xUAzHrJQCeUoZH1SrkADNxaLZfzGpzKhEZ3L",
        "data": {
          "amount": 273864349842360n,
          "closeAuthority": {
            "__option": "None",
          },
          "delegate": {
            "__option": "None",
          },
          "delegatedAmount": 0n,
          "extensions": {
            "__option": "Some",
            "value": [
              {
                "__kind": "ImmutableOwner",
              },
              {
                "__kind": "TransferHookAccount",
                "transferring": false,
              },
            ],
          },
          "isNative": {
            "__option": "None",
          },
          "mint": "FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo",
          "owner": "HWdpqHAJ1U3hmFpqJg5tJVrjaCJ7PuzB6j1VQf5VDqgJ",
          "state": 1,
        },
        "executable": false,
        "lamports": 2108880n,
        "programAddress": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
        "space": 175n,
      }
    `);
    expect(ctx.account).not.toBeUndefined();
    expect(ctx.account).toMatchInlineSnapshot(`
      {
        "address": "B3iqP1N6xUAzHrJQCeUoZH1SrkADNxaLZfzGpzKhEZ3L",
        "data": {
          "amount": 273864349842360n,
          "closeAuthority": {
            "__option": "None",
          },
          "delegate": {
            "__option": "None",
          },
          "delegatedAmount": 0n,
          "extensions": {
            "__option": "Some",
            "value": [
              {
                "__kind": "ImmutableOwner",
              },
              {
                "__kind": "TransferHookAccount",
                "transferring": false,
              },
            ],
          },
          "isNative": {
            "__option": "None",
          },
          "mint": "FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo",
          "owner": "HWdpqHAJ1U3hmFpqJg5tJVrjaCJ7PuzB6j1VQf5VDqgJ",
          "state": 1,
        },
        "executable": false,
        "lamports": 2108880n,
        "programAddress": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
        "space": 175n,
      }
    `);

    await ctx.resolveAccount(true);
  });

  test('FragmetricMetadataContext', async () => {
    const feed1 = new FragmetricMetadataContext(
      program,
      'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo'
    );
    const feed2 = new FragmetricMetadataContext(
      program,
      'xxxSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo'
    );
    await expect(
      Promise.all([feed1.resolveAccount(), feed2.resolveAccount()])
    ).resolves.toMatchObject([
      {
        address: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
        type: 'FRAGMETRIC_RESTAKED_TOKEN',
        displayName: 'Fragmetric Restaked SOL',
        symbol: 'fragSOL',
        // ... omitted
      },
      null,
    ]);

    // cannot create a feed in a circular way
    expect(() => FragmetricMetadataContext.from(feed2)).toThrowError(
      'circular'
    );

    const feed3 = FragmetricMetadataContext.from(
      new BaseAccountContext(program, '11111111111111111111111111111111')
    );
    await expect(feed3.resolveAccount()).resolves.toBeNull();
  });
});
