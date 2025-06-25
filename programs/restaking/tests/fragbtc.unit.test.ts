import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeFragBTC } from './fragbtc.unit.init';

describe('restaking.fragBTC unit test', async () => {
  const testCtx = initializeFragBTC(await createTestSuiteContext());

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  const { validator, feePayer, restaking, solv, initializationTasks } = testCtx;
  const ctx = restaking.fragBTC;

  /* create users **/
  const [signer1, signer2] = await Promise.all([
    validator
      .newSigner('fragBTCTestSigner1', 100_000_000_000n)
      .then(async (signer) => {
        await Promise.all([
          validator.airdropToken(
            signer.address,
            'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
            100_000_000_000n
          ),
          validator.airdropToken(
            signer.address,
            'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
            100_000_000_000n
          ),
          validator.airdropToken(
            signer.address,
            '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh',
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
            'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
            100_000_000_000n
          ),
          validator.airdropToken(
            signer.address,
            'cbbtcf3aa214zXHbiAZQwf4122FBYbraNdFqgw4iMij',
            100_000_000_000n
          ),
          validator.airdropToken(
            signer.address,
            '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh',
            100_000_000_000n
          ),
        ]);
        return signer;
      }),
    validator.airdrop(restaking.knownAddresses.fundManager, 100_000_000_000n),
  ]);
  const user1 = ctx.user(signer1);
  const user2 = ctx.user(signer2);

  /** deposit **/
  test('user can update reward pools and sync with global reward account anytime', async () => {
    await expectMasked(
      user1.deposit.execute(
        {
          assetMint: 'zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg',
          assetAmount: 100_000_000n,
        },
        { signers: [signer1] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 100000000n,
          "assetMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
          "metadata": null,
        },
        "events": {
          "unknown": [],
          "userCreatedOrUpdatedFundAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            "userFundAccount": "DdVfA4rT4tSJRMi688zb5SeZitH91AcKXU79Q4A3pCHg",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            "userRewardAccount": "8XYL5QhcGSVZFX1wCdvVatXhehd7fwx9CYY4og1Fobt9",
          },
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 100000000n,
            "fundAccount": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
            "mintedReceiptTokenAmount": 100000000n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
            },
            "updatedUserRewardAccounts": [
              "8XYL5QhcGSVZFX1wCdvVatXhehd7fwx9CYY4og1Fobt9",
            ],
            "user": "AWb2qUvuFzbVN5Eu7tZY8gM745pus5DhTGgo8U8Bd8X2",
            "userFundAccount": "DdVfA4rT4tSJRMi688zb5SeZitH91AcKXU79Q4A3pCHg",
            "userReceiptTokenAccount": "31GDTcPyFTexmiwSvLTZAcY2JSxafoy3yVavK4iAYaLE",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "CHBjDuyN1JxAfeYY7wz68b78E6uPpek1TKqjKeyDJZD6",
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

  /** delegate */
  test('delegate reward account from user2 to user1', async () => {
    // for delegate, do deposit first to create reward account
    await expectMasked(
      user2.deposit.execute(
        {
          assetMint: '3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh',
          assetAmount: 100_000_000n,
        },
        { signers: [signer2] }
      )
    ).resolves.toMatchInlineSnapshot(`
      {
        "args": {
          "applyPresetComputeUnitLimit": true,
          "assetAmount": 100000000n,
          "assetMint": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
          "metadata": null,
        },
        "events": {
          "unknown": [],
          "userCreatedOrUpdatedFundAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
            "userFundAccount": "4qyutWBhvuM6k73kehvo6drqQg93aLJaEiPo4ticeQE1",
          },
          "userCreatedOrUpdatedRewardAccount": {
            "created": true,
            "receiptTokenAmount": 0n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
            "userRewardAccount": "5vjNrtqwKRB68nTfwN3gZayEADap2947MuZp43bCdLo1",
          },
          "userDepositedToFund": {
            "contributionAccrualRate": {
              "__option": "None",
            },
            "depositedAmount": 100000000n,
            "fundAccount": "BEpVRdWw6VhvfwfQufB9iqcJ6acf51XRP1jETCvGDBVE",
            "mintedReceiptTokenAmount": 100000000n,
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "supportedTokenMint": {
              "__option": "Some",
              "value": "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh",
            },
            "updatedUserRewardAccounts": [
              "5vjNrtqwKRB68nTfwN3gZayEADap2947MuZp43bCdLo1",
            ],
            "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
            "userFundAccount": "4qyutWBhvuM6k73kehvo6drqQg93aLJaEiPo4ticeQE1",
            "userReceiptTokenAccount": "HJVHLQtg6MJpTtyzbTCEzdLxLF6XShRvT7gcYbBUwdzD",
            "userSupportedTokenAccount": {
              "__option": "Some",
              "value": "ALt5RC3MSK2K9DNk7Ad9pk5vuCzStXKywHa6M1sZGVmt",
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
            "receiptTokenMint": "ExBpou3QupioUjmHbwGQxNVvWvwE3ZpfzMzyXdWZhzZz",
            "user": "CCgWk6XBuLqZTcDuJa4ZKGUM22cLXUFAxJSHiZdm3T3b",
            "userRewardAccount": "5vjNrtqwKRB68nTfwN3gZayEADap2947MuZp43bCdLo1",
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
});
