import assert from 'assert';
import { RpcFankorError } from './rpc_errors';

describe('RpcFankorError tests', () => {
    it('test', () => {
        const code = 1006;
        const name = 'InvalidPda';
        const message = 'The provided PDA (x) does not match expected one (y).';
        const log = `Program log: FankorError occurred. Error Name: ${name}. Error Code: ${code}. Error Message: ${message}`;

        let logs = [
            'another log',
            'Program log: FankorError occurred.',
            'Program log: FankorError occurred. Error Name: ',
            'Program log: FankorError occurred. Error Name: {}',
            'Program log: FankorError occurred. Error Name: {}. Error Code: ',
            'Program log: FankorError occurred. Error Name: {}. Error Code: {}',
            'Program log: FankorError occurred. Error Name: {}. Error Code: {}. Error Message:',
            log,
        ];
        const error = RpcFankorError.fromLogs(logs);

        if (error === null) {
            throw new Error('error is null');
        }

        assert.strictEqual(error.code, code, 'Invalid error code');
        assert.strictEqual(error.name, name, 'Invalid error name');
        assert.strictEqual(error.message, message, 'Invalid error message');
    });
});
