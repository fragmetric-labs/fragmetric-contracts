import resolve from '@rollup/plugin-node-resolve';
import typescript from 'rollup-plugin-typescript2';
import html from '@rollup/plugin-html';

export default {
    input: 'src/index.ts',
    output: {
        file: 'dist/bundle.js',
        format: 'iife', // Immediately Invoked Function Expression for browser
        name: 'example',
        sourcemap: true,
        globals: {
            '@fragmetric-labs/sdk': 'fragmetric',
        },
    },
    plugins: [
        resolve({ browser: true }),
        typescript( { tsconfig: 'tsconfig.json' }),
        html({
            template: () => `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Fragmetric SDK Example</title>
  <script src="bundle.js"></script>
</head>
<body>
  <h1>Fragmetric SDK Example</h1>
</body>
</html>
      `,
        }),
    ],
};
