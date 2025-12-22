(() => {
  if (window.__GHOSTCORD_BOOTSTRAPPED__) return;
  window.__GHOSTCORD_BOOTSTRAPPED__ = true;

  console.log("%c[Ghostcord] Injected", "color:#7fffd4;font-weight:bold");

  const PERF_STYLE_ID = "__ghostcord_perf_css__";
  const THEME_STYLE_ID = "ghostcord-theme";
  const SETTINGS_PANEL_ID = "__ghostcord_settings_panel__";
  const SETTINGS_ITEM_ID = "__ghostcord_settings_item__";

  const PERF_CSS = `
    /* Ghostcord Performance Mode */
    *, *::before, *::after {
      animation: none !important;
      transition: none !important;
    }
    [style*="backdrop-filter"], [class*="backdrop"], [class*="blur"] {
      backdrop-filter: none !important;
      filter: none !important;
    }
    * {
      box-shadow: none !important;
      text-shadow: none !important;
    }
    img, video, canvas {
      will-change: auto !important;
    }
    [class*="avatar"] *, [class*="banner"] * {
      animation: none !important;
    }
  `;

  function injectStyle(id, css) {
    if (!document.documentElement) {
      document.addEventListener("DOMContentLoaded", () => injectStyle(id, css), { once: true });
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
    document.getElementById(id)?.remove();
  }

  function ensureRuntime() {
    if (!window.__GHOSTCORD__) {
      window.__GHOSTCORD__ = {
        injectedAt: performance.now(),
        perfEnabled: true,
        blockersEnabled: false,
        blockersInstalled: false,
        blockersOriginals: null,
        currentConfig: null
      };
    }
  }

  function applyPerfCss() {
    injectStyle(PERF_STYLE_ID, PERF_CSS);
  }

  function applyThemeFromConfig(cfg) {
    if (!cfg?.enable_theme) {
      removeStyle(THEME_STYLE_ID);
      return;
    }
    if (cfg.theme_css?.trim()) {
      injectStyle(THEME_STYLE_ID, cfg.theme_css);
    } else {
      removeStyle(THEME_STYLE_ID);
    }
  }

  function applyPerfFromConfig(cfg) {
    ensureRuntime();
    window.__GHOSTCORD__.perfEnabled = !!(cfg?.enable_perf_css);
    if (window.__GHOSTCORD__.perfEnabled) {
      applyPerfCss();
    } else {
      removeStyle(PERF_STYLE_ID);
    }
  }

  function applyVencordFromConfig(cfg) {
    ensureRuntime();
    const enabled = !!(cfg?.enable_vencord);
    if (enabled && !window.__GHOSTCORD__.vencordLoaded) {
      if (!window.__TAURI__?.core?.invoke) return;
      window.__GHOSTCORD__.vencordLoaded = true;
      window.__TAURI__.core
        .invoke('apply_vencord_to_main')
        .catch((err) => {
          window.__GHOSTCORD__.vencordLoaded = false;
          console.warn('[Ghostcord] Vencord loader failed', err);
        });
    }
  }

  function getUrlString(input) {
    if (!input) return '';
    if (typeof input === 'string') return input;
    if (input.url) return input.url;
    return '';
  }

  function isBlockedUrl(url, method) {
    if (!url) return false;
    try {
      const parsed = new URL(url, window.location.origin);
      const host = parsed.hostname.toLowerCase();
      const pathname = parsed.pathname.toLowerCase();
      const blockedHost = host.includes('discord.com') || host.includes('discordapp.com');
      if (!blockedHost && !host.includes('sentry.io') && !host.includes('sentry.discord')) {
        return false;
      }
      if (host.includes('sentry.io') || host.includes('sentry.discord')) {
        return true;
      }
      if (method && method.toUpperCase() !== 'POST') {
        return false;
      }
      return (
        /\/api\/v\d+\/science\b/.test(pathname) ||
        /\/api\/v\d+\/track\b/.test(pathname) ||
        /\/api\/v\d+\/users\/@me\/analytics\b/.test(pathname) ||
        /\/api\/v\d+\/applications\/\d+\/analytics\b/.test(pathname)
      );
    } catch (_) {
      return false;
    }
  }

  function installBlockers() {
    ensureRuntime();
    if (window.__GHOSTCORD__.blockersInstalled) return;

    const originals = {
      fetch: window.fetch,
      xhrOpen: window.XMLHttpRequest && window.XMLHttpRequest.prototype.open,
      xhrSend: window.XMLHttpRequest && window.XMLHttpRequest.prototype.send,
      sendBeacon: navigator.sendBeacon
    };

    window.fetch = function(input, init) {
      try {
        const url = getUrlString(input);
        const method = init?.method || input?.method;
        if (isBlockedUrl(url, method)) {
          console.warn('[Ghostcord] Blocked fetch:', url);
          return Promise.resolve(new Response('', { status: 204 }));
        }
      } catch (err) {
        console.warn('[Ghostcord] fetch blocker error', err);
      }
      return originals.fetch.call(this, input, init);
    };

    if (window.XMLHttpRequest && originals.xhrOpen && originals.xhrSend) {
      window.XMLHttpRequest.prototype.open = function(method, url, async, user, password) {
        try {
          this.__ghostcord_method = method;
          this.__ghostcord_blocked = isBlockedUrl(url, method);
          if (this.__ghostcord_blocked) {
            console.warn('[Ghostcord] Blocked XHR:', url);
          }
        } catch (err) {
          console.warn('[Ghostcord] XHR open blocker error', err);
        }
        return originals.xhrOpen.call(this, method, url, async, user, password);
      };
      window.XMLHttpRequest.prototype.send = function(body) {
        if (this.__ghostcord_blocked) {
          try {
            this.abort();
          } catch (_) {}
          return;
        }
        return originals.xhrSend.call(this, body);
      };
    }

    if (originals.sendBeacon) {
      navigator.sendBeacon = function(url, data) {
        try {
          if (isBlockedUrl(url, 'POST')) {
            console.warn('[Ghostcord] Blocked beacon:', url);
            return true;
          }
        } catch (err) {
          console.warn('[Ghostcord] beacon blocker error', err);
        }
        return originals.sendBeacon.call(this, url, data);
      };
    }

    window.__GHOSTCORD__.blockersOriginals = originals;
    window.__GHOSTCORD__.blockersInstalled = true;
  }

  function uninstallBlockers() {
    ensureRuntime();
    if (!window.__GHOSTCORD__.blockersInstalled) return;
    const originals = window.__GHOSTCORD__.blockersOriginals;
    if (originals?.fetch) window.fetch = originals.fetch;
    if (originals?.xhrOpen) window.XMLHttpRequest.prototype.open = originals.xhrOpen;
    if (originals?.xhrSend) window.XMLHttpRequest.prototype.send = originals.xhrSend;
    if (originals?.sendBeacon) navigator.sendBeacon = originals.sendBeacon;
    window.__GHOSTCORD__.blockersInstalled = false;
  }

  function applyBlockersFromConfig(cfg) {
    ensureRuntime();
    window.__GHOSTCORD__.blockersEnabled = !!(cfg?.enable_blockers);
    if (window.__GHOSTCORD__.blockersEnabled) {
      installBlockers();
    } else {
      uninstallBlockers();
    }
  }

  function applyAllFromConfig(cfg) {
    window.__GHOSTCORD__.currentConfig = cfg;
    applyPerfFromConfig(cfg);
    applyThemeFromConfig(cfg);
    applyVencordFromConfig(cfg);
    applyBlockersFromConfig(cfg);
  }

  window.__GHOSTCORD_APPLY_CONFIG__ = applyAllFromConfig;

  // Settings Panel Creation
  function createSettingsPanel() {
    const panel = document.createElement("div");
    panel.id = SETTINGS_PANEL_ID;
    panel.style.cssText = `
      display: none;
      padding: 60px 40px 80px;
      max-width: 740px;
      min-height: 100%;
      color: var(--text-normal, #dcddde);
    `;

    panel.innerHTML = `
      <style>
        #${SETTINGS_PANEL_ID} h1 {
          font-size: 20px;
          line-height: 24px;
          font-weight: 600;
          margin: 0 0 20px;
          color: var(--header-primary, #fff);
        }
        #${SETTINGS_PANEL_ID} h2 {
          font-size: 12px;
          line-height: 16px;
          font-weight: 700;
          text-transform: uppercase;
          margin: 20px 0 8px;
          color: var(--header-secondary, #b9bbbe);
        }
        #${SETTINGS_PANEL_ID} .setting-row {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 12px 0;
          border-bottom: 1px solid var(--background-modifier-accent, #4f545c);
        }
        #${SETTINGS_PANEL_ID} .setting-label {
          flex: 1;
        }
        #${SETTINGS_PANEL_ID} .setting-label-title {
          font-size: 16px;
          font-weight: 500;
          margin-bottom: 4px;
        }
        #${SETTINGS_PANEL_ID} .setting-label-desc {
          font-size: 14px;
          color: var(--text-muted, #72767d);
        }
        #${SETTINGS_PANEL_ID} .switch {
          position: relative;
          display: inline-block;
          width: 44px;
          height: 24px;
          background: var(--background-accent, #4f545c);
          border-radius: 12px;
          cursor: pointer;
          transition: background 0.15s;
        }
        #${SETTINGS_PANEL_ID} .switch.active {
          background: var(--brand-experiment, #5865f2);
        }
        #${SETTINGS_PANEL_ID} .switch-slider {
          position: absolute;
          top: 3px;
          left: 3px;
          width: 18px;
          height: 18px;
          background: white;
          border-radius: 50%;
          transition: transform 0.15s;
        }
        #${SETTINGS_PANEL_ID} .switch.active .switch-slider {
          transform: translateX(20px);
        }
        #${SETTINGS_PANEL_ID} textarea {
          width: 100%;
          min-height: 100px;
          padding: 10px;
          background: var(--background-secondary, #2f3136);
          border: 1px solid var(--background-tertiary, #202225);
          border-radius: 3px;
          color: var(--text-normal, #dcddde);
          font-family: 'Consolas', 'Monaco', monospace;
          font-size: 14px;
          resize: vertical;
          margin-top: 8px;
        }
        #${SETTINGS_PANEL_ID} .button-row {
          display: flex;
          gap: 8px;
          margin-top: 12px;
        }
        #${SETTINGS_PANEL_ID} button {
          padding: 8px 16px;
          border: none;
          border-radius: 3px;
          font-weight: 500;
          cursor: pointer;
          font-size: 14px;
        }
        #${SETTINGS_PANEL_ID} .btn-primary {
          background: var(--brand-experiment, #5865f2);
          color: white;
        }
        #${SETTINGS_PANEL_ID} .btn-secondary {
          background: var(--background-accent, #4f545c);
          color: var(--text-normal, #dcddde);
        }
        #${SETTINGS_PANEL_ID} .status-message {
          padding: 10px;
          margin: 10px 0;
          border-radius: 3px;
          display: none;
        }
        #${SETTINGS_PANEL_ID} .status-message.success {
          background: rgba(59, 165, 93, 0.3);
          color: #3ba55d;
        }
        #${SETTINGS_PANEL_ID} .status-message.error {
          background: rgba(237, 66, 69, 0.3);
          color: #ed4245;
        }
        #${SETTINGS_PANEL_ID} input[type="text"] {
          flex: 1;
          padding: 10px;
          background: var(--background-secondary, #2f3136);
          border: 1px solid var(--background-tertiary, #202225);
          border-radius: 3px;
          color: var(--text-normal, #dcddde);
          font-size: 14px;
        }
        #${SETTINGS_PANEL_ID} .file-input-row {
          display: flex;
          gap: 8px;
          margin-top: 8px;
        }
      </style>

      <h1>⚡ Ghostcord Lite</h1>
      <div id="status-message" class="status-message"></div>

      <h2>Performance</h2>
      <div class="setting-row">
        <div class="setting-label">
          <div class="setting-label-title">Analytics Blocker</div>
          <div class="setting-label-desc">Block Discord analytics and tracking</div>
        </div>
        <div class="switch" id="switch-blockers">
          <div class="switch-slider"></div>
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-label">
          <div class="setting-label-title">Performance Mode</div>
          <div class="setting-label-desc">Remove animations and effects for better performance</div>
        </div>
        <div class="switch" id="switch-perf">
          <div class="switch-slider"></div>
        </div>
      </div>

      <h2>Customization</h2>
      <div class="setting-row">
        <div class="setting-label">
          <div class="setting-label-title">Custom Theme</div>
          <div class="setting-label-desc">Enable BetterDiscord CSS themes</div>
        </div>
        <div class="switch" id="switch-theme">
          <div class="switch-slider"></div>
        </div>
      </div>

      <div style="margin-top: 12px;">
        <div class="setting-label-desc" style="margin-bottom: 4px;">Theme File (.theme.css)</div>
        <div class="file-input-row">
          <input type="text" id="theme-path" placeholder="Select a theme file..." />
          <button class="btn-secondary" id="btn-browse">Browse</button>
        </div>
        <div class="setting-label-desc" style="margin-top: 8px;">Or paste CSS directly below (overrides file):</div>
        <textarea id="theme-css" placeholder="/* Paste custom CSS here */"></textarea>
      </div>

      <h2>Plugins</h2>
      <div class="setting-row">
        <div class="setting-label">
          <div class="setting-label-title">Vencord (Experimental)</div>
          <div class="setting-label-desc">Enable the Vencord plugin loader</div>
        </div>
        <div class="switch" id="switch-vencord">
          <div class="switch-slider"></div>
        </div>
      </div>
      <div class="setting-label-desc" style="margin-top: 6px;">
        Vencord is GPL-3.0. Source: <a href="https://github.com/Vencord/Vencord" target="_blank" rel="noreferrer">github.com/Vencord/Vencord</a>
      </div>

      <div class="button-row">
        <button class="btn-primary" id="btn-save">Save & Apply</button>
        <button class="btn-secondary" id="btn-reload">Reload Config</button>
      </div>
    `;

    return panel;
  }

  function showStatus(message, isError = false) {
    const status = document.querySelector('#status-message');
    if (!status) return;
    status.textContent = message;
    status.className = 'status-message ' + (isError ? 'error' : 'success');
    status.style.display = 'block';
    setTimeout(() => {
      status.style.display = 'none';
    }, 3000);
  }

  async function loadConfigToUI() {
    try {
      const cfg = await window.__TAURI__.core.invoke('load_config');
      window.__GHOSTCORD__.currentConfig = cfg;

      const toggleSwitch = (id, value) => {
        const sw = document.getElementById(id);
        if (sw) sw.classList.toggle('active', !!value);
      };

      toggleSwitch('switch-blockers', cfg.enable_blockers);
      toggleSwitch('switch-perf', cfg.enable_perf_css);
      toggleSwitch('switch-theme', cfg.enable_theme);
      toggleSwitch('switch-vencord', cfg.enable_vencord);

      const pathInput = document.getElementById('theme-path');
      const cssInput = document.getElementById('theme-css');
      if (pathInput) pathInput.value = cfg.theme_path || '';
      if (cssInput) cssInput.value = cfg.theme_css || '';

      console.log('[Ghostcord] Config loaded to UI');
    } catch (err) {
      console.error('[Ghostcord] Failed to load config:', err);
      showStatus('Failed to load settings', true);
    }
  }

  function applyConfigToUI(cfg) {
    if (!cfg) return;
    const toggleSwitch = (id, value) => {
      const sw = document.getElementById(id);
      if (sw) sw.classList.toggle('active', !!value);
    };

    toggleSwitch('switch-blockers', cfg.enable_blockers);
    toggleSwitch('switch-perf', cfg.enable_perf_css);
    toggleSwitch('switch-theme', cfg.enable_theme);
    toggleSwitch('switch-vencord', cfg.enable_vencord);

    const pathInput = document.getElementById('theme-path');
    const cssInput = document.getElementById('theme-css');
    if (pathInput) pathInput.value = cfg.theme_path || '';
    if (cssInput) cssInput.value = cfg.theme_css || '';
  }

  async function saveConfigFromUI() {
    try {
      const getSwitch = (id) => document.getElementById(id)?.classList.contains('active');
      
      const cfg = {
        enable_blockers: getSwitch('switch-blockers'),
        enable_perf_css: getSwitch('switch-perf'),
        enable_vencord: getSwitch('switch-vencord'),
        enable_theme: getSwitch('switch-theme'),
        theme_path: document.getElementById('theme-path')?.value.trim() || null,
        theme_css: document.getElementById('theme-css')?.value.trim() || null
      };

      await window.__TAURI__.core.invoke('save_config', { cfg });
      await window.__TAURI__.core.invoke('apply_config_to_main', { cfg });
      
      showStatus('✓ Settings saved and applied!');
      console.log('[Ghostcord] Config saved:', cfg);
    } catch (err) {
      console.error('[Ghostcord] Failed to save config:', err);
      showStatus('Failed to save: ' + err, true);
    }
  }

  async function browseThemeFile() {
    try {
      const picked = await window.__TAURI__.core.invoke('pick_theme_file');
      if (picked) {
        const pathInput = document.getElementById('theme-path');
        if (pathInput) pathInput.value = picked;
        console.log('[Ghostcord] Theme file selected:', picked);
      }
    } catch (err) {
      console.error('[Ghostcord] Failed to pick file:', err);
      showStatus('Failed to select file', true);
    }
  }

  function attachSettingsPanelHandlers() {
    const panel = document.getElementById(SETTINGS_PANEL_ID);
    if (!panel) return;

    // Toggle switches
    ['switch-blockers', 'switch-perf', 'switch-theme', 'switch-vencord'].forEach(id => {
      const sw = document.getElementById(id);
      if (sw) {
        sw.addEventListener('click', () => {
          sw.classList.toggle('active');
        });
      }
    });

    // Buttons
    const btnSave = document.getElementById('btn-save');
    const btnReload = document.getElementById('btn-reload');
    const btnBrowse = document.getElementById('btn-browse');

    if (btnSave) btnSave.addEventListener('click', saveConfigFromUI);
    if (btnReload) btnReload.addEventListener('click', loadConfigToUI);
    if (btnBrowse) btnBrowse.addEventListener('click', browseThemeFile);

    // Load initial config
    loadConfigToUI();
  }

  function showSettingsPanel() {
    const panel = document.getElementById(SETTINGS_PANEL_ID);
    if (!panel) return;

    // Hide other settings panels
    const contentRegion = document.querySelector('[class*="contentRegion"]');
    if (contentRegion) {
      const children = contentRegion.children;
      for (let i = 0; i < children.length; i++) {
        children[i].style.display = 'none';
      }
    }

    panel.style.display = 'block';
    loadConfigToUI();
  }

  function injectSettingsPanel() {
    // Find the settings content region
    const contentRegion = document.querySelector('[class*="contentRegion"]') || 
                         document.querySelector('[class*="content"]');
    
    if (!contentRegion) {
      console.warn('[Ghostcord] Settings content region not found, retrying...');
      setTimeout(injectSettingsPanel, 1000);
      return;
    }

    if (document.getElementById(SETTINGS_PANEL_ID)) return;

    const panel = createSettingsPanel();
    contentRegion.appendChild(panel);
    attachSettingsPanelHandlers();
    
    console.log('[Ghostcord] Settings panel injected');
  }

  function findSettingsNav() {
    const normalize = (text) => (text || '').toLowerCase().replace(/\s+/g, ' ').trim();

    const ariaNav = document.querySelector('nav[aria-label*="User Settings"]');
    if (ariaNav) return ariaNav;

    const modal = document.querySelector('[role="dialog"], [aria-modal="true"]');
    const scope = modal || document;

    const tablists = Array.from(scope.querySelectorAll('[role="tablist"]'));
    for (const el of tablists) {
      const text = normalize(el.textContent || '');
      if (text.includes('my account') || text.includes('user settings')) {
        return el;
      }
    }

    const tabs = Array.from(scope.querySelectorAll('[role="tab"]'));
    for (const tab of tabs) {
      const text = normalize(tab.textContent || '');
      if (text.includes('my account') || text.includes('user settings')) {
        return tab.closest('[role="tablist"]') || tab.parentElement;
      }
    }

    const ariaTablist = Array.from(scope.querySelectorAll('[aria-label*="Settings"]'));
    for (const el of ariaTablist) {
      const role = el.getAttribute('role');
      if (role === 'tablist') return el;
    }

    return null;
  }

  function ensureSettingsMenuItem() {
    const nav = findSettingsNav();
    if (!nav) {
      setTimeout(ensureSettingsMenuItem, 1000);
      return;
    }
    if (document.getElementById(SETTINGS_ITEM_ID)) return;

    const item = document.createElement('div');
    item.id = SETTINGS_ITEM_ID;
    item.setAttribute('role', 'tab');
    item.textContent = 'Ghostcord Lite';
    item.style.cssText = `
      padding: 6px 10px;
      margin: 2px 0;
      border-radius: 4px;
      cursor: pointer;
      user-select: none;
      font-size: 16px;
      font-weight: 500;
      color: var(--interactive-normal, #b9bbbe);
    `;
    
    item.addEventListener('mouseenter', () => {
      item.style.background = 'var(--background-modifier-hover, rgba(79, 84, 92, 0.16))';
    });
    item.addEventListener('mouseleave', () => {
      item.style.background = 'transparent';
    });
    item.addEventListener('click', () => {
      // Remove active state from other items
      nav.querySelectorAll('[role="tab"]').forEach(tab => {
        tab.style.background = 'transparent';
      });
      item.style.background = 'var(--background-modifier-selected, rgba(79, 84, 92, 0.32))';
      
      injectSettingsPanel();
      showSettingsPanel();
    });

    nav.appendChild(item);
    console.log('[Ghostcord] Settings menu item added');
  }

  function setupSettingsMenuObserver() {
    const observer = new MutationObserver(() => {
      ensureSettingsMenuItem();
    });
    
    const target = document.body || document.documentElement;
    if (target) {
      observer.observe(target, { childList: true, subtree: true });
    }
    
    ensureSettingsMenuItem();
  }

  function setupDevtoolsShortcut() {
    document.addEventListener('keydown', (e) => {
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === 'i') {
        e.preventDefault();
        e.stopPropagation();
        if (window.__TAURI__) {
          window.__TAURI__.window.getCurrent().openDevtools();
        }
      }
    }, true);
  }


  // Initialize
  ensureRuntime();
  applyPerfCss();
  applyBlockersFromConfig(window.__GHOSTCORD__.currentConfig);
  
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
      setupSettingsMenuObserver();
      setupDevtoolsShortcut();
    });
  } else {
    setupSettingsMenuObserver();
    setupDevtoolsShortcut();
  }

  // Re-inject on navigation
  const pushState = history.pushState;
  history.pushState = function(...args) {
    pushState.apply(this, args);
    setTimeout(() => {
      ensureSettingsMenuItem();
      if (window.__GHOSTCORD__?.perfEnabled) applyPerfCss();
    }, 100);
  };

  window.addEventListener('popstate', () => {
    setTimeout(() => {
      ensureSettingsMenuItem();
      if (window.__GHOSTCORD__?.perfEnabled) applyPerfCss();
    }, 100);
  });

})();
