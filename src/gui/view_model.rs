use audio_visualizer::controller::channel::ViewFrame;
use audio_visualizer::controller::stream::Settings;
use audio_visualizer::{Controller, InputDevice};
use cpal::HostId;
use egui::Color32;
use egui_plot::PlotPoints;
use std::sync::mpsc::Receiver;

type GeneralSettings = Settings;
type DataReceiver = Receiver<ViewFrame>;

pub struct AudioVisualizerViewModel {
    controller: Controller,
    hosts: Vec<HostId>,
    devices: Vec<InputDevice>,
    effects: Vec<&'static str>,
    receiver: DataReceiver,

    pub selected_host: usize,
    pub selected_device: usize,
    pub selected_effect: usize,
    pub settings: GeneralSettings,
    pub color: [u8; 3],
}

pub struct PlotData {
    pub effect: PlotPoints,
    pub fft: PlotPoints,
    pub melbank: PlotPoints,
    pub color: Color32,
}

impl AudioVisualizerViewModel {

    pub fn new() -> AudioVisualizerViewModel {
        let mut controller = Controller::new();
        let hosts = controller.get_available_hosts().unwrap();
        let devices = controller.get_available_input_devices().unwrap();
        let effects = controller.get_effects();
        let settings = Settings::default();

        let first_effect = effects[0];

        // Open the stream
        let rx = controller.open_stream(0, first_effect, settings).unwrap();

        AudioVisualizerViewModel {
            controller,
            hosts,
            devices,
            effects,
            receiver: rx,
            selected_host: 0,
            selected_device: 0,
            selected_effect: 0,
            settings,
            color: [255, 255, 255],
        }
    }

    pub fn get_hosts(&self) -> Vec<HostId> {
        self.hosts.clone()
    }

    pub fn get_devices(&self) -> Vec<InputDevice> {
        self.devices.clone()
    }

    pub fn get_effects(&self) -> Vec<&'static str> {
        self.effects.clone()
    }

    pub fn get_selected_host(&self) -> &'static str {
        self.hosts[self.selected_host].name()
    }

    pub fn get_selected_device(&self) -> &str {
        self.devices[self.selected_device].name.as_str()
    }

    pub fn get_selected_effect(&self) -> &'static str {
        self.effects[self.selected_effect]
    }

    pub fn click_update_host(&mut self, host: &HostId) {
        //Update controller if the host was clicked and load new input devices
        self.controller.update_host(*host).unwrap();
        self.devices = self.controller.get_available_input_devices().unwrap();
        self.selected_device = 0;
    }

    pub fn click_update_controller(&mut self, device: &InputDevice) {
        // Update the device inside the controller and update the stream

        self.controller.update_device(device.id).unwrap();
        if let Ok(rx) = self.controller.update_stream(self.effects[self.selected_effect], self.settings) {
            self.receiver = rx;
        }
    }

    pub fn click_update_color(&mut self) {
        self.controller.update_color(self.color)
    }

    pub fn click_update_settings(&mut self) {
        self.controller.update_stream_settings(self.settings)
    }

    pub fn click_update_effect(&mut self) {
        self.controller.update_effect(self.effects[self.selected_effect])
    }

    pub fn receive_plot_update(&self) -> Option<(PlotPoints, Color32)> {
        // Receive data
        if let Ok(frame) = self.receiver.try_recv() {
            let color = Color32::from_rgb(frame.color.0, frame.color.1, frame.color.2);
            return Some((frame.effect.to_plot_points(), color));
        }
        None
    }
}

trait MapToPlotPoints {
    fn to_plot_points(self) -> PlotPoints;
}

impl MapToPlotPoints for Vec<f32> {
    fn to_plot_points(self) -> PlotPoints {
        // Map the values to a list of plot points
        // Use the iterator as x and the vec as y
        (0..self.len())
            .map(|i| { [i as f64, self[i] as f64] })
            .collect()
    }
}