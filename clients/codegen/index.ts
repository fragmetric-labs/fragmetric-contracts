import { rootNodeFromAnchor } from '@codama/nodes-from-anchor';
import { renderJavaScriptVisitor, renderRustVisitor } from '@codama/renderers';
import * as chalk from 'chalk';
import chokidar from 'chokidar';
import * as codama from 'codama';
import * as crypto from 'crypto';
import * as fs from 'fs';
import * as path from 'path';

import codegenConfig from './codegen.config';

generate()
  .then(() => {
    console.log(`${chalk.green('[codegen] generation completed')}`);
    if (codegenConfig.watch) startWatchers();
  })
  .catch((err) => {
    console.log(`${chalk.red('[codegen] generation failed')}`, err);
  });

async function generate(targetId?: string) {
  for (const [id, config] of Object.entries(codegenConfig.targets)) {
    if (!!targetId && targetId != id) continue;

    // prepare generation
    const idlContent = fs.readFileSync(
      path.join(__dirname, config.idlFilePath),
      'utf-8'
    );
    const outputHash = crypto
      .createHash('sha256')
      .update(JSON.stringify({ id, config }) + idlContent)
      .digest('hex');
    const outputHashShort = outputHash.substring(0, 8);

    const jsOutputDir = path.join(
      __dirname,
      codegenConfig.outputBaseDir.javascript,
      id
    );
    const rustOutputDir = path.join(
      __dirname,
      codegenConfig.outputBaseDir.rust,
      id
    );

    const jsOutputHashFilePath = path.join(jsOutputDir, 'codegen.lock');
    const jsGen =
      config.javascript !== false &&
      (codegenConfig.skipHashCheck ||
        !checkOutputHashEqualsTo(outputHash, jsOutputHashFilePath));
    const rustOutputHashFilePath = path.join(rustOutputDir, 'codegen.lock');
    const rustGen =
      config.rust !== false &&
      (codegenConfig.skipHashCheck ||
        !checkOutputHashEqualsTo(outputHash, rustOutputHashFilePath));

    if (jsGen || rustGen) {
      const idl = JSON.parse(idlContent);
      const ctx = codama.createFromRoot(rootNodeFromAnchor(idl) as any);

      config.visitors?.forEach((createVisitor) => {
        createVisitor(idl).forEach((visitor) => ctx.update(visitor));
      });

      if (jsGen) {
        await ctx.accept(
          renderJavaScriptVisitor(jsOutputDir, {
            deleteFolderBeforeRendering: true,
            formatCode: true,
          })
        );
        fs.writeFileSync(jsOutputHashFilePath, outputHash);
        console.log(
          `${chalk.green('[codegen] generated')} ${chalk.yellow(
            id
          )} ${chalk.dim(`(js:${outputHashShort})`)}`
        );
      }
      if (rustGen) {
        ctx.accept(
          renderRustVisitor(rustOutputDir, {
            crateFolder: rustOutputDir,
            deleteFolderBeforeRendering: true,
            formatCode: true,
            traitOptions: {
              useFullyQualifiedName: false,
              ...config.rustTraitOptions,
            },
          })
        );
        fs.writeFileSync(rustOutputHashFilePath, outputHash);
        console.log(
          `${chalk.green('[codegen] generated')} ${chalk.yellow(
            id
          )} ${chalk.dim(`(rust:${outputHashShort})`)}`
        );
      }
    }
    if (!jsGen) {
      console.log(
        `${chalk.dim(
          `[codegen] skipped ${chalk.dim(id)} (js:${outputHashShort})`
        )}`
      );
    }
    if (!rustGen) {
      console.log(
        `${chalk.dim(
          `[codegen] skipped ${chalk.dim(id)} (rust:${outputHashShort})`
        )}`
      );
    }
  }
}

function checkOutputHashEqualsTo(hash: string, hashFilePath: string): boolean {
  if (fs.existsSync(hashFilePath)) {
    return fs.readFileSync(hashFilePath).toString() == hash;
  }
  return false;
}

function startWatchers() {
  for (const [id, config] of Object.entries(codegenConfig.targets)) {
    const idlFilePath = path.join(__dirname, config.idlFilePath);
    chokidar
      .watch(idlFilePath, { ignoreInitial: true })
      .on('change', (filePath) => {
        console.log(
          `${chalk.blue('[codegen] detected changes in')} ${chalk.yellow(
            id
          )}: ${chalk.dim(filePath)}`
        );
        generate(id)
          .then(() =>
            console.log(
              `${chalk.green('[codegen] generation completed after change')}`
            )
          )
          .catch((err) =>
            console.error(
              `${chalk.red('[codegen] generation failed after change')}`,
              err
            )
          );
      })
      .on('error', (error) =>
        console.error(`${chalk.red('[codegen] watcher error')}`, error)
      );
    console.log(
      `${chalk.blue('[codegen] watching changes in')} ${chalk.yellow(
        id
      )}: ${chalk.dim(idlFilePath)}`
    );
  }
}
