import assert from 'assert';
import { FnkInt } from './integers';
import BN from 'bn.js';
import { FnkBorshWriter } from '../../serializer';
import { FnkBorshReader } from '../../deserializer';

describe('FnkInt Tests', () => {
    const schema = FnkInt;

    it('test_serialize_flag_format_one_byte', () => {
        // Positive
        let max = 1 << 5;
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

        // Negative
        for (let number = 1; number < max; number += 1) {
            const writer = new FnkBorshWriter();
            schema.serialize(writer, new BN(-number));

            let actual = writer.buffer.slice(0, writer.length);
            let expected = new BN(number).toBuffer('le');
            expected[0] |= 0x20;

            assert(
                actual.equals(expected),
                `${actual.toString('hex')} != ${expected.toString('hex')}`
            );
            assert(actual.length == 1, `Invalid length: ${actual.length} != 1`);
        }
    });

    it('test_serialize_flag_format_two_bytes', () => {
        let max = 1 << 13;
        for (let number = 1 << 5; number < max; number += 1) {
            // Positive
            let writer = new FnkBorshWriter();
            schema.serialize(writer, new BN(number));

            let actual = writer.buffer.slice(0, writer.length);

            assert(
                actual[0] === (0x40 | (number & 0x1f)),
                `[0] ${actual[0]} != ${0x40 | (number & 0x1f)}`
            );
            assert(
                actual[1] === number >> 5,
                `[1] ${actual[1]} != ${number >> 5}`
            );
            assert(actual.length == 2, `Invalid length: ${actual.length} != 2`);

            // Negative
            writer = new FnkBorshWriter();
            schema.serialize(writer, new BN(-number));

            actual = writer.buffer.slice(0, writer.length);

            assert(
                actual[0] === (0x40 | 0x20 | (number & 0x1f)),
                `[0] ${actual[0]} != ${0x40 | 0x20 | (number & 0x1f)}`
            );
            assert(
                actual[1] === number >> 5,
                `[1] ${actual[1]} != ${number >> 5}`
            );
            assert(actual.length == 2, `Invalid length: ${actual.length} != 2`);
        }
    });

    it('test_serialize_length_format', () => {
        // Three bytes
        let numBytes = 2;
        let max = 1 << 16;
        for (let number = 1 << 14; number < max; number += 1) {
            // Positive
            let writer = new FnkBorshWriter();
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

            // Negative
            writer = new FnkBorshWriter();
            schema.serialize(writer, new BN(-number));

            actual = writer.buffer.slice(0, writer.length);
            expected = Buffer.concat([
                Buffer.from([0x80 | 0x40 | (numBytes - 2)]),
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

        // Rest until 8 bytes
        for (let numBytes = 3; numBytes < 8; numBytes += 1) {
            let low = new BN(1).shln((numBytes - 1) << 3);
            let high = new BN(1).shln(numBytes << 3).subn(1);

            for (let number of [low, high]) {
                // Positive
                let writer = new FnkBorshWriter();
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

                // Negative
                writer = new FnkBorshWriter();
                schema.serialize(writer, number.neg());

                actual = writer.buffer.slice(0, writer.length);
                expected = Buffer.concat([
                    Buffer.from([0x80 | 0x40 | (numBytes - 2)]),
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

        // 8 bytes
        numBytes = 8;
        for (let number of [
            new BN(1).shln(7 << 3),
            new BN(1).shln(7 << 3).neg(),
            new BN(1).shln(63).neg(),
            new BN(1).shln(63 - 1),
        ]) {
            if (!number.isNeg()) {
                // Positive
                let writer = new FnkBorshWriter();
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
            } else {
                // Negative
                let writer = new FnkBorshWriter();
                schema.serialize(writer, number);

                let actual = writer.buffer.slice(0, writer.length);
                let expected = Buffer.concat([
                    Buffer.from([0x80 | 0x40 | (numBytes - 2)]),
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
            new BN(1).neg(),
            new BN(1).shln(5).sub(new BN(1)),
            new BN(1).shln(5).sub(new BN(1)).neg(),
            new BN(1).shln(5),
            new BN(1).shln(5).neg(),
            new BN(1).shln(13).sub(new BN(1)),
            new BN(1).shln(13).sub(new BN(1)).neg(),
            new BN(1).shln(13),
            new BN(1).shln(13).neg(),
            new BN(1).shln(16),
            new BN(1).shln(16).neg(),
            new BN(1).shln(24),
            new BN(1).shln(24).neg(),
            new BN(1).shln(32),
            new BN(1).shln(32).neg(),
            new BN(1).shln(40),
            new BN(1).shln(40).neg(),
            new BN(1).shln(48),
            new BN(1).shln(48).neg(),
            new BN(1).shln(56),
            new BN(1).shln(56).neg(),
            new BN(2).pow(new BN(8)).subn(1),
            new BN(2).pow(new BN(16)).subn(1),
            new BN(2).pow(new BN(32)).subn(1),
            new BN(2).pow(new BN(63)).subn(1).divn(2),
            new BN(2).pow(new BN(63)).subn(1).sub(new BN(1)),
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
