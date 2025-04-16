import { Rpc, RpcSubscriptions, SolanaRpcSubscriptionsApi } from '@solana/kit';
import {
  Runtime,
  RuntimeCluster,
  RuntimeFactory,
  RuntimeRPCMethods,
  RuntimeRPCOptionalMethods,
} from './runtime';

export type SVMRuntimeConfig = {
  type?: 'svm';
  rpc: Rpc<RuntimeRPCMethods> & Partial<Rpc<RuntimeRPCOptionalMethods>>;
  rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;
  cluster: RuntimeCluster;
};

export const createSVMRuntime: RuntimeFactory<SVMRuntimeConfig> = ({
  rpc,
  rpcSubscriptions,
  cluster,
}): Runtime => {
  return {
    type: 'svm',
    cluster,
    rpc: rpc as any,
    rpcSubscriptions: rpcSubscriptions,
  };
};
