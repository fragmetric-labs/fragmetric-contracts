import { Rpc, RpcSubscriptions, SolanaRpcSubscriptionsApi } from '@solana/kit';
import {
  Runtime,
  RuntimeCluster,
  RuntimeFactory,
  RuntimeRPCMethods,
  RuntimeRPCOptionalMethods,
} from './runtime';

export type SolanaRuntimeConfig = {
  type?: 'solana';
  rpc: Rpc<RuntimeRPCMethods> & Partial<Rpc<RuntimeRPCOptionalMethods>>;
  rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;
  cluster: RuntimeCluster;
};

export const createSolanaRuntime: RuntimeFactory<SolanaRuntimeConfig> = ({
  rpc,
  rpcSubscriptions,
  cluster,
}): Runtime => {
  return {
    type: 'solana',
    cluster,
    rpc: rpc as any,
    rpcSubscriptions: rpcSubscriptions,
  };
};
