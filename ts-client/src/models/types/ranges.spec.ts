import assert from 'assert';
import {BinaryReader, BinaryWriter} from 'borsh';
import {FnkRange, FnkURange} from './ranges';
import BN from 'bn.js';

describe('FnkInt Tests', () => {
    it('test_serialize_unsigned_range_full', () => {
        let fnk_range = FnkURange.newUnbounded(5);
        let serializer = new BinaryWriter();
        serializer.writeFnkURange(fnk_range);

        let buffer = serializer.buf.slice(0, serializer.length);
        assert(buffer[0] === 5);
        assert(buffer[1] === 0);

        let deserializer = new BinaryReader(buffer);
        let actual = deserializer.readFnkURange();
        let expected = fnk_range;
        assert(actual.from.value.eq(expected.from.value), `${actual.from.value} != ${expected.from.value}`);
        assert(actual.to.value.eq(expected.to.value), `${actual.to.value} != ${expected.to.value}`);
    });

    it('test_serialize_unsigned_range_positive', () => {
        let fnk_range = FnkURange.fromNumbers(5, 10);
        let serializer = new BinaryWriter();
        serializer.writeFnkURange(fnk_range);

        let buffer = serializer.buf.slice(0, serializer.length);
        assert(buffer[0] === 5);
        assert(buffer[1] === 6);

        let deserializer = new BinaryReader(buffer);
        let actual = deserializer.readFnkURange();
        let expected = fnk_range;
        assert(actual.from.value.eq(expected.from.value), `${actual.from.value} != ${expected.from.value}`);
        assert(actual.to.value.eq(expected.to.value), `${actual.to.value} != ${expected.to.value}`);
    });

    it('test_serialize_unsigned_range_negative', () => {
        let fnk_range = FnkURange.fromNumbers(0, new BN(2).pow(new BN(64)).subn(1).subn(5));
        let serializer = new BinaryWriter();
        serializer.writeFnkURange(fnk_range);

        let buffer = serializer.buf.slice(0, serializer.length);
        assert(buffer[0] === 0);
        assert(buffer[1] === (5 | 0x20));

        let deserializer = new BinaryReader(buffer);
        let actual = deserializer.readFnkURange();
        let expected = fnk_range;
        assert(actual.from.value.eq(expected.from.value), `${actual.from.value} != ${expected.from.value}`);
        assert(actual.to.value.eq(expected.to.value), `${actual.to.value} != ${expected.to.value}`);
    });

    it('test_serialize_unsigned_range_negative', () => {
        for (let i of [new BN(0), new BN(1), new BN(2), new BN(2).pow(new BN(64)).subn(1).divn(3),
            new BN(2).pow(new BN(64)).subn(1).divn(2).subn(1), new BN(2).pow(new BN(64)).subn(1).divn(2),
            new BN(2).pow(new BN(64)).subn(1).divn(2).addn(1), new BN(2).pow(new BN(64)).subn(1).subn(2),
            new BN(2).pow(new BN(64)).subn(1).subn(1), new BN(2).pow(new BN(64)).subn(1)]) {
            let fnk_range = FnkURange.fromNumbers(0, i);
            let serializer = new BinaryWriter();
            serializer.writeFnkURange(fnk_range);

            let buffer = serializer.buf.slice(0, serializer.length);
            let deserializer = new BinaryReader(buffer);
            let actual = deserializer.readFnkURange();
            let expected = fnk_range;
            assert(actual.from.value.eq(expected.from.value), `${actual.from.value} != ${expected.from.value}`);
            assert(actual.to.value.eq(expected.to.value), `${actual.to.value} != ${expected.to.value}`);
        }
    });

    it('test_serialize_signed_range', () => {
        for (let i of [new BN(2).pow(new BN(63)).neg(), new BN(2).pow(new BN(63)).neg().addn(1),
            new BN(2).pow(new BN(63)).neg().addn(2), new BN(2).pow(new BN(63)).neg().divn(2).subn(1),
            new BN(2).pow(new BN(63)).neg().divn(2), new BN(2).pow(new BN(63)).neg().divn(2).addn(1),
            new BN(2).pow(new BN(63)).neg().divn(3), new BN(-2), new BN(-1), new BN(0)]) {
            let fnk_range = FnkRange.fromNumbers(i, 0);
            let serializer = new BinaryWriter();
            serializer.writeFnkRange(fnk_range);

            let buffer = serializer.buf.slice(0, serializer.length);
            let deserializer = new BinaryReader(buffer);
            let actual = deserializer.readFnkRange();
            let expected = fnk_range;
            assert(actual.from.value.eq(expected.from.value), `${actual.from.value} != ${expected.from.value}`);
            assert(actual.to.value.eq(expected.to.value), `${actual.to.value} != ${expected.to.value}`);
        }

        for (let i of [new BN(0), new BN(1), new BN(2), new BN(2).pow(new BN(63)).subn(1).divn(3),
            new BN(2).pow(new BN(63)).subn(1).divn(2).subn(1), new BN(2).pow(new BN(63)).subn(1).divn(2),
            new BN(2).pow(new BN(63)).subn(1).divn(2).addn(1), new BN(2).pow(new BN(63)).subn(1).subn(2),
            new BN(2).pow(new BN(63)).subn(1).subn(1), new BN(2).pow(new BN(63)).subn(1)]) {
            let fnk_range = FnkRange.fromNumbers(0, i);
            let serializer = new BinaryWriter();
            serializer.writeFnkRange(fnk_range);

            let buffer = serializer.buf.slice(0, serializer.length);
            let deserializer = new BinaryReader(buffer);
            let actual = deserializer.readFnkRange();
            let expected = fnk_range;
            assert(actual.from.value.eq(expected.from.value), `${actual.from.value} != ${expected.from.value}`);
            assert(actual.to.value.eq(expected.to.value), `${actual.to.value} != ${expected.to.value}`);
        }
    });
    it('test_serialize_signed_range_full', () => {
        let fnk_range = FnkRange.fromNumbers(new BN(2).pow(new BN(63)).neg(), new BN(2).pow(new BN(63)).subn(1));
        let serializer = new BinaryWriter();
        serializer.writeFnkRange(fnk_range);

        let buffer = serializer.buf.slice(0, serializer.length);
        let deserializer = new BinaryReader(buffer);
        let actual = deserializer.readFnkRange();
        let expected = fnk_range;
        assert(actual.from.value.eq(expected.from.value), `${actual.from.value} != ${expected.from.value}`);
        assert(actual.to.value.eq(expected.to.value), `${actual.to.value} != ${expected.to.value}`);
    });

});