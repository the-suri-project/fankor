import assert from 'assert';
import { FnkBorshWriter } from '../../serializer';
import { FnkBorshReader } from '../../deserializer';
import { U64, U8 } from '../unsigned';
import { FnkBSet } from './bset';

describe('FnkBSet Tests', () => {
    it('test_serialize_deserialize_empty', () => {
        const schema = FnkBSet(U64);

        const data = [];
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        let buffer = writer.buffer.slice(0, writer.length);
        assert(buffer[0] === 0);
        assert(buffer[1] === data.length);
        assert(buffer[2] === 0);
        assert(buffer[3] === 0);
        assert(buffer.length === 4);

        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(
            actual.length === expected.length,
            `Length: ${actual} != ${expected}`
        );
    });

    it('test_serialize_deserialize_data', () => {
        const schema = FnkBSet(U8);

        const data = [1, 2, 3];
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        let buffer = writer.buffer.slice(0, writer.length);
        assert(buffer[0] === data.length);
        assert(buffer[1] === 0);
        assert(buffer[2] === 2);
        assert(buffer[3] === 0);
        assert(buffer[4] === data[0]);
        assert(buffer[5] === 0);
        assert(buffer[6] === 0);
        assert(buffer[7] === 0);
        assert(buffer[8] === 0);
        assert(buffer[9] === 0);
        assert(buffer[10] === data[1]);
        assert(buffer[11] === 1);
        assert(buffer[12] === 0);
        assert(buffer[13] === 3);
        assert(buffer[14] === 0);
        assert(buffer[15] === 1);
        assert(buffer[16] === data[2]);
        assert(buffer[17] === 0);
        assert(buffer[18] === 0);
        assert(buffer[19] === 0);
        assert(buffer[20] === 0);
        assert(buffer[21] === 0);
        assert(buffer.length === 2 + 2 + data.length * (1 + 2 + 2 + 1));

        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(
            actual.length === expected.length,
            `Length: ${actual.length} != ${expected.length}`
        );

        for (let i = 0; i < actual.length; i += 1) {
            assert(
                actual[i] === expected[i],
                `[${i}]: ${actual[i]} != ${expected[i]}`
            );
        }
    });

    it('test_serialize_deserialize_data_with_different_lengths', () => {
        const schema = FnkBSet(U8);

        for (let j = 1; j < 100; j += 1) {
            const data: number[] = [];

            for (let i = 0; i < j; i += 1) {
                data.push(i);
            }

            const writer = new FnkBorshWriter();
            schema.serialize(writer, data);

            let buffer = writer.buffer.slice(0, writer.length);
            assert(buffer.length === 2 + 2 + data.length * (1 + 2 + 2 + 1));

            const reader = new FnkBorshReader(buffer);
            let actual = schema.deserialize(reader);
            let expected = data;
            assert(
                actual.length === expected.length,
                `[${j}]Length: ${actual.length} != ${expected.length}`
            );

            for (let i = 0; i < j; i += 1) {
                assert(
                    actual[i] === expected[i],
                    `[${j}][${i}]: ${actual[i]} != ${expected[i]}`
                );
            }
        }
    });
});
