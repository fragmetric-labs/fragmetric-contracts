import * as util from 'node:util';
import {
  ProgramDerivedContext,
  RuntimeContextOptions,
  TransactionTemplateOverrides,
} from '../../context';

const AlreadyReportedErrorSymbol = Symbol.for('AlreadyReportedError');
export function isAlreadyReportedError(err: any): boolean {
  return err && (err as any)[AlreadyReportedErrorSymbol] == true;
}

export function createDefaultTransactionExecutionHooks(opts?: {
  tag?: string;
  inspection?: boolean;
  mergeWith?: RuntimeContextOptions['transaction']['executionHooks'];
}): NonNullable<
  TransactionTemplateOverrides<
    ProgramDerivedContext<any>,
    any
  >['executionHooks']
> {
  const log = opts?.tag ? logger.withTag(opts.tag) : logger;

  return {
    onSignature: (parent, signature, args) => {
      if (loggerFormat === 'pretty') {
        log.start(
          `${'Signed'.padEnd(10, ' ')} ${createTransactionInspectionURL(signature, parent.runtime.cluster)}`
        );
      } else {
        log.start('transaction signed', {
          signature,
          parent: parent.toContextDescription(),
        });
      }
      opts?.mergeWith?.onSignature?.(parent, signature, args);
    },
    onError: (parent, err, args) => {
      if (loggerFormat === 'pretty') {
        log.error(util.inspect(err, false, 10, true));
      } else {
        log.error('transaction error', {
          error: util.inspect(err, false, 10, false),
          parent: parent.toContextDescription(),
        });
      }
      Object.defineProperty(err, AlreadyReportedErrorSymbol, {
        value: true,
        enumerable: false,
        configurable: true,
      });
      opts?.mergeWith?.onError?.(parent, err, args);
    },
    onResult: (parent, result, args) => {
      let common: any = {
        succeeded: result.succeeded,
        slot: result.slot,
        args,
        events: result.events,
      };
      if (opts?.inspection) {
        const tx = result.result
          ? compileTransaction(result.result.transaction)
          : undefined;
        const txSize = tx
          ? tx.messageBytes.length + Object.keys(tx.signatures).length * 64 + 1
          : undefined;
        common = {
          ...common,
          inspection: {
            logs: result.result?.meta?.logMessages,
            feeLamports: result.result?.meta?.fee,
            computeUnits: result.result?.meta?.computeUnitsConsumed,
            computeUnitsUsage: result.result?.meta?.computeUnitsConsumed
              ? Math.ceil(
                  (Number(result.result.meta.computeUnitsConsumed) * 100) /
                    1_400_000
                ) / 100
              : undefined,
            transactionSizeBytes: txSize,
            transactionSizeUsage: txSize
              ? Math.ceil((txSize * 100) / 1232) / 100
              : undefined,
          },
        };
      }
      if (result.succeeded) {
        if (loggerFormat === 'pretty') {
          log.success(
            `${'Confirmed'.padEnd(10, ' ')} ${parent.toContextDescription().properties.description}`,
            util.inspect(common)
          );
        } else {
          log.success('transaction confirmed', {
            signature: result.signature,
            ...common,
            parent: parent.toContextDescription(),
          });
        }
      } else {
        if (loggerFormat === 'pretty') {
          log.fail(
            `${'Failed'.padEnd(10, ' ')} ${parent.toContextDescription().properties.description}`,
            util.inspect(common)
          );
        } else {
          log.fail('transaction failed', {
            signature: result.signature,
            ...common,
            parent: parent.toContextDescription(),
          });
        }
      }
      opts?.mergeWith?.onResult?.(parent, result, args);
    },
  };
}

import { compileTransaction } from '@solana/kit';
import terminalLink$ from 'terminal-link';
import { logger, loggerFormat } from './logger';
const terminalLink: typeof terminalLink$ =
  (terminalLink$ as any).default ?? terminalLink$;

export function createTransactionInspectionURL(
  signature: string,
  cluster: string | null
): string {
  if (!cluster || cluster == 'local') return signature;
  return terminalLink(
    signature,
    `https://explorer.solana.com/tx/${signature}?cluster=${cluster}`
  );
}
