// See https://svelte.dev/docs/kit/types#app.d.ts

declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    // interface PageState {}
    // interface Platform {}
  }

  interface Window {
    kitodo: {
      invoke<T>(command: string, payload?: Record<string, unknown>): Promise<T>;
      window: {
        isMaximized(): Promise<boolean>;
        toggleMaximize(): Promise<boolean>;
        close(): Promise<void>;
      };
      shell: {
        openExternal(url: string): Promise<void>;
      };
      notifications: {
        getSettings(): Promise<{ enabled: boolean; supported: boolean }>;
        setEnabled(enabled: boolean): Promise<{ enabled: boolean; supported: boolean }>;
      };
    };
  }
}

export {};
