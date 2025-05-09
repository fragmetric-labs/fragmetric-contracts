import { createConsola, LogObject } from 'consola';
import * as util from 'node:util';
import stripAnsi from 'strip-ansi';

// enable pretty inspection and JSON serialization ...
const inspectOptions = util.inspect.defaultOptions;
inspectOptions.showProxy = false;
inspectOptions.showHidden = false;
inspectOptions.depth = 10;
inspectOptions.maxArrayLength = 100;
inspectOptions.colors = !process.env.CI;
inspectOptions.customInspect = true;
inspectOptions.numericSeparator = true;

// well... refactor it later when possible
export let loggerFormat = 'pretty';
export const logger = setLogger({ format: loggerFormat as any });

export function setLogger(options?: { format?: 'pretty' | 'json' }) {
  loggerFormat = options?.format == 'json' ? 'json' : 'pretty';
  const logger = createConsola(
    loggerFormat == 'json'
      ? {
          reporters: [
            {
              log(logObj: LogObject) {
                for (let i = 0; i < logObj.args.length; i++) {
                  if (typeof logObj.args[i] === 'string') {
                    logObj.args[i] = stripAnsi(logObj.args[i]);
                  }
                }
                process.stdout.write(JSON.stringify(logObj, null, 2) + '\n');
              },
            },
          ],
          formatOptions: {
            date: true,
          },
        }
      : {
          fancy: true,
          formatOptions: {
            date: false,
            compact: false,
          },
        }
  );
  // logger.wrapConsole();
  return logger;
}
