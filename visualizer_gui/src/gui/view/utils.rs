use egui::{InnerResponse, Ui};

/// Shortcut to build a settings grid
pub fn settings_grid<R>(id: &'static str, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            content(ui)
        })
}