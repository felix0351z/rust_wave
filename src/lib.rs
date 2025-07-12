use crate::controller::stream::{Settings, Stream};
use crate::ControllerError::CPALError;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{HostId, SampleFormat, SupportedStreamConfig};
use std::error::Error;
use std::ops::Deref;
use thiserror::Error;
use log::info;
use crate::controller::channel::{Receiver, ViewFrame};
use crate::controller::effects::{AudioEffect, EffectDescription};
use crate::controller::effects::fft::FftEffect;
use crate::controller::effects::melbank::MelbankEffect;
use crate::controller::effects::shine::ShineEffect;
use crate::controller::effects::spectrum::SpectrumEffect;
use crate::controller::sender::Sender;

pub mod controller;

#[cfg(test)]
mod tests;

pub const FPS: usize = 100;

// Help to declare all method results with the ControllerError Type
type Result<T> = std::result::Result<T, ControllerError>;

// The main controller of the program
pub struct Controller {
    host: Option<cpal::Host>,
    device: Option<cpal::Device>,
    stream_handler: Stream,
    sender: Sender,
    effects: Vec<EffectDescription>
}

// All Errors which can occur during the program runtime from the controller
#[derive(Debug, Error)]
pub enum  ControllerError {

    // CPAL library which returned an error. Can be any audio-device related issues
    #[error("Failure in the cpal audio library")]
    CPALError(Box<dyn Error>),
    #[error("Failure in the sacn sender library")]
    SacnError(sacn::error::errors::Error),
    #[error("No device found")]
    NoDeviceFound,
    #[error("No supported audio callback config")]
    NoSupportedConfig,
    #[error("The given ID is not available")]
    NoValidId
}

#[derive(Clone)]
pub struct InputDevice {
    pub id: usize,
    pub name: String,
}



impl Controller {

    pub fn new() -> Controller {
        let effects: Vec<EffectDescription> = register_effects! {
            "Melbank" => MelbankEffect::new,
            "Spectrum" => SpectrumEffect::new,
            "Shine" => ShineEffect::new,
            "FFT (View Only)" => FftEffect::new
        };

        Controller {
            host: None,
            device: None,
            stream_handler: Stream::new(),
            sender: Sender::new_multicast_sender(),
            effects
        }
    }

    /// Get all available cpal host and initialize with the first available host
    pub fn get_available_hosts(&mut self) -> Result<Vec<HostId>> {
        let hosts = cpal::available_hosts();

        // initialize with the first host
        self.update_host(hosts[0])?;
        Ok(hosts)
    }

    /// Update the used host and set the selected device to 0
    pub fn update_host(&mut self, id: HostId) -> Result<()> {
        println!("Select host {}", id.name());

        let new = cpal::host_from_id(id)
            .map_err(|e| {CPALError(e.into())})?;

        self.host = Some(new);
        Ok(())
    }

    /// Get a list of all effects
    pub fn get_effects(&self) -> Vec<&'static str> {
        self.effects.iter()
            .map(|it| it.name)
            .collect::<Vec<_>>()
    }

    pub fn update_effect(&mut self, effect: &'static str)  {
        if let Some(effect) = self.effects.iter().find(|it| it.name == effect) {
            // Build the effect and send it to the stream
            let built = (effect.factory)();
            self.stream_handler.update_effect(built)
        }
    }

    pub fn get_available_input_devices(&self) -> Result<Vec<InputDevice>> {
        let devices = self.host.as_ref()
            .expect("No available host found")
            .input_devices()
            .map_err(|e| CPALError(e.into()))?;


        let mut out: Vec<InputDevice> = Vec::new();

        //TODO: Map error to log

        for (id, device) in devices.enumerate() {
            if let Ok(name) = device.name() {
                out.push(InputDevice {id, name});
            }
        }
        Ok(out)
    }

    pub fn update_device(&mut self, id: usize) -> Result<()> {

        let devices = self.host.as_ref()
            .expect("No available host found")
            .input_devices()
            .map_err(|e| CPALError(e.into()))?;

        let selected = devices.skip(id).next()
            .expect("");
        info!("Select device: {:?}", selected.name());

        // Update the device and start run the stream on the new device
        self.device = Some(selected);
        Ok(())
    }

    pub fn open_stream(&mut self, device: usize, effect: &'static str, settings: Settings) -> Result<std::sync::mpsc::Receiver<ViewFrame>> {
        self.update_device(device)?;
        self.update_stream(effect, settings)
    }

    pub fn update_stream(&mut self, effect: &'static str, settings: Settings) -> Result<std::sync::mpsc::Receiver<ViewFrame>> {
        info!("Opening stream");

        // Check if a valid device & config are available
        let device = self.device.as_ref().ok_or(ControllerError::NoDeviceFound)?;
        let configs = device.supported_input_configs()
            .map_err(|e| CPALError(e.into()))?;

        // Only use the config with the sample format f32
        let mut config: Option<SupportedStreamConfig> = None;
        for config_range in configs {
            if config_range.sample_format() == SampleFormat::F32 {
                config = Some(config_range.with_max_sample_rate());
                break;
            };
        }

        // Get the effect
        let effect = self.effects.iter()
            .find(|it| it.name == effect)
            .ok_or(ControllerError::NoValidId)?;

        // Build the effect
        let built = (effect.factory)();

        // If a valid config with f32 was found create the stream
        if let Some(config) = config {
            // Start the stream and if an error occurs, notify the view
            let rx = self.stream_handler.open(device, config.into(), settings, built)
                .map_err(|e| CPALError(e.into()))?;

            // Start the sacn sender
            let Receiver { rx_sacn, rx_view } = rx;
            self.sender.listen(rx_sacn);

            Ok(rx_view)
        } else {
            // if no valid config was found return error
            Err(ControllerError::NoSupportedConfig)
        }
    }

    pub fn update_stream_settings(&mut self, settings: Settings) {
        self.stream_handler.update_settings(settings)
    }

    pub fn update_color(&mut self, color: (u8, u8, u8)) {
        self.stream_handler.update_color(color)
    }


}




