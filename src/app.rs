use eframe::egui;
use crate::data::{Entry, EntryType, Ledger};
use crate::lang::{self, Language};
use crate::settings::Settings;
use crate::storage;
use crate::toast::Toasts;
use crate::updater::UpdateInfo;

const GREEN:    egui::Color32 = egui::Color32::from_rgb(16,  185, 129);
const RED:      egui::Color32 = egui::Color32::from_rgb(244,  63,  94);
const DIM_TEXT: egui::Color32 = egui::Color32::from_rgb(161, 161, 170);
const INDIGO:   egui::Color32 = egui::Color32::from_rgb(99,  102, 241);

#[derive(PartialEq)]
enum Tab { Ledger, Settings }

pub struct NotaApp {
    ledger:   Ledger,
    settings: Settings,
    // Form fields
    new_date:        String,
    new_description: String,
    new_entry_type:  EntryType,
    new_product:     String,
    new_vat:         String,
    new_price:       String,
    price_is_incl:   bool,
    new_attachment:  Option<String>,
    // UI state
    active_tab:           Tab,
    search_query:         String,
    selected_description: Option<String>,
    toasts:               Toasts,
    // Splash + update check
    splash_done:          bool,
    splash_born:          std::time::Instant,
    update_handle:        Option<std::thread::JoinHandle<Option<UpdateInfo>>>,
}

impl NotaApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let settings = storage::load_settings();
        apply_nota_style(&cc.egui_ctx);
        cc.egui_ctx.set_pixels_per_point(settings.ui_scale);

        let ledger = storage::load().unwrap_or_default();
        let default_vat = settings.default_vat.clone();

        // Spawn update check in background — won't block the splash
        let handle = std::thread::spawn(crate::updater::check);

        NotaApp {
            ledger,
            settings,
            new_date:        String::new(),
            new_description: String::new(),
            new_entry_type:  EntryType::Income,
            new_product:     String::new(),
            new_vat:         default_vat,
            new_price:       String::new(),
            price_is_incl:   false,
            new_attachment:  None,
            active_tab:      Tab::Ledger,
            search_query:    String::new(),
            selected_description: None,
            toasts:          Toasts::new(),
            splash_done:     false,
            splash_born:     std::time::Instant::now(),
            update_handle:   Some(handle),
        }
    }
}

impl eframe::App for NotaApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.ctx().set_pixels_per_point(self.settings.ui_scale);
        let ctx = ui.ctx().clone();

        // ── Splash screen ──────────────────────────────────────────────
        if !self.splash_done {
            self.draw_splash(ui);
            return;
        }

        let t = lang::get(self.settings.language);

        // ── Top bar ────────────────────────────────────────────────────
        egui::Panel::top("top_bar")
            .exact_size(56.0)
            .show_inside(ui, |ui| {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.heading(&t.app_name);
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    if ui.selectable_label(self.active_tab == Tab::Ledger,   &t.tab_ledger).clicked()   { self.active_tab = Tab::Ledger;   }
                    if ui.selectable_label(self.active_tab == Tab::Settings, &t.tab_settings).clicked() { self.active_tab = Tab::Settings; }

                    if self.active_tab == Tab::Ledger {
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        let income   = self.ledger.total_income();
                        let expenses = self.ledger.total_expenses();
                        let net      = income - expenses;
                        let vat_due  = self.ledger.total_vat_collected() - self.ledger.total_vat_deductible();

                        ui.label(format!("{}: {:.2} €", t.total_income,   income));
                        ui.separator();
                        ui.label(format!("{}: {:.2} €", t.total_expenses, expenses));
                        ui.separator();
                        ui.colored_label(if net >= 0.0 { GREEN } else { RED }, format!("{}: {:.2} €", t.net, net));
                        ui.separator();
                        ui.label(format!("{}: {:.2} €", t.vat_payable, vat_due));
                    }
                });
            });

        match self.active_tab {
            Tab::Ledger   => self.show_ledger(ui),
            Tab::Settings => self.show_settings(ui, frame),
        }

        // ── Overlays ──────────────────────────────────────────────────

        // Description popup — full text in a modal card
        if let Some(desc) = self.selected_description.clone() {
            let modal = egui::Modal::new(egui::Id::new("desc_modal"))
                .show(&ctx, |ui| {
                    ui.set_min_width(360.0);
                    ui.set_max_width(560.0);
                    ui.strong(&lang::get(self.settings.language).col_desc);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.add(egui::Label::new(&desc).wrap());
                    ui.add_space(12.0);
                    if ui.button("  Close  ").clicked() {
                        self.selected_description = None;
                    }
                });
            if modal.should_close() { self.selected_description = None; }
        }

        // Toast notifications
        self.toasts.draw(&ctx);
    }
}

impl NotaApp {

    fn draw_splash(&mut self, ui: &mut egui::Ui) {
        let elapsed = self.splash_born.elapsed();
        let check_done = self.update_handle.as_ref()
            .map(|h| h.is_finished())
            .unwrap_or(true);

        // Transition to main app after minimum 1.5s and check complete
        if elapsed.as_secs_f32() > 1.5 && check_done {
            if let Some(handle) = self.update_handle.take() {
                if let Ok(Some(info)) = handle.join() {
                    self.toasts.info(format!(
                        "Update {} available — download at nota.app",
                        info.version
                    ));
                }
            }
            self.splash_done = true;
            ui.ctx().request_repaint();
            return;
        }

        // Keep animating
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(200));

        // Fix theme consistency: wrap splash layout inside a themed CentralPanel
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let available = ui.available_size();
            ui.add_space((available.y / 2.0 - 100.0).max(0.0));
            ui.vertical_centered(|ui| {
                // Logo mark — the N from the SVG, rendered large
                ui.add(egui::Label::new(
                    egui::RichText::new("N")
                        .size(80.0)
                        .color(INDIGO)
                        .strong()
                ));
                ui.add_space(8.0);
                ui.add(egui::Label::new(
                    egui::RichText::new("Nota")
                        .size(28.0)
                        .color(egui::Color32::from_rgb(244, 244, 245))
                        .strong()
                ));
                ui.add_space(4.0);
                ui.colored_label(DIM_TEXT, format!("v{}", env!("CARGO_PKG_VERSION")));
                ui.add_space(32.0);

                // Animated dots
                let dots = match ((elapsed.as_secs_f32() * 2.5) as u32) % 4 {
                    0 => "   ", 1 => ".  ", 2 => ".. ", _ => "...",
                };
                ui.colored_label(DIM_TEXT, format!("Checking for updates{}", dots));
            });
        });
    }

    // ──────────────────────────────────────────────────────────────────
    //  Ledger tab
    // ──────────────────────────────────────────────────────────────────

    fn show_ledger(&mut self, ui: &mut egui::Ui) {
        let t = lang::get(self.settings.language);

        egui::Panel::left("form_panel")
            .exact_size(300.0)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(12.0);
                    ui.strong(&t.new_entry);
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(12.0);

                    ui.label(&t.type_label);
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.new_entry_type, EntryType::Income,  &t.income);
                        ui.radio_value(&mut self.new_entry_type, EntryType::Expense, &t.expense);
                    });

                    ui.add_space(8.0);
                    ui.label(&t.date_label);
                    ui.text_edit_singleline(&mut self.new_date);

                    ui.add_space(8.0);
                    ui.label(&t.desc_label);
                    ui.text_edit_singleline(&mut self.new_description);

                    ui.add_space(8.0);
                    ui.label(&t.product_label);
                    ui.text_edit_singleline(&mut self.new_product);

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(&t.vat_label);
                            ui.add(egui::TextEdit::singleline(&mut self.new_vat).desired_width(70.0));
                        });
                        ui.vertical(|ui| {
                            ui.label(&t.price_label);
                            ui.horizontal(|ui| {
                                ui.radio_value(&mut self.price_is_incl, false, &t.price_excl_radio);
                                ui.radio_value(&mut self.price_is_incl, true,  &t.price_incl_radio);
                            });
                            ui.add(egui::TextEdit::singleline(&mut self.new_price).desired_width(130.0));
                        });
                    });

                    if let (Ok(vat), Ok(price)) = (
                        self.new_vat.trim().parse::<f64>(),
                        self.new_price.trim().parse::<f64>(),
                    ) {
                        let (excl, vat_amt, incl) = calc_vat(vat, price, self.price_is_incl);
                        ui.add_space(8.0);
                        ui.colored_label(DIM_TEXT, format!(
                            "{}  {:.2} €    {}  {:.2} €    {}  {:.2} €",
                            t.preview_excl, excl, t.preview_vat, vat_amt, t.preview_incl, incl,
                        ));
                    }

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(&t.btn_attach).clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter(t.attach_filter.clone(), &["pdf", "png", "jpg", "jpeg"])
                                .pick_file()
                            {
                                match storage::save_attachment(&path) {
                                    Ok(name) => { self.new_attachment = Some(name); }
                                    Err(e)   => { self.toasts.error(format!("{}: {}", t.err_attach, e)); }
                                }
                            }
                        }
                        if let Some(ref name) = self.new_attachment {
                            ui.colored_label(DIM_TEXT, name.as_str());
                        }
                    });

                    ui.add_space(16.0);
                    if ui.button(format!("  {}  ", t.btn_add_entry)).clicked() {
                        self.try_add_entry();
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);

                    if ui.button(format!("  {}  ", t.btn_save_csv)).clicked() {
                        match storage::save(&self.ledger) {
                            Ok(_)  => self.toasts.success(t.saved.clone()),
                            Err(e) => self.toasts.error(format!("{}: {}", t.err_save, e)),
                        }
                    }
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.colored_label(DIM_TEXT, "🔍");
                ui.add(egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text(&t.search_hint)
                    .desired_width(320.0));
                if !self.search_query.is_empty() && ui.small_button("✕").clicked() {
                    self.search_query.clear();
                }
                ui.add_space(16.0);
                ui.colored_label(DIM_TEXT, format!("{} {}", self.ledger.entries.len(), t.col_num));
            });
            ui.add_space(12.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                let query = self.search_query.to_lowercase();

                let filtered: Vec<(usize, &Entry)> = self.ledger.entries
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| {
                        query.is_empty()
                            || e.desc.to_lowercase().contains(&query)
                            || e.product.to_lowercase().contains(&query)
                            || e.date.contains(&query)
                    })
                    .collect();

                let mut to_delete: Option<usize>              = None;
                let mut open_file: Option<std::path::PathBuf> = None;
                let mut show_desc: Option<String>             = None;

                egui::Grid::new("ledger_grid")
                    .striped(true)
                    .spacing([16.0, 8.0])
                    .min_col_width(40.0)
                    .max_col_width(260.0)
                    .show(ui, |ui| {
                        ui.strong(&t.col_num);
                        ui.strong(&t.col_date);
                        ui.strong(&t.col_type);
                        ui.strong(&t.col_desc);
                        ui.strong(&t.col_product);
                        ui.strong(&t.col_vat_pct);
                        ui.strong(&t.col_excl);
                        ui.strong(&t.col_vat_eur);
                        ui.strong(&t.col_incl);
                        ui.strong("📎");
                        ui.strong("");
                        ui.end_row();

                        for (real_idx, entry) in filtered {
                            ui.label(entry.number.to_string());
                            ui.label(&entry.date);

                            let (clr, type_str) = match entry.entry_type {
                                EntryType::Income  => (GREEN, &t.income),
                                EntryType::Expense => (RED,   &t.expense),
                            };
                            ui.colored_label(clr, type_str);

                            let is_long = entry.desc.chars().count() > 32;
                            let d_short = truncate_chars(&entry.desc, 32);
                            let d_text  = egui::RichText::new(&d_short)
                                .color(if is_long {
                                    egui::Color32::from_rgb(190, 190, 210)
                                } else {
                                    egui::Color32::from_rgb(244, 244, 245)
                                });
                            let d_label = egui::Label::new(d_text).sense(egui::Sense::click());
                            let d_resp  = ui.add(d_label);
                            if is_long {
                                let d_resp = d_resp
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .on_hover_text("Click to read full description");
                                if d_resp.clicked() {
                                    show_desc = Some(entry.desc.clone());
                                }
                            }

                            let p_short = truncate_chars(&entry.product, 20);
                            let p_resp  = ui.label(&p_short);
                            if entry.product.chars().count() > 20 {
                                p_resp.on_hover_text(&entry.product);
                            }

                            ui.label(format!("{:.1}%",  entry.vat_percent));
                            ui.label(format!("{:.2} €", entry.price_excl));
                            ui.label(format!("{:.2} €", entry.vat_amount));
                            ui.label(format!("{:.2} €", entry.price_incl));

                            match &entry.attachment {
                                Some(name) => {
                                    if ui.link("📎").on_hover_text(name.as_str()).clicked() {
                                        open_file = Some(storage::attachment_path(name));
                                    }
                                }
                                None => { ui.label(""); }
                            }

                            if ui.small_button("✕").on_hover_text("Delete").clicked() {
                                to_delete = Some(real_idx);
                            }
                            ui.end_row();
                        }
                    });

                if let Some(idx) = to_delete {
                    self.ledger.entries.remove(idx);
                    self.auto_save();
                }
                if let Some(path) = open_file {
                    if let Err(e) = open::that_detached(&path) {
                        self.toasts.error(format!("{}: {}", lang::get(self.settings.language).err_open_file, e));
                    }
                }
                if let Some(desc) = show_desc {
                    self.selected_description = Some(desc);
                }
            });
        });
    }

    // ──────────────────────────────────────────────────────────────────
    //  Settings tab
    // ──────────────────────────────────────────────────────────────────

    fn show_settings(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let t    = lang::get(self.settings.language);
        let prev = self.settings.clone();

        // Fix theme + layouts crash: encapsulate content in a CentralPanel
        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(16.0);
                ui.heading(&t.settings_heading);
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(12.0);

                ui.strong(&t.lang_section);
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.settings.language, Language::Finnish, "Suomi");
                    ui.radio_value(&mut self.settings.language, Language::English, "English");
                });

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(12.0);

                ui.strong(&t.ui_scale_label);
                ui.add_space(6.0);
                ui.add(egui::Slider::new(&mut self.settings.ui_scale, 0.75_f32..=2.0_f32)
                    .step_by(0.05)
                    .text("×"));
                ui.colored_label(DIM_TEXT, &t.ui_scale_hint);

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(12.0);

                ui.strong(&t.company_section);
                ui.add_space(6.0);
                ui.label(&t.company_name_lbl);
                ui.text_edit_singleline(&mut self.settings.company_name);
                ui.add_space(8.0);
                ui.label(&t.default_vat_lbl);
                ui.add(egui::TextEdit::singleline(&mut self.settings.default_vat).desired_width(80.0));

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(12.0);

                ui.strong(&t.export_section);
                ui.add_space(6.0);
                if ui.button(&t.btn_export_csv).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name("nota_export.csv")
                        .add_filter("CSV", &["csv"])
                        .save_file()
                    {
                        match storage::export_csv(&self.ledger, &path) {
                            Ok(_)  => self.toasts.success(t.export_ok.clone()),
                            Err(e) => self.toasts.error(format!("{}: {}", t.err_export, e)),
                        }
                    }
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(12.0);

                ui.strong(&t.about_section);
                ui.add_space(6.0);
                ui.label(&t.about_desc);
                ui.label(format!("{}: {}", t.about_version, env!("CARGO_PKG_VERSION")));
                ui.label(format!("{}: MIT", t.about_license));
                ui.add_space(8.0);
                if ui.button("Check for updates").clicked() {
                    let handle = std::thread::spawn(crate::updater::check);
                    self.update_handle = Some(handle);
                    self.toasts.info("Checking for updates...");
                }
            });
        });

        if self.settings != prev {
            let _ = storage::save_settings(&self.settings);
        }
    }

    fn try_add_entry(&mut self) {
        let t = lang::get(self.settings.language);

        if !is_valid_finnish_date(self.new_date.trim()) {
            self.toasts.error(t.err_date.clone()); return;
        }
        if self.new_description.trim().is_empty() {
            self.toasts.error(t.err_desc.clone()); return;
        }
        if self.new_product.trim().is_empty() {
            self.toasts.error(t.err_product.clone()); return;
        }

        let vat: f64 = match self.new_vat.trim().parse() {
            Ok(v) => v, Err(_) => { self.toasts.error(t.err_vat.clone()); return; }
        };
        let price: f64 = match self.new_price.trim().parse() {
            Ok(p) => p, Err(_) => { self.toasts.error(t.err_price.clone()); return; }
        };

        let price_excl = if self.price_is_incl {
            (price / (1.0 + vat / 100.0) * 100.0).round() / 100.0
        } else { price };

        let number = self.ledger.next_entry_number();
        let entry  = Entry::new(
            number,
            self.new_date.trim().to_string(),
            self.new_description.trim().to_string(),
            self.new_entry_type.clone(),
            self.new_product.trim().to_string(),
            vat,
            price_excl,
            self.new_attachment.take(),
        );

        self.ledger.entries.push(entry);
        self.auto_save();

        self.new_description.clear();
        self.new_product.clear();
        self.new_price.clear();
        self.toasts.success(format!("{}{}", t.entry_added, number));
    }

    fn auto_save(&mut self) {
        let t = lang::get(self.settings.language);
        if let Err(e) = storage::save(&self.ledger) {
            self.toasts.error(format!("{}: {}", t.err_autosave, e));
        }
    }
}

// ── Theme ─────────────────────────────────────────────────────────────────

pub fn apply_nota_style(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing   = egui::vec2(12.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 6.0);
    style.spacing.window_margin  = egui::Margin::same(16);
    style.spacing.interact_size  = egui::vec2(40.0, 24.0);
    style.visuals = nota_theme();
    ctx.set_global_style(style);
}

fn nota_theme() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    let bg_app     = egui::Color32::from_rgb(24,  24,  27);
    let bg_panel   = egui::Color32::from_rgb(39,  39,  42);
    let bg_deep    = egui::Color32::from_rgb(9,   9,   11);
    let border_dim = egui::Color32::from_rgb(63,  63,  70);
    let border_med = egui::Color32::from_rgb(82,  82,  91);
    let text_main  = egui::Color32::from_rgb(244, 244, 245);
    let accent     = egui::Color32::from_rgb(99,  102, 241);
    let accent_dim = egui::Color32::from_rgb(67,  56,  202);

    v.panel_fill          = bg_panel;
    v.window_fill         = bg_panel;
    v.extreme_bg_color    = bg_deep;
    v.faint_bg_color      = bg_app;
    v.override_text_color = Some(text_main);
    v.window_stroke       = egui::Stroke::new(1.0, border_dim);

    v.widgets.noninteractive.bg_fill       = bg_app;
    v.widgets.noninteractive.bg_stroke     = egui::Stroke::new(1.0, border_dim);
    v.widgets.noninteractive.fg_stroke     = egui::Stroke::new(1.0, egui::Color32::from_rgb(161, 161, 170));
    v.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);
    v.widgets.inactive.bg_fill             = bg_panel;
    v.widgets.inactive.bg_stroke           = egui::Stroke::new(1.0, border_dim);
    v.widgets.inactive.fg_stroke           = egui::Stroke::new(1.0, text_main);
    v.widgets.inactive.corner_radius       = egui::CornerRadius::same(6);
    v.widgets.hovered.bg_fill              = border_dim;
    v.widgets.hovered.bg_stroke            = egui::Stroke::new(1.0, border_med);
    v.widgets.hovered.fg_stroke            = egui::Stroke::new(1.0, text_main);
    v.widgets.hovered.corner_radius        = egui::CornerRadius::same(6);
    v.widgets.active.bg_fill               = accent_dim;
    v.widgets.active.bg_stroke             = egui::Stroke::new(1.0, accent);
    v.widgets.active.fg_stroke             = egui::Stroke::new(1.0, text_main);
    v.widgets.active.corner_radius         = egui::CornerRadius::same(6);
    v.widgets.open.bg_fill                 = bg_panel;
    v.widgets.open.bg_stroke               = egui::Stroke::new(1.0, border_med);
    v.widgets.open.corner_radius           = egui::CornerRadius::same(6);
    v.selection.bg_fill = accent_dim;
    v.selection.stroke  = egui::Stroke::new(1.0, accent);
    v.window_shadow = egui::epaint::Shadow {
        offset: [0, 8], blur: 24, spread: 0,
        color: egui::Color32::from_black_alpha(80),
    };
    v
}

// ── Utilities ─────────────────────────────────────────────────────────────

fn truncate_chars(s: &str, max: usize) -> String {
    let mut iter = s.chars();
    let head: String = iter.by_ref().take(max).collect();
    if iter.next().is_some() { format!("{}…", head) } else { head }
}

fn calc_vat(vat_pct: f64, price: f64, is_incl: bool) -> (f64, f64, f64) {
    if is_incl {
        let excl    = (price / (1.0 + vat_pct / 100.0) * 100.0).round() / 100.0;
        let vat_amt = ((price - excl) * 100.0).round() / 100.0;
        (excl, vat_amt, price)
    } else {
        let vat_amt = (price * vat_pct / 100.0 * 100.0).round() / 100.0;
        (price, vat_amt, ((price + vat_amt) * 100.0).round() / 100.0)
    }
}

fn is_valid_finnish_date(s: &str) -> bool {
    let p: Vec<&str> = s.split('.').collect();
    if p.len() != 3 || p[2].len() != 4 { return false; }
    let d: u32 = match p[0].parse() { Ok(v) => v, Err(_) => return false };
    let m: u32 = match p[1].parse() { Ok(v) => v, Err(_) => return false };
    let y: u32 = match p[2].parse() { Ok(v) => v, Err(_) => return false };
    d >= 1 && d <= 31 && m >= 1 && m <= 12 && y >= 1900 && y <= 2200
}