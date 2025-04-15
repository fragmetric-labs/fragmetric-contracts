import { createSolanaRpc, createSolanaRpcSubscriptions } from '@solana/kit';
import { describe, expect, test } from 'vitest';
import { createRuntime } from './index';

describe('createRuntime with SVMRuntimeOptions', () => {
  test('create SVMRuntime with rpc, cluster options', async () => {
    const runtime = createRuntime({
      type: 'svm',
      rpc: createSolanaRpc('https://api.devnet.solana.com'),
      rpcSubscriptions: createSolanaRpcSubscriptions(
        'wss://api.devnet.solana.com'
      ),
      cluster: 'devnet',
    });

    expect(runtime.cluster).equals('devnet');
  });

  test('create SVMRuntime without type options', async () => {
    const runtime = createRuntime({
      rpc: createSolanaRpc('http://0.0.0.0:8888'),
      rpcSubscriptions: createSolanaRpcSubscriptions('ws://0.0.0.0:8889'),
      cluster: 'local',
    });

    expect(runtime.cluster).equals('local');
  });
});
