export class FnkBorshError extends Error {
    originalMessage: string;
    fieldPath: string[];

    // CONSTRUCTORS -----------------------------------------------------------

    constructor(message: string) {
        super(message);
        this.fieldPath = [];
        this.originalMessage = message;
    }

    // METHODS ----------------------------------------------------------------

    addToFieldPath(fieldName: string): void {
        this.fieldPath.splice(0, 0, fieldName);
        // NOTE: Modifying message directly as jest doesn't use .toString()
        this.message = this.originalMessage + ': ' + this.fieldPath.join('.');
    }
}
