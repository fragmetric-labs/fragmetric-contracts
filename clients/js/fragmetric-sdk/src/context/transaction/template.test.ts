import {
  getCreateAccountWithSeedInstruction,
  getInitializeNonceAccountInstruction,
  getNonceSize,
  getTransferSolInstruction,
  SYSTEM_PROGRAM_ADDRESS,
} from '@solana-program/system';
import {
  AccountRole,
  address,
  Address,
  createAddressWithSeed,
  createKeyPairSignerFromBytes,
  createNoopSigner,
  getBase58Decoder,
  getBase64EncodedWireTransaction,
  getCompiledTransactionMessageDecoder,
  getTransactionDecoder,
  Lamports,
  lamports,
  SignatureBytes,
} from '@solana/kit';
import { LiteSVM } from 'litesvm';
import { describe, expect, test } from 'vitest';
import {
  getUserDepositedToFundDecoder,
  getUserDepositedToFundDiscriminatorBytes,
} from '../../generated/restaking';
import { ProgramContext } from '../program';
import { TransactionTemplateContext } from './template';

const signer = await createKeyPairSignerFromBytes(
  Uint8Array.from([
    18, 99, 108, 102, 2, 206, 6, 7, 168, 174, 190, 163, 59, 172, 204, 141, 105,
    14, 181, 146, 108, 161, 134, 128, 169, 57, 152, 205, 238, 237, 220, 216,
    150, 75, 239, 172, 33, 80, 166, 64, 55, 49, 182, 185, 30, 49, 104, 33, 14,
    163, 68, 64, 59, 209, 64, 244, 34, 15, 83, 110, 17, 139, 78, 4,
  ])
);

describe('can parse transaction result and events', async () => {
  const program = ProgramContext.devnet(
    process.env.SOLANA_RPC_DEVNET,
    undefined,
    'frag9zfFME5u1SNhUYGa4cXLzMKgZXF3xwZ2Y1KCYTQ'
  );

  const svm = new LiteSVM().withBuiltins().withSysvars().withSplPrograms();

  test('can sign transaction with default signers', async () => {
    const program = ProgramContext.connect({
      type: 'litesvm',
      svm,
    });
    await program.runtime.rpc.requestAirdrop!(
      signer.address,
      lamports(1_000_000_000n)
    ).send();
    await expect(
      new TransactionTemplateContext(program, null, {
        feePayer: signer,
      }).execute(null)
    ).resolves.toMatchObject({
      events: {
        unknown: [],
      },
      result: {
        meta: {
          computeUnitsConsumed: 0n,
          err: null,
          fee: 0n,
          innerInstructions: [],
          loadedAddresses: {
            readonly: [],
            writable: [],
          },
          logMessages: [],
          postBalances: [],
          postTokenBalances: [],
          preBalances: [],
          preTokenBalances: [],
          rewards: [],
          status: {
            Ok: null,
          },
        },
        slot: 0n,
        transaction: {
          feePayer: {
            address: 'B7hKA5KCnPb465QCJMHwfGfyfzemQ3Nvvxsa9K7cQTUw',
          },
          instructions: [],
          lifetimeConstraint: {
            blockhash: '3UY9LH9KcRXfXFSXZ3P6aoak2NeXtYrQQnuN79pGNiSw',
            lastValidBlockHeight: 18446744073709551615n,
          },
          version: 0,
        },
        version: 0,
      },
      signature:
        '5dCJnMbsqdgvnto5pTPrpDS1z1iKkmtcoDPTaJTuQuCZWWP8omWzQHVdi4Pn1P5D9FS3r27TzepfFZVknGZCqVtL',
    });
  });

  test('execute an empty transaction to confirm and fetch', async () => {
    await expect(
      new TransactionTemplateContext(program, null, {
        feePayer: signer,
      }).execute(null)
    ).resolves.toMatchObject({
      events: {
        unknown: [],
      },
      result: {
        meta: {
          innerInstructions: [],
          loadedAddresses: {
            readonly: [],
            writable: [],
          },
          logMessages: [],
        },
        transaction: {
          feePayer: {
            address: 'B7hKA5KCnPb465QCJMHwfGfyfzemQ3Nvvxsa9K7cQTUw',
          },
          instructions: [],
          lifetimeConstraint: {},
          version: 0,
        },
        version: 0n,
      },
    });
  });

  test('directly parse a transaction to decode events', async () => {
    await expect(
      new TransactionTemplateContext(program, null, {}).parse(
        '5ruoVL4kGU3LCcPqsbs7qU2bTD5AwPa6PLB5KM166hKLweDSKrEzo2WufCLe9QEgyJcUcB3j8NsBHkPQxpTAVTJS' // operator run fund commands failed
      )
    ).resolves.toMatchObject({
      succeeded: false,
    });

    await expect(
      new TransactionTemplateContext(program, null, {
        anchorEventDecoders: {
          UserDepositedToFund: {
            discriminator: getUserDepositedToFundDiscriminatorBytes(),
            decoder: getUserDepositedToFundDecoder(),
          },
        },
      }).parse(
        '2sth1831gHoTkQa4wkGsiEpegSjd5gxbYUASecAnozCvxFHogdaQ752eFmXyb6bMBp3FFBUjqYkGFNxbFQqfRLoQ' // deposit
        // '4zxMkCBiVYQ7TqGy8zW9CxVVVVPV1rhFhoykB6K4iDKDrCMifuBGcMUJbaagQQU9E146n2WAAqhgbvEiGVBFeUyd' // request withdrawal
      )
    ).resolves.toMatchObject({
      succeeded: true,
      events: {
        unknown: [],
        UserDepositedToFund: {
          contributionAccrualRate: {
            __option: 'Some',
            value: 110,
          },
          depositedAmount: 1000000n,
          fundAccount: '4YHmpuyY54Bsj61qNxYGgtQy8xhacfnhdZ6W92rqB64w',
          mintedReceiptTokenAmount: 945689n,
          receiptTokenMint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
          supportedTokenMint: {
            __option: 'None',
          },
          updatedUserRewardAccounts: [
            'CCmAQhBS1FxU5TEbtmMCXZez1skdtZ2RfW5KR7xUjDFE',
          ],
          user: '91zBeWL8kHBaMtaVrHwWsck1UacDKvje82QQ3HE2k8mJ',
          userFundAccount: '3jrmHNGtqncqvfKq5o1SDrYywrVPqoUJJ5NDPGeu1uE6',
          userReceiptTokenAccount:
            'BQ2F6kmvoabdahJJKnwsDY91E8d6AskC7vEszeCfaVTG',
          userSupportedTokenAccount: {
            __option: 'None',
          },
          walletProvider: {
            __option: 'Some',
            value: 'PHANTOM',
          },
        },
      },
      signature:
        '2sth1831gHoTkQa4wkGsiEpegSjd5gxbYUASecAnozCvxFHogdaQ752eFmXyb6bMBp3FFBUjqYkGFNxbFQqfRLoQ',
      result: {
        meta: {
          // .. omitted
        },
        transaction: {
          // .. omitted
        },
      },
    });
  });
});

describe('default signer works', async () => {
  const svm = new LiteSVM().withBuiltins().withSysvars().withSplPrograms();

  test('can sign transaction with default signers', async () => {
    const program = ProgramContext.connect(
      {
        type: 'litesvm',
        svm,
      },
      {
        transaction: {
          feePayer: signer.address,
          signers: [
            createNoopSigner(signer.address),
            createNoopSigner(signer.address),
            signer,
          ],
        },
      }
    );
    await program.runtime.rpc.requestAirdrop!(
      signer.address,
      lamports(1_000_000_000n)
    ).send();

    const txTemplate1 = new TransactionTemplateContext(program, null, {
      instructions: [
        getTransferSolInstruction({
          amount: 1000n as Lamports,
          destination: signer.address as Address,
          source: createNoopSigner(signer.address),
        }),
      ],
    });

    const sig = await txTemplate1.sendAndConfirm(null);

    await expect(
      program.runtime.rpc.getTransaction(sig).send()
    ).resolves.not.toThrow();
  });
});

describe.each([
  [
    'litesvm',
    ProgramContext.connect({
      type: 'litesvm',
      svm: new LiteSVM().withBuiltins().withSysvars().withSplPrograms(),
    }),
  ],
  ['devnet', ProgramContext.devnet(process.env.SOLANA_RPC_DEVNET)],
])('handle transactions on %s', async (title, program) => {
  if (program.runtime.type == 'litesvm') {
    await program.runtime.rpc.requestAirdrop!(
      signer.address,
      lamports(1_000_000_000n)
    ).send();
  }

  // skip possibly a single slot for expiration test
  async function sleep() {
    if (program.runtime.config.type == 'litesvm') {
      const svm = program.runtime.config.svm;
      svm.expireBlockhash();
      svm.warpToSlot(svm.getClock().slot + 1n);
      return;
    }
    return new Promise((resolve) => setTimeout(resolve, 10 * 1000));
  }

  test('can build transaction (manual blockhash)', async () => {
    const tx = new TransactionTemplateContext(program, null, {
      // can give address or signer here
      feePayer: signer.address,
      // also can give extra signers for missing ones
      signers: [],
    });

    // no signer yet
    await expect(tx.sendAndConfirm(null)).rejects.toThrowError(
      'Transaction is missing signatures'
    );

    // invalid block hash
    await expect(
      tx.sendAndConfirm(null, {
        // can give additional signers here
        signers: [signer],
        recentBlockhash: {
          blockhash: 'FwqW96qQwKVARG8D9hWLcYrkALBRpBemmpuaY7FdXxxx',
          lastValidBlockHeight: 777n,
        },
      })
    ).rejects.toThrowError('Transaction simulation failed');
  });

  test('can build transaction (automatic blockhash)', async () => {
    const txTemplate = new TransactionTemplateContext(program, null, {
      feePayer: signer,
    });

    // get a serialized one from template
    const tx1 = await txTemplate.serializeToBase64(null);
    await expect(
      program.runtime.rpc
        .simulateTransaction(tx1, { encoding: 'base64' })
        .send()
    ).resolves.not.toThrow();

    await expect(
      program.runtime.rpc.sendTransaction(tx1, { encoding: 'base64' }).send()
    ).resolves.toBeTypeOf('string');

    await sleep();

    await expect(
      program.runtime.rpc.sendTransaction(tx1, { encoding: 'base64' }).send()
    ).rejects.toThrowError('Transaction simulation failed');

    await sleep();

    // send and confirm new tx with fresh blockhash (automatically fetched)
    await expect(txTemplate.sendAndConfirm(null)).resolves.not.toThrow();
  });

  test('can build transaction (durable nonce) and use .send()', async () => {
    const nonceAccountAddress = await createAddressWithSeed({
      baseAddress: signer.address,
      programAddress: SYSTEM_PROGRAM_ADDRESS,
      seed: 'nonce',
    });

    if (program.runtime.type == 'litesvm') {
      const txTemplate1 = new TransactionTemplateContext(program, null, {
        feePayer: signer,
        instructions: [
          async () => {
            const nonceAccountSize = BigInt(getNonceSize());
            const createAccount = getCreateAccountWithSeedInstruction({
              payer: signer,
              newAccount: nonceAccountAddress,
              base: signer.address,
              baseAccount: signer,
              seed: 'nonce',
              amount: await program.runtime.rpc
                .getMinimumBalanceForRentExemption(nonceAccountSize)
                .send(),
              space: nonceAccountSize,
              programAddress: SYSTEM_PROGRAM_ADDRESS,
            });
            const initializeNonceAccount = getInitializeNonceAccountInstruction(
              {
                nonceAccount: nonceAccountAddress,
                nonceAuthority: signer.address,
              }
            );

            return [createAccount, initializeNonceAccount];
          },
        ],
      });

      // create a nonce account
      await expect(
        txTemplate1.sendAndConfirm(null, {}, { encoding: 'base64' })
      ).resolves.not.toThrowError();
    }

    // send tx with durable nonce
    const txTemplate2 = new TransactionTemplateContext(program, null, {
      feePayer: signer,
      durableNonce: {
        nonceAccountAddress: nonceAccountAddress,
      },
      instructions: [
        getTransferSolInstruction({
          amount: 1000n as Lamports,
          destination: signer.address as Address,
          source: createNoopSigner(signer.address),
        }),
      ],
    });

    // can repetitively send txs by building tx with fresh nonce
    const sig1 = await txTemplate2.sendAndConfirm(null);

    const sig2 = await txTemplate2.sendAndConfirm(null);

    expect(sig1).not.toEqual(sig2);

    // fails to send multiple time with a same tx
    const tx3Serialized = await txTemplate2.serialize(null);
    const tx3Wired = getBase64EncodedWireTransaction(tx3Serialized);
    const tx3Decoded = getCompiledTransactionMessageDecoder().decode(
      getTransactionDecoder().decode(Buffer.from(tx3Wired, 'base64'))
        .messageBytes
    );
    expect(tx3Decoded.lifetimeToken).toEqual(
      'nonce' in tx3Serialized.lifetimeConstraint
        ? tx3Serialized.lifetimeConstraint.nonce
        : tx3Serialized.lifetimeConstraint.blockhash
    );
    const tx3WiredSig = getBase58Decoder().decode(
      Object.values(
        getTransactionDecoder().decode(Buffer.from(tx3Wired, 'base64'))
          .signatures
      )[0]!
    );

    const sig3 = await program.runtime.rpc
      .sendTransaction(tx3Wired, { encoding: 'base64' })
      .send();
    await sleep();

    expect(sig2).not.toEqual(sig3);
    expect(sig3).toEqual(tx3WiredSig);
    await expect(
      program.runtime.rpc
        .sendTransaction(tx3Wired, { encoding: 'base64' })
        .send()
    ).rejects.toThrowError('simulation failed');
    await expect(
      program.runtime.rpc
        .getTransaction(sig3, { maxSupportedTransactionVersion: 0 })
        .send()
    ).resolves.toMatchObject({
      meta: {
        logMessages: [
          'Program 11111111111111111111111111111111 invoke [1]',
          'Program 11111111111111111111111111111111 success',
          'Program 11111111111111111111111111111111 invoke [1]',
          'Program 11111111111111111111111111111111 success',
        ],
      },
    });

    // send and confirm new tx with fresh nonce
    await expect(txTemplate2.sendAndConfirm(null)).resolves.toBeTypeOf(
      'string'
    );
  });

  test('can sign transaction with various signer types', async () => {
    const txTemplate1 = new TransactionTemplateContext(program, null, {
      feePayer: signer.address, // just address is given
      instructions: [
        getTransferSolInstruction({
          amount: 1000n as Lamports,
          destination: signer.address as Address,
          source: createNoopSigner(signer.address),
        }),
      ],
    });

    await expect(txTemplate1.assemble(null)).resolves.not.toThrow();
    await expect(txTemplate1.serialize(null)).rejects.toThrowError(
      `Transaction is missing signatures for addresses: B7hKA5KCnPb465QCJMHwfGfyfzemQ3Nvvxsa9K7cQTUw`
    );
    await expect(txTemplate1.sendAndConfirm(null)).rejects.toThrowError(
      `Transaction is missing signatures for addresses: B7hKA5KCnPb465QCJMHwfGfyfzemQ3Nvvxsa9K7cQTUw`
    );

    // valid signer
    await expect(
      txTemplate1.simulate(null, { signers: [signer] })
    ).resolves.not.toThrow();

    // invalid signing
    await expect(
      txTemplate1.simulate(
        null,
        {
          signers: [
            {
              address: signer.address,
              async signTransactions(transactions, config) {
                return transactions.map((tx) => {
                  return {
                    [signer.address]: new Uint8Array([
                      1, 2, 3,
                    ]) as SignatureBytes,
                  };
                });
              },
            },
          ],
        },
        { sigVerify: true }
      )
    ).rejects.toThrowError('signature');

    // modifying signer works
    await expect(
      txTemplate1.simulate(null, {
        signers: [
          {
            address: signer.address,
            async modifyAndSignTransactions(transactions, config) {
              const signatureMaps = await signer.signTransactions(transactions);
              return transactions.map((tx) => {
                return {
                  ...tx,
                  signatures: {
                    ...tx.signatures,
                    ...signatureMaps[0],
                  },
                };
              });
            },
          },
        ],
      })
    ).resolves.not.toThrow();

    // sending signer who doesn't share the signature to the app
    await expect(
      txTemplate1.send(null, {
        signers: [
          {
            address: signer.address,
            async signAndSendTransactions(transactions, config?) {
              return signer
                .signTransactions(transactions)
                .then((signatureMaps) => {
                  return signatureMaps.map((signatureMap, i) => {
                    const tx = transactions[i];

                    // here custom tx sending logic...

                    // return tx signature
                    return signatureMap[signer.address];
                  });
                });
            },
          },
        ],
      })
    ).resolves.not.toThrow();
  });

  test.skipIf(program.runtime.type == 'litesvm')(
    'tx with address lookup table',
    async () => {
      const txTemplate1 = new TransactionTemplateContext(program, null, {
        feePayer: signer,
        addressLookupTables: [
          '6VHmiiuZAW2PVoY5N16oqs8wYVkXnfmZBcM7Vkbb76jH', // existing on devnet
          'AQtDes99nLUnSK6BQJgj9KJ6b3eDv8bUUxGCmnEJUkY5', // not existing on devnet
        ],
        instructions: [
          getTransferSolInstruction({
            amount: 500_000n as Lamports,
            destination: signer.address,
            source: signer,
          }),
          {
            accounts: [
              {
                address: address(
                  'DaFJQBXyuFAgyCSpLs7db3H9YiiuWGfVMXocumEqFNdu'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  'FRAGJ157KSDfGvBJtCSrsTWUqFnZhrw4aC8N8LqHuoos'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  'CxfTU9Vt9YpDwtQj5HnUfqHrHez7LTwD4UVXn4gHvfUW'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  'EQmDqtcHbTU19twRoRPmjntd9zG3PeeuG4eB8ex3TUio'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  '2AWzCFX96yRXbMz8Ew1yj372GbDUVynWVpRpmGdVqME8'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  '3cXLrxLPg7e7W2UeL7iNdkVxqeHPLLTTH7zQow9zvJH5'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  '4tRSH1SurNoW57srvmtU37EciRRRFH116st5q5DccMiP'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address('nVu5UFUhraA1LKJuDzKNX9F4m3v1bAPXixu65fFEfMb'),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  'APCaPNBDgdiJiy8ie54GmQu4PHpJtxcfR75twPedBt4s'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  'D4PehTja14kQFYLmNTzN2x2dfxMURbG7ks6b4SpTr5yq'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address('Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8'),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  '9eZbWiHsPRsxLSiHxzg2pkXsAuQMwAjQrda7C7e21Fw6'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  'AHHvidK4yCtTems94FUkKvsdJDXXSEwY4kj7TaDrvnoH'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address('UwuSgAq4zByffCGCrWH87DsjfsewYjuqHfJEpzw1Jq3'),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  '7dCQpU5w6Xz3aAnpFrXByBg9LxLdz33deUCrWJAVcNaE'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  '6VSjoP9hyHKKNZfcDzrAKRKWKSnyKhzLgBR9dtewPN9z'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address('W1MLgLiJ3XPdkWGYTxDcRvtmLip6m23nbPG5QF3Hb6x'),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  'HkN9jQzcHfMz5MWMbVwd7eDBL8uZHh2PebsE4iBoduQF'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  '4D3wGahA5fbUEHjkBn719b7C8uTYjQVB75ZeEeJGvjN9'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  '2kisLCMCZLPPyoR1R15AhPjRKj1aU8P6paPhznM7GNKj'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  'FKirD7tzEXk2yUHQ2kLiXGZA5aH76A8Qizeo2FSFKVFc'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  '2zafawvhvPfGb54NzE2Cp5SipvyPvQQBy5NNMKGfMfXL'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  '9GN9cmL5R4mcKtBU16JXS9XNxFzABTbYhNBXySsgYtLk'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  '5soeDM2NddDan7DPtmfakh3hUVZWCcqzDrSdEhPTopDb'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  'AGpGjQ8665wx6pM268dFRg1WteNxDxHTbwDyT6eQYxuS'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address('SRCMj3B7cYjvwTtqJxUSptgJPWkL8bHLrQme6q4zHn7'),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  'F8QYVvPxrtyYRiKdFdcHqGYxGUeTwvR8JNuNS3HuZpfb'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  'FAkEjwHSbxkojmdiMurSXR11dU5jbfoqVhtjFCXbM1hh'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address('fragkamrANLvuZYQPcmPsCATQAabkqNGH6gxqqPG3aP'),
                role: AccountRole.READONLY,
              },
              {
                address: address('RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q'),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  '4vvKh3Ws4vGzgXRVdo8SdL4jePXDvCqKVmi21BCBGwvn'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  'BQQVo6sz9pTjD1P88C7WgCo4ABLxr8PM6Ycu4fzDZmBQ'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address('WFRGJnQt5pK8Dv4cDAbrSsgPcmboysrmX3RYhmRRyTR'),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  '56CtxyY6EsP45vvpJUspAyERcf7uCsMh3QEM9KT9VkAj'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  'FVmBQTGa3mCuaT7drZtguVjTCpM1ue6qUpJDb5YhNEYm'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  '8WfG1BMRH3ciHFwAG4FksTGuHaq63jExGcrqFAHKqy9N'
                ),
                role: AccountRole.READONLY,
              },
              {
                address: address(
                  'BBtrmwew5PgCTteBd39acqkfNB7tcccQNCrvtsZGMykU'
                ),
                role: AccountRole.READONLY,
              },
            ],
            programAddress: SYSTEM_PROGRAM_ADDRESS,
            data: Uint8Array.from([1, 2, 3, 4]),
          },
        ],
      });
      await expect(
        txTemplate1.serialize(null).then((tx) => tx.messageBytes.length)
      ).resolves.toBeLessThan(300);
    }
  );
});
