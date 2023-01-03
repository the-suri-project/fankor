import {FnkBorshReader} from '../deserializer';
import {FnkBorshWriter} from '../serializer';
import {FnkBorshError} from '../errors';
import {FnkBorshSchema} from '../index';

export class BoolSchema implements FnkBorshSchema<boolean> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: boolean) {
        if (value) {
            writer.writeByte(1);
        } else {
            writer.writeByte(0);
        }
    }

    deserialize(reader: FnkBorshReader): boolean {
        const discriminant = reader.readByte();

        if (discriminant === 0) {
            return false;
        } else if (discriminant === 1) {
            return true;
        } else {
            throw new FnkBorshError(`Invalid discriminant for bool: ${discriminant}`);
        }
    }
}

export const Bool = new BoolSchema();