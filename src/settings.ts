import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

type AppConfig = {
  theme_path: string | null;
  theme_css: string | null;
  enable_theme: boolean;
  enable_blockers: boolean;
  enable_perf_css: boolean;
  enable_vencord: boolean;
};

const $ = <T extends HTMLElement>(id: string) =>
  document.getElementById(id) as T;

async function load() {
  const cfg = await invoke<AppConfig>("load_config");

  $<HTMLInputElement>("enable_blockers").checked = !!cfg.enable_blockers;
  $<HTMLInputElement>("enable_perf_css").checked = !!cfg.enable_perf_css;
  $<HTMLInputElement>("enable_vencord").checked = !!cfg.enable_vencord;
  $<HTMLInputElement>("enable_theme").checked = !!cfg.enable_theme;
  $<HTMLInputElement>("theme_path").value = cfg.theme_path ?? "";
  $<HTMLTextAreaElement>("theme_css").value = cfg.theme_css ?? "";
}

async function save() {
  const cfg: AppConfig = {
    enable_blockers: $<HTMLInputElement>("enable_blockers").checked,
    enable_perf_css: $<HTMLInputElement>("enable_perf_css").checked,
    enable_vencord: $<HTMLInputElement>("enable_vencord").checked,
    enable_theme: $<HTMLInputElement>("enable_theme").checked,
    theme_path: $<HTMLInputElement>("theme_path").value.trim() || null,
    theme_css: $<HTMLTextAreaElement>("theme_css").value.trim() || null
  };
  await invoke("save_config", { cfg });
  await invoke("apply_config_to_main", { cfg });

  // apply via backend to avoid webview event permission issues
}

$<HTMLButtonElement>("theme_pick").addEventListener("click", async () => {
  const picked = await invoke<string | null>("pick_theme_file");
  if (picked) $<HTMLInputElement>("theme_path").value = picked;
});

$<HTMLButtonElement>("save").addEventListener("click", save);
$<HTMLButtonElement>("close").addEventListener("click", async () => {
  await getCurrentWindow().hide();
});

load();
