import replace from '@rollup/plugin-replace';
import nodeResolve from '@rollup/plugin-node-resolve';
import nodePolyfills from 'rollup-plugin-polyfill-node';
import json from '@rollup/plugin-json';
import commonjs from '@rollup/plugin-commonjs';
import typescript from 'rollup-plugin-typescript2';
import { terser } from 'rollup-plugin-terser';

import packageJson from './package.json' with { type: 'json' };

const generateConfig = (format, browser = false) => {
    if (!['cjs', 'esm', 'umd'].includes(format)) {
        throw "unsupported output format";
    }

    return {
        input: 'src/index.ts',
        output: [
            {
                file: `dist/index${browser ? '.browser' : ''}.${format}.js`,
                format,
                sourcemap: true,
                name: format === 'umd' ? 'fragmetricSDK' : undefined,
                globals: format === 'umd' ? { '@solana/web3.js': 'solanaWeb3' } : undefined,
                exports: format === 'cjs' ? 'named' : undefined,
            },
        ],
        plugins: [
            replace({
                preventAssignment: true,
                'process.env.NODE_ENV': JSON.stringify('production'),
            }),
            commonjs({
                esmExternals: browser,
            }),
            json(), // handle JSON imports
            nodeResolve({
                browser,
                preferBuiltins: !browser,
            }),
            typescript({
                tsconfig: 'tsconfig.json',
            }),
            ...(format === 'umd' ? [ terser() ] : []), // minifies the bundle
        ],
        external: [
            ...Object.keys(packageJson.peerDependencies || {}),
            ...(browser ? [] : Object.keys(packageJson.dependencies || {}))
        ],
    };
};

export default [
    generateConfig('cjs', false),
    generateConfig('cjs', true),
    generateConfig('esm', false),
    generateConfig('esm', true),
    generateConfig('umd', true),
];
