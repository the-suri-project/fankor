import { PublicKey } from '@solana/web3.js';

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

    if (v.map) {
        return v.map(clone);
    }

    if (v instanceof PublicKey || isPublicKey(v)) {
        return v;
    }

    if ((v as any).clone) {
        return v.clone();
    }

    const result: any = {};
    for (const [k, value] of Object.entries(v)) {
        result[k] = clone(value);
    }

    return result;
}

function isPublicKey(v: any): v is PublicKey {
    if (v instanceof PublicKey) {
        return true;
    }

    const prototype: any = Object.getPrototypeOf(v);

    return (
        typeof prototype?.constructor === 'function' &&
        typeof prototype?.toBase58 === 'function' &&
        typeof prototype?.toBuffer === 'function' &&
        typeof prototype?.equals === 'function' &&
        typeof prototype?.toBytes === 'function' &&
        typeof prototype?.toJSON === 'function' &&
        typeof prototype?.constructor?.createProgramAddress === 'function' &&
        typeof prototype?.constructor?.createProgramAddressSync ===
            'function' &&
        typeof prototype?.constructor?.findProgramAddress === 'function' &&
        typeof prototype?.constructor?.findProgramAddressSync === 'function'
    );
}
