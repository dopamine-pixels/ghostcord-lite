#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{WebviewUrl, WebviewWindowBuilder};

mod commands;
mod config;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::load_config,
            commands::save_config,
            commands::read_theme_file,
            commands::pick_theme_file
        ])
        .setup(|app| {
            // This script is injected into EVERY page load, before any JS runs
            let ghostcord_init = r#"
(() => {
  if (window.__GHOSTCORD_BOOTSTRAPPED__) return;
  window.__GHOSTCORD_BOOTSTRAPPED__ = true;

  console.log(
    "%c[Ghostcord] injected (init script)",
    "color:#7fffd4;font-weight:bold"
  );

  const PERF_STYLE_ID = "__ghostcord_perf_css__";
  const THEME_STYLE_ID = "ghostcord-theme";

  const PERF_CSS = `
    /* === Ghostcord Performance Mode === */

    /* Kill animations & transitions */
    *, *::before, *::after {
      animation: none !important;
      transition: none !important;
    }

    /* Disable expensive blur/backdrop effects */
    [style*="backdrop-filter"],
    [class*="backdrop"],
    [class*="blur"] {
      backdrop-filter: none !important;
      filter: none !important;
    }

    /* Remove heavy shadows */
    * {
      box-shadow: none !important;
      text-shadow: none !important;
    }

    /* Reduce GPU churn on avatars/media */
    img, video, canvas {
      will-change: auto !important;
    }

    /* Disable animated avatars/banners */
    [class*="avatar"] *,
    [class*="banner"] * {
      animation: none !important;
    }
  `;

  function applyPerfCss() {
    if (document.getElementById(PERF_STYLE_ID)) return;
    if (!document.documentElement) {
      document.addEventListener("DOMContentLoaded", applyPerfCss, { once: true });
      return;
    }

    const style = document.createElement("style");
    style.id = PERF_STYLE_ID;
    style.textContent = PERF_CSS;
    document.documentElement.appendChild(style);

    console.log("[Ghostcord] performance CSS applied");
  }

  function injectStyle(id, css) {
    if (!document.documentElement) {
      document.addEventListener("DOMContentLoaded", () => injectStyle(id, css), {
        once: true
      });
      return;
    }

    let el = document.getElementById(id);
    if (!el) {
      el = document.createElement("style");
      el.id = id;
      document.documentElement.appendChild(el);
    }
    el.textContent = css;
  }

  function removeStyle(id) {
    const el = document.getElementById(id);
    if (el) el.remove();
  }

  async function applyThemeFromConfig(cfg) {
    if (!cfg || !cfg.enable_theme) {
      removeStyle(THEME_STYLE_ID);
      return;
    }

    if (cfg.theme_css && cfg.theme_css.trim()) {
      injectStyle(THEME_STYLE_ID, cfg.theme_css);
      return;
    }

    if (cfg.theme_path && cfg.theme_path.trim()) {
      try {
        const css = await window.__TAURI__?.core?.invoke("read_theme_file", {
          path: cfg.theme_path
        });
        if (css) injectStyle(THEME_STYLE_ID, css);
      } catch (err) {
        console.warn("[Ghostcord] failed to read theme file", err);
        removeStyle(THEME_STYLE_ID);
      }
      return;
    }

    removeStyle(THEME_STYLE_ID);
  }

  function applyPerfFromConfig(cfg) {
    ensureRuntime();
    window.__GHOSTCORD__.perfEnabled = !!(cfg && cfg.enable_perf_css);
    if (window.__GHOSTCORD__.perfEnabled) applyPerfCss();
    else removeStyle(PERF_STYLE_ID);
  }

  function ensureRuntime() {
    if (!window.__GHOSTCORD__) {
      window.__GHOSTCORD__ = {
        injectedAt: performance.now(),
        perfCss: true,
        perfEnabled: true
      };
    }
  }

  // Initial apply
  ensureRuntime();
  applyPerfCss();

  async function loadAndApplyConfig() {
    const invoke = window.__TAURI__?.core?.invoke;
    if (!invoke) return;
    try {
      const cfg = await invoke("load_config");
      applyPerfFromConfig(cfg);
      await applyThemeFromConfig(cfg);
    } catch (err) {
      console.warn("[Ghostcord] failed to load config", err);
    }
  }

  loadAndApplyConfig();

  // Re-apply after SPA navigation
  const push = history.pushState;
  history.pushState = function (...args) {
    push.apply(this, args);
    ensureRuntime();
    if (window.__GHOSTCORD__.perfEnabled) applyPerfCss();
  };

  window.addEventListener("popstate", () => {
    if (window.__GHOSTCORD__?.perfEnabled) applyPerfCss();
  });
  document.addEventListener("readystatechange", () => {
    if (window.__GHOSTCORD__?.perfEnabled) applyPerfCss();
  });

  const eventApi = window.__TAURI__?.event;
  if (eventApi?.listen) {
    eventApi.listen("config-changed", (evt) => {
      const cfg = evt?.payload;
      applyPerfFromConfig(cfg);
      applyThemeFromConfig(cfg);
    });
  }
})();
"#;


            WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External("https://discord.com/app".parse().unwrap()),
            )
            .title("Ghostcord Lite")
            .inner_size(1100.0, 780.0)
            .resizable(true)
            .initialization_script(ghostcord_init)
            .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running Ghostcord Lite");
}
