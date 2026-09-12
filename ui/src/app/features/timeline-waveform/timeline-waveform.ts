import { Component } from '@angular/core';

/**
 * Waveform visualization, loop-section selection, and marker positioning.
 * Empty-state only for now: `waveform_ready` isn't emitted by anything yet
 * (`Audio Engine` doesn't compute waveforms until stem import and playback
 * are implemented). See docs/architecture.md#events-table.
 */
@Component({
  selector: 'stemloft-timeline-waveform',
  templateUrl: './timeline-waveform.html',
  styleUrl: './timeline-waveform.css',
})
export class TimelineWaveform {}
