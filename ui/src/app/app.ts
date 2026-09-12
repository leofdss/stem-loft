import { Component, inject } from '@angular/core';
import { IpcEvents } from './core/ipc-events';
import { ChordTabView } from './features/chord-tab-view/chord-tab-view';
import { ErrorModal } from './features/error-modal/error-modal';
import { StemImportScreen } from './features/stem-import-screen/stem-import-screen';
import { StemMixer } from './features/stem-mixer/stem-mixer';
import { TimelineWaveform } from './features/timeline-waveform/timeline-waveform';
import { TransportControls } from './features/transport-controls/transport-controls';

/**
 * The app's single screen -- see docs/architecture.md's "Presentation
 * Layer — Angular" table. Just lays out the feature components; it holds
 * no state and no logic of its own (see
 * docs/architecture.md#stack-and-platform).
 *
 * `Automatic Separation` isn't mounted here yet -- it's explicitly a
 * future feature (docs/architecture.md#current-status-and-future-work),
 * not part of what's currently in development.
 */
@Component({
  selector: 'stemloft-root',
  imports: [
    ChordTabView,
    ErrorModal,
    StemImportScreen,
    StemMixer,
    TimelineWaveform,
    TransportControls,
  ],
  templateUrl: './app.html',
  styleUrl: './app.css',
})
export class App {
  private readonly ipcEvents = inject(IpcEvents);

  constructor() {
    void this.ipcEvents.listen();
  }
}
