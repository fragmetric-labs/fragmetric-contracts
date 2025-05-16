import { createSolanaRpc, createSolanaRpcSubscriptions } from '@solana/kit';
import { describe, expect, test } from 'vitest';
import { createRuntime } from './index';

describe('createRuntime with SVMRuntimeOptions', () => {
  test('create SVMRuntime with rpc, cluster options', async () => {
    const rpcUrl = process.env.SOLANA_RPC_DEVNET
      ? process.env.SOLANA_RPC_DEVNET
      : 'https://api.devnet.solana.com';
    const rpcSubscriptionsUrl = rpcUrl
      .replace('https://', 'wss://')
      .replace('http://', 'ws://');
    const runtime = createRuntime({
      type: 'svm',
      rpc: createSolanaRpc(rpcUrl),
      rpcSubscriptions: createSolanaRpcSubscriptions(rpcSubscriptionsUrl),
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
