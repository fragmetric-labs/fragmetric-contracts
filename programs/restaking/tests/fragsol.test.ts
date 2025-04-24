import { afterAll, beforeAll, describe } from 'vitest';
import { createTestSuiteContext } from '../../testutil';
import { initializeFragSOL } from './fragsol';
import { fragSOLConfigurationTest } from './fragsol.config';
import { fragSOLDepositTest } from './fragsol.deposit';
import { fragSOLPlaygroundTest } from './fragsol.playground';

describe('restaking.fragSOL test', async () => {
  const ctx = initializeFragSOL(
    await createTestSuiteContext({ programs: { solv: false } })
  );

  beforeAll(() => ctx.initializationTasks);
  afterAll(() => ctx.validator.quit());

  await fragSOLConfigurationTest(ctx);
  await fragSOLDepositTest(ctx);
  await fragSOLPlaygroundTest(ctx);
});
