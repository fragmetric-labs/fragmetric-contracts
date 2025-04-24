import { afterAll, beforeAll, describe } from 'vitest';
import { createTestSuiteContext } from '../../testutil';
import { initializeFragSOL } from './fragsol';
import { fragSOLConfigurationTest } from './fragsol.test.config';
import { fragSOLDepositTest } from './fragsol.test.deposit';

describe('restaking.fragSOL test', async () => {
  const ctx = initializeFragSOL(
    await createTestSuiteContext({ programs: { solv: false } })
  );

  beforeAll(() => ctx.initializationTasks);
  afterAll(() => ctx.validator.quit());

  await fragSOLConfigurationTest(ctx);
  await fragSOLDepositTest(ctx);
});
