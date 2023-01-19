import assert from 'assert';
import { equals } from './equality';
import { Keypair } from '@solana/web3.js';
import BN from 'bn.js';

describe('Equality Tests', () => {
    it('basics', () => {
        {
            const a = null;
            const b = null;
            assert(equals(a, b), '1');
        }

        {
            const a = true;
            const b = true;
            assert(equals(a, b), '2');
        }

        {
            const a = 'xyz';
            const b = 'xyz';
            assert(equals(a, b), '3');
        }

        let data = [1, BigInt(1), new BN(1)];
        for (let a of data) {
            for (let b of data) {
                assert(equals(a, b), '4');
            }
        }
    });

    it('arrays', () => {
        {
            const a = [1, 2, 3];
            const b = [1, 2, 3];
            assert(equals(a, b), '1');
        }

        {
            const a = [1, true, 3];
            const b = [1, 2, 3];
            assert(!equals(a, b), '2');
        }
    });

    it('objects', () => {
        let aKeypair = Keypair.generate();
        let bKeypair = Keypair.generate();

        {
            const a = aKeypair;
            const b = bKeypair;
            assert(equals(a, a), '1.a');
            assert(equals(b, b), '1.b');
            assert(!equals(a, b), '1');
        }

        {
            const a = aKeypair.publicKey;
            const b = bKeypair.publicKey;
            assert(equals(a, a), '2.a');
            assert(equals(b, b), '2.b');
            assert(!equals(a, b), '2');
        }

        {
            const a = aKeypair.secretKey;
            const b = bKeypair.secretKey;
            assert(equals(a, a), '3.a');
            assert(equals(b, b), '3.b');
            assert(!equals(a, b), '3');
        }

        {
            const a = {
                type: 'a',
                value: null,
            };
            const b = {
                type: 'b',
                value: 4,
            };
            assert(equals(a, a), '3.a');
            assert(equals(b, b), '3.b');
            assert(!equals(a, b), '3');
        }
    });

    it('nested objects', () => {
        {
            const a = {
                type: 'a',
                value: {
                    value: null,
                },
            };
            const b = {
                type: 'b',
                value: {
                    type: 'c',
                    value: null,
                },
            };
            assert(equals(a, a), '1.a');
            assert(equals(b, b), '1.b');
            assert(!equals(a, b), '1');
        }
    });
});
