// Serial number allocator
// (c) 2024 Ross Younger

// Note: This used to use async-mutex.
// Observing that JS is single threaded without pre-emption, then provided nextSerial
// is atomic and non-async then the overhead of a mutex isn't necessary.

class SerialAllocator {
    private _next: number;
    constructor() {
        this._next = 1;
    }
    next(): number {
        let rv = this._next;
        if (++this._next == Number.MAX_SAFE_INTEGER) { this._next = 0; }
        return rv;
    }
}

let _gSerial = new SerialAllocator();
export function nextSerial(): number {
    return _gSerial.next();
}
