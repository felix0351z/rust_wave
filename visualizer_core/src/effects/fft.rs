use super::*;
use crate::stream::channel::{Frame, ViewFrame};

pub struct FftEffect {
    gain_filter: GainFilter
}

impl FftEffect {
    pub fn new() -> FftEffect {
        FftEffect {
            gain_filter: GainFilter::gain_settings()
        }
    }
}

impl AudioEffect for FftEffect {

    fn visualize(&mut self, data: AudioData) -> Vec<f32> {

        let mut buffer = data.power_spectrum.to_vec();
        apply_gain_filter(&mut buffer, &mut self.gain_filter);

        buffer
    }

    fn transpose_animation(&mut self, data: AudioData) -> Frame {
        let color = data.color;
        let effect = self.visualize(data);

        Frame {
            data: None,
            view: Some(ViewFrame { effect, color })
        }
    }


}