use crate::controller::effects::*;

pub struct MelbankEffect {
    gain_filter: GainFilter,
    smooth_filter: SmoothingFilter,
}


impl MelbankEffect {

    pub fn new() -> MelbankEffect {
        MelbankEffect {
            gain_filter: GainFilter::gain_settings(),
            smooth_filter: SmoothingFilter::smoothing_settings(),
        }
    }
}

impl AudioEffect for MelbankEffect {

    fn visualize(&mut self, data: AudioData) -> Vec<f32> {
        let mut buffer = data.melbank.to_vec();

        apply_gain_filter(&mut buffer, &mut self.gain_filter);
        apply_smoothing_filter(&mut buffer, &mut self.smooth_filter);

        buffer
    }
}