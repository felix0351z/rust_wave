use gui::view::{AudioVisualizerView};

mod gui;

///TODO: FFT als Effekte registrieren, anstatt in UI versuchen zu implementieren
//TODO: Farbauswahl Menü / Vorgefertigte Farben
//TODO: Zusätzliche Filter Parameter zur GUI hinzufügen => Evtl. Filter anpassen auf mutability
//TODO: Presets um die Einstellungen passend zu Bass, Sprache etc. anzupassen

pub const APP_NAME: &'static str = "Audio Visualizer";
pub const LED_SIZE: u8 = 60;

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
