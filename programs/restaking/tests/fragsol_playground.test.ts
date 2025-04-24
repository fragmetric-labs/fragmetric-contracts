import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext } from '../../testutil';
import { initializeFragSOL } from './fragsol';

describe('restaking.fragSOL new test', async () => {
  const ctx = initializeFragSOL(
    await createTestSuiteContext({ programs: { solv: false } })
  );

  beforeAll(() => ctx.initializationTasks);
  afterAll(() => ctx.validator.quit());

  test(`new test example`, async () => {
    // implement some test suite then merge into an existing suite to reduce number of test suites if possible
    expect(true).toBeTruthy();
  });
});
