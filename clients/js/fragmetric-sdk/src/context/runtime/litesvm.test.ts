import { LiteSVM } from 'litesvm';
import { describe, expect, test } from 'vitest';
import { createRuntime } from './index';

describe('createRuntime with LiteSVMRuntimeOptions', () => {
  test('create LiteSVMRuntime with type, svm options', async () => {
    const runtime = createRuntime({
      type: 'litesvm',
      svm: LiteSVM.default(),
    });

    expect(runtime.cluster).equals(
      'local',
      'litesvm runtime cluster is always local'
    );
  });
});
