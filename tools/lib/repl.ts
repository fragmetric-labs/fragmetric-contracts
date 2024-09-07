import * as repl from 'repl';
import * as fs from 'fs';
import * as path from 'path';
import readline from "readline";

export function startREPL(context: Object = {}) {
    const shell = repl.start("> ");
    shell.displayPrompt();
    shell.context.fs = fs;
    shell.context.path = path;
}

export function askOnce<T extends string = string>(question: string): Promise<T> {
    return new Promise<T>((resolve) => {
        const rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout
        });
        rl.question(question, (answer: string) => {
            const normalizedAnswer = answer.trim().toLowerCase() as T;
            resolve(normalizedAnswer);
            rl.close();
        });
    })
}