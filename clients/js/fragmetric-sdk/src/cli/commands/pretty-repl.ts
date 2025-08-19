import type repl from 'repl';

export function start(...args: Parameters<typeof repl.start>): ReturnType<typeof repl.start> {
  if (process?.versions?.bun) {
    throw new Error(`REPL is not supported on Bun runtime`);
  }
  return require('pretty-repl').start(...args);
}