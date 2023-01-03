import { FnkUIntSchema } from './unsigned';
import BN from 'bn.js';
import { FnkIntSchema } from './integers';
import { FnkBorshReader } from '../../deserializer';
import { FnkBorshWriter } from '../../serializer';
import { FnkBorshError } from '../../errors';
import { FnkBorshSchema } from '../../index';

const ZERO = new BN(0);
const U64_MAX_VALUE = new BN('18446744073709551615'); // 2^64 - 1
const I64_MIN_VALUE = new BN('-9223372036854775808'); // -2^63
const I64_MAX_VALUE = new BN('9223372036854775807'); // 2^63 - 1

export class FnkURange {
    readonly from: BN;
    readonly to: BN;

    // CONSTRUCTORS -----------------------------------------------------------

    constructor(from: BN | bigint | number, to: BN | bigint | number) {
        from =
            typeof from === 'bigint' ? new BN(from.toString()) : new BN(from);
        to = typeof to === 'bigint' ? new BN(to.toString()) : new BN(to);

        if (from.gt(to)) {
            throw new FnkBorshError('Invalid range: from > to');
        }

        if (from.lt(ZERO)) {
            throw new RangeError('from(FnkUInt) cannot be negative');
        }

        if (from.gt(U64_MAX_VALUE)) {
            throw new RangeError(
                'from(FnkUInt) cannot be greater than 2^64 - 1'
            );
        }

        if (to.lt(ZERO)) {
            throw new RangeError('to(FnkUInt) cannot be negative');
        }

        if (to.gt(U64_MAX_VALUE)) {
            throw new RangeError('to(FnkUInt) cannot be greater than 2^64 - 1');
        }

        this.from = from;
        this.to = to;
    }

    static newUnbounded(from: BN | bigint | number) {
        return new FnkURange(from, U64_MAX_VALUE);
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export const TFnkURange = () => new FnkURangeSchema();

export class FnkURangeSchema implements FnkBorshSchema<FnkURange> {
    // METHODS ----------------------------------------------------------------

    private pointAndLength(value: FnkURange): { point: BN; length: BN } {
        let point = value.from;
        let distanceToEnd = U64_MAX_VALUE.sub(value.to);

        // Shortcut for unbounded ranges.
        if (distanceToEnd === ZERO) {
            let length = ZERO;

            return {
                point,
                length,
            };
        }

        let distanceToStart = value.to.sub(value.from).addn(1);

        let length: BN;

        if (distanceToEnd.lte(distanceToStart)) {
            length = distanceToEnd.neg();
        } else {
            length = distanceToStart;
        }

        return {
            point,
            length,
        };
    }

    serialize(writer: FnkBorshWriter, value: FnkURange) {
        let { point, length } = this.pointAndLength(value);

        new FnkUIntSchema().serialize(writer, point);
        new FnkIntSchema().serialize(writer, length);
    }

    deserialize(reader: FnkBorshReader): FnkURange {
        const point = new FnkUIntSchema().deserialize(reader);
        const length = new FnkIntSchema().deserialize(reader);

        let to = length.lten(0)
            ? U64_MAX_VALUE.sub(length.abs())
            : point.add(length).subn(1);

        return new FnkURange(point, to);
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export class FnkRange {
    readonly from: BN;
    readonly to: BN;

    // CONSTRUCTORS -----------------------------------------------------------

    constructor(from: BN | bigint | number, to: BN | bigint | number) {
        from =
            typeof from === 'bigint' ? new BN(from.toString()) : new BN(from);
        to = typeof to === 'bigint' ? new BN(to.toString()) : new BN(to);

        if (from.gt(to)) {
            throw new FnkBorshError('Invalid range: from > to');
        }

        if (from.lt(I64_MIN_VALUE)) {
            throw new RangeError('from(FnkInt) cannot be lower than -2^63');
        }

        if (from.gt(I64_MAX_VALUE)) {
            throw new RangeError(
                'from(FnkInt) cannot be greater than 2^63 - 1'
            );
        }

        if (to.lt(I64_MIN_VALUE)) {
            throw new RangeError('to(FnkInt) cannot be lower than -2^63');
        }

        if (to.gt(I64_MAX_VALUE)) {
            throw new RangeError('to(FnkInt) cannot be greater than 2^63 - 1');
        }

        this.from = from;
        this.to = to;
    }

    static newUnbounded(from: BN | bigint | number) {
        return new FnkRange(from, I64_MAX_VALUE);
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export const TFnkRange = () => new FnkRangeSchema();

export class FnkRangeSchema implements FnkBorshSchema<FnkRange> {
    // METHODS ----------------------------------------------------------------
    serialize(writer: FnkBorshWriter, value: FnkRange) {
        const fnkIntSchema = new FnkIntSchema();
        fnkIntSchema.serialize(writer, value.from);
        fnkIntSchema.serialize(writer, value.to);
    }

    deserialize(reader: FnkBorshReader): FnkRange {
        const fnkIntSchema = new FnkIntSchema();
        const from = fnkIntSchema.deserialize(reader);
        const to = fnkIntSchema.deserialize(reader);

        return new FnkRange(from, to);
    }
}
