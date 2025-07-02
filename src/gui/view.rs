use crate::gui::view_model::AudioVisualizerViewModel;
use eframe::emath::Vec2b;
use eframe::{App, Frame};
use egui::{Context, InnerResponse, Ui};
use egui_plot::Line;
use std::fmt::Debug;
use strum::IntoEnumIterator;

/// The App
pub struct AudioVisualizerView {
    vm: AudioVisualizerViewModel
}

/// Initialize the app
impl Default for AudioVisualizerView {
    fn default() -> Self {
        let vm = AudioVisualizerViewModel::new();
        AudioVisualizerView {
            vm
        }
    }
}

/// UI Top Root
impl App for AudioVisualizerView {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show control window
            self.show_window(ui);
            // Show plot
            self.show_plot(ui);
        });
    }
}


/// All UI Panels
impl AudioVisualizerView {


    fn show_plot(&mut self, ui: &mut Ui) {
        let update = self.vm.receive_plot_update();

        ui.ctx().request_repaint();

        egui_plot::Plot::new("audio_plot")
            //.view_aspect(2.0)
            .allow_zoom(Vec2b::new(false, true))
            .show(ui, |plot_ui| {

                if let Some(update) = update {
                    plot_ui.set_plot_bounds(update.bounds);

                    //  Draw the effect
                    let line = Line::new(update.points).color(update.color);
                    plot_ui.line(line);
                }
            });
    }

    fn show_window(&mut self, ui: &mut Ui) {

        egui::Window::new("Settings").show(&ui.ctx(), |ui| {
            // Audio Source Settings
            settings_grid("grid_audio_source", ui, |ui| {
                grid_audio_source(ui, &mut self.vm);
            });
        });
    }
}


/// Shortcut to build a settings grid
fn settings_grid<R>(id: &'static str, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            content(ui)
        })
}

fn grid_audio_source(ui: &mut Ui, vm: &mut AudioVisualizerViewModel) {

    // ComboBox Host
    ui.label("Host");
    egui::ComboBox::from_id_salt("a")
        .selected_text(vm.get_selected_host())
        .show_ui(ui, |ui| {
            for (i, host) in vm.get_hosts().iter().enumerate() {
                if ui.selectable_value(&mut vm.selected_host, i, host.name()).clicked() {
                    // Notify when another host was selected
                    vm.click_update_host(host);
                }
            }
        });
    ui.end_row();

    // Combobox Input device
    ui.label("Input device");
    egui::ComboBox::from_id_salt("b")
        .selected_text(vm.get_selected_device())
        .show_ui(ui, |ui| {
            for (i, device) in vm.get_devices().iter().enumerate() {
                if ui.selectable_value(&mut vm.selected_device, i, &device.name).clicked() {
                    // Notify when another device was selected
                    vm.click_update_controller(device);
                }
            }
        });
    ui.end_row();

    ui.label("Plot");
    egui::ComboBox::from_id_salt("pgg")
        .selected_text(vm.get_selected_effect())
        .show_ui(ui, |ui| {
            for (i, effect_name) in vm.get_effects().iter().enumerate() {
                if ui.selectable_value(&mut vm.selected_effect, i, *effect_name).clicked() {
                    // Notify when another host was selected
                    vm.click_update_effect();
                }
            }
        });
    ui.end_row();

    ui.label("Logarithmic Scale");
    ui.checkbox(&mut vm.use_logarithmic_scale, "");
    ui.end_row();

    ui.label("Amount Bins");
    if ui.add(egui::Slider::new(&mut vm.settings.n_bins, 1..=100)).dragged() {
        vm.click_update_settings();
    }
    ui.end_row();

    ui.label("Minimum frequency");
    if ui.add(egui::Slider::new(&mut vm.settings.min_frequency, 0..=14000)).dragged() {
        vm.click_update_settings();
    }
    ui.end_row();

    ui.label("Maximum frequency");
    if ui.add(egui::Slider::new(&mut vm.settings.max_frequency, 100..=20000)).dragged() {
        vm.click_update_settings();
    }

    ui.end_row();

    ui.label("Color");
    if ui.color_edit_button_srgb(&mut vm.color).changed() {
        vm.click_update_color()
    };
    ui.end_row();


}

