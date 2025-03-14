import { TransactionSignerResolver } from '../address';

export function createLedgerSignerResolver(): TransactionSignerResolver {
  throw new Error(`createLedgerSignerResolver is not supported in browser`);
}

export function createTransactionSignerResolvers(): Record<
  string,
  TransactionSignerResolver
> {
  throw new Error(
    `createTransactionSignerResolvers is not supported in browser`
  );
}
