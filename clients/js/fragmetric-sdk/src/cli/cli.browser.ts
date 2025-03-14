export function startCommandLineInterface() {
  throw new Error('startCommandLineInterface is not supported in browser');
}

export function createTransactionInspectionURL(
  signature: string,
  cluster: string | null
): string {
  return signature;
}
