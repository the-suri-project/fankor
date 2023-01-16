import { FnkBorshReader } from '../deserializer';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshSchema } from '../borsh';

export class UnitSchema implements FnkBorshSchema<null> {
    // METHODS ----------------------------------------------------------------

    serialize(writer: FnkBorshWriter, value: null) {}

    deserialize(reader: FnkBorshReader): null {
        return null;
    }
}

export const Unit = new UnitSchema();
