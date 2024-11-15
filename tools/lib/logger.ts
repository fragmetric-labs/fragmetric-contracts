// @ts-ignore
import { AnsiLogger, LogLevel } from 'node-ansi-logger';

export type { LogLevel };

const LOG_PAD_SMALL = 32;
const LOG_PAD_LARGE = 64;

export function getLogger(name: string) {
    const ciMode = !!process.env.CI;
    return {
        logger: new AnsiLogger({
            logName: name,
            logDebug: true,
            logLevel: LogLevel.DEBUG,
            logWithColors: !ciMode,
            logTimestampFormat: ciMode ? 6 : 4,
            logCustomTimestampFormat: 'CI',
        }),
        LOG_PAD_SMALL: LOG_PAD_SMALL - name.length,
        LOG_PAD_LARGE: LOG_PAD_LARGE - name.length,
    };
}
