import { afterAll, beforeAll, describe } from 'vitest';
import { createTestSuiteContext } from '../../testutil';
import { initializeFragBTC } from './fragbtc';
import { fragBTCConfigurationTest } from './fragbtc.test.config';
import { fragBTCDepositTest } from './fragbtc.test.deposit';
import { fragBTCOperationTest } from './fragbtc.test.operation';

describe('restaking.fragBTC test', async () => {
  const testCtx = initializeFragBTC(
    await createTestSuiteContext({ programs: { solv: true } })
  );

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  await fragBTCConfigurationTest(testCtx);
  await fragBTCDepositTest(testCtx);
  await fragBTCOperationTest(testCtx);
});
