use std::default::Default;
use egui::{Style, Visuals};

mod view;
mod view_model;

pub const APP_NAME: &'static str = "Audio Visualizer";

fn main() {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1500.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(APP_NAME, options, Box::new(|creation_context| {
        let style = Style {
            visuals: Visuals::dark(),
            ..Style::default()
        };
        creation_context.egui_ctx.set_style(style);
        Ok(Box::<view::AudioVisualizerView>::default())
    })).expect("Failed to run app");
}
