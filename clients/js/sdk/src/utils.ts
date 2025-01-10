import * as anchor from '@coral-xyz/anchor';
import * as web3 from '@solana/web3.js';
import BN from 'bn.js';
import chalk from 'chalk';

const nodeCustomInspectSymbol = Symbol.for("nodejs.util.inspect.custom");

const web3PublicKeyExtensionEnabled = typeof (web3.PublicKey.prototype as any).extensionDisabled === 'undefined';

if (web3PublicKeyExtensionEnabled) {
    (web3.PublicKey.prototype as any)[nodeCustomInspectSymbol] = (anchor.web3.PublicKey.prototype as any)[nodeCustomInspectSymbol] = function () {
        return chalk.blue(this.toString());
    }
}

const bnExtensionEnabled = typeof (BN.prototype as any).extensionDisabled === 'undefined';

if (bnExtensionEnabled) {
    BN.prototype.toJSON = (BN.prototype as any)[nodeCustomInspectSymbol] = function() {
        return chalk.yellow(this.toString());
    };

    // eg. (100).bn => new BN(100)
    Object.defineProperty(Number.prototype, 'bn', {
        get: function toBN() {
            return new BN(this);
        },
    });
}

export const BN_U64_MAX = new BN("18446744073709551615");

export { BN };
