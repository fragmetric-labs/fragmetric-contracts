import { afterAll, beforeAll, describe } from 'vitest';
import { createTestSuiteContext } from '../../testutil';
import { initializeFragBTC } from './fragbtc';
import { fragBTCConfigurationTest } from './fragbtc.config';
import { fragBTCDepositTest } from './fragbtc.deposit';
import { fragBTCOperationTest } from './fragbtc.operation';

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
