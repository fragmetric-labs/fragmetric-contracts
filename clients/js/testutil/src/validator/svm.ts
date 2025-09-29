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

  private static async createInstance(
    instanceNo: number,
    options: TestValidatorOptions<'svm'> & { warpSlot?: bigint },
    retryCount: number = 0
  ): Promise<{
    process: child_process.ChildProcess;
    ledgerPath: string;
    rpcURL: string;
    rpcSubscriptionsURL: string;
  }> {
    const logger = SVMValidator.createLogger(options);
    try {
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
            : [['--warp-slot', options.warpSlot.toString()]]
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

        // create a local wallet if does not exits
        child_process.execSync(
          `cat ~/.config/solana/id.json >/dev/null 2>&1 || { mkdir -p ~/.config/solana && solana-keygen new --no-bip39-passphrase -o ~/.config/solana/id.json; }`
        );

        const childProcess = child_process.spawn(cmd, args, {
          detached: false,
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
          const logs = data.toString().trim().split('\n');
          for (const log of logs) {
            if (log.startsWith('Error') || log.startsWith('Notice')) {
              logger(log);
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
                resolved = true;
                await new Promise((resolve) => setTimeout(resolve, 5000));

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
        });
      });
      return instance;
    } catch (err) {
      if (
        (err as any).toString().includes('Address already in use') &&
        retryCount < 10
      ) {
        await new Promise((resolve) => setTimeout(resolve, 1000));
        return SVMValidator.createInstance(instanceNo, options, retryCount + 1);
      }
      throw err;
    }
  }

  private static instanceNo = 0;
  private rpc: Rpc<SolanaRpcApi>;
  private rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;

  protected static createLogger(options: TestValidatorOptions<'svm'>) {
    if (options.debug) {
      if (options.tag) {
        return (msg: string) => console.log(`[${options.tag}] ${msg}`);
      }
      return (msg: string) => console.log(msg);
    }
    return (msg: string) => {};
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

  async quit() {
    this.logger('STOPPING VALIDATOR');
    this.process.kill('SIGINT');
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }

  async airdrop(pubkey: string, lamports: bigint): Promise<void> {
    const prevLamports = (await this.rpc.getBalance(pubkey as Address).send())
      .value;
    if (this.options.debug) {
      this.logger(`AIRDROP ${lamports} TO ${pubkey} (${prevLamports})`);
    }

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
            setTimeout(resolve, Math.floor(Math.random() * 5000) + 1000)
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
              setTimeout(resolve, Math.floor(Math.random() * 5000) + 1000)
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

      if (currentProcessedSlot >= slot) {
        this.logger(`NO NEED TO WARP TO SLOT ${slot}`);
        return;
      } else if (currentFullSnapshotSlot >= targetSnapshotSlot) {
        this.logger(`FULL SNAPSHOT TAKEN AT SLOT ${currentFullSnapshotSlot}`);
        break;
      }
      if (this.options.debug) {
        this.logger(
          `WAIT UNTIL TAKING FULL SNAPSHOT AFTER SLOT ${targetSnapshotSlot}`
        );
      }
    }

    this.process.kill('SIGINT');
    await new Promise((resolve) => setTimeout(resolve, 1000));
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
