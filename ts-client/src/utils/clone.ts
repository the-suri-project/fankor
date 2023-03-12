import { Keypair, PublicKey } from '@solana/web3.js';

export function clone(v: any): any {
    switch (typeof v) {
        case 'object':
            break;
        case 'undefined':
        case 'boolean':
        case 'number':
        case 'string':
        case 'symbol':
        case 'bigint':
        case 'function':
            return v;
    }

    if (v === null) {
        return null;
    }

    if (v instanceof Uint8Array) {
        return v.slice();
    }

    if (v instanceof Keypair || v instanceof PublicKey) {
        return v;
    }

    if ((v as any).clone) {
        return v.clone();
    }

    if (Array.isArray(v)) {
        return v.map(clone);
    }

    const result: any = {};
    for (const [k, value] of Object.entries(v)) {
        result[k] = clone(value);
    }

    return result;
}
