import assert from 'assert';
import {FnkUInt} from './unsigned';
import BN from 'bn.js';
import {FnkBorshWriter} from '../../serializer';
import {FnkBorshReader} from '../../deserializer';

describe('FnkUInt Tests', () => {
    const schema = FnkUInt;

    it('test_serialize_as_one_byte_flag_format', () => {
        for (const number of [0, 1, 42, 63]) {
            const writer = new FnkBorshWriter();
            schema.serialize(writer, number);

            let actual = writer.buffer.slice(0, writer.length);
            let expected = Buffer.from([number]);
            assert(actual.equals(expected), `${actual.toString('hex')} != ${expected.toString('hex')}`);
        }
    });

    it('test_serialize_as_two_bytes_flag_format', () => {
        let number = 0b0001_0101_0101_0101;
        const writer = new FnkBorshWriter();
        schema.serialize(writer, number);

        let actual = writer.buffer.slice(0, writer.length);
        let expected = Buffer.from([0b0101_0101, 0b0101_0101]);
        assert(actual.equals(expected), `${actual.toString('hex')} != ${expected.toString('hex')}`);
    });

    it('test_serialize_as_two_bytes_length_format', () => {
        let number = 0x2AAA;
        const writer = new FnkBorshWriter();
        schema.serialize(writer, number);

        let actual = writer.buffer.slice(0, writer.length);
        let expected = Buffer.from([2 | 0x80, 0b1010_1010, 0b10_1010]);
        assert(actual.equals(expected), `${actual.toString('hex')} != ${expected.toString('hex')}`);
    });

    it('test_serialize_as_bytes_length_format', () => {
        let number = new BN(0x1AA);
        for (let i = 3; i < 9; i += 1) {
            number = number.shln(8).or(new BN(0xAA));

            const writer = new FnkBorshWriter();
            schema.serialize(writer, number);

            let bytes = [i | 0x80];

            for (let j = 1; j < i; j += 1) {
                bytes.push(0b1010_1010);
            }

            bytes.push(0b1);

            let actual = writer.buffer.slice(0, writer.length);
            let expected = Buffer.from(bytes);
            assert(actual.equals(expected), `For(${i}): ${actual.toString('hex')} != ${expected.toString('hex')}`);
        }

        number = new BN('18446744073709551615');
        const writer = new FnkBorshWriter();
        schema.serialize(writer, number);

        let bytes = [8 | 0x80];

        for (let j = 1; j < 9; j += 1) {
            bytes.push(0b1111_1111);
        }

        let actual = writer.buffer.slice(0, writer.length);
        let expected = Buffer.from(bytes);
        assert(actual.equals(expected), `For(MAX): ${actual.toString('hex')} != ${expected.toString('hex')}`);
    });

    it('test_deserialize', () => {
        for (let number of
            [new BN(0), new BN(1), new BN(2).pow(new BN(8)).sub(new BN(1)), new BN(2).pow(new BN(16)).sub(new BN(1)),
                new BN(2).pow(new BN(32)).sub(new BN(1)), new BN(2).pow(new BN(64)).divn(2),
                new BN(2).pow(new BN(64)).sub(new BN(1))]) {
            const writer = new FnkBorshWriter();
            schema.serialize(writer, number);

            const reader = new FnkBorshReader(writer.buffer.slice(0, writer.length));
            let de_number = schema.deserialize(reader);

            assert(reader.offset === reader.buffer.length,
                `For(${number}): offset(${reader.offset}) != length(${reader.buffer.length})`);

            let actual = number;
            let expected = de_number;
            assert(actual.eq(expected), `For(${number}): ${actual.toString('hex')} != ${expected.toString('hex')}`);
        }
    });

    it('test_deserialize_long', () => {
        for (let number = 64; number < 1000; number += 1) {
            const writer = new FnkBorshWriter();
            schema.serialize(writer, number);

            const reader = new FnkBorshReader(writer.buffer.slice(0, writer.length));
            let de_number = schema.deserialize(reader);

            assert(reader.offset === reader.buffer.length,
                `For(${number}): offset(${reader.offset}) != length(${reader.buffer.length})`);

            let actual = new BN(number);
            let expected = de_number;
            assert(actual.eq(expected), `For(${number}): ${actual.toString('hex')} != ${expected.toString('hex')}`);
        }
    });
});