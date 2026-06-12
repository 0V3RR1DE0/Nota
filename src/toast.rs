use std::time::Instant;
use eframe::egui;

const LIFETIME_SECS: f32 = 4.0;
const FADE_START:    f32 = 3.0;

#[derive(Clone, Copy, PartialEq)]
pub enum ToastKind { Success, Error, Info }

pub struct Toast {
    pub message: String,
    pub kind:    ToastKind,
    born:        Instant,
    id:          usize,
}

pub struct Toasts {
    items:   Vec<Toast>,
    next_id: usize,
}

impl Toasts {
    pub fn new() -> Self { Toasts { items: Vec::new(), next_id: 0 } }

    pub fn success(&mut self, msg: impl Into<String>) { self.push(msg, ToastKind::Success); }
    pub fn error  (&mut self, msg: impl Into<String>) { self.push(msg, ToastKind::Error);   }
    pub fn info   (&mut self, msg: impl Into<String>) { self.push(msg, ToastKind::Info);    }

    fn push(&mut self, msg: impl Into<String>, kind: ToastKind) {
        if self.items.len() >= 5 { self.items.remove(0); } // cap stack
        self.items.push(Toast { message: msg.into(), kind, born: Instant::now(), id: self.next_id });
        self.next_id += 1;
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        self.items.retain(|t| t.born.elapsed().as_secs_f32() < LIFETIME_SECS);
        if self.items.is_empty() { return; }

        // Keep repainting while toasts are visible (for fade-out)
        ctx.request_repaint_after(std::time::Duration::from_millis(50));

        let mut y = 64.0_f32; // below the top bar

        for toast in &self.items {
            let age     = toast.born.elapsed().as_secs_f32();
            let opacity = if age > FADE_START {
                1.0 - (age - FADE_START) / (LIFETIME_SECS - FADE_START)
            } else { 1.0 };

            let (bg, icon) = match toast.kind {
                ToastKind::Success => (egui::Color32::from_rgb(16,  185, 129), "✓"),
                ToastKind::Error   => (egui::Color32::from_rgb(244,  63,  94), "✕"),
                ToastKind::Info    => (egui::Color32::from_rgb(99,  102, 241), "ℹ"),
            };

            egui::Area::new(egui::Id::new(("toast", toast.id)))
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, y))
                .order(egui::Order::Foreground)
                .interactable(false)
                .show(ctx, |ui| {
                    ui.set_opacity(opacity);
                    egui::Frame::default()
                        .fill(bg)
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::same(10))
                        .show(ui, |ui| {
                            ui.set_min_width(200.0);
                            ui.set_max_width(380.0);
                            ui.horizontal(|ui| {
                                ui.colored_label(egui::Color32::WHITE, icon);
                                ui.add(egui::Label::new(
                                    egui::RichText::new(&toast.message).color(egui::Color32::WHITE)
                                ).wrap());
                            });
                        });
                });

            y += 52.0;
        }
    }
}