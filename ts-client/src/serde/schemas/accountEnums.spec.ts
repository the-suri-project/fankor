import assert from 'assert';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshReader } from '../deserializer';
import { U8 } from './unsigned';
import { TEnum } from './enums';

describe('Account Enums Tests', () => {
    it('test_serialize_deserialize_data', () => {
        const schema = TEnum([
            [0, 'A', U8],
            [1, 'B', U8],
        ] as const);

        {
            const data = {
                type: 'A',
                value: 0,
            };
            const writer = new FnkBorshWriter();
            schema.serialize(writer, data);

            const buffer = writer.buffer.slice(0, writer.length);
            assert(buffer[0] === 0);

            const reader = new FnkBorshReader(buffer);
            let actual = schema.deserialize(reader);
            let expected = data;
            assert(
                actual.type === expected.type,
                `type: ${actual.type} != ${expected.type}`
            );
            assert(
                actual.value === expected.value,
                `value: ${actual.value} != ${expected.value}`
            );
        }

        {
            const data = {
                type: 'B',
                value: 1,
            };
            const writer = new FnkBorshWriter();
            schema.serialize(writer, data);

            const buffer = writer.buffer.slice(0, writer.length);
            assert(buffer[0] === 1);

            const reader = new FnkBorshReader(buffer);
            let actual = schema.deserialize(reader);
            let expected = data;
            assert(
                actual.type === expected.type,
                `type: ${actual.type} != ${expected.type}`
            );
            assert(
                actual.value === expected.value,
                `value: ${actual.value} != ${expected.value}`
            );
        }
    });
});
