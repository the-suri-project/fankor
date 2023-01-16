import assert from 'assert';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshReader } from '../deserializer';
import { Keypair } from '@solana/web3.js';
import { TPublicKey } from './public_keys';

describe('PublicKeys Tests', () => {
    it('test_serialize_deserialize', () => {
        const keypair = Keypair.generate();
        const data = keypair.publicKey;
        const schema = TPublicKey;
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        const buffer = writer.buffer.slice(0, writer.length);
        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(actual.equals(expected), `${actual} != ${expected}`);
    });
});
