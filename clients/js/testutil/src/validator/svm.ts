import {
  AccountInfoBase,
  AccountInfoWithBase64EncodedData,
  Address,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  Lamports,
  Rpc,
  RpcSubscriptions,
  SolanaError,
  SolanaRpcApi,
  SolanaRpcSubscriptionsApi,
} from '@solana/kit';
import * as web3 from '@solana/web3.js';
import * as child_process from 'node:child_process';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import * as stream from 'node:stream';
import {
  GetSlotOptions,
  TestValidator,
  TestValidatorOptions,
  TestValidatorRuntime,
} from './validator';

export class SVMValidator extends TestValidator<'svm'> {
  static async initialize(
    options: TestValidatorOptions<'svm'>
  ): Promise<TestValidator<'svm'>> {
    const instanceNo = options.instanceNo ?? ++SVMValidator.instanceNo;
    const instance = await SVMValidator.createInstance(instanceNo, options);
    return new SVMValidator(
      instanceNo,
      options,
      instance.process,
      instance.ledgerPath,
      instance.rpcURL,
      instance.rpcSubscriptionsURL
    );
  }

  private static localWalletInitialization: null | Promise<void> = null;

  private static async createInstance(
    instanceNo: number,
    options: TestValidatorOptions<'svm'> & { warpSlot?: bigint }
  ): Promise<{
    process: child_process.ChildProcess;
    ledgerPath: string;
    rpcURL: string;
    rpcSubscriptionsURL: string;
  }> {
    // create a local wallet if does not exits
    if (!SVMValidator.localWalletInitialization) {
      SVMValidator.localWalletInitialization = new Promise(
        (resolve, reject) => {
          try {
            child_process.execSync(
              `cat ~/.config/solana/id.json >/dev/null 2>&1 || (mkdir -p ~/.config/solana && solana-keygen new --no-bip39-passphrase -o ~/.config/solana/id.json)`
            );
            resolve();
          } catch (e) {
            reject(e);
          }
        }
      );
    }
    await SVMValidator.localWalletInitialization;

    const logger = SVMValidator.createLogger(options);
    const instance = await new Promise<{
      process: child_process.ChildProcess;
      ledgerPath: string;
      rpcURL: string;
      rpcSubscriptionsURL: string;
    }>(async (resolve, reject) => {
      const ledgerPath = path.join(
        os.tmpdir(),
        '@fragmetric-labs',
        'testutil',
        'ledgers',
        instanceNo.toString()
      );
      fs.mkdirSync(ledgerPath, { recursive: true });
      const cmd = 'solana-test-validator';
      const argsBuilder = [
        ['--ledger', ledgerPath],
        ['--rpc-port', (18900 + (instanceNo - 1) * 4).toString()],
        ['--faucet-port', (18900 + (instanceNo - 1) * 4 + 2).toString()],
        ['--gossip-port', (18900 + (instanceNo - 1) * 4 + 3).toString()],
      ].concat(
        !options.warpSlot
          ? [
              ['--faucet-sol', 1_000_000n.toString()],
              ['--faucet-time-slice-secs', '0'],
              ['--limit-ledger-size', options.limitLedgerSize.toString()],
              ['--slots-per-epoch', options.slotsPerEpoch.toString()],
              ['--ticks-per-slot', options.ticksPerSlot.toString()],
              ['--reset'],
            ]
          : [
              ['--warp-slot', options.warpSlot.toString()],
              // ['--log'],
            ]
      );

      if (options.mock && !options.warpSlot) {
        function resolvePath(p: string) {
          return path.isAbsolute(p) ? p : path.join(options.mock!.rootDir, p);
        }

        for (const program of options.mock.programs) {
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

          argsBuilder.push([
            '--bpf-program',
            pubkey,
            resolvePath(program.soFilePath),
          ]);
        }

        for (const account of options.mock.accounts) {
          if ('jsonFileDirPath' in account) {
            argsBuilder.push([
              '--account-dir',
              resolvePath(account.jsonFileDirPath),
            ]);
          } else if ('jsonFilePath' in account) {
            argsBuilder.push([
              '--account',
              account.pubkey ?? '-',
              resolvePath(account.jsonFilePath),
            ]);
          } else {
            // create temporary account file to load inline account mocks
            const accountFilePath = path.join(
              os.tmpdir(),
              `${instanceNo}_mock_${account.pubkey}.json`
            );
            const accountFile = JSON.stringify(
              account,
              (key, value) => {
                if (typeof value === 'bigint') {
                  if (value > Number.MAX_SAFE_INTEGER) {
                    return Number.MAX_SAFE_INTEGER; // TODO: how to set max rentEpoch while converting BigInt to Number ?
                  } else {
                    return Number(value);
                  }
                }
                return value;
              },
              2
            );
            fs.writeFileSync(accountFilePath, accountFile, 'utf8');
            argsBuilder.push(['--account', '-', accountFilePath]);
          }
        }
      }

      const args = argsBuilder.flat();
      const cmdString = `${cmd} ${args.join(' ')}`;
      logger(cmdString);

      const childProcess = child_process.spawn(cmd, args, {
        detached: false,
        shell: false,
        stdio: ['ignore', 'pipe', 'pipe'],
      });

      // gather validator info from log
      let rpcURL: string;
      let rpcSubscriptionsURL: string;
      const rpcURLPrefix = 'JSON RPC URL: ';
      const rpcSubscriptionsURLPrefix = 'WebSocket PubSub URL: ';
      const processingLogPrefix = 'Processed Slot: ';

      let stdout: stream.Readable | null = null;
      const stderr = childProcess.stderr.on('data', (data) => {
        stderr.destroy();
        stdout?.destroy();
        reject(new Error(`${cmdString}\n${data.toString()}`));
      });
      let resolved = false;
      let processingDebugLogPrintCount = 0;
      stdout = childProcess.stdout.on('data', async (data) => {
        try {
          const logs = data.toString().trim().split('\n');
          for (const log of logs) {
            if (log.startsWith('Error') || log.startsWith('Notice')) {
              console.error(log);
              stderr.destroy();
              stdout?.destroy();
              reject(new Error(`${cmdString}\n${data.toString()}`));
              break;
            } else if (log.startsWith(rpcURLPrefix)) {
              logger(log);
              rpcURL = log.substring(rpcURLPrefix.length).trim();
            } else if (log.includes(rpcSubscriptionsURLPrefix)) {
              logger(log);
              rpcSubscriptionsURL = log
                .substring(rpcSubscriptionsURLPrefix.length)
                .trim();
            } else if (log.includes(processingLogPrefix)) {
              if (!resolved) {
                logger(log);

                resolve({
                  process: childProcess,
                  ledgerPath,
                  rpcURL,
                  rpcSubscriptionsURL,
                });
                await new Promise((resolve) => setTimeout(resolve, 1000));
                resolved = true;

                if (stderr) stderr.pause();
                if (!options.debug) {
                  stdout?.pause();
                  break;
                }
              } else {
                if (processingDebugLogPrintCount++ % 16 == 0) {
                  logger(log);
                }
              }
            }
          }
        } catch (err) {
          console.error(err);
          reject(err);
        }
      });
    });
    return instance;
  }

  private static instanceNo = 0;
  private readonly rpc: Rpc<SolanaRpcApi>;
  private readonly rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;

  protected static createLogger(options: TestValidatorOptions<'svm'>) {
    if (options.tag) {
      return (msg: string) => console.log(`[${options.tag}] ${msg}`);
    }
    return (msg: string) => console.log(msg);
  }

  private logger = SVMValidator.createLogger(this.options);

  private constructor(
    private readonly instanceNo: number,
    public readonly options: TestValidatorOptions<'svm'>,
    private process: child_process.ChildProcess,
    private readonly ledgerPath: string,
    private readonly rpcURL: string,
    private readonly rpcSubscriptionsURL: string
  ) {
    super();

    this.rpc = createSolanaRpc(this.rpcURL);
    this.rpcSubscriptions = createSolanaRpcSubscriptions(
      this.rpcSubscriptionsURL
    );
  }

  get runtime(): TestValidatorRuntime<'svm'> {
    return {
      type: 'svm',
      instanceNo: this.instanceNo,
      cluster: 'local',
      rpc: this.rpc,
      rpcSubscriptions: this.rpcSubscriptions,
      rpcURL: this.rpcURL,
      rpcSubscriptionsURL: this.rpcSubscriptionsURL,
    };
  }

  private quitting: Promise<void> | null = null;

  async quit() {
    if (this.quitting) {
      return this.quitting;
    }

    return (this.quitting = new Promise<void>((resolve, reject) => {
      let i = 0;
      let exited = false;
      const process = this.process;
      process.once('exit', () => {
        this.logger('STOPPED VALIDATOR');
        exited = true;
      });
      const reporter = setInterval(() => {
        if (exited) {
          clearInterval(reporter);
          setTimeout(resolve, 1000);
          return;
        }
        try {
          this.logger(`STOPPING VALIDATOR... (${++i})`);
          process.kill('SIGTERM');
        } catch (err) {
          console.error(err);
        }
        if (i == 12) {
          this.logger(`NO EXIT EVENT FROM VALIDATOR... (${i})`);
          clearInterval(reporter!);
          reject('stopping validator timed out');
        }
      }, 1000);
    }).finally(() => {
      this.quitting = null;
    }));
  }

  async airdrop(pubkey: string, lamports: bigint): Promise<void> {
    const prevLamports = (await this.rpc.getBalance(pubkey as Address).send())
      .value;
    // if (this.options.debug) {
    this.logger(`AIRDROP ${lamports} TO ${pubkey} (${prevLamports})`);
    // }

    let retriesOnBlockErrors = 0;
    while (retriesOnBlockErrors < 100) {
      try {
        const signature = await this.rpc
          .requestAirdrop(pubkey as Address, lamports as Lamports, {
            commitment: 'confirmed',
          })
          .send();
        const abortController = new AbortController();
        const signatureNotifications = await this.rpcSubscriptions
          .signatureNotifications(signature)
          .subscribe({ abortSignal: abortController.signal });
        const timeoutTimer = setTimeout(() => {
          if (!abortController.signal.aborted) {
            abortController.abort(`TIMEOUT`);
            // console.error('AIRDROP CONFIRMATION TIMED OUT');
          }
        }, 20 * 1000);
        for await (const res of signatureNotifications) {
          if (res.value.err) {
            abortController.abort();
            throw res.value.err;
          }
          break;
        }
        clearTimeout(timeoutTimer);

        let retryOnSlowBalanceUpdate = 0;
        while (retryOnSlowBalanceUpdate < 5) {
          const currentLamports = (
            await this.rpc.getBalance(pubkey as Address).send()
          ).value;
          if (currentLamports == prevLamports + lamports) {
            return;
          }

          console.error(
            `Rechecking the balance update from airdrop (${retryOnSlowBalanceUpdate}): ${pubkey} (${prevLamports} + ${lamports} => ${currentLamports})\nSignature: ${signature}`
          );
          retryOnSlowBalanceUpdate++;
          await new Promise((resolve) =>
            setTimeout(resolve, Math.floor(Math.random() * 2000) + 500)
          );
        }
      } catch (err) {
        if (err instanceof SolanaError) {
          const blockError =
            /network has progressed|blockhash not found|already been processed/i;
          const causeMsg = err.cause?.toString() || '';
          const msg = `${err.message}${causeMsg ? ` - ${causeMsg}` : ''}`;
          if (blockError.test(msg)) {
            console.error(
              `Retrying the same airdrop transaction (${retriesOnBlockErrors}): ${msg}`
            );
            retriesOnBlockErrors++;
            await new Promise((resolve) =>
              setTimeout(resolve, Math.floor(Math.random() * 2000) + 500)
            );
            continue;
          }
        }

        throw err;
      }
    }
  }

  async getAccount(
    pubkey: string
  ): Promise<(AccountInfoBase & AccountInfoWithBase64EncodedData) | null> {
    const res = await this.rpc
      .getAccountInfo(pubkey as Address, {
        encoding: 'base64',
        commitment: 'confirmed',
      })
      .send();
    return res.value;
  }

  async getSlot(opts?: GetSlotOptions): Promise<bigint> {
    const commitment = opts?.commitment ?? 'confirmed';
    return this.rpc.getSlot({ commitment }).send();
  }

  private async getFullSnapshotSlot(): Promise<bigint> {
    return this.rpc
      .getHighestSnapshotSlot()
      .send()
      .then(
        (snapshot) => snapshot.full,
        () => 0n
      );
  }

  async warpToSlot(slot: bigint): Promise<void> {
    this.logger(`PREPARE WARP TO SLOT ${slot}`);
    const targetSnapshotSlot = await this.getSlot({ commitment: 'processed' });

    while (true) {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      const [currentProcessedSlot, currentFullSnapshotSlot] = await Promise.all(
        [this.getSlot({ commitment: 'processed' }), this.getFullSnapshotSlot()]
      );

      if (slot <= currentProcessedSlot) {
        this.logger(`NO NEED TO WARP TO SLOT ${slot}`);
        return;
      } else if (
        // Preferring halting to restarting, takes 150 slot (60s with ticksPerSlot=64: 1 slot ~= 400ms)
        // So use "160 * 64 / ticksPerSlot" slots as a buffer
        slot - currentProcessedSlot <
        (150 * 64) / this.options.ticksPerSlot
      ) {
        this.logger(`NO NEED TO WARP TO SLOT ${slot} (WAITING)`);
        continue;
      } else if (currentFullSnapshotSlot >= targetSnapshotSlot) {
        this.logger(`FULL SNAPSHOT TAKEN AT SLOT ${currentFullSnapshotSlot}`);
        break;
      }
      this.logger(
        `WAIT UNTIL TAKING FULL SNAPSHOT AFTER SLOT ${targetSnapshotSlot}`
      );
    }

    await this.quit();
    const newInstance = await SVMValidator.createInstance(this.instanceNo, {
      ...this.options,
      warpSlot: slot,
    });

    this.process = newInstance.process;
  }

  readonly canDangerouslyAirdropNonMintableToken = false;

  async dangerouslyAirdropNonMintableToken(
    pubkey: string,
    mockMint: string,
    amount: bigint
  ) {
    throw new Error(
      'dangerouslyAirdropNonMintableToken: unsupported runtime type'
    );
  }
}
