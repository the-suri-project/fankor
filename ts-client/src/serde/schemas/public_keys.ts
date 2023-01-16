import { FnkBorshReader } from '../deserializer';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshSchema } from '../borsh';
import { PUBLIC_KEY_LENGTH, PublicKey } from '@solana/web3.js';

export class PublicKeySchema implements FnkBorshSchema<PublicKey> {
    // METHODS ----------------------------------------------------------------
    serialize(writer: FnkBorshWriter, value: PublicKey) {
        const bytes = value.toBuffer();
        writer.writeBuffer(bytes);
    }

    deserialize(reader: FnkBorshReader): PublicKey {
        const endIndex = reader.offset + PUBLIC_KEY_LENGTH;
        const buffer = reader.buffer.slice(reader.offset, endIndex);
        const result = new PublicKey(buffer);
        reader.offset += PUBLIC_KEY_LENGTH;

        return result;
    }
}

export const TPublicKey = new PublicKeySchema();
