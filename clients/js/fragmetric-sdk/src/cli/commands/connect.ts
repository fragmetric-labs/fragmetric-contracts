import chalk from 'chalk';
import { Command } from 'commander';
import * as module from 'node:module';
import * as os from 'node:os';
import * as path from 'node:path';
import * as readline from 'node:readline';
import * as repl from 'node:repl';
import * as util from 'node:util';
import * as prepl from 'pretty-repl';
import * as sdk from '../../index';
import * as sdkConstants from '../../context/constants';
import { RootCommandOptions } from '../cli.node';
import { isAlreadyReportedError, logger } from '../utils';

export const connectCommand = new Command()
  .name('connect')
  .description('Create a REPL to interact with programs.')
  .configureHelp({ showGlobalOptions: true })
  .option('-e, --eval <EXPRESSION>', 'Evaluate an expression and quit.')
  .action(async function () {
    const expression: string | undefined = this.opts().eval;
    const rootOptions = this.parent!.opts() as RootCommandOptions;

    const endpoint = chalk.dim(
      `${rootOptions.url.length > 35 ? rootOptions.url.substring(0, 33) + '..' : rootOptions.url}`
    );
    const cluster =
      rootOptions.cluster == 'mainnet'
        ? chalk.bgRed.white(' mainnet ')
        : rootOptions.cluster == 'devnet'
          ? chalk.bgYellow.black(' devnet ')
          : chalk.bgWhite.black(` ${rootOptions.cluster} `);

    if (!expression) {
      logger.box(
        `Connected to ${chalk.green('Fragmetric')} programs ${rootOptions.cluster} REPL.\n- Press ${chalk.bold('TAB')} to autocomplete.\n- Can use ${chalk.bold('_')} to refer the previous evaluation result.`
      );
    }

    // fetch all programs data eagerly
    Promise.all(
      Object.values(rootOptions.context.programs).map((program) =>
        program.resolve()
      )
    ).catch(logger.warn);

    // create REPL server
    const server = prepl.start({
      preview: true,
      prompt: expression ? '' : `${cluster} ${endpoint} > `,
      writer: util.inspect,
    });

    server.setupHistory(
      path.join(os.homedir(), '.fragmetric_history'),
      (err) => {
        if (err) {
          logger.warn(err);
        }
      }
    );

    // set root context
    const context = {
      ...rootOptions.context.programs,
      ...rootOptions.context,
      ...sdkConstants,
      sdk,
    };
    Object.assign(server.context, context);

    // set custom completer
    const originalCompleter = server.completer.bind(
      server
    ) as readline.AsyncCompleter;
    const customCompleter: readline.AsyncCompleter = (line, callback) => {
      originalCompleter(line, (err, result) => {
        if (result) {
          if (!line && !result[1]) {
            result[0] = Object.keys(context);
          } else {
            result[0] = result[0].filter((token) => {
              return !(
                /^[A-Z]/.test(token) ||
                /(\.|^)__[^.]*$/.test(token) || // only hide final segments starting with `__`
                /(?:__proto__|constructor|hasOwnProperty|isPrototypeOf|propertyIsEnumerable|toJSON|toString|toLocaleString|valueOf)$/.test(
                  token
                ) ||
                module.builtinModules.includes(token)
              );
            });
          }
        }
        callback(err, result);
      });
    };
    Object.assign(server, { completer: customCompleter });

    // set custom eval
    const startSpinner = () => {
      // not for --eval option
      if (expression) {
        return () => {};
      }

      // control stdin
      let stopSpinner = () => {};
      const muteAndAbort = (chunk: any) => {
        if (
          chunk === '\x03' ||
          (Buffer.isBuffer(chunk) && chunk.toString() === '\x03')
        ) {
          Object.values(rootOptions.context.programs).forEach((program) => {
            program.runtime.abortController.abort();
          });
          stopSpinner();
        }
      };

      const listeners = server.input.listeners('data');
      listeners.forEach((listener) =>
        server.input.off('data', listener as any)
      );
      server.input.on('data', muteAndAbort);

      // display loading indicator
      const loadingIndicatorOutputs = [
        '⠋',
        '⠙',
        '⠹',
        '⠸',
        '⠼',
        '⠴',
        '⠦',
        '⠧',
        '⠇',
        '⠏',
      ];
      let loadingIndicatorFrame = 0;
      const originalWriteOut = process.stdout.write;
      const originalWriteErr = process.stderr.write;
      let intervalId: NodeJS.Timeout | null = setInterval(() => {
        const indicator =
          loadingIndicatorOutputs[
            loadingIndicatorFrame % loadingIndicatorOutputs.length
          ];
        originalWriteOut.call(
          process.stdout,
          `\r${chalk.bold.blue(indicator)} Press Ctrl+C to abort`
        );
        loadingIndicatorFrame++;
      }, 100);

      // setup cleaner
      stopSpinner = () => {
        if (intervalId) {
          clearTimeout(intervalId);
          intervalId = null;
          readline.clearLine(process.stdout, 0);
          readline.cursorTo(process.stdout, 0);
          process.stdout.write = originalWriteOut;
          process.stderr.write = originalWriteErr;

          // restore listeners
          listeners.forEach((listener) =>
            server.input.on('data', listener as any)
          );
          server.input.off('data', muteAndAbort);
          server.displayPrompt();
        }
      };

      // intercept stdin/out
      process.stdout.write = function (chunk: any, encoding?: any, cb?: any) {
        originalWriteOut.call(
          process.stdout,
          '\r'.padEnd(process.stdout.columns, ' ') + '\r'
        );
        return originalWriteOut.call(process.stdout, chunk, encoding, cb);
      };
      process.stderr.write = function (chunk: any, encoding?: any, cb?: any) {
        originalWriteOut.call(
          process.stdout,
          '\r'.padEnd(process.stdout.columns, ' ') + '\r'
        );
        return originalWriteErr.call(process.stderr, chunk, encoding, cb);
      };
      return stopSpinner;
    };

    const originalEval = server.eval.bind(server);
    const customEval: repl.REPLEval = async (
      cmd,
      context,
      filename,
      callback
    ) => {
      // preview
      if (cmd.endsWith('} catch {}')) {
        return originalEval(cmd, context, filename, callback);
      }

      let stopSpinner = () => {};
      try {
        const result = await new Promise((resolve, reject) => {
          try {
            originalEval(cmd, context, filename, async (err, result) => {
              if (err) {
                if (
                  err.name === 'SyntaxError' &&
                  /^(Unexpected end of input|Unexpected token)/.test(
                    err.message
                  )
                ) {
                  return reject(new repl.Recoverable(err));
                }
                return reject(err);
              } else {
                if (typeof result?.then == 'function') {
                  stopSpinner = startSpinner();
                  try {
                    resolve(await result);
                  } catch (err) {
                    reject(err);
                  }
                } else {
                  resolve(result);
                }
              }
            });
          } catch (err) {
            reject(err);
          }
        });

        readline.clearLine(process.stdout, 0);
        readline.cursorTo(process.stdout, 0);
        callback(null, result);
      } catch (err) {
        readline.clearLine(process.stdout, 0);
        readline.cursorTo(process.stdout, 0);
        if (isAlreadyReportedError(err)) {
          if (expression) {
            callback(err as Error, undefined);
          } else {
            callback(null, undefined);
          }
        } else {
          callback(err as Error, undefined);
        }
      } finally {
        stopSpinner();
      }
    };
    Object.assign(server, { eval: customEval });

    // run given expression directly
    if (expression) {
      server.eval(expression, server.context, '', async (err, res) => {
        server.close();
        logger.restoreConsole();

        if (err) {
          if (!isAlreadyReportedError(err)) {
            console.error(err);
          }
          process.exit(1);
        } else {
          if (rootOptions.format == 'json') {
            console.log(JSON.stringify(res, null, 2));
          } else {
            console.log(util.inspect(res, false, null, true));
          }
          process.exit(0);
        }
      });
    } else {
      server.displayPrompt();
    }
  });
