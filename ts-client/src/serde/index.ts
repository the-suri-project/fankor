import {FnkBorshReadSchema} from './deserializer';
import {FnkBorshWriteSchema} from './serializer';

export * from './schemas';
export * from './deserializer';
export * from './errors';
export * from './serializer';

export type FnkBorshSchema<T> =
    FnkBorshReadSchema<T>
    & FnkBorshWriteSchema<T>;