import assert from 'assert';
import { FnkRange, FnkRangeSchema, FnkURange, FnkURangeSchema } from './ranges';
import BN from 'bn.js';
import { FnkBorshWriter } from '../../serializer';
import { FnkBorshReader } from '../../deserializer';

describe('FnkRange and FnkURange Tests', () => {
    let uIntSchema = new FnkURangeSchema();
    let intSchema = new FnkRangeSchema();

    it('test_serialize_unsigned_range_full', () => {
        let fnk_range = FnkURange.newUnbounded(5);
        const writer = new FnkBorshWriter();
        uIntSchema.serialize(writer, fnk_range);

        let buffer = writer.buffer.slice(0, writer.length);
        assert(buffer[0] === 5);
        assert(buffer[1] === 0);

        const reader = new FnkBorshReader(buffer);
        let actual = uIntSchema.deserialize(reader);
        let expected = fnk_range;
        assert(
            actual.from.eq(expected.from),
            `${actual.from} != ${expected.from}`
        );
        assert(actual.to.eq(expected.to), `${actual.to} != ${expected.to}`);
    });

    it('test_serialize_unsigned_range_positive', () => {
        let fnk_range = new FnkURange(5, 10);
        const writer = new FnkBorshWriter();
        uIntSchema.serialize(writer, fnk_range);

        let buffer = writer.buffer.slice(0, writer.length);
        assert(buffer[0] === 5);
        assert(buffer[1] === 6);

        const reader = new FnkBorshReader(buffer);
        let actual = uIntSchema.deserialize(reader);
        let expected = fnk_range;
        assert(
            actual.from.eq(expected.from),
            `${actual.from} != ${expected.from}`
        );
        assert(actual.to.eq(expected.to), `${actual.to} != ${expected.to}`);
    });

    it('test_serialize_unsigned_range_negative', () => {
        let fnk_range = new FnkURange(
            0,
            new BN(2).pow(new BN(64)).subn(1).subn(5)
        );
        const writer = new FnkBorshWriter();
        uIntSchema.serialize(writer, fnk_range);

        let buffer = writer.buffer.slice(0, writer.length);
        assert(buffer[0] === 0);
        assert(buffer[1] === (5 | 0x20));

        const reader = new FnkBorshReader(buffer);
        let actual = uIntSchema.deserialize(reader);
        let expected = fnk_range;
        assert(
            actual.from.eq(expected.from),
            `${actual.from} != ${expected.from}`
        );
        assert(actual.to.eq(expected.to), `${actual.to} != ${expected.to}`);
    });

    it('test_serialize_unsigned_range_negative', () => {
        for (let i of [
            new BN(0),
            new BN(1),
            new BN(2),
            new BN(2).pow(new BN(64)).subn(1).divn(3),
            new BN(2).pow(new BN(64)).subn(1).divn(2).subn(1),
            new BN(2).pow(new BN(64)).subn(1).divn(2),
            new BN(2).pow(new BN(64)).subn(1).divn(2).addn(1),
            new BN(2).pow(new BN(64)).subn(1).subn(2),
            new BN(2).pow(new BN(64)).subn(1).subn(1),
            new BN(2).pow(new BN(64)).subn(1),
        ]) {
            let fnk_range = new FnkURange(0, i);
            const writer = new FnkBorshWriter();
            uIntSchema.serialize(writer, fnk_range);

            let buffer = writer.buffer.slice(0, writer.length);
            const reader = new FnkBorshReader(buffer);
            let actual = uIntSchema.deserialize(reader);
            let expected = fnk_range;
            assert(
                actual.from.eq(expected.from),
                `${actual.from} != ${expected.from}`
            );
            assert(actual.to.eq(expected.to), `${actual.to} != ${expected.to}`);
        }
    });

    it('test_serialize_signed_range', () => {
        for (let i of [
            new BN(2).pow(new BN(63)).neg(),
            new BN(2).pow(new BN(63)).neg().addn(1),
            new BN(2).pow(new BN(63)).neg().addn(2),
            new BN(2).pow(new BN(63)).neg().divn(2).subn(1),
            new BN(2).pow(new BN(63)).neg().divn(2),
            new BN(2).pow(new BN(63)).neg().divn(2).addn(1),
            new BN(2).pow(new BN(63)).neg().divn(3),
            new BN(-2),
            new BN(-1),
            new BN(0),
        ]) {
            let fnk_range = new FnkRange(i, 0);
            const writer = new FnkBorshWriter();
            intSchema.serialize(writer, fnk_range);

            let buffer = writer.buffer.slice(0, writer.length);
            const reader = new FnkBorshReader(buffer);
            let actual = intSchema.deserialize(reader);
            let expected = fnk_range;
            assert(
                actual.from.eq(expected.from),
                `${actual.from} != ${expected.from}`
            );
            assert(actual.to.eq(expected.to), `${actual.to} != ${expected.to}`);
        }

        for (let i of [
            new BN(0),
            new BN(1),
            new BN(2),
            new BN(2).pow(new BN(63)).subn(1).divn(3),
            new BN(2).pow(new BN(63)).subn(1).divn(2).subn(1),
            new BN(2).pow(new BN(63)).subn(1).divn(2),
            new BN(2).pow(new BN(63)).subn(1).divn(2).addn(1),
            new BN(2).pow(new BN(63)).subn(1).subn(2),
            new BN(2).pow(new BN(63)).subn(1).subn(1),
            new BN(2).pow(new BN(63)).subn(1),
        ]) {
            let fnk_range = new FnkRange(0, i);
            const writer = new FnkBorshWriter();
            intSchema.serialize(writer, fnk_range);

            let buffer = writer.buffer.slice(0, writer.length);
            const reader = new FnkBorshReader(buffer);
            let actual = intSchema.deserialize(reader);
            let expected = fnk_range;
            assert(
                actual.from.eq(expected.from),
                `${actual.from} != ${expected.from}`
            );
            assert(actual.to.eq(expected.to), `${actual.to} != ${expected.to}`);
        }
    });

    it('test_serialize_signed_range_full', () => {
        let fnk_range = new FnkRange(
            new BN(2).pow(new BN(63)).neg(),
            new BN(2).pow(new BN(63)).subn(1)
        );
        const writer = new FnkBorshWriter();
        intSchema.serialize(writer, fnk_range);

        let buffer = writer.buffer.slice(0, writer.length);
        const reader = new FnkBorshReader(buffer);
        let actual = intSchema.deserialize(reader);
        let expected = fnk_range;
        assert(
            actual.from.eq(expected.from),
            `${actual.from} != ${expected.from}`
        );
        assert(actual.to.eq(expected.to), `${actual.to} != ${expected.to}`);
    });
});
