import * as web3 from "@solana/web3.js";
import {ProgramTransactionSigner} from "./program_transaction";
import {Program} from "./program";

export class ProgramAddressBook<NAME extends string> {
    public readonly entries = new Map<NAME, web3.PublicKey>();
    private readonly program: Program<any>;

    constructor({ program }: { program: Program<any> }) {
        this.program = program;
    }

    public add(name: NAME, address: web3.PublicKey) {
        this.entries.set(name, address);
    }

    public addAll(addresses: Partial<{[name in NAME]: web3.PublicKey}>) {
        for (let [name, address] of Object.entries(addresses)) {
            this.entries.set(name as NAME, address as web3.PublicKey);
        }
    }

    public get(name: NAME): web3.PublicKey|null {
        return this.entries.get(name) ?? null;
    }

    public async sendLookupTableSyncTransactions({ lookupTableAddress = null, payer, authority, signer = this.program.transactionHandler?.signer ?? null }: {
        lookupTableAddress: web3.PublicKey|null,
        payer: web3.PublicKey,
        authority: web3.PublicKey,
        signer?: ProgramTransactionSigner<'payer'|'authority'> | null,
    }) {
        const exists = lookupTableAddress ? await this.program.connection.getAccountInfo(lookupTableAddress).then(() => true).catch(() => false) : false;
        if (!exists) {
            const recentSlot = await this.program.connection.getSlot({commitment: 'recent'});
            const [createIx, newLookupTableAddress] = web3.AddressLookupTableProgram.createLookupTable({
                authority,
                payer,
                recentSlot,
            });
            await this.program.createTransactionMessage({
                descriptions: [`create a new address lookup table`, { newLookupTableAddress }],
                instructions: [createIx],
                signers: { payer, authority },
            })
                .send({ signer });
            lookupTableAddress = newLookupTableAddress;
        }

        let lookupTable = await this.program.connection
            .getAddressLookupTable(lookupTableAddress!, { commitment: 'confirmed' })
            .then(res => res.value);

        const existingAddresses = new Set(lookupTable?.state.addresses.map(a => a.toString()) ?? []);
        const newAddresses = Array.from(this.entries.values()).filter(address => !existingAddresses.has(address.toString()));
        const listOfNewAddresses = newAddresses.reduce((listOfNewAddresses, address) =>  {
            if (!listOfNewAddresses[0] || listOfNewAddresses[0].length == 27) { // 27 (addresses) + 5 (admin/authority, payer, alt_program, alt, system_program)
                listOfNewAddresses.unshift([address]);
            } else {
                listOfNewAddresses[0].push(address);
            }
            return listOfNewAddresses;
        }, [] as web3.PublicKey[][]);

        if (listOfNewAddresses.length > 0) {
            let size = existingAddresses.size;
            for (let addresses of listOfNewAddresses) {
                await this.program.createTransactionMessage({
                    descriptions: [`extend the address lookup table (${size} + ${addresses.length})`, { addresses }],
                    instructions: [
                        web3.AddressLookupTableProgram.extendLookupTable({
                            lookupTable: lookupTableAddress!,
                            authority,
                            payer,
                            addresses,
                        }),
                    ],
                    signers: { payer, authority },
                })
                    .send({ signer });
                size += addresses.length;
            }

            lookupTable = await this.program.connection
                .getAddressLookupTable(lookupTableAddress!, { commitment: 'confirmed' })
                .then(res => res.value!);
        }

        return lookupTable!;
    }
}
