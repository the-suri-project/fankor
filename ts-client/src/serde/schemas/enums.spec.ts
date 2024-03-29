import assert from 'assert';
import { TStruct } from './structs';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshReader } from '../deserializer';
import { TString } from './strings';
import { U8 } from './unsigned';
import { TEnum } from './enums';
import { Bool } from './bools';

describe('Enums Tests', () => {
    it('test_serialize_deserialize_data', () => {
        const schema = TEnum([
            [0, 'A', TString],
            [1, 'B', U8],
        ] as const);

        {
            const data = {
                type: 'A',
                value: 'x',
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
                value: 2,
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

    it('test_serialize_deserialize_data_no_value', () => {
        const schema = TEnum([
            [0, 'A'],
            [1, 'B'],
        ] as const);

        {
            const data = {
                type: 'A',
                value: 'x',
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
                (actual as any).value === undefined,
                `value: ${(actual as any).value} != undefined`
            );
        }
    });

    it('test_serialize_deserialize_real', () => {
        const schemaA = TStruct([['value', U8]] as const);
        const schemaB = TStruct([['value', Bool]] as const);
        const schema = TEnum([
            [0, 'A', schemaA],
            [1, 'B', schemaB],
        ] as const);

        {
            const data = {
                type: 'A',
                value: {
                    value: 2,
                },
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
                actual.value.value === expected.value.value,
                `value.value: ${actual.value.value} != ${expected.value.value}`
            );
        }

        {
            const data = {
                type: 'B',
                value: {
                    value: true,
                },
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
                actual.value.value === expected.value.value,
                `value.value: ${actual.value.value} != ${expected.value.value}`
            );
        }
    });
});
