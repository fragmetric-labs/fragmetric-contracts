import * as repl from 'repl';
import * as fs from 'fs';
import * as path from 'path';

export function startREPL(context: Object = {}) {
    const shell = repl.start("> ");
    shell.displayPrompt();
    shell.context.fs = fs;
    shell.context.path = path;
}