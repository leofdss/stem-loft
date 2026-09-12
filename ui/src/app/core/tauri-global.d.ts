/**
 * Ambient types for the subset of Tauri v2's injected `window.__TAURI__`
 * global that this app uses.
 *
 * Deliberately typed by hand instead of importing `@tauri-apps/api`: the
 * Angular layer uses only native framework features, no third-party (npm)
 * package beyond what Angular itself already provides (see
 * docs/architecture.md#stack-and-platform). Tauri's `app.withGlobalTauri`
 * option (set in `src-tauri/tauri.conf.json`) injects this object into the
 * WebView itself, at runtime -- no npm install needed to call into it, only
 * these type declarations so TypeScript knows its shape.
 */
export {};

declare global {
  interface Window {
    readonly __TAURI__: {
      readonly core: {
        invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
      };
      readonly event: {
        listen<T>(
          event: string,
          handler: (event: { payload: T }) => void,
        ): Promise<() => void>;
      };
    };
  }
}
