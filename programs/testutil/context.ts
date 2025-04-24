import { TestValidator, TestValidatorType } from '@fragmetric-labs/testutil';

import * as _sdk from '@fragmetric-labs/sdk';
declare type SDK = typeof _sdk;
/** ⚠️ DO NOT import '@fragmetric-labs/sdk' directly in test suites.
 * The above import is only used to extract type information from the SDK source.
 *
 * At runtime, the actual SDK is loaded dynamically via `import()` based on the `useDistSDK` flag and `process.env.CI`,
 * to allow tests to run either against the on-the-fly source (`src/`) or the compiled dist bundle (`dist/`).
 */

// node ESM loader fallback for __dirname
import * as path from 'node:path';
import * as url from 'node:url';
const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

export type TestSuiteContext = Awaited<
  ReturnType<typeof createTestSuiteContext>
>;

export async function createTestSuiteContext(config?: {
  validator?: TestValidatorType;
  debug?: boolean;
  slotsPerEpoch?: bigint;
  ticksPerSlot?: number;
  useDistSDK?: boolean;
  programs?: { restaking?: boolean; solv?: boolean };
}) {
  const resolvedConfig = {
    slotsPerEpoch: 432000n,
    ticksPerSlot: 64,
    validator:
      process.env.RUNTIME?.toLowerCase() == 'svm'
        ? ('svm' as const)
        : ('litesvm' as const),
    debug: !!process.env.DEBUG,
    useDistSDK: !!process.env.CI,
    ...config,
    programs: {
      restaking: true,
      solv: true,
      ...config?.programs,
    },
  };

  const validator = await TestValidator.create({
    type: resolvedConfig.validator,
    slotsPerEpoch: resolvedConfig.slotsPerEpoch,
    ticksPerSlot: resolvedConfig.ticksPerSlot,
    debug: resolvedConfig.debug,
    tag: process.env.VITEST_WORKER_ID
      ? `worker-${process.env.VITEST_WORKER_ID}`
      : undefined,
    instanceNo: process.env.VITEST_WORKER_ID
      ? parseInt(process.env.VITEST_WORKER_ID)
      : undefined,
    mock: {
      rootDir: __dirname,
      programs: [
        ...(resolvedConfig.programs.restaking
          ? [
              {
                // keypairFilePath: '../../target/deploy/restaking-keypair.json',
                pubkey: '4qEHCzsLFUnw8jmhmRSmAK5VhZVoSD1iVqukAf92yHi5',
                soFilePath: '../../target/deploy/restaking.so',
              },
              {
                pubkey: 'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
                soFilePath: '../restaking/tests/mocks/metaplex.so',
              },
              {
                pubkey: 'SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy',
                soFilePath: '../restaking/tests/mocks/spl_stake_pool.so',
              },
              {
                pubkey: 'MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD',
                soFilePath: '../restaking/tests/mocks/marinade_stake_pool.so',
              },
              {
                pubkey: 'SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY',
                soFilePath:
                  '../restaking/tests/mocks/sanctum_single_validator_stake_pool.so',
              },
              {
                pubkey: 'Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8',
                soFilePath: '../restaking/tests/mocks/jito_vault.so',
              },
              {
                pubkey: 'RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q',
                soFilePath: '../restaking/tests/mocks/jito_restaking.so',
              },
              {
                pubkey: 'whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc',
                soFilePath: '../restaking/tests/mocks/orca_whirlpool.so',
              },
            ]
          : []),
        ...(resolvedConfig.programs.solv
          ? [
              {
                // keypairFilePath: '../../target/deploy/solv-keypair.json',
                pubkey: '9beGuWXNoKPKCApT6xJUm5435Fz8EMGzoTTXgkcf3zAz',
                soFilePath: '../../target/deploy/solv.so',
              },
            ]
          : []),
      ],
      accounts: [
        ...(resolvedConfig.programs.restaking
          ? [
              {
                jsonFileDirPath: '../restaking/tests/mocks',
              },
            ]
          : []),
        ...(resolvedConfig.programs.solv
          ? [
              {
                jsonFileDirPath: '../solv/tests/mocks',
              },
            ]
          : []),
      ],
    },
  });

  const feePayer = 'GiDkDCZjVC8Nk1Fd457qGSV2g3MQX62n7cV5CvgFyGfF';
  await validator.airdrop(feePayer, 1_000_000_000_000n);

  const sdk: SDK = await import(
    config?.useDistSDK
      ? '@fragmetric-labs/sdk'
      : '../../clients/js/fragmetric-sdk/src'
  );
  const executionHooks = sdk.createDefaultTransactionExecutionHooks({
    tag: process.env.VITEST_WORKER_ID
      ? `worker-${process.env.VITEST_WORKER_ID}`
      : undefined,
    inspection: true,
  });
  if (!resolvedConfig.debug) {
    // hide too verbose logs except error logs
    delete executionHooks.onSignature;
    delete executionHooks.onResult;
  }
  const rpcConfig =
    resolvedConfig.validator == 'litesvm'
      ? {
          accountCacheTTLSeconds: 1,
          accountDeduplicationIntervalSeconds: 0,
          accountBatchIntervalMilliseconds: 0,
          blockhashCacheTTLMilliseconds: 50,
          blockhashBatchIntervalMilliseconds: 0,
        }
      : undefined;
  const transactionConfig = {
    feePayer: feePayer,
    signers: [
      ...sdk.createTransactionSignerResolvers({
        rootDir: __dirname,
        keypairs: [
          '../../keypairs/shared_wallet_GiDkDCZjVC8Nk1Fd457qGSV2g3MQX62n7cV5CvgFyGfF.json',
          ...(resolvedConfig.programs.restaking
            ? ['../../keypairs/restaking']
            : []),
          ...(resolvedConfig.programs.solv ? ['../../keypairs/solv'] : []),
        ],
      }),
    ],
    executionHooks: executionHooks,
  };

  const restaking = resolvedConfig.programs.restaking
    ? sdk.RestakingProgram.connect(validator.runtime, {
        rpc: rpcConfig,
        transaction: transactionConfig,
        // debug: true,
      })
    : (undefined as unknown as _sdk.RestakingProgram);

  const solv = resolvedConfig.programs.solv
    ? sdk.SolvVaultProgram.connect(validator.runtime, {
        rpc: rpcConfig,
        transaction: transactionConfig,
        // debug: true,
      })
    : (undefined as unknown as _sdk.SolvVaultProgram);

  return { validator, sdk, solv, restaking, feePayer };
}
