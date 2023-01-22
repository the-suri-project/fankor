import { AccountInfo, PublicKey } from '@solana/web3.js';

export * from './accounts';
export * from './serde';
export * from './utils';
export * from './errors';

/**
 * Data information returned by lots of functions.
 */
export interface AccountResult<T> {
    address: PublicKey;
    account: AccountInfo<Buffer>;
    data: T;
}
