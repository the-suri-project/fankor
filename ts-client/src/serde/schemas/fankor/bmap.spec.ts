import assert from 'assert';
import { FnkBorshWriter } from '../../serializer';
import { FnkBorshReader } from '../../deserializer';
import { U64, U8 } from '../unsigned';
import { TPublicKey } from '../public_keys';
import { FnkBMap } from './bmap';

describe('FnkBMap Tests', () => {
    it('test_serialize_deserialize_empty', () => {
        const schema = FnkBMap({
            keySchema: TPublicKey,
            valueSchema: U64,
        });

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
        const schema = FnkBMap({
            keySchema: U8,
            valueSchema: U8,
        });

        const data = [
            {
                key: 1,
                value: 2,
            },
            {
                key: 3,
                value: 4,
            },
            {
                key: 5,
                value: 6,
            },
        ];
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        let buffer = writer.buffer.slice(0, writer.length);
        assert(buffer[0] === data.length);
        assert(buffer[1] === 0);
        assert(buffer[2] === 2);
        assert(buffer[3] === 0);
        assert(buffer[4] === data[0].key);
        assert(buffer[5] === data[0].value);
        assert(buffer[6] === 0);
        assert(buffer[7] === 0);
        assert(buffer[8] === 0);
        assert(buffer[9] === 0);
        assert(buffer[10] === 0);
        assert(buffer[11] === data[1].key);
        assert(buffer[12] === data[1].value);
        assert(buffer[13] === 1);
        assert(buffer[14] === 0);
        assert(buffer[15] === 3);
        assert(buffer[16] === 0);
        assert(buffer[17] === 1);
        assert(buffer[18] === data[2].key);
        assert(buffer[19] === data[2].value);
        assert(buffer[20] === 0);
        assert(buffer[21] === 0);
        assert(buffer[22] === 0);
        assert(buffer[23] === 0);
        assert(buffer[24] === 0);
        assert(buffer.length === 2 + 2 + data.length * (1 + 1 + 2 + 2 + 1));

        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(
            actual.length === expected.length,
            `Length: ${actual.length} != ${expected.length}`
        );

        for (let i = 0; i < actual.length; i += 1) {
            assert(
                actual[i].key === expected[i].key,
                `[${i}].key: ${actual[i].key} != ${expected[i].key}`
            );
            assert(
                actual[i].value === expected[i].value,
                `[${i}].value: ${actual[i].value} != ${expected[i].value}`
            );
        }
    });

    it('test_serialize_deserialize_data_with_different_lengths', () => {
        const schema = FnkBMap({
            keySchema: U8,
            valueSchema: U8,
        });

        for (let j = 1; j < 100; j += 1) {
            const data: { key: number; value: number }[] = [];

            for (let i = 0; i < j; i += 1) {
                data.push({
                    key: i,
                    value: i,
                });
            }

            const writer = new FnkBorshWriter();
            schema.serialize(writer, data);

            let buffer = writer.buffer.slice(0, writer.length);
            assert(buffer.length === 2 + 2 + data.length * (1 + 1 + 2 + 2 + 1));

            const reader = new FnkBorshReader(buffer);
            let actual = schema.deserialize(reader);
            let expected = data;
            assert(
                actual.length === expected.length,
                `[${j}]Length: ${actual.length} != ${expected.length}`
            );

            for (let i = 0; i < j; i += 1) {
                assert(
                    actual[i].key === expected[i].key,
                    `[${j}][${i}].key: ${actual[i].key} != ${expected[i].key}`
                );
                assert(
                    actual[i].value === expected[i].value,
                    `[${j}][${i}].value: ${actual[i].value} != ${expected[i].value}`
                );
            }
        }
    });
});
