import alias from '@rollup/plugin-alias';
import commonjs from '@rollup/plugin-commonjs';
import json from '@rollup/plugin-json';
import nodeResolve from '@rollup/plugin-node-resolve';
import replace from '@rollup/plugin-replace';
import { builtinModules } from 'node:module';
import { defineConfig } from 'rollup';
import del from 'rollup-plugin-delete';
import esbuild from 'rollup-plugin-esbuild';
import polyfillNode from 'rollup-plugin-polyfill-node'; // for Buffer/crypto/etc
import packageJSON from './package.json' with { type: 'json' };

export default defineConfig([
  {
    input: 'src/index.ts',
    output: [
      {
        file: 'dist/index.browser.mjs',
        format: 'esm',
        sourcemap: true,
      },
    ],
    plugins: [
      del({ targets: 'dist/*' }),

      // rewrite paths before anything else
      alias({
        entries: [
          {
            find: './polyfills.node',
            replacement: './polyfills.browser',
          },
          {
            find: './signer.node',
            replacement: './signer.browser',
          },
          {
            find: './litesvm.node',
            replacement: './litesvm.browser',
          },
          {
            find: './cli.node',
            replacement: './cli.browser',
          },
          {
            find: './__devtools',
            replacement: './__devtools/.dist',
          },
        ],
      }),

      // replace env vars and globals before parsing files
      replace({
        preventAssignment: true,
        values: {
          'process.env.NODE_ENV': JSON.stringify('production'),
        },
      }),

      // polyfill Node.js globals and built-ins for browser
      polyfillNode(),

      // resolve modules (with browser-friendly versions preferred)
      nodeResolve({
        browser: true,
        preferBuiltins: false,
      }),

      // handle CommonJS modules
      commonjs(),

      // handle JSON imports
      json(),

      // compile TypeScript last (after aliasing and resolving modules)
      esbuild({
        keepNames: true,
      }),
    ],
  },
  {
    input: 'src/index.ts',
    output: [
      {
        file: 'dist/index.node.cjs',
        format: 'cjs',
        sourcemap: true,
      },
    ],
    plugins: [
      // rewrite paths before anything else
      alias({
        entries: [
          {
            find: './__devtools',
            replacement: './__devtools/.dist',
          },
        ],
      }),

      nodeResolve({
        preferBuiltins: true,
      }),
      commonjs(),
      json(),
      esbuild({
        keepNames: true,
      }),
    ],
    external: [
      ...builtinModules,
      ...Object.keys(packageJSON.optionalDependencies ?? {}),
      ...Object.keys(packageJSON.dependencies ?? {}),
    ],
  },
]);
