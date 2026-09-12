import { Component } from '@angular/core';

/**
 * Volume, mute, and solo per stem. Empty-state only for now: there's no
 * event yet that reports which stems belong to the open project (that
 * arrives once `Stem Importer`/`Project Persistence` are implemented and
 * the project's state is exposed to the UI). Wiring `set_stem_volume`,
 * `set_stem_mute`, and `set_stem_solo` (already available on `Ipc`, see
 * `core/ipc.ts`) to real per-stem controls is the next step once that
 * state exists -- not before, to avoid inventing a stem list on the
 * Angular side that could drift from the core's.
 */
@Component({
  selector: 'stemloft-stem-mixer',
  templateUrl: './stem-mixer.html',
  styleUrl: './stem-mixer.css',
})
export class StemMixer {}
