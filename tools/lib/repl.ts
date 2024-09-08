import * as repl from 'pretty-repl';
import readline from "readline";

export function startREPL(args: {prompt?: string, context: object}) {
    const replServer = repl.start({
        preview: true,
        // useColors: true,
        prompt: args.prompt ?? '> ',
    });
    replServer.displayPrompt();
    Object.assign(replServer.context, args.context ?? {});
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