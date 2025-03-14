import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['**/*.test.ts'],
    passWithNoTests: true,
    testTimeout: 5 * 60 * 1000,
  },
});
