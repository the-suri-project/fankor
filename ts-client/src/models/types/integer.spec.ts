import assert from 'assert';
import {FnkInt} from './integers';
import {BinaryReader, BinaryWriter} from 'borsh';
import BN from 'bn.js';

describe('FnkInt Tests', () => {
    it('test_serialize_as_one_byte_flag_format', () => {
        for (const number of [0, 1, 15, 31]) {
            // Positive
            {
                let fnk_number = new FnkInt(number);
                let serializer = new BinaryWriter();
                serializer.writeFnkInt(fnk_number);

                let actual = serializer.buf.slice(0, serializer.length);
                let expected = Buffer.from([number]);
                assert(actual.equals(expected), `${actual.toString('hex')} != ${expected.toString('hex')}`);
            }

            if (number == 0) {
                continue;
            }

            // Negative
            {
                let fnk_number = new FnkInt(-number);
                let serializer = new BinaryWriter();
                serializer.writeFnkInt(fnk_number);

                let actual = serializer.buf.slice(0, serializer.length);
                let expected = Buffer.from([number | 0x20]);
                assert(actual.equals(expected), `${actual.toString('hex')} != ${expected.toString('hex')}`);
            }
        }
    });

    it('test_serialize_as_two_bytes_flag_format', () => {
        // Positive
        {
            let fnk_number = new FnkInt(0b1010_1010_1010);
            let serializer = new BinaryWriter();
            serializer.writeFnkInt(fnk_number);

            let actual = serializer.buf.slice(0, serializer.length);
            let expected = Buffer.from([0b0100_1010, 0b0101_0101]);
            assert(actual.equals(expected), `${actual.toString('hex')} != ${expected.toString('hex')}`);
        }

        // Negative
        {
            let fnk_number = new FnkInt(-0b1010_1010_1010);
            let serializer = new BinaryWriter();
            serializer.writeFnkInt(fnk_number);

            let actual = serializer.buf.slice(0, serializer.length);
            let expected = Buffer.from([0b0110_1010, 0b0101_0101]);
            assert(actual.equals(expected), `${actual.toString('hex')} != ${expected.toString('hex')}`);
        }
    });

    it('test_serialize_as_two_bytes_length_format', () => {
        // Positive
        {
            let fnk_number = new FnkInt(0x1555);
            let serializer = new BinaryWriter();
            serializer.writeFnkInt(fnk_number);

            let actual = serializer.buf.slice(0, serializer.length);
            let expected = Buffer.from([2 | 0x80, 0b0101_0101, 0b1_0101]);
            assert(actual.equals(expected), `${actual.toString('hex')} != ${expected.toString('hex')}`);
        }

        // Negative
        {
            let fnk_number = new FnkInt(-0x1555);
            let serializer = new BinaryWriter();
            serializer.writeFnkInt(fnk_number);

            let actual = serializer.buf.slice(0, serializer.length);
            let expected = Buffer.from([2 | 0x80 | 0x40, 0b0101_0101, 0b1_0101]);
            assert(actual.equals(expected), `${actual.toString('hex')} != ${expected.toString('hex')}`);
        }
    });

    it('test_serialize_as_bytes_length_format', () => {
        // Positive
        {
            let number = new BN(0x1AA);
            for (let i = 3; i < 9; i += 1) {
                number = number.shln(8).or(new BN(0xAA));

                let fnk_number = new FnkInt(number);
                let serializer = new BinaryWriter();
                serializer.writeFnkInt(fnk_number);

                let bytes = [i | 0x80];

                for (let j = 1; j < i; j += 1) {
                    bytes.push(0b1010_1010);
                }

                bytes.push(0b1);

                let actual = serializer.buf.slice(0, serializer.length);
                let expected = Buffer.from(bytes);
                assert(actual.equals(expected), `For(${i}): ${actual.toString('hex')} != ${expected.toString('hex')}`);
            }

            number = new BN('9223372036854775807');
            let fnk_number = new FnkInt(number);
            let serializer = new BinaryWriter();
            serializer.writeFnkInt(fnk_number);

            let bytes = [8 | 0x80];

            for (let j = 1; j < 8; j += 1) {
                bytes.push(0b1111_1111);
            }
            bytes.push(0b0111_1111);

            let actual = serializer.buf.slice(0, serializer.length);
            let expected = Buffer.from(bytes);
            assert(actual.equals(expected), `For(MAX): ${actual.toString('hex')} != ${expected.toString('hex')}`);
        }

        // Negative
        {
            let number = new BN(0x1AA);
            for (let i = 3; i < 9; i += 1) {
                number = number.shln(8).or(new BN(0xAA));

                let fnk_number = new FnkInt(number.neg());
                let serializer = new BinaryWriter();
                serializer.writeFnkInt(fnk_number);

                let bytes = [i | 0x80| 0x40];

                for (let j = 1; j < i; j += 1) {
                    bytes.push(0b1010_1010);
                }

                bytes.push(0b1);

                let actual = serializer.buf.slice(0, serializer.length);
                let expected = Buffer.from(bytes);
                assert(actual.equals(expected), `For(${i}): ${actual.toString('hex')} != ${expected.toString('hex')}`);
            }

            {
                number = new BN('-9223372036854775807');
                let fnk_number = new FnkInt(number);
                let serializer = new BinaryWriter();
                serializer.writeFnkInt(fnk_number);

                let bytes = [8 | 0x80 | 0x40];

                for (let j = 1; j < 8; j += 1) {
                    bytes.push(0b1111_1111);
                }
                bytes.push(0b0111_1111);

                let actual = serializer.buf.slice(0, serializer.length);
                let expected = Buffer.from(bytes);
                assert(actual.equals(expected), `For(MAX): ${actual.toString('hex')} != ${expected.toString('hex')}`);
            }

            {
                number = new BN('-9223372036854775808');
                let fnk_number = new FnkInt(number);
                let serializer = new BinaryWriter();
                serializer.writeFnkInt(fnk_number);

                let bytes = [8 | 0x80 | 0x40];

                for (let j = 1; j < 8; j += 1) {
                    bytes.push(0b0);
                }
                bytes.push(0b01000_0000);

                let actual = serializer.buf.slice(0, serializer.length);
                let expected = Buffer.from(bytes);
                assert(actual.equals(expected), `For(MAX): ${actual.toString('hex')} != ${expected.toString('hex')}`);
            }
        }
    });

    it('test_deserialize', () => {
        for (let number of
            [new BN(0), new BN(1),
                new BN(2).pow(new BN(8)).sub(new BN(1)),
                new BN(2).pow(new BN(16)).sub(new BN(1)),
                new BN(2).pow(new BN(32)).sub(new BN(1)),
                new BN(2).pow(new BN(7)).neg(),
                new BN(2).pow(new BN(7)).sub(new BN(1)),
                new BN(2).pow(new BN(15)).neg(),
                new BN(2).pow(new BN(15)).sub(new BN(1)),
                new BN(2).pow(new BN(31)).neg(),
                new BN(2).pow(new BN(31)).sub(new BN(1)),
                new BN(2).pow(new BN(63)).neg(),
                new BN(2).pow(new BN(63)).div(new BN(2)).neg(),
                new BN(2).pow(new BN(63)).div(new BN(2)),
                new BN(2).pow(new BN(63)).sub(new BN(1)),
            ]) {
            let fnk_number = new FnkInt(number);
            let serializer = new BinaryWriter();
            serializer.writeFnkInt(fnk_number);

            let deserializer = new BinaryReader(serializer.buf.slice(0, serializer.length));
            let de_fnk_number = deserializer.readFnkInt();

            assert(deserializer.offset === deserializer.buf.length,
                `For(${number}): offset(${deserializer.offset}) != length(${deserializer.buf.length})`);

            let actual = fnk_number.value;
            let expected = de_fnk_number.value;
            assert(actual.eq(expected), `For(${number}): ${actual.toString('hex')} != ${expected.toString('hex')}`);
        }
    });

    it('test_deserialize_long', () => {
        for (let number = -1000; number < 1000; number += 1) {
            let fnk_number = new FnkInt(number);
            let serializer = new BinaryWriter();
            serializer.writeFnkInt(fnk_number);

            let deserializer = new BinaryReader(serializer.buf.slice(0, serializer.length));
            let de_fnk_number = deserializer.readFnkInt();

            assert(deserializer.offset === deserializer.buf.length,
                `For(${number}): offset(${deserializer.offset}) != length(${deserializer.buf.length})`);

            let actual = fnk_number.value;
            let expected = de_fnk_number.value;
            assert(actual.eq(expected), `For(${number}): ${actual.toString('hex')} != ${expected.toString('hex')}`);
        }
    });
});