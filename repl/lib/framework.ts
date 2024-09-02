import * as anchor from "@coral-xyz/anchor";
import {Program} from "@coral-xyz/anchor";
import { DepositProgram } from "@target/types/deposit_program";

export class Framework {
    public readonly provider = anchor.AnchorProvider.env();
    public get wallet() { return this.provider.wallet };
    public get connection() { return this.provider.connection };

    public readonly restaking: Program<DepositProgram> = anchor.workspace.Restaking;

    constructor() {
        anchor.setProvider(this.provider);
    }
}
