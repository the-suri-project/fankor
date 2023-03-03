import assert from 'assert';
import { FnkUInt } from './unsigned';
import BN from 'bn.js';
import { FnkBorshWriter } from '../../serializer';
import { FnkBorshReader } from '../../deserializer';

describe('FnkUInt Tests', () => {
    const schema = FnkUInt;

    it('test_serialize_flag_format_one_byte', () => {
        let max = 1 << 6;
        for (let number = 0; number < max; number += 1) {
            const writer = new FnkBorshWriter();
            schema.serialize(writer, new BN(number));

            let actual = writer.buffer.slice(0, writer.length);
            let expected = new BN(number).toBuffer('le');
            assert(
                actual.equals(expected),
                `${actual.toString('hex')} != ${expected.toString('hex')}`
            );
            assert(actual.length == 1, `Invalid length: ${actual.length} != 1`);
        }
    });

    it('test_serialize_flag_format_two_bytes', () => {
        let max = 1 << 14;
        for (let number = 1 << 6; number < max; number += 1) {
            const writer = new FnkBorshWriter();
            schema.serialize(writer, new BN(number));

            let actual = writer.buffer.slice(0, writer.length);

            assert(
                actual[0] === (0x40 | (number & 0x3f)),
                `[0] ${actual[0]} != ${0x40 | (number & 0x3f)}`
            );
            assert(
                actual[1] === number >> 6,
                `[1] ${actual[1]} != ${number >> 6}`
            );
            assert(actual.length == 2, `Invalid length: ${actual.length} != 2`);
        }
    });

    it('test_serialize_length_format', () => {
        // Three bytes
        let numBytes = 2;
        let max = 1 << 16;
        for (let number = 1 << 14; number < max; number += 1) {
            const writer = new FnkBorshWriter();
            schema.serialize(writer, new BN(number));

            let actual = writer.buffer.slice(0, writer.length);
            let expected = Buffer.concat([
                Buffer.from([0x80 | (numBytes - 2)]),
                new BN(number).toBuffer('le'),
            ]);
            assert(
                actual.equals(expected),
                `${actual.toString('hex')} != ${expected.toString('hex')}`
            );
            assert(
                actual.length == numBytes + 1,
                `Invalid length: ${actual.length} != ${numBytes}`
            );
        }

        // Rest
        for (let numBytes = 3; numBytes <= 8; numBytes += 1) {
            let low = new BN(1).shln((numBytes - 1) << 3);
            let high = new BN(1).shln(numBytes << 3).subn(1);

            for (let number of [low, high]) {
                const writer = new FnkBorshWriter();
                schema.serialize(writer, number);

                let actual = writer.buffer.slice(0, writer.length);
                let expected = Buffer.concat([
                    Buffer.from([0x80 | (numBytes - 2)]),
                    number.toBuffer('le'),
                ]);
                assert(
                    actual.equals(expected),
                    `${actual.toString('hex')} != ${expected.toString('hex')}`
                );
                assert(
                    actual.length == numBytes + 1,
                    `Invalid length: ${actual.length} != ${numBytes}`
                );
            }
        }
    });

    it('test serialize/deserialize', () => {
        for (let number of [
            new BN(0),
            new BN(1),
            new BN(1).shln(6).sub(new BN(1)),
            new BN(1).shln(6),
            new BN(1).shln(14).sub(new BN(1)),
            new BN(1).shln(14),
            new BN(1).shln(16),
            new BN(1).shln(24),
            new BN(1).shln(32),
            new BN(1).shln(40),
            new BN(1).shln(48),
            new BN(1).shln(56),
            new BN(2).pow(new BN(8)),
            new BN(2).pow(new BN(16)),
            new BN(2).pow(new BN(32)),
            new BN(2).pow(new BN(64)).divn(2),
            new BN(2).pow(new BN(64)).sub(new BN(1)),
        ]) {
            const writer = new FnkBorshWriter();
            schema.serialize(writer, number);

            const reader = new FnkBorshReader(
                writer.buffer.slice(0, writer.length)
            );
            let de_number = schema.deserialize(reader);

            assert(
                reader.offset === reader.buffer.length,
                `For(${number}): offset(${reader.offset}) != length(${reader.buffer.length})`
            );

            let actual = number;
            let expected = de_number;
            assert(
                actual.eq(expected),
                `For(${number}): ${actual.toString(
                    'hex'
                )} != ${expected.toString('hex')}`
            );
        }
    });
});
