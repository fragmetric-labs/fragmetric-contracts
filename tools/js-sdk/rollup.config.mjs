import replace from '@rollup/plugin-replace';
import alias from '@rollup/plugin-alias';
import nodeResolve from '@rollup/plugin-node-resolve';
import json from '@rollup/plugin-json';
import commonjs from '@rollup/plugin-commonjs';
import typescript from 'rollup-plugin-typescript2';
import { terser } from 'rollup-plugin-terser';

import packageJson from './package.json' with { type: 'json' };

const generateConfig = (format, browser = false, generateTypes = false) => {
    if (!['cjs', 'esm', 'umd'].includes(format)) {
        throw "unsupported output format";
    }

    return {
        input: 'src/index.ts',
        output: [
            {
                file: `lib/index${browser ? '.browser' : ''}.${format}.js`,
                format,
                sourcemap: true,
                name: format === 'umd' ? 'fragmetricSDK' : undefined,
                globals: format === 'umd' ? { '@solana/web3.js': 'solanaWeb3' } : undefined,
                exports: format === 'cjs' ? 'named' : undefined,
                interop: 'auto',
            },
        ],
        plugins: [
            replace({
                preventAssignment: true,
                'process.env.NODE_ENV': JSON.stringify('production'),
            }),
            ...(browser ? [
                alias({
                    entries: [
                        {
                            find: './ledger_signer_impl',
                            replacement: './ledger_signer_impl.browser',
                        },
                    ],
                }),
            ] : []),
            commonjs({
                esmExternals: true,
                transformMixedEsModules: true,
            }),
            json(), // handle JSON imports
            nodeResolve({
                browser,
                preferBuiltins: !browser,
            }),
            typescript({
                tsconfig: 'tsconfig.json',
                tsconfigOverride: {
                    compilerOptions: {
                        declaration: generateTypes,
                        declarationMap: generateTypes,
                    },
                },
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
    generateConfig('cjs', false, true),
    ...(process.env.BUILD_NODE_ONLY ? [] : [
        generateConfig('cjs', true),
        generateConfig('esm', false),
        generateConfig('esm', true),
        generateConfig('umd', true),
    ]),
];
