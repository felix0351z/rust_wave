use std::error::Error;
use std::sync::{Arc, Mutex};
use log::error;

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::InputCallbackInfo;

use channel::{Receiver, Sender};
use super::ControllerError;
use super::dsp::tick;
use super::effects::AudioEffect;

pub mod channel;

#[derive(Debug, Copy, Clone)]
pub struct Settings {
    pub n_bins: usize,
    pub min_frequency: u16,
    pub max_frequency: u16,
}
impl Default for Settings {
    fn default() -> Self {
        Settings {
            n_bins: 60,
            min_frequency: 20,
            max_frequency: 12000,
        }
    }
}

pub struct InnerStream {
    pub last_frame: Vec<f32>,
    pub settings: Settings,
    pub sample_rate: u32,
    pub sender: Sender,
    pub color: [u8; 3],
    pub effect: Box<dyn AudioEffect>,
}



pub struct Stream {
    cpal_stream: Option<cpal::Stream>,
    buffer: Option<Arc<Mutex<InnerStream>>>,
}
impl Stream {

    pub fn new() -> Self {
        Stream {
            cpal_stream: None,
            buffer: None,
        }
    }

    pub fn open(
        &mut self,
        device: &cpal::Device,
        config: cpal::StreamConfig,
        settings: Settings,
        effect: Box<dyn AudioEffect>,
    ) -> Result<Receiver, Box<dyn Error>> {
        // Run the processing stream on another thread and share the data between a channel
        let (tx, rx) = channel::new();

        // Create a buffer for the thread
        let buffer = Arc::new(Mutex::new(
            InnerStream {
                last_frame: Vec::new(),
                settings,
                sample_rate: config.sample_rate.0,
                sender: tx,
                color: [255u8; 3],
                effect
            }
        ));
        self.buffer = Some(buffer.clone());

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _info: &InputCallbackInfo| tick(data, buffer.clone()),
            move |error| {
                error!("Stream error: {:?}", error);
            },
            None
        )?;

        stream.play()?;
        self.cpal_stream = Some(stream);

        Ok(rx)
    }

    /// Update the stream settings
    /// If no stream has been started yet, this change has no effect
    pub fn update_settings(&mut self, settings: Settings) {
        // Update to access the stream
        if let Some(guard) = self.buffer.as_deref() {
            if let Ok(mut buffer) = guard.lock() {
                buffer.settings = settings;
            }
        }
    }

    /// Update the color of the effect.
    /// If the effect produces his own color, these change will have no effect.
    pub fn update_color(&mut self, rgb: [u8; 3]) {
        // Try to access the stream and lock the buffer
        if let Some(buffer) = self.buffer.as_deref() {
            if let Ok(mut buffer) = buffer.lock() {
                // Update the color, if available
                buffer.color = rgb;
            }
        }
    }

    /// Update the selected effect
    pub fn update_effect(&mut self, effect: Box<dyn AudioEffect>) {
        // Try to access the stream and lock the buffer
        if let Some(buffer) = self.buffer.as_deref() {
            if let Ok(mut buffer) = buffer.lock() {
                buffer.effect = effect;
            }
        }
    }

    pub fn is_color_selection_available(&self) -> crate::Result<bool> {
        let guard = self.buffer.as_deref()
            .ok_or(ControllerError::NoStream)?
            .lock()
            .unwrap();

        Ok(!guard.effect.disable_color_wheel())
    }

}