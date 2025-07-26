pub mod color_slider;
mod utils;

use crate::gui::view::color_slider::color_slider;
use crate::gui::view::utils::settings_grid;
use crate::gui::view_model::AudioVisualizerViewModel;
use eframe::emath::Vec2b;
use eframe::{App, Frame};
use egui::ecolor::Hsva;
use egui::{remap_clamp, Color32, Context, Ui};
use egui_plot::Line;

/// The App
pub struct AudioVisualizerView {
    vm: AudioVisualizerViewModel
}

/// Initialize the app
impl Default for AudioVisualizerView {
    fn default() -> Self {
        Self { vm: AudioVisualizerViewModel::new() }
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
                    let line = Line::new("", update.points).color(update.color);
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

fn grid_audio_source(ui: &mut Ui, vm: &mut AudioVisualizerViewModel) {

    // ComboBox Host
    ui.label("Host");
    egui::ComboBox::from_id_salt("a")
        .selected_text(vm.get_selected_host())
        .show_ui(ui, |ui| {
            for (i, host) in vm.get_hosts().iter().enumerate() {
                if ui.selectable_value(&mut vm.selected_host, i, host.name()).clicked() {
                    if vm.selected_host != i {
                        // Notify when another host was selected
                        vm.click_update_host(host);
                    }
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


    if !vm.color_selection_enabled {
        ui.disable()
    }
    ui.label("Hue");
    let (_, hue_changed) = color_slider(ui, "Hue_Slider", &mut vm.color.hue, 0..=360, |value| {
        let hsv = Hsva { h: value, s: 1.0, v: 1.0, a: 1.0, };
        Color32::from(hsv)
    });
    ui.end_row();

    ui.label("Saturation");
    let (_, sat_changed) = color_slider(ui, "Saturation_Slider", &mut vm.color.saturation, 0..=255, |value| {
        let hue = remap_clamp(
            vm.color.hue as f32,
            0f32..=360f32,
            0f32..=1f32
        );

        let hsv = Hsva { h: hue, s: value, v: 1.0, a: 1.0, };
        Color32::from(hsv)
    });

    if hue_changed || sat_changed {
        vm.click_update_color()
    }
    ui.end_row();


}

