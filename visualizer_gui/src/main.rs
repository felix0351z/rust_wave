use std::default::Default;

mod view;
mod view_model;
mod utils;

pub const APP_NAME: &'static str = "Audio Visualizer";

fn main() {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1500.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(APP_NAME, options, Box::new(|creation_context| {
        creation_context.egui_ctx.set_theme(egui::Theme::Dark);
        Ok(Box::<view::AudioVisualizerView>::default())
    })).expect("Failed to run app");
}
