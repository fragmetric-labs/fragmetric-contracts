import { describe, expect, test } from 'vitest';
import { RestakingProgram } from './program';
import { testSignerResolver } from './testing_fixture';

describe('RestakingUserContext on devnet', async () => {
  const program = RestakingProgram.devnet(process.env.SOLANA_RPC_DEVNET);
  const signer = await testSignerResolver();
  const user = program.fragSOL.user(signer.address);

  test('can get user data and related accounts', async () => {
    await user.resolveAccount();
    expect(user.account?.lamports).toBeTypeOf('bigint');
    expect(user.address).toEqual(
      'B7hKA5KCnPb465QCJMHwfGfyfzemQ3Nvvxsa9K7cQTUw'
    );

    await expect(user.fund.resolveAccount()).resolves.toMatchObject({
      data: {
        receiptTokenMint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
        user: 'B7hKA5KCnPb465QCJMHwfGfyfzemQ3Nvvxsa9K7cQTUw',
      },
    });
    expect(user.fund.address).toMatchInlineSnapshot(
      `"CW16ZmLa25VLbJRxn27RALa53HAtwQRrVd3uaARtZzTi"`
    );

    await expect(user.reward.resolveAccount()).resolves.toMatchObject({
      data: {
        receiptTokenMint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
        user: 'B7hKA5KCnPb465QCJMHwfGfyfzemQ3Nvvxsa9K7cQTUw',
      },
    });
    expect(user.reward.address).toMatchInlineSnapshot(
      `"9XJL2MHBoqgkJq2YoGoj7Y3D5Aua7ASx9twhqvjTu3e9"`
    );

    await expect(user.receiptToken.resolveAccount()).resolves.toMatchObject({
      data: {
        mint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
        owner: 'B7hKA5KCnPb465QCJMHwfGfyfzemQ3Nvvxsa9K7cQTUw',
      },
    });

    expect(user.toContextTreeString()).toBeTypeOf('string');
  });

  // CU: 0.8M
  test('can execute depositTransaction', async () => {
    await expect(
      user.deposit.execute(
        {
          assetMint: null,
          assetAmount: 10n,
        },
        {
          signers: [signer],
          // executionHooks: {
          //   onResult: (parent, result, args) => {
          //     console.log('result', parent.toString(), result, args);
          //   },
          //   onSignature: (parent, sig) => {
          //     console.log('signature', parent.toString(), sig);
          //   },
          //   onError: (parent, err) => {
          //     console.log('err', parent.toString(), err);
          //   },
          // },
        }
      )
    ).resolves.toMatchObject({
      succeeded: true,
      events: {
        userDepositedToFund: {
          contributionAccrualRate: {
            __option: 'None',
          },
          depositedAmount: 10n,
          fundAccount: '4YHmpuyY54Bsj61qNxYGgtQy8xhacfnhdZ6W92rqB64w',
          mintedReceiptTokenAmount: 9n,
          receiptTokenMint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
          supportedTokenMint: {
            __option: 'None',
          },
          updatedUserRewardAccounts: [
            '9XJL2MHBoqgkJq2YoGoj7Y3D5Aua7ASx9twhqvjTu3e9',
          ],
          user: 'B7hKA5KCnPb465QCJMHwfGfyfzemQ3Nvvxsa9K7cQTUw',
          userFundAccount: 'CW16ZmLa25VLbJRxn27RALa53HAtwQRrVd3uaARtZzTi',
          userReceiptTokenAccount:
            'EN2SRVYVVko9XVC5sieRGAPMd2oCTbKFwScTYe3WpRo1',
          userSupportedTokenAccount: {
            __option: 'None',
          },
          walletProvider: {
            __option: 'None',
          },
        },
      },
    });
  });

  test('can execute depositTransaction with metadata (fails as unauthorized)', async () => {
    await expect(
      user.deposit
        .execute(
          {
            assetMint: null,
            assetAmount: 10n,
            metadata: {
              user: signer.address,
              contributionAccrualRate: 130,
              expiredAt: new Date(new Date().getTime() + 1000 * 30),
              walletProvider: 'BACKPACK',
              signerKeyPair: signer.keyPair,
            },
          },
          {
            signers: [signer],
          }
        )
        .catch((err) => {
          return err.context.logs.join('\n');
        })
    ).resolves.toContain(
      'Error Code: InvalidSignatureError. Error Number: 6003. Error Message: signature verification failed.'
    );
  });

  test('can execute requestWithdrawalTransaction, cancelWithdrawalRequestTransaction and withdrawTransaction', async () => {
    // CU 0.21M
    const u = await user.resolve();

    for (const request of u!.withdrawalRequests) {
      if (request.state == 'claimable') {
        await expect(
          user.withdraw.execute(
            {
              assetMint: request?.supportedAssetMint ?? null,
              requestId: (request?.requestId ?? 0n) + 100000n,
            },
            { signers: [signer] },
            { skipPreflight: true }
          )
        ).rejects.toThrowError('invalid context');
      } else if (request.state == 'cancelable') {
        await expect(
          user.cancelWithdrawalRequest.execute(
            { assetMint: null, requestId: request.requestId },
            { signers: [signer] }
          )
        ).resolves.toMatchObject({
          events: {
            unknown: [],
            userCanceledWithdrawalRequestFromFund: {
              batchId: request.batchId,
              fundAccount: '4YHmpuyY54Bsj61qNxYGgtQy8xhacfnhdZ6W92rqB64w',
              receiptTokenMint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
              requestId: request.requestId,
              requestedReceiptTokenAmount: 5n,
              supportedTokenMint: {
                __option: 'None',
              },
              updatedUserRewardAccounts: [
                '9XJL2MHBoqgkJq2YoGoj7Y3D5Aua7ASx9twhqvjTu3e9',
              ],
              user: 'B7hKA5KCnPb465QCJMHwfGfyfzemQ3Nvvxsa9K7cQTUw',
              userFundAccount: 'CW16ZmLa25VLbJRxn27RALa53HAtwQRrVd3uaARtZzTi',
              userReceiptTokenAccount:
                'EN2SRVYVVko9XVC5sieRGAPMd2oCTbKFwScTYe3WpRo1',
            },
          },
          succeeded: true,
        });
      }
    }

    await user.deposit.execute(
      {
        assetMint: null,
        assetAmount: 10n,
      },
      {
        signers: [signer],
      }
    );

    // CU 0.51M
    try {
      await expect(
        user.requestWithdrawal.execute(
          { assetMint: null, receiptTokenAmount: 5n },
          { signers: [signer] }
        )
      ).resolves.toMatchObject({
        succeeded: true,
        events: {
          unknown: [],
          userRequestedWithdrawalFromFund: {
            fundAccount: '4YHmpuyY54Bsj61qNxYGgtQy8xhacfnhdZ6W92rqB64w',
            receiptTokenMint: 'FRAGSEthVFL7fdqM8hxfxkfCZzUvmg21cqPJVvC1qdbo',
            requestedReceiptTokenAmount: 5n,
            supportedTokenMint: {
              __option: 'None',
            },
            updatedUserRewardAccounts: [
              '9XJL2MHBoqgkJq2YoGoj7Y3D5Aua7ASx9twhqvjTu3e9',
            ],
            user: 'B7hKA5KCnPb465QCJMHwfGfyfzemQ3Nvvxsa9K7cQTUw',
            userFundAccount: 'CW16ZmLa25VLbJRxn27RALa53HAtwQRrVd3uaARtZzTi',
            userReceiptTokenAccount:
              'EN2SRVYVVko9XVC5sieRGAPMd2oCTbKFwScTYe3WpRo1',
          },
        },
      });
    } catch (err: any) {
      expect(err.cause.context.logs).toContain(
        'Program log: AnchorError thrown in programs/restaking/src/modules/fund/user_fund_account.rs:120. Error Code: FundExceededMaxWithdrawalRequestError. Error Number: 6043. Error Message: fund: exceeded max withdrawal request per user.'
      );
    }
  });
});
