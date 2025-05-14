import {
  AccountInfoBase,
  AccountInfoWithBase64EncodedData,
  Address,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  Lamports,
  Rpc,
  RpcSubscriptions,
  SolanaRpcApi,
  SolanaRpcSubscriptionsApi,
} from '@solana/kit';
import * as web3 from '@solana/web3.js';
import * as child_process from 'node:child_process';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import * as stream from 'node:stream';
import * as url from 'node:url';
import util from 'node:util';
import {
  GetSlotOptions,
  TestValidator,
  TestValidatorOptions,
  TestValidatorRuntime,
} from './validator';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

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
          ['--rpc-port', (18900 + (instanceNo - 1) * 3).toString()],
          ['--faucet-port', (18900 + (instanceNo - 1) * 3 + 2).toString()],
          !options.warpSlot ? ['--faucet-sol', 1_000_000n.toString()] : [],
          options.slotsPerEpoch && !options.warpSlot
            ? ['--slots-per-epoch', options.slotsPerEpoch.toString()]
            : [],
          options.ticksPerSlot && !options.warpSlot
            ? ['--ticks-per-slot', options.ticksPerSlot.toString()] // TODO: verify whether manipulating ticks-per-slot is appropriate
            : [],
          options.warpSlot
            ? ['--warp-slot', options.warpSlot.toString()]
            : ['--reset'],
        ];

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
                      return Number.MAX_SAFE_INTEGER; // TODO: how to set max rentEpoch while convert BigInt to Number ?
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
        stdout = childProcess.stdout.on('data', async (data) => {
          const logs = data.toString().trim().split('\n');
          for (const log of logs) {
            logger(log);
            if (log.startsWith('Error: ')) {
              stderr.destroy();
              stdout?.destroy();
              reject(new Error(`${cmdString}\n${data.toString()}`));
              break;
            } else if (log.startsWith(rpcURLPrefix)) {
              rpcURL = log.substring(rpcURLPrefix.length).trim();
            } else if (log.includes(rpcSubscriptionsURLPrefix)) {
              rpcSubscriptionsURL = log
                .substring(rpcSubscriptionsURLPrefix.length)
                .trim();
            } else if (log.includes(processingLogPrefix)) {
              if (!resolved) {
                resolve({
                  process: childProcess,
                  ledgerPath,
                  rpcURL,
                  rpcSubscriptionsURL,
                });
                resolved = true;
                if (stderr) stderr.pause();
                if (!options.debug) {
                  stdout?.pause();
                  break;
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
    const signature = await this.rpc
      .requestAirdrop(pubkey as Address, lamports as Lamports)
      .send();
    const abortController = new AbortController();
    const signatureNotifications = await this.rpcSubscriptions
      .signatureNotifications(signature)
      .subscribe({ abortSignal: abortController.signal });
    const timeoutTimer = setTimeout(() => {
      if (!abortController.signal.aborted) {
        abortController.abort(`test validator airdrop timed out: ${signature}`);
      }
    }, 10000);
    for await (const res of signatureNotifications) {
      if (res.value.err) {
        abortController.abort();
        throw new Error(
          `test validator airdrop failed: ${signature}\n${util.inspect(res)}`
        );
      }
    }
    clearTimeout(timeoutTimer);
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

  async getSlot(opts: GetSlotOptions = {}): Promise<bigint> {
    const commitment = opts.commitment ?? 'finalized';
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
}
