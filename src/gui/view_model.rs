use audio_visualizer::controller::channel::ViewFrame;
use audio_visualizer::controller::stream::Settings;
use audio_visualizer::{Controller, InputDevice};
use cpal::HostId;
use egui::{remap_clamp, Color32};
use egui_plot::{PlotBounds, PlotPoints};
use std::sync::mpsc::Receiver;
use egui::ecolor::Hsva;

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
    pub use_logarithmic_scale: bool,
    pub settings: GeneralSettings,
    pub color: ColorState,
    pub color_selection_enabled: bool,
}

pub struct PlotUpdate<'a> {
    pub points: PlotPoints<'a>,
    pub color: Color32,
    pub bounds: PlotBounds,
}

#[derive(Clone, Copy, Default)]
pub struct ColorState {
    pub hue: u16,
    pub saturation: u16,
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
            use_logarithmic_scale: false,
            settings,
            color: ColorState::default(),
            color_selection_enabled: true
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
        let hue = remap_clamp(
            self.color.hue as f32,
            0f32..=360f32,
            0f32..=1f32
        );
        let sat = remap_clamp(
            self.color.saturation as f32,
            0f32..=255f32,
            0f32..=1f32
        );

        let color = Color32::from(Hsva { h: hue, s: sat, v: 1.0, a: 1.0, });
        let rgb = [color.r(), color.g(), color.b()];
        self.controller.update_color(rgb)
    }

    pub fn click_update_settings(&mut self) {
        self.controller.update_stream_settings(self.settings)
    }

    pub fn click_update_effect(&mut self) {
        self.controller.update_effect(self.effects[self.selected_effect]);

        if let Ok(color_selection_available) = self.controller.is_color_selection_available() {
            self.color_selection_enabled = color_selection_available;
        }
    }

    pub fn receive_plot_update(&self) -> Option<PlotUpdate> {
        // Receive data
        if let Ok(frame) = self.receiver.try_recv() {
            let color = Color32::from_rgb(frame.color[0], frame.color[1], frame.color[2]);
            let points = frame.effect.to_plot_points(self.use_logarithmic_scale);

            let point_len = points.points().len() as f64;
            let bounds = if self.use_logarithmic_scale {
                PlotBounds::from_min_max([0.8, 0.0], [point_len.log10(), 1.0])
            } else {
               PlotBounds::from_min_max([0.0, 0.0], [point_len, 1.0])
            };

            return Some(PlotUpdate {
                points,
                color,
                bounds,
            });
        }
        None
    }
}

trait MapToPlotPoints {
    fn to_plot_points(self, logarithmic_scale: bool) -> PlotPoints<'static>;
}

impl MapToPlotPoints for Vec<f32> {
    fn to_plot_points(self, logarithmic_scale: bool) -> PlotPoints<'static> {
        // Map the values to a list of plot points
        // Use the iterator as x and the vec as y
        (0..self.len())
            .map(|i| {
                let x = if logarithmic_scale { (i as f64).log10() } else { i as f64 };
                let y = self[i] as f64;
                [x, y] })
            .collect()
    }
}