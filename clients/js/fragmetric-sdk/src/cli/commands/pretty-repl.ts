import type repl from 'repl';

export function start(
  ...args: Parameters<typeof repl.start>
): Promise<ReturnType<typeof repl.start>> {
  if (process?.versions?.bun) {
    throw new Error(`REPL is not supported on Bun runtime`);
  }
  return import('pretty-repl').then((m) => m.start(...args));
}
