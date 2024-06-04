// Serial number allocator
// (c) 2024 Ross Younger

import { Mutex } from 'async-mutex'

export class SerialAllocator {
    private _next: number;
    private _mutex: Mutex;
    constructor() {
        this._mutex = new Mutex();
        this._next = 1;
    }
    async next() {
        return await this._mutex.runExclusive(() => {
            var rv = this._next;
            if (++this._next == Number.MAX_SAFE_INTEGER) { this._next = 0; }
            return rv;
        });
    }
}

let _gSerial = new SerialAllocator();
export async function nextSerial(): Promise<number> {
    return _gSerial.next();
}