import assert from 'assert';
import { Struct } from './structs';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshReader } from '../deserializer';
import { TString } from './strings';
import { U8 } from './unsigned';
import { Bool } from './bools';

describe('Structs Tests', () => {
    it('test_serialize_deserialize_empty', () => {
        const data = {};
        const schema = Struct([] as const);
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        let buffer = writer.buffer.slice(0, writer.length);
        assert(buffer.length === 0);

        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(Object.keys(actual).length === Object.keys(expected).length,
            `Keys: ${Object.keys(actual).length} != ${Object.keys(expected).length}`);
    });

    it('test_serialize_deserialize_data', () => {
        const data = {
            a: 'x',
            b: 1,
            c: true
        };
        const schema = Struct([['a', TString], ['b', U8], ['c', Bool]] as const);
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        const buffer = writer.buffer.slice(0, writer.length);
        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(Object.keys(actual).length === Object.keys(expected).length,
            `Keys: ${Object.keys(actual).length} != ${Object.keys(expected).length}`);
        assert(actual.a === expected.a, `a: ${actual.a} != ${expected.a}`);
        assert(actual.b === expected.b, `b: ${actual.b} != ${expected.b}`);
        assert(actual.c === expected.c, `c: ${actual.c} != ${expected.c}`);
    });

    it('test_serialize_deserialize_nested', () => {
        const data = {
            a: 'x',
            b: {
                c: 1
            },
            d: true
        };
        const schema = Struct([['a', TString], ['b', Struct([['c', U8]] as const)], ['d', Bool]] as const);
        const writer = new FnkBorshWriter();
        schema.serialize(writer, data);

        const buffer = writer.buffer.slice(0, writer.length);
        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(Object.keys(actual).length === Object.keys(expected).length,
            `Keys: ${Object.keys(actual).length} != ${Object.keys(expected).length}`);
        assert(actual.a === expected.a, `a: ${actual.a} != ${expected.a}`);
        assert(actual.b.c === expected.b.c, `b.c: ${actual.b.c} != ${expected.b.c}`);
        assert(actual.d === expected.d, `d: ${actual.d} != ${expected.d}`);
    });
});