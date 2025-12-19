declare function invoke<T = any>(cmd: string, args?: Record<string, any>): Promise<T>;

export async function applyTheme(path: string) {
  const css = await invoke<string>("read_theme_file", { path });
  injectStyle("ghostcord-theme", css);
}

export function applyThemeCss(css: string) {
  injectStyle("ghostcord-theme", css);
}

export function applyPerfCss() {
  injectStyle("ghostcord-perf", PERF_CSS);
}

function injectStyle(id: string, css: string) {
  let el = document.getElementById(id) as HTMLStyleElement | null;
  if (!el) {
    el = document.createElement("style");
    el.id = id;
    document.documentElement.appendChild(el);
  }
  el.textContent = css;
}

const PERF_CSS = `
/* Ghostcord performance mode: reduce paint/layout churn */
* { transition: none !important; animation: none !important; }

img, video { image-rendering: auto; }

/* kill expensive blur/backdrop */
[style*="backdrop-filter"], [style*="filter: blur"] {
  backdrop-filter: none !important;
  filter: none !important;
}

/* reduce shadows */
* { box-shadow: none !important; text-shadow: none !important; }

/* optional: reduce profile/banner animations */
[class*="avatar"] * { animation: none !important; }
`;
