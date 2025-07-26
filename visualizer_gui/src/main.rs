use gui::view::{AudioVisualizerView};

mod gui;

//TODO: API refactoring

pub const APP_NAME: &'static str = "Audio Visualizer";

fn main() {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1500.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(APP_NAME, options, Box::new(|_cc| {
        Ok(Box::<AudioVisualizerView>::default())
    })).expect("Failed to run app");
}
