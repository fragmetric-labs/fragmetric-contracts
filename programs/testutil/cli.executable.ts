#!/usr/bin/env pnpm tsx --no-warnings

import { initializeFragBTC } from '../restaking/tests/fragbtc.init';
import { initializeFragJTO } from '../restaking/tests/fragjto.init';
import { initializeFragSOL } from '../restaking/tests/fragsol.init';
import { initializeCBBTCVault } from '../solv/tests/cbbtc.init';
import { initializeWBTCVault } from '../solv/tests/wbtc.init';
import { initializeZBTCVault } from '../solv/tests/zbtc.init';
import { createTestSuiteContext } from './context';

createTestSuiteContext({ validator: 'litesvm' })
  .then(async (ctx) => {
    // init receipt tokens sequentially
    return Promise.resolve()
      .then(() => {
        if (!process.env.PROGRAM || process.env.PROGRAM == 'restaking') {
          return Promise.resolve()
            .then(() => {
              ctx.sdk.logger.start('Initialize fragSOL...');
              return initializeFragSOL(ctx).initializationTasks;
            })
            .then(() => {
              ctx.sdk.logger.start('Initialize fragJTO...');
              return initializeFragJTO(ctx).initializationTasks;
            })
            .then(() => {
              ctx.sdk.logger.start('Initialize fragBTC...');
              return initializeFragBTC(ctx).initializationTasks;
            })
            .then(() => {});
        }
        return Promise.resolve();
      })
      .then(() => {
        if (!process.env.PROGRAM || process.env.PROGRAM == 'solv') {
          return Promise.resolve()
            .then(() => {
              ctx.sdk.logger.start('Initialize zBTC Vault...');
              return initializeZBTCVault(ctx).initializationTasks;
            })
            .then(() => {
              ctx.sdk.logger.start('Initialize cbBTC Vault...');
              return initializeCBBTCVault(ctx).initializationTasks;
            })
            .then(() => {
              ctx.sdk.logger.start('Initialize wBTC Vault...');
              return initializeWBTCVault(ctx).initializationTasks;
            })
            .then(() => {});
        }
        return Promise.resolve();
      })
      .catch((err: any) => {
        ctx.sdk.logger.error(err);
        process.exit(1);
      })
      .then(() => {
        ctx.sdk.startCommandLineInterface({
          contextOverrides: {
            programs: { restaking: ctx.restaking, solv: ctx.solv },
            validator: ctx.validator,
          },
        });
      });
  })
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });
