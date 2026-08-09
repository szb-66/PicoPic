export function isTauriRuntime(scope: typeof globalThis = globalThis): boolean {
  return "__TAURI_INTERNALS__" in scope;
}
