import assert from 'assert';
import {BinaryReader, BinaryWriter} from 'borsh';
import {FnkString} from './strings';

describe('FnkString Tests', () => {
    it('test_serialize_deserialize', () => {
        for (const text of ['', 'Hello world!']) {
            let fnk_string = new FnkString(text);
            let serializer = new BinaryWriter();
            serializer.writeFnkString(fnk_string);

            let buffer = serializer.buf.slice(0, serializer.length);
            assert(buffer[0] === text.length);
            assert(buffer.slice(1).equals(Buffer.from(text, 'utf8')), `${buffer.slice(1).toString("hex")} != ${Buffer.from(text, 'utf8').toString("hex")}`);
            assert(buffer.length === text.length + 1);

            let deserializer = new BinaryReader(buffer);
            let deserialized = deserializer.readFnkString();

            let actual = deserialized.value;
            let expected = text;
            assert(actual === expected, `${actual} != ${expected}`);
        }
    });
});