use serde::{Deserialize, Serialize};
use crate::lang::Language;

// All persistent user preferences — saved to Nota/settings.json
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub language:              Language,
    pub company_name:          String,
    pub default_vat:           String,
    pub screenshot_protection: bool,
    pub ui_scale:              f32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language:              Language::Finnish,
            company_name:          String::new(),
            default_vat:           "25.5".to_string(),
            screenshot_protection: false,
            ui_scale:              1.0,
        }
    }
}