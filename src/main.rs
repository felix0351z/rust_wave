use gui::view::{AudioVisualizerView};

mod gui;

//TODO: Farbauswahl Menü / Vorgefertigte Farben
//TODO: Zusätzliche Filter Parameter zur GUI hinzufügen => Evtl. Filter anpassen auf mutability
//TODO: Presets um die Einstellungen passend zu Bass, Sprache etc. anzupassen
//TODO: Letzte Animationen übernehmen aus Python

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
