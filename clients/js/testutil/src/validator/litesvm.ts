import {
  AccountInfoBase,
  AccountInfoWithBase64EncodedData,
  AccountInfoWithPubkey,
  address,
  Base64EncodedDataResponse,
  lamports,
} from '@solana/kit';
import * as web3 from '@solana/web3.js';
import fs from 'fs';
import { LiteSVM } from 'litesvm';
import path from 'path';
import {
  TestValidator,
  TestValidatorOptions,
  TestValidatorRuntime,
} from './validator';

export class LiteSVMValidator extends TestValidator<'litesvm'> {
  static async initialize(
    options: TestValidatorOptions<'litesvm'>
  ): Promise<TestValidator<'litesvm'>> {
    const instanceNo = options.instanceNo ?? ++LiteSVMValidator.instanceNo;
    const instance = await this.createInstance(instanceNo, options);
    return new LiteSVMValidator(
      instanceNo,
      options,
      instance.svm,
      instance.clockTimeout
    );
  }

  private static async createInstance(
    instanceNo: number,
    options: TestValidatorOptions<'litesvm'>
  ) {
    const svm = new LiteSVM().withSysvars().withBuiltins().withSplPrograms();

    if (options.mock) {
      function resolvePath(p: string) {
        return path.isAbsolute(p) ? p : path.join(options.mock!.rootDir, p);
      }

      function setMockAccount(
        pubkey: string | null,
        filePathOrData:
          | string
          | AccountInfoWithPubkey<
              AccountInfoBase & AccountInfoWithBase64EncodedData
            >
      ) {
        try {
          const file =
            typeof filePathOrData == 'string'
              ? (JSON.parse(
                  fs.readFileSync(filePathOrData).toString()
                ) as AccountInfoWithPubkey<
                  AccountInfoBase & AccountInfoWithBase64EncodedData
                >)
              : filePathOrData;

          svm.setAccount(new web3.PublicKey(pubkey ?? file.pubkey), {
            data: Uint8Array.from(
              Buffer.from(file.account.data[0], file.account.data[1])
            ),
            executable: file.account.executable,
            lamports: Number(file.account.lamports.valueOf()),
            owner: new web3.PublicKey(file.account.owner.toString()),
            rentEpoch:
              file.account.rentEpoch > BigInt(Number.MAX_SAFE_INTEGER)
                ? undefined
                : Number(file.account.rentEpoch),
          });
        } catch (err) {
          throw new Error(
            `failed to mock account: ${filePathOrData.toString()} - ${err}`
          );
        }
      }

      for (const program of options.mock.programs) {
        try {
          const pubkey =
            'keypairFilePath' in program
              ? web3.Keypair.fromSecretKey(
                  Uint8Array.from(
                    JSON.parse(
                      fs
                        .readFileSync(resolvePath(program.keypairFilePath))
                        .toString()
                    )
                  )
                ).publicKey.toString()
              : program.pubkey;
          (program as any).pubkey = pubkey;

          svm.addProgramFromFile(
            new web3.PublicKey(pubkey),
            resolvePath(program.soFilePath)
          );
        } catch (err) {
          throw new Error(
            `failed to mock program: ${program.soFilePath} - ${err}`
          );
        }
      }

      for (const account of options.mock.accounts) {
        if ('jsonFileDirPath' in account) {
          const resolvedDirPath = resolvePath(account.jsonFileDirPath);
          for (const filePath of fs.readdirSync(resolvedDirPath)) {
            if (filePath.endsWith('.json')) {
              setMockAccount(null, path.join(resolvedDirPath, filePath));
            }
          }
        } else if ('jsonFilePath' in account) {
          setMockAccount(
            account.pubkey ?? null,
            resolvePath(account.jsonFilePath)
          );
        } else {
          setMockAccount(account.pubkey, account);
        }
      }
    }

    if (typeof options.slotsPerEpoch != 'undefined') {
      const schedule = svm.getEpochSchedule();
      schedule.slotsPerEpoch = options.slotsPerEpoch;
      schedule.warmup = true;
      schedule.firstNormalSlot = 0n;
      schedule.firstNormalEpoch = 0n;
      schedule.leaderScheduleSlotOffset = 0n;
      svm.setEpochSchedule(schedule);
    }

    const clockTimeout = setInterval(
      () => {
        const clock = svm.getClock();
        clock.slot++;
        clock.unixTimestamp = BigInt(Math.floor(Date.now() / 1000));
        svm.setClock(clock);
      },
      (400 / 64) * options.ticksPerSlot
    );

    return {
      svm,
      clockTimeout,
    };
  }

  private static instanceNo = 0;

  private constructor(
    private readonly instanceNo: number,
    public readonly options: TestValidatorOptions<'litesvm'>,
    private readonly svm: LiteSVM,
    private readonly clockTimeout: NodeJS.Timeout
  ) {
    super();
  }

  get runtime(): TestValidatorRuntime<'litesvm'> {
    return {
      type: 'litesvm',
      instanceNo: this.instanceNo,
      svm: this.svm,
    };
  }

  async quit() {
    clearInterval(this.clockTimeout);
  }

  async getSlot(): Promise<bigint> {
    return this.svm.getClock().slot;
  }

  async warpToSlot(slot: bigint): Promise<void> {
    const currentSlot = this.svm.getClock().slot;
    if (slot < currentSlot) {
      throw new Error(
        `warp slot (${slot}) cannot be less than the working bank slot (${currentSlot})`
      );
    }
    this.svm.warpToSlot(slot);
  }

  async airdrop(pubkey: string, lamports: bigint): Promise<void> {
    const result = this.svm.airdrop(new web3.PublicKey(pubkey), lamports);
    if (!result) {
      throw new Error(
        `failed to airdrop: pubkey=${pubkey}, lamports=${lamports}`
      );
    }
    if ('err' in result) {
      throw new Error(
        `failed to airdrop: pubkey=${pubkey}, lamports=${lamports}, err=${result.toString()}`
      );
    }
  }

  async getAccount(
    pubkey: string
  ): Promise<(AccountInfoBase & AccountInfoWithBase64EncodedData) | null> {
    const account = this.svm.getAccount(new web3.PublicKey(pubkey));
    return account
      ? {
          data: [
            Buffer.from(account.data).toString('base64'),
            'base64',
          ] as Base64EncodedDataResponse,
          executable: account.executable,
          lamports: lamports(BigInt(account.lamports)),
          owner: address(account.owner.toString()),
          rentEpoch: account.rentEpoch
            ? BigInt(account.rentEpoch)
            : BigInt('18446744073709551615'),
          space: BigInt(account.data.length),
        }
      : null;
  }
}
