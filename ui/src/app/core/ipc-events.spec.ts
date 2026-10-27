import { TestBed } from '@angular/core/testing';
import { IpcEvents } from './ipc-events';
import { Score } from './score';

describe('IpcEvents', () => {
  let handlers: Map<string, (event: { payload: unknown }) => void>;

  beforeEach(() => {
    handlers = new Map();
    Object.defineProperty(window, '__TAURI__', {
      configurable: true,
      value: {
        core: { invoke: () => Promise.resolve(undefined) },
        event: {
          listen: (name: string, handler: (event: { payload: unknown }) => void) => {
            handlers.set(name, handler);
            return Promise.resolve(() => undefined);
          },
        },
      },
    });
  });

  afterEach(() => {
    Reflect.deleteProperty(window, '__TAURI__');
  });

  it('relays score_loaded as the score Signal, as the core sent it', async () => {
    const events = TestBed.inject(IpcEvents);
    await events.listen();
    expect(events.score()).toBeNull();

    const score = { title: 'Study', chords: [] } as unknown as Score;
    handlers.get('score_loaded')!({ payload: score });

    expect(events.score()).toBe(score);
  });

  it('relays score_parse_error for the error modal', async () => {
    const events = TestBed.inject(IpcEvents);
    await events.listen();

    handlers.get('score_parse_error')!({ payload: { line: 3, message: 'two chords' } });

    expect(events.scoreParseError()).toEqual({ line: 3, message: 'two chords' });
  });

  it('drops the previous score when a reload fails', async () => {
    // Added after review.
    const events = TestBed.inject(IpcEvents);
    await events.listen();
    handlers.get('score_loaded')!({ payload: { title: 'Old' } as unknown as Score });

    handlers.get('score_parse_error')!({ payload: { line: 1, message: 'broken' } });

    expect(events.score()).toBeNull();
  });
});
