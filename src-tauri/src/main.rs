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
            commands::apply_config_to_main,
            commands::open_settings
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
  const SETTINGS_ITEM_ID = "__ghostcord_settings_item__";
  const DEVTOOLS_URL = "https://ghostcord.local/devtools";

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

  function openSettingsWindow() {
    try {
      window.location.href = "https://ghostcord.local/settings";
    } catch (err) {
      console.warn("[Ghostcord] open_settings nav failed", err);
    }
  }

  function findSettingsNav() {
    const ariaNav = document.querySelector(
      'nav[aria-label*="User Settings"], div[aria-label*="User Settings"]'
    );
    if (ariaNav) return ariaNav;

    const modal =
      document.querySelector('[role="dialog"]') ||
      document.querySelector('[aria-modal="true"]');
    const scope = modal || document;

    const tablists = Array.from(scope.querySelectorAll('[role="tablist"]'));
    for (const el of tablists) {
      const text = (el.textContent || "").toLowerCase();
      if (text.includes("my account") || text.includes("user settings")) {
        return el;
      }
    }

    const candidates = Array.from(
      scope.querySelectorAll('[role="listbox"], [role="tablist"], nav')
    );
    for (const el of candidates) {
      const text = (el.textContent || "").toLowerCase();
      if (text.includes("user settings") && text.includes("my account")) {
        return el;
      }
    }
    return null;
  }

  function ensureSettingsMenuItem() {
    const nav = findSettingsNav();
    if (!nav) return;
    if (document.getElementById(SETTINGS_ITEM_ID)) return;

    const item = document.createElement("div");
    item.id = SETTINGS_ITEM_ID;
    item.setAttribute("role", "tab");
    item.textContent = "Ghostcord Lite";
    item.style.padding = "6px 10px";
    item.style.margin = "2px 0";
    item.style.borderRadius = "4px";
    item.style.cursor = "pointer";
    item.style.userSelect = "none";
    item.style.fontSize = "14px";
    item.style.color = "var(--interactive-normal, #b5bac1)";
    item.addEventListener("mouseenter", () => {
      item.style.background = "rgba(255,255,255,0.06)";
    });
    item.addEventListener("mouseleave", () => {
      item.style.background = "transparent";
    });
    item.addEventListener("click", openSettingsWindow);

    nav.appendChild(item);
  }

  function setupSettingsMenuObserver() {
    if (!document.documentElement) {
      document.addEventListener("DOMContentLoaded", setupSettingsMenuObserver, {
        once: true
      });
      return;
    }
    const target = document.body || document.documentElement;
    if (!target) return;
    const observer = new MutationObserver(ensureSettingsMenuItem);
    observer.observe(target, {
      childList: true,
      subtree: true
    });
    ensureSettingsMenuItem();
  }

  function setupDevtoolsShortcut() {
    document.addEventListener(
      "keydown",
      (e) => {
        const key = (e.key || "").toLowerCase();
        if ((e.ctrlKey || e.metaKey) && e.shiftKey && key === "i") {
          e.preventDefault();
          e.stopPropagation();
          window.location.href = DEVTOOLS_URL;
        }
      },
      true
    );
  }

  // Initial apply
  ensureRuntime();
  applyPerfCss();
  setupSettingsMenuObserver();
  setupDevtoolsShortcut();

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


            let app_handle = app.handle().clone();
            WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External("https://discord.com/app".parse().unwrap()),
            )
            .title("Ghostcord Lite")
            .inner_size(1100.0, 780.0)
            .resizable(true)
            .on_navigation(move |url| {
                if url.as_str() == "https://ghostcord.local/settings" {
                    let _ = commands::open_settings(app_handle.clone());
                    return false;
                }
                if url.as_str() == "https://ghostcord.local/devtools" {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        window.open_devtools();
                    }
                    return false;
                }
                true
            })
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
