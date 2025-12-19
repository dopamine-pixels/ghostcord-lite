export async function maybeLoadVencord() {
  const state = (window as any).__GHOSTCORD__;
  if (state.vencordLoaded) return;

  // placeholder for vencord plugins
  console.warn("[Ghostcord] Vencord mode enabled (stub). Implement loader here.");
  state.vencordLoaded = true;
}
