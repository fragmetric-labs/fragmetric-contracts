import { afterAll, beforeAll, describe } from 'vitest';
import { initializeFragJTO } from './fragjto';
import { fragJTOConfigurationTest } from './fragjto.test.config';
import { createTestSuiteContext } from './utils';

describe('restaking.fragJTO test', async () => {
  const ctx = initializeFragJTO(await createTestSuiteContext());

  beforeAll(() => ctx.initializationTasks);
  afterAll(() => ctx.validator.quit());

  await fragJTOConfigurationTest(ctx);
});
