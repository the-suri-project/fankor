import assert from 'assert';
import { TStruct } from './structs';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshReader } from '../deserializer';
import { TString } from './strings';
import { U8 } from './unsigned';
import { Bool } from './bools';
import { TEnum } from './enums';
import { FnkMap } from './fankor';
import { Vec } from './vectors';

describe('Structs Tests', () => {
    it('test_serialize_deserialize_empty', () => {
        const data = {};
        const schema = TStruct([] as const);
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        let buffer = writer.buffer.slice(0, writer.length);
        assert(buffer.length === 0);

        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(
            Object.keys(actual).length === Object.keys(expected).length,
            `Keys: ${Object.keys(actual).length} != ${
                Object.keys(expected).length
            }`
        );
    });

    it('test_serialize_deserialize_data', () => {
        const data = {
            a: 'x',
            b: 1,
            c: true,
        };
        const schema = TStruct([
            ['a', TString],
            ['b', U8],
            ['c', Bool],
        ] as const);
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        const buffer = writer.buffer.slice(0, writer.length);
        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(
            Object.keys(actual).length === Object.keys(expected).length,
            `Keys: ${Object.keys(actual).length} != ${
                Object.keys(expected).length
            }`
        );
        assert(actual.a === expected.a, `a: ${actual.a} != ${expected.a}`);
        assert(actual.b === expected.b, `b: ${actual.b} != ${expected.b}`);
        assert(actual.c === expected.c, `c: ${actual.c} != ${expected.c}`);
    });

    it('test_serialize_deserialize_nested', () => {
        const data = {
            a: 'x',
            b: {
                c: 1,
            },
            d: {
                type: 'A',
                value: 2,
            },
        };
        const schema = TStruct([
            ['a', TString],
            ['b', TStruct([['c', U8]] as const)],
            [
                'd',
                TEnum([
                    [0, 'A', U8],
                    [1, 'B', TString],
                ] as const),
            ],
        ] as const);
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        const buffer = writer.buffer.slice(0, writer.length);
        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(
            Object.keys(actual).length === Object.keys(expected).length,
            `Keys: ${Object.keys(actual).length} != ${
                Object.keys(expected).length
            }`
        );
        assert(actual.a === expected.a, `a: ${actual.a} != ${expected.a}`);
        assert(
            actual.b.c === expected.b.c,
            `b.c: ${actual.b.c} != ${expected.b.c}`
        );
        assert(
            actual.d.type === expected.d.type,
            `d.type: ${actual.d.type} != ${expected.d.type}`
        );
        assert(
            actual.d.value === expected.d.value,
            `d.value: ${actual.d.value} != ${expected.d.value}`
        );
    });

    it('test_serialize_deserialize_nested_map', () => {
        const data = {
            a: 'x',
            b: {
                c: 1,
            },
            d: [
                {
                    key: 2,
                    value: 'x',
                },
            ],
            e: [3],
        };
        const schema = TStruct([
            ['a', TString],
            ['b', TStruct([['c', U8]] as const)],
            [
                'd',
                FnkMap({
                    keySchema: U8,
                    valueSchema: TString,
                }),
            ],
            ['e', Vec(U8)],
        ] as const);
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        const buffer = writer.buffer.slice(0, writer.length);
        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(
            Object.keys(actual).length === Object.keys(expected).length,
            `Keys: ${Object.keys(actual).length} != ${
                Object.keys(expected).length
            }`
        );
        assert(actual.a === expected.a, `a: ${actual.a} != ${expected.a}`);
        assert(
            actual.b.c === expected.b.c,
            `b.c: ${actual.b.c} != ${expected.b.c}`
        );
        assert(
            actual.d.length === expected.d.length,
            `d.length: ${actual.d.length} != ${expected.d.length}`
        );

        let actual0 = actual.d[0];
        let expected0 = expected.d[0];
        assert(
            actual0.value === expected0.value,
            `d[0].value: ${actual0.value} != ${expected0.value}`
        );
    });
});
