import { Component, inject, signal } from '@angular/core';
import { Ipc } from '../../core/ipc';

/**
 * Flow for selecting stem files and importing them into the project. Fires
 * `import_stems` as soon as the user confirms a selection -- no client-side
 * queue, no optimistic update: the file list isn't added to any "project"
 * Signal here, since this component has no such state yet (that will come
 * from a `Project`-shaped Event once `Stem Importer` is implemented).
 */
@Component({
  selector: 'stemloft-stem-import-screen',
  templateUrl: './stem-import-screen.html',
  styleUrl: './stem-import-screen.css',
})
export class StemImportScreen {
  private readonly ipc = inject(Ipc);

  protected readonly pending = signal(false);
  protected readonly lastError = signal<string | null>(null);
  protected readonly selectedNames = signal<readonly string[]>([]);

  protected onFilesSelected(event: Event): void {
    const input = event.target as HTMLInputElement;
    const files = input.files ? Array.from(input.files) : [];
    this.selectedNames.set(files.map((file) => file.name));
  }

  protected async onImport(): Promise<void> {
    // A browser <input type="file"> only exposes each File's name, not its
    // full filesystem path -- wiring a real path (or a Tauri file-picker
    // dialog plugin) is part of implementing `import_stems` for real, not a
    // scaffold concern. This sends what's already available so the round
    // trip to the (stub) Command is real, not simulated.
    const filePaths = this.selectedNames();
    if (filePaths.length === 0) {
      return;
    }

    this.pending.set(true);
    this.lastError.set(null);
    try {
      await this.ipc.importStems([...filePaths]);
    } catch (error) {
      this.lastError.set(String(error));
    } finally {
      this.pending.set(false);
    }
  }
}
