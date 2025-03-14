import { Context } from '../context';
import { RuntimeContext } from '../runtime';
import { ProgramContext } from './context';

export abstract class ProgramDerivedContext<
  P extends Context<any>,
> extends Context<P> {
  get program() {
    return this.__memoized('program', () => {
      const runtime = this.__findParentContext(ProgramContext);
      if (!runtime) {
        throw new Error('ProgramContext not found in the context hierarchy');
      }
      return runtime as ProgramContext;
    });
  }

  get runtime() {
    return this.__memoized('runtime', () => {
      const runtime = this.__findParentContext(RuntimeContext);
      if (!runtime) {
        throw new Error('RuntimeContext not found in the context hierarchy');
      }
      return runtime;
    });
  }

  // safely get runtime options for test cases
  protected get __maybeRuntimeOptions() {
    return this.__memoized('__maybeRuntimeOptions', () => {
      return this.__findParentContext(RuntimeContext)?.options;
    });
  }

  protected get __debug() {
    return this.__memoized('__debug', () => {
      return this.__maybeRuntimeOptions?.debug == true;
    });
  }
}
