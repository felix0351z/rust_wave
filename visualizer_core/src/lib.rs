use std::error::Error;
use log::info;

use thiserror::Error;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{SampleFormat, SupportedStreamConfig};

use sender::SacnSender;
use stream::Stream;
use stream::channel::{Receiver, ViewFrame};
use effects::*;

/// All digital signal processing related stuff
mod dsp;
/// the audio stream who processes all data
mod stream;
/// math utils
mod math;
/// all audio effects
mod effects;
/// the sacn sender to send the effects over the network
mod sender;

#[cfg(test)]
mod test;

// Export all needed utilities
pub use cpal::HostId;
pub use stream::Settings;
pub use stream::channel::ViewFrame as StreamFrame;
use crate::ControllerError::NoValidEffectName;

// Help to declare all method results with the ControllerError Type
pub type Result<T> = std::result::Result<T, ControllerError>;

// The main lib of the program
pub struct Controller {
    host: Option<cpal::Host>,
    device: Option<cpal::Device>,
    stream_handler: Stream,
    sender: SacnSender,
    effects: Vec<EffectDescription>
}

/// All errors that can occur during the program's runtime
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
    #[error("The given Effect ID is not available")]
    NoValidEffectName,
    #[error("No Stream created yet")]
    NoStream
}

/// The audio input device used by the program to receive audio signals.
/// This could be your microphone for example.
#[derive(Clone)]
pub struct InputDevice {
    pub id: usize,
    pub name: String,
}



impl Controller {

    /// Create a new controller
    pub fn new() -> Controller {
        let effects: Vec<EffectDescription> = register_effects! {
            "Melbank" => MelbankEffect::new,
            "Spectrum" => SpectrumEffect::new,
            "Shine" => ShineEffect::new,
            "Energy" => EnergyEffect::new,
            "Bass" => BassEffect::new,
            "Color Spectrum (Data Only)" => ColorSpectrumEffect::new,
            "FFT (View Only)" => FftEffect::new
        };

        Controller {
            host: None,
            device: None,
            stream_handler: Stream::new(),
            sender: SacnSender::new_multicast_sender(),
            effects
        }
    }

    /// Get all available hosts and initialize with the first available host
    pub fn get_available_hosts(&mut self) -> Result<Vec<HostId>> {
        let hosts = cpal::available_hosts();

        // initialize with the first host
        self.change_host(hosts[0])?;
        Ok(hosts)
    }

    /// Get a list of all input devices for your current selected host
    pub fn get_available_input_devices(&self) -> Result<Vec<InputDevice>> {
        let devices = self.host.as_ref()
            .expect("No available host found")
            .input_devices()
            .map_err(|e| ControllerError::CPALError(e.into()))?;

        let mut out: Vec<InputDevice> = Vec::new();
        for (id, device) in devices.enumerate() {
            if let Ok(name) = device.name() {
                out.push(InputDevice {id, name});
            }
        }
        Ok(out)
    }

    /// Get a list of all available audio effects
    pub fn get_effects(&self) -> Vec<&'static str> {
        self.effects.iter()
            .map(|it| it.name)
            .collect::<Vec<_>>()
    }

    /// Change the used host and set the selected device to 0
    pub fn change_host(&mut self, id: HostId) -> Result<()> {
        info!("Select host {}", id.name());

        let new = cpal::host_from_id(id)
            .map_err(|e| {ControllerError::CPALError(e.into())})?;

        self.host = Some(new);
        Ok(())
    }

    /// Change the selected input device
    pub fn change_input_device(&mut self, id: usize) -> Result<()> {
        let devices = self.host.as_ref()
            .expect("No available host found")
            .input_devices()
            .map_err(|e| ControllerError::CPALError(e.into()))?;

        let selected = devices.skip(id).next()
            .expect("");
        info!("Select device: {:?}", selected.name());

        // Update the device and start run the stream on the new device
        self.device = Some(selected);
        Ok(())
    }

    /// Update an existing audio stream or open a new one with the given attributes
    pub fn update_stream(&mut self, device: usize, effect: &'static str, settings: Settings, color: [u8; 3]) -> Result<std::sync::mpsc::Receiver<ViewFrame>> {
        self.change_input_device(device)?;
        self.open_stream(effect, settings, color)
    }

    /// Update the audio settings of the stream
    pub fn update_stream_settings(&mut self, settings: Settings) {
        self.stream_handler.update_settings(settings)
    }


    /// Change the current audio effect
    pub fn update_effect(&mut self, effect: &'static str) -> Result<()> {
        let effect = self.effects.iter()
            .find(|it| it.name == effect)
            .ok_or(NoValidEffectName)?;

        // Build the effect and send it to the stream
        let built = (effect.factory)();
        self.stream_handler.update_effect(built);
        Ok(())
    }

    /// Change the effect color
    pub fn update_color(&mut self, color: [u8; 3]) {
        self.stream_handler.update_color(color)
    }

    /// If the current effect produces his own color this value will be false
    pub fn is_color_selection_used(&self) -> Result<bool> {
        self.stream_handler.is_color_selection_used()
    }

    fn open_stream(&mut self, effect: &'static str, settings: Settings, color: [u8; 3]) -> Result<std::sync::mpsc::Receiver<ViewFrame>> {
        info!("Opening stream");

        // Check if a valid device & config are available
        let device = self.device.as_ref().ok_or(ControllerError::NoDeviceFound)?;
        let configs = device.supported_input_configs()
            .map_err(|e| ControllerError::CPALError(e.into()))?;

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
            .ok_or(ControllerError::NoValidEffectName)?;

        // Build the effect
        let built = (effect.factory)();

        // If a valid config with f32 was found create the stream
        if let Some(config) = config {
            // Start the stream and if an error occurs, notify the view
            let rx = self.stream_handler.open(device, config.into(), settings,  color,  built)
                .map_err(|e| ControllerError::CPALError(e.into()))?;

            // Start the sacn sender
            let Receiver { rx_sacn, rx_view } = rx;
            self.sender.listen(rx_sacn);

            Ok(rx_view)
        } else {
            // if no valid config was found return error
            Err(ControllerError::NoSupportedConfig)
        }
    }

}




