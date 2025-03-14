import path from 'path';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  resolve: {
    alias: {
      '@fragmetric-labs/sdk': path.resolve(
        __dirname,
        './clients/js/fragmetric-sdk/src'
      ),
      '@fragmetric-labs/testutil': path.resolve(
        __dirname,
        './clients/js/testutil/src'
      ),
    },
  },
  test: {
    coverage: {
      provider: 'v8',
      reportsDirectory: './coverage',
      reportOnFailure: false,
      exclude: [
        '**/node_modules/**',
        '**/{dist,lib,generated,.coverage}/**',
        '**/*.{d.ts,js,config.**}',
        '{.git,.github,.anchor,.idea,target,tests,tools}/**',
      ],
    },
    include: ['**/*.test.ts'],
    passWithNoTests: true,
    testTimeout: 10 * 60 * 1000,
    hookTimeout: 5 * 60 * 1000,

    pool: 'forks', // parallel run for `describe`s
    sequence: {
      concurrent: false, // sequential run for `test`s
    },

    onConsoleLog(log: string, type: 'stdout' | 'stderr'): boolean | void {
      return !!(process.env.CI || process.env.DEBUG || type == 'stderr');
    },
  },
});
