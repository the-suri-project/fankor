import assert from 'assert';
import {FnkBorshWriter} from '../../serializer';
import {FnkBorshReader} from '../../deserializer';
import {FnkVec} from './vectors';
import {TString} from '../strings';
import {U8} from '../unsigned';

describe('FnkVec Tests', () => {
    it('test_serialize_deserialize_empty', () => {
        const schema = FnkVec(TString);
        const data = [];
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        let buffer = writer.buffer.slice(0, writer.length);
        assert(buffer[0] === data.length);
        assert(buffer.length === 1);

        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(actual.length === expected.length, `Length: ${actual} != ${expected}`);
    });

    it('test_serialize_deserialize_bytes', () => {
        const schema = FnkVec(U8);
        const data = [0, 1, 2, 3];
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        let buffer = writer.buffer.slice(0, writer.length);
        assert(buffer[0] === data.length);
        assert(buffer[1] === data[0]);
        assert(buffer[2] === data[1]);
        assert(buffer[3] === data[2]);
        assert(buffer[4] === data[3]);
        assert(buffer.length === data.length + 1);

        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(actual.length === expected.length, `Length: ${actual.length} != ${expected.length}`);
        assert(actual[0] === expected[0], `[0]: ${actual[0]} != ${expected[0]}`);
        assert(actual[1] === expected[1], `[1]: ${actual[1]} != ${expected[1]}`);
        assert(actual[2] === expected[2], `[2]: ${actual[2]} != ${expected[2]}`);
        assert(actual[3] === expected[3], `[3]: ${actual[3]} != ${expected[3]}`);
    });

    it('test_serialize_deserialize_data', () => {
        const schema = FnkVec(TString);
        const data = ['a', 'b'];
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        let buffer = writer.buffer.slice(0, writer.length);
        assert(buffer[0] === data.length);
        assert(buffer[1] === 1);
        assert(buffer[2] === 0);
        assert(buffer[3] === 0);
        assert(buffer[4] === 0);
        assert(buffer[5] === 'a'.charCodeAt(0));
        assert(buffer[6] === 1);
        assert(buffer[7] === 0);
        assert(buffer[8] === 0);
        assert(buffer[9] === 0);
        assert(buffer[10] === 'b'.charCodeAt(0));
        assert(buffer.length === data.reduce((acc, s) => acc + s.length + 4, 1));

        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(actual.length === expected.length, `Length: ${actual.length} != ${expected.length}`);
        assert(actual[0] === expected[0], `[0]: ${actual[0]} != ${expected[0]}`);
        assert(actual[1] === expected[1], `[1]: ${actual[1]} != ${expected[1]}`);
    });
});