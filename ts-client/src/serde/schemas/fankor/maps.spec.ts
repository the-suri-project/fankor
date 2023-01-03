import assert from 'assert';
import {FnkBorshWriter} from '../../serializer';
import {FnkBorshReader} from '../../deserializer';
import {FnkMap} from './maps';
import {TString} from '../strings';
import {U8} from '../unsigned';

describe('FnkMap Tests', () => {
    it('test_serialize_deserialize_empty', () => {
        const schema = FnkMap({
            keySchema: TString,
            valueSchema: U8,
        });

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

    it('test_serialize_deserialize_data', () => {
        const schema = FnkMap({
            keySchema: TString,
            valueSchema: TString,
        });

        const data = [{
            key: 'a',
            value: 'b',
        }];
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
        assert(buffer.length === data.reduce((acc, {
            key,
            value,
        }) => acc + key.length + value.length + 8, 1));

        const reader = new FnkBorshReader(buffer);
        let actual = schema.deserialize(reader);
        let expected = data;
        assert(actual.length === expected.length, `Length: ${actual.length} != ${expected.length}`);
        assert(actual[0].key === expected[0].key, `[0].key: ${actual[0].key} != ${expected[0].key}`);
        assert(actual[0].value === expected[0].value, `[0].value: ${actual[0].value} != ${expected[0].value}`);
    });
});