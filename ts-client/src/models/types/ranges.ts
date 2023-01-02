import {BinaryReader, BinaryWriter, BorshError} from 'borsh';
import {FnkUInt} from './unsigned';
import BN from 'bn.js';
import {FnkInt} from './integers';

const ZERO = new BN(0);

export class FnkURange {
    readonly from: FnkUInt;
    readonly to: FnkUInt;

    // CONSTRUCTORS -----------------------------------------------------------

    constructor(from: FnkUInt, to: FnkUInt) {
        if (from.value.gt(to.value)) {
            throw new BorshError('Invalid range: from > to');
        }

        this.from = from;
        this.to = to;
    }

    static newUnbounded(from: BN | number | string) {
        return new FnkURange(new FnkUInt(from), FnkUInt.maxValue());
    }

    static fromNumbers(from: BN | number | string, to: BN | number | string) {
        return new FnkURange(new FnkUInt(from), new FnkUInt(to));
    }

    static fromNumbersUnbounded(from: BN | number | string) {
        return new FnkURange(new FnkUInt(from), FnkUInt.maxValue());
    }

    // GETTERS ----------------------------------------------------------------

    pointAndLength(): { point: FnkUInt, length: FnkInt } {
        let point = this.from;
        let distanceToEnd = FnkUInt.maxValue().value.sub(this.to.value);

        // Shortcut for unbounded ranges.
        if (distanceToEnd === ZERO) {
            let length = new FnkInt(ZERO);

            return {
                point,
                length,
            };
        }

        let distanceToStart = this.to.value.sub(this.from.value).addn(1);

        let length: FnkInt;

        if (distanceToEnd.lte(distanceToStart)) {
            length = new FnkInt(distanceToEnd.neg());
        } else {
            length = new FnkInt(distanceToStart);
        }

        return {
            point,
            length,
        };
    }

    // METHODS ----------------------------------------------------------------

    borshSerialize(writer: BinaryWriter) {
        writer.writeFnkURange(this);
    }

    borshDeserialize(reader: BinaryReader): FnkURange {
        return reader.readFnkURange();
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

declare module 'borsh' {
    export interface BinaryWriter {
        writeFnkURange(value: FnkURange);
    }

    export interface BinaryReader {
        readFnkURange(): FnkURange;
    }
}

(BinaryWriter.prototype as any).writeFnkURange = function (value: FnkURange) {
    const writer = this as unknown as BinaryWriter;

    let {
        point,
        length,
    } = value.pointAndLength();

    writer.writeFnkUInt(point);
    writer.writeFnkInt(length);
};

(BinaryReader.prototype as any).readFnkURange = function () {
    const reader = this as unknown as BinaryReader;

    const point = reader.readFnkUInt();
    const length = reader.readFnkInt();

    let to = length.value.lten(0) ? FnkUInt.maxValue().value.sub(length.value.abs()) :
        point.value.add(length.value).subn(1);

    return new FnkURange(point, new FnkUInt(to));
};

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

export class FnkRange {
    readonly from: FnkInt;
    readonly to: FnkInt;

    // CONSTRUCTORS -----------------------------------------------------------

    constructor(from: FnkInt, to: FnkInt) {
        if (from.value.gt(to.value)) {
            throw new BorshError('Invalid range: from > to');
        }

        this.from = from;
        this.to = to;
    }

    static newUnbounded(from: BN | number | string) {
        return new FnkRange(new FnkInt(from), FnkInt.maxValue());
    }

    static fromNumbers(from: BN | number | string, to: BN | number | string) {
        return new FnkRange(new FnkInt(from), new FnkInt(to));
    }

    static fromNumbersUnbounded(from: BN | number | string) {
        return new FnkRange(new FnkInt(from), FnkInt.maxValue());
    }

    // METHODS ----------------------------------------------------------------

    borshSerialize(writer: BinaryWriter) {
        writer.writeFnkRange(this);
    }

    borshDeserialize(reader: BinaryReader): FnkRange {
        return reader.readFnkRange();
    }
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

declare module 'borsh' {
    export interface BinaryWriter {
        writeFnkRange(value: FnkRange);
    }

    export interface BinaryReader {
        readFnkRange(): FnkRange;
    }
}

(BinaryWriter.prototype as any).writeFnkRange = function (value: FnkRange) {
    const writer = this as unknown as BinaryWriter;

    writer.writeFnkInt(value.from);
    writer.writeFnkInt(value.to);
};

(BinaryReader.prototype as any).readFnkRange = function () {
    const reader = this as unknown as BinaryReader;

    const from = reader.readFnkInt();
    const to = reader.readFnkInt();

    return new FnkRange(from, to);
};