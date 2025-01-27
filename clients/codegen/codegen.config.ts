export const codegenConfig: {
  targets: {
    [id: string]: {
      idlFilePath: string;
      javascript?: boolean;
      rust?: boolean;
    };
  };
  outputBaseDir: {
    javascript: string;
    rust: string;
  };
  skipHashCheck?: boolean;
  watch?: boolean;
} = {
  outputBaseDir: {
    javascript: '../js',
    rust: '../rust',
  },
  skipHashCheck: typeof process.env.ALL !== 'undefined',
  watch: typeof process.env.WATCH !== 'undefined',
  targets: {
    'fragmetric-restaking': {
      idlFilePath: typeof process.env.LOCAL !== 'undefined' ? '../../target/idl/restaking.json' : './idls/fragmetric-restaking.json',
      rust: true,
      // JS SDK IS NOT USING THIS YET, STILL STICK TO @solana/web3.js@1
      javascript: false,
    },
    'jito-restaking': {
      idlFilePath: './idls/jito-restaking.json',
      rust: false,
    },
    'jito-vault': {
      idlFilePath: './idls/jito-vault.json',
      rust: false,
    },
    'marinade': {
      idlFilePath: './idls/marinade.json',
    },
    'whirlpool': {
      idlFilePath: './idls/whirlpool.json',
    },
  },
};

export default codegenConfig;