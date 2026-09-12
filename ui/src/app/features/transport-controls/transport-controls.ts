import { Component, inject, signal, type WritableSignal } from '@angular/core';
import { Ipc } from '../../core/ipc';

/**
 * Play, pause, and stop. Each button disables itself from the click until
 * its own `Command`'s response arrives -- preventing a double click from
 * resending it -- while the other two stay free to fire their own
 * `Command`s independently. This is the UI reflecting "in progress," not a
 * queue: the queue itself lives in the Rust core. See
 * docs/architecture.md#design-note-commands-queue-in-the-rust-core.
 *
 * No optimistic update: this component doesn't have (and shouldn't invent)
 * a "currently playing" Signal of its own yet -- that state will come from
 * the `transport_state_changed` event once `Audio Engine` emits it. Until
 * then, a rejected `Command` is only shown as inline status text, not
 * silently swallowed.
 */
@Component({
  selector: 'stemloft-transport-controls',
  templateUrl: './transport-controls.html',
  styleUrl: './transport-controls.css',
})
export class TransportControls {
  private readonly ipc = inject(Ipc);

  protected readonly playPending = signal(false);
  protected readonly pausePending = signal(false);
  protected readonly stopPending = signal(false);
  protected readonly lastError = signal<string | null>(null);

  protected async onPlay(): Promise<void> {
    await this.run(this.playPending, () => this.ipc.play());
  }

  protected async onPause(): Promise<void> {
    await this.run(this.pausePending, () => this.ipc.pause());
  }

  protected async onStop(): Promise<void> {
    await this.run(this.stopPending, () => this.ipc.stop());
  }

  private async run(pending: WritableSignal<boolean>, call: () => Promise<void>) {
    pending.set(true);
    this.lastError.set(null);
    try {
      await call();
    } catch (error) {
      this.lastError.set(String(error));
    } finally {
      pending.set(false);
    }
  }
}
