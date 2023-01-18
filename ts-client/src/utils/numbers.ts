import BN from 'bn.js';

export function numberToBN(number: BN | bigint | number | string): BN {
    return new BN(typeof number == 'bigint' ? number.toString() : number);
}
