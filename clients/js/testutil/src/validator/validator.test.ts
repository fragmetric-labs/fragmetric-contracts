import { address } from '@solana/kit';
import * as web3 from '@solana/web3.js';
import { afterAll, describe, expect, test } from 'vitest';
import { TestValidator } from './validator';

describe.each(['litesvm', 'solana'] as const)(
  'TestValidator with %s',
  async (validatorType) => {
    const validator1 = await TestValidator.create({
      type: validatorType,
      slotsPerEpoch: 432n,
      ticksPerSlot: 16,
      mock: {
        rootDir: __dirname,
        programs: [
          {
            keypairFilePath:
              './fixture/dummy_program_keypair_A8GusXzc4DTHU1R4Pf9cYMVrvXrhShWUCzYuZaGTTAQR.json',
            soFilePath: './fixture/marinade_stake_pool.so',
          },
          {
            pubkey: '111111111111111111111111111111MS',
            soFilePath: './fixture/marinade_stake_pool.so',
          },
        ],
        accounts: [
          {
            jsonFilePath: './fixture/msol/mSOL_mint.json',
            pubkey: '111111111111111111111111111111MM',
          },
          {
            jsonFileDirPath: './fixture/msol',
          },
        ],
      },
      debug: !!process.env.DEBUG,
    });

    const validator2 = await TestValidator.create({
      type: validatorType,
      slotsPerEpoch: 32n,
      ticksPerSlot: 64,
    });

    afterAll(async () => {
      await Promise.all([validator1.quit(), validator2.quit()]);
    });

    test('mock accounts', async () => {
      const programAccount1 = await validator1.getAccount(
        'A8GusXzc4DTHU1R4Pf9cYMVrvXrhShWUCzYuZaGTTAQR'
      );
      const programAccount2 = await validator1.getAccount(
        '111111111111111111111111111111MS'
      );
      if (validatorType == 'litesvm') {
        expect(programAccount1!.owner).eq(
          'BPFLoader2111111111111111111111111111111111'
        );
        expect(programAccount1).toMatchObject(programAccount2!);
      } else {
        expect(programAccount1!.owner).eq(
          'BPFLoaderUpgradeab1e11111111111111111111111'
        );
        expect(programAccount1!.lamports).eq(programAccount2!.lamports); // cannot compare two accounts directly in solana-test-validator
      }

      const mintAccount1 = await validator1.getAccount(
        'mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So'
      );
      const mintAccount2 = await validator1.getAccount(
        '111111111111111111111111111111MM'
      );
      expect(mintAccount1).toMatchObject(mintAccount2!);
      expect(mintAccount1?.owner).eq(
        'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'
      );
    });

    test('airdrop, getAccount', async () => {
      const user = address(web3.PublicKey.unique().toString());
      await expect(
        validator1.airdrop(user, 1_000_000_000n)
      ).resolves.not.toThrow();

      const userAccount = await validator1.getAccount(user);
      expect(userAccount?.lamports).eq(1_000_000_000n);

      let userAccountFromAnotherValidator = await validator2.getAccount(user);
      expect(userAccountFromAnotherValidator).null;
      await expect(
        validator2.airdrop(user, 500_000_000n)
      ).resolves.not.toThrow();

      userAccountFromAnotherValidator = await validator2.getAccount(user);
      expect(userAccountFromAnotherValidator?.lamports).eq(500_000_000n);

      expect(userAccountFromAnotherValidator).toMatchSnapshot();
      expect(userAccount).toMatchSnapshot();
    });

    test('getSlot, skipSlots, getEpoch, skipEpoch', async () => {
      const firstSlot = await validator1.getSlot();
      const firstEpoch = await validator1.getEpoch();

      await new Promise((resolve) => setTimeout(resolve, 400));
      const secondSlot = await validator1.getSlot();
      expect(secondSlot).toBeGreaterThan(firstSlot);
      expect(await validator2.getSlot()).toBeLessThan(secondSlot);

      await validator1.skipEpoch();
      const secondEpoch = await validator1.getEpoch();
      expect(firstEpoch + 1n).toEqual(secondEpoch);
    });

    test('newSigner', async () => {
      const user1 = await validator1.newSigner('user1');
      const user2 = await validator1.newSigner('user2', 500n);
      expect(user1.address).not.toEqual(user2.address);
      await expect(
        validator1.getAccount(user1.address).then((u) => u?.lamports)
      ).resolves.toEqual(100_000_000_000n);
      await expect(
        validator1.getAccount(user2.address).then((u) => u?.lamports)
      ).resolves.toEqual(1000000n); // due to minimum rent

      const user1b = await validator2.newSigner('user1');
      const user2b = await validator2.newSigner('user2');
      expect(user1.address).toEqual(user1b.address);
      expect(user2.address).toEqual(user2b.address);
    });
  }
);
