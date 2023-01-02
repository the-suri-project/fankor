import assert from 'assert';
import {FnkUInt} from './unsigned';
import {BinaryReader, BinaryWriter} from 'borsh';
import BN from 'bn.js';

describe('FnkUInt Tests', () => {
    it('test_serialize_as_one_byte_flag_format', () => {
        for (const number of [0, 1, 42, 63]) {
            let fnk_number = new FnkUInt(number);
            let serializer = new BinaryWriter();
            serializer.writeFnkUInt(fnk_number);

            let actual = serializer.buf.slice(0, serializer.length);
            let expected = Buffer.from([number]);
            assert(actual.equals(expected), `${actual.toString('hex')} != ${expected.toString('hex')}`);
        }
    });

    it('test_serialize_as_two_bytes_flag_format', () => {
        let fnk_number = new FnkUInt(0b0001_0101_0101_0101);
        let serializer = new BinaryWriter();
        serializer.writeFnkUInt(fnk_number);

        let actual = serializer.buf.slice(0, serializer.length);
        let expected = Buffer.from([0b0101_0101, 0b0101_0101]);
        assert(actual.equals(expected), `${actual.toString('hex')} != ${expected.toString('hex')}`);
    });

    it('test_serialize_as_two_bytes_length_format', () => {
        let fnk_number = new FnkUInt(0x2AAA);
        let serializer = new BinaryWriter();
        serializer.writeFnkUInt(fnk_number);

        let actual = serializer.buf.slice(0, serializer.length);
        let expected = Buffer.from([2 | 0x80, 0b1010_1010, 0b10_1010]);
        assert(actual.equals(expected), `${actual.toString('hex')} != ${expected.toString('hex')}`);
    });

    it('test_serialize_as_bytes_length_format', () => {
        let number = new BN(0x1AA);
        for (let i = 3; i < 9; i += 1) {
            number = number.shln(8).or(new BN(0xAA));

            let fnk_number = new FnkUInt(number);
            let serializer = new BinaryWriter();
            serializer.writeFnkUInt(fnk_number);

            let bytes = [i | 0x80];

            for (let j = 1; j < i; j += 1) {
                bytes.push(0b1010_1010);
            }

            bytes.push(0b1);

            let actual = serializer.buf.slice(0, serializer.length);
            let expected = Buffer.from(bytes);
            assert(actual.equals(expected), `For(${i}): ${actual.toString('hex')} != ${expected.toString('hex')}`);
        }

        number = new BN('18446744073709551615');
        let fnk_number = new FnkUInt(number);
        let serializer = new BinaryWriter();
        serializer.writeFnkUInt(fnk_number);

        let bytes = [8 | 0x80];

        for (let j = 1; j < 9; j += 1) {
            bytes.push(0b1111_1111);
        }

        let actual = serializer.buf.slice(0, serializer.length);
        let expected = Buffer.from(bytes);
        assert(actual.equals(expected), `For(MAX): ${actual.toString('hex')} != ${expected.toString('hex')}`);
    });

    it('test_deserialize', () => {
        for (let number of
            [new BN(0), new BN(1), new BN(2).pow(new BN(8)).sub(new BN(1)), new BN(2).pow(new BN(16)).sub(new BN(1)),
                new BN(2).pow(new BN(32)).sub(new BN(1)), new BN(2).pow(new BN(64)).divn(2),
                new BN(2).pow(new BN(64)).sub(new BN(1))]) {
            let fnk_number = new FnkUInt(number);
            let serializer = new BinaryWriter();
            serializer.writeFnkUInt(fnk_number);

            let deserializer = new BinaryReader(serializer.buf.slice(0, serializer.length));
            let de_fnk_number = deserializer.readFnkUInt();

            assert(deserializer.offset === deserializer.buf.length,
                `For(${number}): offset(${deserializer.offset}) != length(${deserializer.buf.length})`);

            let actual = fnk_number.value;
            let expected = de_fnk_number.value;
            assert(actual.eq(expected), `For(${number}): ${actual.toString('hex')} != ${expected.toString('hex')}`);
        }
    });

    it('test_deserialize_long', () => {
        for (let number = 64; number < 1000; number += 1) {
            let fnk_number = new FnkUInt(number);
            let serializer = new BinaryWriter();
            serializer.writeFnkUInt(fnk_number);

            let deserializer = new BinaryReader(serializer.buf.slice(0, serializer.length));
            let de_fnk_number = deserializer.readFnkUInt();

            assert(deserializer.offset === deserializer.buf.length,
                `For(${number}): offset(${deserializer.offset}) != length(${deserializer.buf.length})`);

            let actual = fnk_number.value;
            let expected = de_fnk_number.value;
            assert(actual.eq(expected), `For(${number}): ${actual.toString('hex')} != ${expected.toString('hex')}`);
        }
    });
});