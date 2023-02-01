import BN from 'bn.js';
import { numberToBN } from './numbers';
import { Keypair, PublicKey } from '@solana/web3.js';

export function equals(a: any, b: any): boolean {
    if (a === b) {
        return true;
    }

    if (a === null) {
        return b === null;
    }

    if (b === null) {
        return false;
    }

    if (a === undefined) {
        return b === undefined;
    }

    if (b === undefined) {
        return false;
    }

    if (a instanceof BN || typeof a === 'number' || typeof a === 'bigint') {
        if (b instanceof BN || typeof b === 'number' || typeof b === 'bigint') {
            return numberToBN(a).eq(numberToBN(b));
        }

        return false;
    }

    if (b instanceof BN || typeof b === 'number' || typeof b === 'bigint') {
        return false;
    }

    if (a instanceof Uint8Array) {
        if (b instanceof Uint8Array) {
            return a.length === b.length && a.every((v, i) => v === b[i]);
        }

        return false;
    }

    if (b instanceof Uint8Array) {
        return false;
    }

    if (a instanceof Keypair) {
        if (b instanceof Keypair) {
            return (
                a.secretKey.length === b.secretKey.length &&
                a.secretKey.every((v, i) => v === b.secretKey[i])
            );
        }

        return false;
    }

    if (b instanceof Keypair) {
        return false;
    }

    if (a instanceof PublicKey) {
        if (b instanceof PublicKey) {
            return a.equals(b);
        }

        return false;
    }

    if (b instanceof PublicKey) {
        return false;
    }

    if ((a as any).equals) {
        return a.equals(b);
    }

    if (Array.isArray(a) && Array.isArray(b)) {
        return a.length === b.length && a.every((v, i) => equals(v, b[i]));
    }

    if (typeof a === 'object' && typeof b === 'object') {
        const aEntries: [string, any][] = Object.entries(a).sort(([a], [b]) =>
            a.localeCompare(b)
        );
        const bEntries: [string, any][] = Object.entries(b).sort(([a], [b]) =>
            a.localeCompare(b)
        );

        return (
            aEntries.length === bEntries.length &&
            aEntries.every(
                ([k, v], i) => k === bEntries[i][0] && equals(v, bEntries[i][1])
            )
        );
    }

    return false;
}
