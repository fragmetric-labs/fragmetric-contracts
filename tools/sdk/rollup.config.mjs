import nodeResolve from '@rollup/plugin-node-resolve';
import nodePolyfills from 'rollup-plugin-polyfill-node';
import json from '@rollup/plugin-json';
import commonjs from '@rollup/plugin-commonjs';
import typescript from 'rollup-plugin-typescript2';
import { terser } from 'rollup-plugin-terser';

import packageJson from './package.json' with { type: 'json' };

export default {
    input: 'src/index.ts',
    output: [
        {
            file: packageJson.main,   // CommonJS output
            format: 'cjs',
            sourcemap: true,
        },
        {
            file: packageJson.module, // ES module output
            format: 'es',
            sourcemap: true,
        },
        {
            file: packageJson.browser, // UMD output (for browsers)
            format: 'umd',
            name: 'fragmetric',
            sourcemap: true,
            globals: {
                // '@coral-xyz/anchor': 'anchor',
                // '@solana/web3.js': 'web3',
            },
        },
    ],
    plugins: [
        nodeResolve({ browser: true }), // resolves Node modules so they can be bundled
        nodePolyfills(), // polifills for Node native modules for browser
        json(), // resolve JSON files
        commonjs(), // converts CJS modules to ES6 so Rollup can process them
        typescript({
            tsconfig: 'tsconfig.json',
            declaration: true,
            declarationDir: 'dist/types',
        }),
        terser(), // minifies the bundle
    ],
    // excludes direct dependencies from bundle
    external: [
        ...Object.keys(packageJson.dependencies || {}),
        ...Object.keys(packageJson.peerDependencies || {}),
    ]
};
