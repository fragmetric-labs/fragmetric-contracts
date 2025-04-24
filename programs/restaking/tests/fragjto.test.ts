import { afterAll, beforeAll, describe } from 'vitest';
import { createTestSuiteContext } from '../../testutil';
import { initializeFragJTO } from './fragjto';
import { fragJTOConfigurationTest } from './fragjto.config';

describe('restaking.fragJTO test', async () => {
  const ctx = initializeFragJTO(
    await createTestSuiteContext({ programs: { solv: false } })
  );

  beforeAll(() => ctx.initializationTasks);
  afterAll(() => ctx.validator.quit());

  await fragJTOConfigurationTest(ctx);
});
