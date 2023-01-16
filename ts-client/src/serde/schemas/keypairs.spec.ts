import assert from 'assert';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshReader } from '../deserializer';
import { Keypair } from '@solana/web3.js';
import { TKeypair } from './keypairs';

describe('Keypairs Tests', () => {
    it('test_serialize_deserialize', () => {
        const data = Keypair.generate();
        const schema = TKeypair;
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        const buffer = writer.buffer.slice(0, writer.length);
        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(
            actual.publicKey.equals(expected.publicKey),
            `Pk: ${actual.publicKey} != ${expected.publicKey}`
        );
        assert(
            actual.secretKey.length == expected.secretKey.length,
            `Sk length: ${actual.secretKey.length} != ${expected.secretKey.length}`
        );

        for (let i = 0; i < actual.secretKey.length; i++) {
            assert(
                actual.secretKey[i] == expected.secretKey[i],
                `Sk[${i}]: ${actual.secretKey[i]} != ${expected.secretKey[i]}`
            );
        }
    });
});
