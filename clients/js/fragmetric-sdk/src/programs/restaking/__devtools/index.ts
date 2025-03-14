import { RestakingProgram } from '../program';
import { mock } from './mock';

/**
 * Dev-Only Tools (`__dev`)
 *
 * This module is excluded from production builds and intended for use in REPL, CLI, test suites.
 * - mocking on-chain accounts
 * - and more...?
 *
 * Example CLI usage:
 *   $ pnpm connect -u m -e "restaking.__dev.mock('fragsol_jito_nsol_vault')"
 *
 * Example TestSuite usage:
 *   restaking.__dev.???;
 */

export function createDevTools(program: RestakingProgram) {
  return {
    mock: mock.bind(program),
  };
}
