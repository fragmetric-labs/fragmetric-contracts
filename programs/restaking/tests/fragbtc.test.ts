import { afterAll, beforeAll, describe } from 'vitest';
import { initializeFragBTC } from './fragbtc';
import { fragBTCConfigurationTest } from './fragbtc.test.config';
import { fragBTCDepositTest } from './fragbtc.test.deposit';
import { createTestSuiteContext } from './utils';

describe('restaking.fragBTC test', async () => {
  const testCtx = initializeFragBTC(await createTestSuiteContext());

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  await fragBTCConfigurationTest(testCtx);
  await fragBTCDepositTest(testCtx);
});
