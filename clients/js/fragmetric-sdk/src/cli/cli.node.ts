import { createSolanaRpc, createSolanaRpcSubscriptions } from '@solana/kit';
import { Command } from 'commander';
import packageJSON from '../../package.json' with { type: 'json' };
import {
  createTransactionSignerResolversMap,
  RuntimeCluster,
  runtimeClusters,
  RuntimeContextPartialOptions,
  setContextCustomInspectionEnabled,
  TransactionSignerResolver,
} from '../context';
import { RestakingProgram, SolvBTCVaultProgram } from '../programs';
import { subCommands } from './commands';
import {
  createBigIntToJSONShim,
  createDefaultTransactionExecutionHooks,
  setLogger,
} from './utils';
export { createDefaultTransactionExecutionHooks, logger } from './utils';

export type RootCommandOptions = {
  url: string;
  ws: string;
  cluster: RuntimeCluster;
  keypairs: string[];
  format: 'pretty' | 'json';
  inspection: boolean;

  context: {
    signers: Record<string, TransactionSignerResolver>;
    programs: {
      restaking: RestakingProgram;
      solv: SolvBTCVaultProgram;
    };
  };
};

type TwoLevelPartial<T> = {
  [K in keyof T]?: T[K] extends object
    ? T[K] extends any[]
      ? T[K]
      : { [P in keyof T[K]]?: T[K][P] }
    : T[K];
};

export type CommandLineInterfaceConfig = {
  rootCommandDefaultOptions?: Omit<Partial<RootCommandOptions>, 'context'>;
  runtimeContextOptions?: RuntimeContextPartialOptions;

  // totally overrides VM context
  contextOverrides?: Partial<RootCommandOptions['context']> &
    Record<string, any>;
};

export function startCommandLineInterface(config?: CommandLineInterfaceConfig) {
  const rootCommand = new Command();

  const LOCAL_RPC_REGEXP =
    /^(http:\/\/)?(localhost|127\.0\.0\.1|0\.0\.0\.0|192\.168\.\d{1,3}\.\d{1,3}|10\.\d{1,3}\.\d{1,3}\.\d{1,3}|172\.(1[6-9]|2\d|3[0-1])\.\d{1,3}\.\d{1,3})(:\d+)?(\/.*)?$/i;

  rootCommand
    .name('fragmetric')
    .description('CLI for Fragmetric programs')
    .version(packageJSON.version)
    .showHelpAfterError()
    .option(
      '-u, --url <URL_OR_MONIKER>',
      `RPC URL or shorthand: [${runtimeClusters.join(', ')}]`,
      config?.rootCommandDefaultOptions?.cluster ?? 'mainnet'
    )
    .option(
      '--ws <URL>',
      'Custom WebSocket RPC URL (overrides derived one)',
      config?.rootCommandDefaultOptions?.ws
    )
    .option(
      '-c, --cluster <CLUSTER>',
      `Program environment when using custom RPC URL (overrides derived one): [${runtimeClusters.join(', ')}]`,
      config?.rootCommandDefaultOptions?.cluster
    )
    .option(
      '-k, --keypairs <KEYPAIRS...>',
      `One or more keypairs to automatically use as signers for transactions. First keypair will be used as feePayer. Accepts: JSON file path, directory of keypairs, base58/JSON literal, or literal for hardware wallets: [ledger].`,
      config?.rootCommandDefaultOptions?.keypairs
    )
    .option(
      '--format <FORMAT>',
      'Set output format for evaluation: [pretty, json]',
      config?.rootCommandDefaultOptions?.format ?? 'pretty'
    )
    .option(
      '--inspection <BOOL>',
      'Set verbose logs in default transaction hooks: [true, false] (default: cluster != "mainnet")',
      (v) => v == 'true',
      config?.rootCommandDefaultOptions?.inspection ?? undefined
    )
    .hook('preSubcommand', async (cmd) => {
      const opts: Partial<RootCommandOptions> = cmd.opts();
      switch (opts.url ?? opts.cluster) {
        case 'mainnet':
        case 'm':
          opts.url = process.env.SOLANA_RPC_MAINNET
            ? process.env.SOLANA_RPC_MAINNET
            : 'https://api.mainnet-beta.solana.com';
          opts.ws =
            opts.ws ??
            opts.url.replace('https://', 'wss://').replace('http://', 'ws://');
          opts.cluster = opts.cluster ?? 'mainnet';
          break;
        case 'devnet':
        case 'd':
          opts.url = process.env.SOLANA_RPC_DEVNET
            ? process.env.SOLANA_RPC_DEVNET
            : 'https://api.devnet.solana.com';
          opts.ws =
            opts.ws ??
            opts.url.replace('https://', 'wss://').replace('http://', 'ws://');
          opts.cluster = opts.cluster ?? 'devnet';
          break;
        case 'testnet':
        case 't':
          opts.url = process.env.SOLANA_RPC_TESTNET
            ? process.env.SOLANA_RPC_TESTNET
            : 'https://api.testnet.solana.com';
          opts.ws =
            opts.ws ??
            opts.url.replace('https://', 'wss://').replace('http://', 'ws://');
          opts.cluster = opts.cluster ?? 'testnet';
          break;
        case 'local':
        case 'l':
          opts.url = 'http://localhost:8899';
          opts.ws = opts.ws ?? 'ws://localhost:8900';
          opts.cluster = opts.cluster ?? 'local';
          break;
        default:
          opts.ws =
            opts.ws ??
            opts.url!.replace('https://', 'wss://').replace('http://', 'ws://');
          opts.cluster =
            opts.cluster ??
            (LOCAL_RPC_REGEXP.test(opts.url!) ? 'local' : 'mainnet');
      }

      // set logger
      opts.format = (opts.format?.toLowerCase() || 'pretty') as any;
      setLogger({ format: opts.format });

      // load keypairs
      const signers = createTransactionSignerResolversMap({
        rootDir: process.cwd(),
        keypairs: opts.keypairs ?? [],
      });

      // handle context overriding
      if (config?.contextOverrides?.programs) {
        const programs = Object.values(config.contextOverrides.programs).filter(
          (program) => !!program
        );
        const firstProgram = programs[0]!;
        const firstRuntime = firstProgram.runtime.toString();
        if (
          programs.some(
            (program) => program.runtime.toString() != firstRuntime.toString()
          )
        ) {
          throw new Error(
            `invalid context override: inconsistent runtime env in programs`
          );
        }

        // to set REPL prompt label
        opts.cluster = firstProgram.runtime.cluster;
        if (firstProgram.runtime.type == 'litesvm') {
          opts.url = 'litesvm://';
        }

        // override transaction hooks
        programs.forEach((program) => {
          program.runtime.options.transaction.executionHooks =
            createDefaultTransactionExecutionHooks({
              mergeWith:
                config?.runtimeContextOptions?.transaction?.executionHooks,
              inspection: opts.inspection ?? opts.cluster != 'mainnet',
            });
        });
      }

      // enable default hook inspection for non-mainnet cluster
      opts.inspection = opts.inspection ?? opts.cluster != 'mainnet';

      setContextCustomInspectionEnabled(true);
      createBigIntToJSONShim();

      // setup vm context
      const programConfig = {
        type: 'svm',
        cluster: opts.cluster,
        rpc: createSolanaRpc(opts.url!),
        rpcSubscriptions: createSolanaRpcSubscriptions(opts.ws!),
      } as const;
      const programOptions = {
        ...config?.runtimeContextOptions,
        transaction: {
          ...config?.runtimeContextOptions?.transaction,
          signers: [
            ...(config?.runtimeContextOptions?.transaction?.signers ?? []),
            ...Object.values(signers),
          ],
          executionHooks: createDefaultTransactionExecutionHooks({
            mergeWith:
              config?.runtimeContextOptions?.transaction?.executionHooks,
            inspection: opts.inspection,
          }),
        },
      };
      opts.context = {
        ...config?.contextOverrides,
        signers: {
          ...signers,
          ...config?.contextOverrides?.signers,
        },
        programs: {
          ...config?.contextOverrides?.programs,
          restaking:
            config?.contextOverrides?.programs?.restaking ??
            RestakingProgram.connect(programConfig, programOptions),
          solv:
            config?.contextOverrides?.programs?.solv ??
            SolvBTCVaultProgram.connect(programConfig, programOptions),
        },
      };
    });

  subCommands.forEach((cmd) => rootCommand.addCommand(cmd));

  rootCommand.parse(process.argv);
  if (!rootCommand.args.length) {
    rootCommand.outputHelp();
  }

  return rootCommand;
}
