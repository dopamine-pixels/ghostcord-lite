#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

mod commands;
mod config;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::ConfigState(Mutex::new(
            config::AppConfig::default(),
        )))
        .invoke_handler(tauri::generate_handler![
            commands::load_config,
            commands::save_config,
            commands::read_theme_file,
            commands::pick_theme_file,
            commands::apply_config_to_main
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

  function applyAllFromConfig(cfg) {
    applyPerfFromConfig(cfg);
    applyThemeFromConfig(cfg);
  }

  window.__GHOSTCORD_APPLY_CONFIG__ = (cfg) => {
    applyAllFromConfig(cfg);
  };

  // Initial apply
  ensureRuntime();
  applyPerfCss();

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

            if let Ok(cfg) = commands::load_config(
                app.handle().clone(),
                app.state::<commands::ConfigState>(),
            ) {
                let _ = commands::apply_config_to_main(app.handle().clone(), cfg);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running Ghostcord Lite");
}
