// @ts-ignore
import { AnsiLogger, LogLevel } from 'node-ansi-logger';

export type { LogLevel };

export const LOG_PAD_SMALL = 16;
export const LOG_PAD_LARGE = 32;

export function getLogger(name: string) {
    return new AnsiLogger({
        logName: name,
        logDebug: true,
        logLevel: LogLevel.DEBUG,
        logWithColors: true,
        logTimestampFormat: 2,
    });
}