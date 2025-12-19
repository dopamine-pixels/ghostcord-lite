use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub theme_path: Option<String>,
    pub theme_css: Option<String>,
    pub enable_theme: bool,
    pub enable_blockers: bool,
    pub enable_perf_css: bool,
    pub enable_vencord: bool,
}

impl AppConfig {
    pub fn sanitize(mut self) -> Self {
        // sensible defaults for 8GB machines
        if self.enable_blockers == false
            && self.enable_perf_css == false
            && self.enable_vencord == false
            && self.enable_theme == false
            && self.theme_path.is_none()
            && self.theme_css.is_none()
        {
            self.enable_blockers = true;
            self.enable_perf_css = true;
            self.enable_vencord = false;
        }
        self
    }
}
