// Generic error handling
// (c) 2024 Ross Younger

import { message } from '@tauri-apps/api/dialog';
import { UnlistenFn, listen } from '@tauri-apps/api/event'

import { GenericError } from './engine_types';

export class ErrorHandler {
  unlisten: UnlistenFn | null = null;

  constructor() {
  }
  async bind_events() {
    this.unlisten = await listen<GenericError>('genericError', (event) => this.on_error(event.payload));
  }

  async on_error(err: GenericError) {
    console.error(`Error from engine: ${err.error}`);
    await message(err.error, { title: 'brot3 engine error', type: 'error' });
  }
}
