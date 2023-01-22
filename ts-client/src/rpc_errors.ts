export class RpcFankorError extends Error {
    // CONSTRUCTORS -----------------------------------------------------------

    constructor(
        public code: number,
        public name: string,
        public message: string
    ) {
        super();
    }

    static fromLogs(logs: string[]) {
        for (let log of logs) {
            if (
                !log.startsWith(
                    'Program log: FankorError occurred. Error Name: '
                )
            ) {
                continue;
            }

            log = log.slice(
                'Program log: FankorError occurred. Error Name: '.length
            );
            let position = log.indexOf('.');
            if (position === -1) {
                continue;
            }

            let name = log.slice(0, position);
            log = log.slice(position);

            if (!log.startsWith('. Error Code: ')) {
                continue;
            }

            log = log.slice('. Error Code: '.length);
            position = log.indexOf('.');
            if (position === -1) {
                continue;
            }

            let code = parseInt(log.slice(0, position));
            if (
                isNaN(code) ||
                code < 0 ||
                code > 0xffffffff ||
                code !== Math.floor(code)
            ) {
                continue;
            }

            log = log.slice(position);

            if (!log.startsWith('. Error Message: ')) {
                continue;
            }

            let message = log.slice('. Error Message: '.length);

            return new RpcFankorError(code, name, message);
        }

        return null;
    }

    // METHODS ----------------------------------------------------------------

    toString(buffer?: Buffer) {
        return `FankorError ${this.name} ${this.code}(0x${this.code.toString(
            16
        )}): ${this.message}`;
    }

    equals(other: RpcFankorError) {
        return (
            this.code === other.code &&
            this.name === other.name &&
            this.message === other.message
        );
    }
}
