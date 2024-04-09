use std::ops::RangeInclusive;

use eframe::egui::{self, Align2, Color32, FontId, Stroke};

#[allow(dead_code)]
fn level_meter_ui(
    ui: &mut egui::Ui,
    lvl_range: RangeInclusive<f32>,
    level_db: f32,
) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(4.0, 1.0);
    let (rect, response) =
        ui.allocate_exact_size(desired_size, egui::Sense::focusable_noninteractive());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let rect = rect.expand(visuals.expansion);
        // background
        ui.painter()
            .rect(rect, 0.0, visuals.bg_fill, visuals.bg_stroke);

        let mut rect = rect;
        *rect.right_mut() = egui::remap(level_db, lvl_range, rect.left()..=rect.right());
        let color = if level_db < 0.0 {
            Color32::GREEN
        } else {
            Color32::RED
        };
        ui.painter().rect(rect, 0.0, color, Stroke::NONE);
        // ui.painter().text(
        //     rect.left_center(),
        //     Align2::LEFT_CENTER,
        //     format!("{:.1} dB", level_db),
        //     FontId::default(),
        //     visuals.text_color(),
        // );
    }

    response
}

// A wrapper that allows the more idiomatic usage pattern: `ui.add(toggle(&mut my_bool))`
/// iOS-style toggle switch.
///
/// ## Example:
/// ``` ignore
/// ui.add(toggle(&mut my_bool));
/// ```
pub fn level_meter<'a>(lvl_range: RangeInclusive<f32>, level_db: f32) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| level_meter_ui(ui, lvl_range, level_db)
}
