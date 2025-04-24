import { afterAll, beforeAll, describe } from 'vitest';
import { createTestSuiteContext } from '../../testutil';
import { initializeZBTCVault } from './zbtc';
import { zBTCVaultConfigurationTest } from './zbtc.test.config';

describe('solv.zBTC vault test', async () => {
  const testCtx = initializeZBTCVault(
    await createTestSuiteContext({ programs: { restaking: false } })
  );

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  await zBTCVaultConfigurationTest(testCtx);
});
