import { consola, LogObject } from 'consola';
import stripAnsi from 'strip-ansi';

// well... refactor it later when possible
const loggerOriginalReporters = [...consola.options.reporters];
export let loggerFormat = 'pretty';
export const logger = setLogger({ format: loggerFormat as any });

export function setLogger(options?: { format?: 'pretty' | 'json' }) {
  switch (options?.format) {
    case 'json':
      loggerFormat = 'json';
      consola.setReporters([
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
      ]);
      consola.options.formatOptions.date = true;
      break;
    default:
      loggerFormat = 'pretty';
      consola.setReporters(loggerOriginalReporters);
      consola.options.formatOptions.date = false;
  }

  consola.wrapConsole();
  return consola;
}
