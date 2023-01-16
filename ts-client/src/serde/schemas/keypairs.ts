import { FnkBorshReader } from '../deserializer';
import { FnkBorshWriter } from '../serializer';
import { FnkBorshSchema } from '../borsh';
import { Keypair, MAX_SEED_LENGTH } from '@solana/web3.js';

export class KeypairSchema implements FnkBorshSchema<Keypair> {
    // METHODS ----------------------------------------------------------------
    serialize(writer: FnkBorshWriter, value: Keypair) {
        const bytes = Buffer.from(value.secretKey.slice(0, 32));
        writer.writeBuffer(bytes);
    }

    deserialize(reader: FnkBorshReader): Keypair {
        const endIndex = reader.offset + MAX_SEED_LENGTH;
        const buffer = reader.buffer.slice(reader.offset, endIndex);
        const result = Keypair.fromSeed(buffer);
        reader.offset += MAX_SEED_LENGTH;

        return result;
    }
}

export const TKeypair = new KeypairSchema();
