import { AnchorIdl } from '@codama/nodes-from-anchor';
import { TraitOptions } from '@codama/renderers-rust/dist/types/utils';
import { Node, Visitor } from 'codama';
import {
  anchorTransformEventsToAccountsVisitor,
  jitoProgramsVisitor,
} from './visitors';

const codegenConfig: {
  targets: {
    [id: string]: {
      idlFilePath: string;
      javascript?: boolean;
      rust?: boolean;
      rustTraitOptions?: TraitOptions;
      visitors?: Array<(idl: AnchorIdl) => Visitor<Node | null>[]>;
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
  skipHashCheck: typeof process.env.FORCE !== 'undefined',
  watch: typeof process.env.WATCH !== 'undefined',
  targets: {
    'fragmetric-sdk/src/generated/restaking': {
      idlFilePath: '../../target/idl/restaking.json',
      rust: false,
      javascript: true,
      visitors: [anchorTransformEventsToAccountsVisitor],
    },
    'fragmetric-sdk/src/generated/solv': {
      idlFilePath: '../../target/idl/solv.json',
      rust: false,
      javascript: true,
      visitors: [anchorTransformEventsToAccountsVisitor],
    },
    'fragmetric-sdk/src/generated/jito_vault': {
      idlFilePath: './idls/jito-vault.json',
      rust: false,
      javascript: true,
      visitors: [jitoProgramsVisitor],
    },
    'fragmetric-sdk/src/generated/jito_restaking': {
      idlFilePath: './idls/jito-restaking.json',
      rust: false,
      javascript: true,
      visitors: [jitoProgramsVisitor],
    },
  },
};

export default codegenConfig;
