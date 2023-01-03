import assert from 'assert';
import { FnkString } from './strings';
import { FnkBorshWriter } from '../../serializer';
import { FnkBorshReader } from '../../deserializer';

describe('FnkString Tests', () => {
    it('test_serialize_deserialize', () => {
        const schema = FnkString;

        for (const text of ['', 'Hello world!']) {
            const writer = new FnkBorshWriter();
            schema.serialize(writer, text);

            let buffer = writer.buffer.slice(0, writer.length);
            assert(buffer[0] === text.length);
            assert(
                buffer.slice(1).equals(Buffer.from(text, 'utf8')),
                `${buffer.slice(1).toString('hex')} != ${Buffer.from(
                    text,
                    'utf8'
                ).toString('hex')}`
            );
            assert(buffer.length === text.length + 1);

            const reader = new FnkBorshReader(buffer);
            let actual = schema.deserialize(reader);
            let expected = text;
            assert(actual === expected, `${actual} != ${expected}`);
        }
    });
});
