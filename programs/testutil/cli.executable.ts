#!/usr/bin/env pnpm tsx --no-warnings

import { initializeFragBTC } from '../restaking/tests/fragbtc';
import { initializeFragJTO } from '../restaking/tests/fragjto';
import { initializeFragSOL } from '../restaking/tests/fragsol';
import { createTestSuiteContext } from './context';

createTestSuiteContext({ validator: 'litesvm' })
  .then(async (ctx) => {
    // init receipt tokens sequentially
    ctx.sdk.logger.start('Initialize fragSOL...');
    return initializeFragSOL(ctx)
      .initializationTasks.then(() => {
        ctx.sdk.logger.start('Initialize fragJTO...');
        return initializeFragJTO(ctx).initializationTasks;
      })
      .then(() => {
        ctx.sdk.logger.start('Initialize fragBTC...');
        return initializeFragBTC(ctx).initializationTasks;
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
