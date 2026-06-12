use std::sync::LazyLock;
use serde::Deserialize;

// Language enum — serializable so it can be saved in settings.json
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Language { Finnish, English }

/// All UI strings for one language.
/// Now uses String so values can come from JSON files.
/// LazyLock means they're initialized once on first use, then cached forever.
#[derive(Debug, Deserialize)]
pub struct T {
    pub app_name:       String,
    pub tab_ledger:     String,
    pub tab_settings:   String,
    pub total_income:   String,
    pub total_expenses: String,
    pub net:            String,
    pub vat_payable:    String,
    pub income:         String,
    pub expense:        String,
    pub new_entry:      String,
    pub type_label:     String,
    pub date_label:     String,
    pub desc_label:     String,
    pub product_label:  String,
    pub vat_label:      String,
    pub price_label:    String,
    pub price_excl_radio: String,
    pub price_incl_radio: String,
    pub preview_excl:   String,
    pub preview_vat:    String,
    pub preview_incl:   String,
    pub btn_attach:     String,
    pub btn_add_entry:  String,
    pub btn_save_csv:   String,
    pub col_num:        String,
    pub col_date:       String,
    pub col_type:       String,
    pub col_desc:       String,
    pub col_product:    String,
    pub col_vat_pct:    String,
    pub col_excl:       String,
    pub col_vat_eur:    String,
    pub col_incl:       String,
    pub saved:          String,
    pub entry_added:    String,
    pub autosaved:      String,
    pub err_date:       String,
    pub err_desc:       String,
    pub err_product:    String,
    pub err_vat:        String,
    pub err_price:      String,
    pub err_save:       String,
    pub err_autosave:   String,
    pub err_attach:     String,
    pub err_open_file:  String,
    pub attach_filter:  String,
    pub search_hint:    String,
    pub settings_heading: String,
    pub lang_section:   String,
    pub ui_scale_label: String,
    pub ui_scale_hint:  String,
    pub company_section: String,
    pub company_name_lbl: String,
    pub default_vat_lbl: String,
    pub export_section: String,
    pub btn_export_csv: String,
    pub export_ok:      String,
    pub err_export:     String,
    pub about_section:  String,
    pub about_desc:     String,
    pub about_version:  String,
    pub about_license:  String,
}

// include_str! bakes the JSON into the binary at compile time.
// The files are real — translators edit them and recompile to publish.
// To add a new language: create locales/sv.json and add a variant + line below.
static FI: LazyLock<T> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../locales/fi.json"))
        .expect("locales/fi.json is invalid — check JSON syntax")
});

static EN: LazyLock<T> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../locales/en.json"))
        .expect("locales/en.json is invalid — check JSON syntax")
});

/// Returns the translation set for the given language.
/// Returns &'static T — safe to call every frame, zero cost after first call.
pub fn get(lang: Language) -> &'static T {
    match lang {
        Language::Finnish => &FI,
        Language::English => &EN,
    }
}