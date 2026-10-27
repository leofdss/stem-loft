import { TestBed } from '@angular/core/testing';
import { Ipc } from './ipc';

/**
 * Pins every `Command`'s name and argument keys to what
 * src-tauri/src/ipc/commands.rs exposes (Tauri maps its snake_case
 * parameters to camelCase), so the two sides can't drift apart silently.
 */
describe('Ipc', () => {
  let calls: { command: string; args?: Record<string, unknown> }[];
  let ipc: Ipc;

  beforeEach(() => {
    calls = [];
    Object.defineProperty(window, '__TAURI__', {
      configurable: true,
      value: {
        core: {
          invoke: (command: string, args?: Record<string, unknown>) => {
            calls.push({ command, args });
            return Promise.resolve(undefined);
          },
        },
        event: { listen: () => Promise.resolve(() => undefined) },
      },
    });
    ipc = TestBed.inject(Ipc);
  });

  afterEach(() => {
    Reflect.deleteProperty(window, '__TAURI__');
  });

  it('sends each Command with the names the Rust core expects', async () => {
    await ipc.isProjectOpen();
    await ipc.createProject('/songs/study', 'Study');
    await ipc.openProject('/songs/study/project.json');
    await ipc.closeProject();
    await ipc.importStems(['/a/bass.wav']);
    await ipc.setMarkers(1, 2);
    await ipc.play();
    await ipc.pause();
    await ipc.stop();
    await ipc.setStemVolume('bass', 0.5);
    await ipc.setStemMute('bass', true);
    await ipc.setStemSolo('bass', false);

    expect(calls).toEqual([
      { command: 'is_project_open', args: undefined },
      { command: 'create_project', args: { dir: '/songs/study', name: 'Study' } },
      { command: 'open_project', args: { path: '/songs/study/project.json' } },
      { command: 'close_project', args: undefined },
      { command: 'import_stems', args: { filePaths: ['/a/bass.wav'] } },
      { command: 'set_markers', args: { startSec: 1, endSec: 2 } },
      { command: 'play', args: undefined },
      { command: 'pause', args: undefined },
      { command: 'stop', args: undefined },
      { command: 'set_stem_volume', args: { stemId: 'bass', volume: 0.5 } },
      { command: 'set_stem_mute', args: { stemId: 'bass', muted: true } },
      { command: 'set_stem_solo', args: { stemId: 'bass', solo: false } },
    ]);
  });
});
