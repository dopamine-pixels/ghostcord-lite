import { installBlockers } from "./blockers";
import { applyTheme, applyThemeCss, applyPerfCss } from "./theme";
import { maybeLoadVencord } from "./plugins";

type AppConfig = {
  theme_path: string | null;
  theme_css: string | null;
  enable_theme: boolean;
  enable_blockers: boolean;
  enable_perf_css: boolean;
  enable_vencord: boolean;
};

export async function applyRuntime(cfg: AppConfig) {
  // Make it idempotent
  if (!(window as any).__GHOSTCORD__) (window as any).__GHOSTCORD__ = {};
  const state = (window as any).__GHOSTCORD__;

  if (cfg.enable_blockers && !state.blockersInstalled) {
    installBlockers();
    state.blockersInstalled = true;
  }

  if (cfg.enable_perf_css) applyPerfCss();
  else removeStyle("ghostcord-perf");

  if (cfg.enable_theme) {
    if (cfg.theme_css) applyThemeCss(cfg.theme_css);
    else if (cfg.theme_path) await applyTheme(cfg.theme_path);
    else removeStyle("ghostcord-theme");
  } else {
    removeStyle("ghostcord-theme");
  }

  if (cfg.enable_vencord) await maybeLoadVencord();
}

function removeStyle(id: string) {
  const el = document.getElementById(id);
  if (el) el.remove();
}
