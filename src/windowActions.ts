export interface WindowActions {
  minimize(): Promise<void>;
  toggleMaximize(): Promise<void>;
  close(): Promise<void>;
}

async function withDesktopWindow(action: "minimize" | "toggleMaximize" | "close"): Promise<void> {
  if (!("__TAURI_INTERNALS__" in globalThis)) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow()[action]();
}

export const desktopWindowActions: WindowActions = {
  minimize: () => withDesktopWindow("minimize"),
  toggleMaximize: () => withDesktopWindow("toggleMaximize"),
  close: () => withDesktopWindow("close"),
};
