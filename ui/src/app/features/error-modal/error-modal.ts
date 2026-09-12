import { Component, computed, inject } from '@angular/core';
import { IpcEvents } from '../../core/ipc-events';

interface CurrentError {
  readonly title: string;
  readonly message: string;
}

/**
 * `audio_error` and `score_parse_error` are always presented as a modal --
 * never a toast or a persistent banner -- see
 * docs/architecture.md#design-note-error-presentation-modal.
 *
 * This only presents the error; dismissing it doesn't pause or undo
 * anything already running in the core. `score_parse_error` in particular
 * must not halt the rest of the app -- `Chord/Tab View` sits in its own
 * error state independently of this modal (see that component), the rest
 * of the screen keeps working normally either way.
 */
@Component({
  selector: 'stemloft-error-modal',
  templateUrl: './error-modal.html',
  styleUrl: './error-modal.css',
})
export class ErrorModal {
  private readonly ipcEvents = inject(IpcEvents);

  /**
   * Whichever error arrived most recently drives the modal -- the two
   * events aren't expected to be simultaneous in practice, and the modal
   * showing one at a time matches "isn't reopened until the next
   * `score_parse_error`" in the design note.
   */
  protected readonly current = computed<CurrentError | null>(() => {
    const audioError = this.ipcEvents.audioError();
    if (audioError) {
      return { title: 'Audio error', message: audioError.message };
    }

    const scoreParseError = this.ipcEvents.scoreParseError();
    if (scoreParseError) {
      return {
        title: `Score error (line ${scoreParseError.line})`,
        message: scoreParseError.message,
      };
    }

    return null;
  });

  protected dismiss(): void {
    this.ipcEvents.dismissAudioError();
    this.ipcEvents.dismissScoreParseError();
  }
}
