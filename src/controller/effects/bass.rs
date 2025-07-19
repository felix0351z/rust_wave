use crate::controller::dsp::apply_mel_matrix;
use crate::controller::dsp::detection::PeakDetector;
use crate::controller::effects::{AudioData, AudioEffect, GainFilter, SmoothingFilter};
use crate::controller::math::gaussian_curve;

const ACCURACY: f32 = 0.1;
const SENSITIVITY: f32 = 1.5;
const GAIN_DECAY: f32 = 0.001;
const SMOOTHING: (f32, f32) = (0.6, 0.05);

pub struct BassEffect {
    gain_filter: GainFilter,
    smoothing_filter: SmoothingFilter,
    peak_detector: PeakDetector
}

impl BassEffect {

    pub fn new() -> Self {
        BassEffect {
            gain_filter: GainFilter::gain_settings(),
            smoothing_filter: SmoothingFilter::smoothing_settings(),
            peak_detector: PeakDetector::new(ACCURACY, SENSITIVITY, GAIN_DECAY, SMOOTHING)
        }
    }


}

impl AudioEffect for BassEffect {
    fn visualize(&mut self, data: AudioData) -> Vec<f32> {
        let size = data.melbank.len();
        let melbank = apply_mel_matrix(data.power_spectrum, 0.0, 200.0, size, data.sample_rate);
        let (output, _) = self.peak_detector.update(melbank.as_slice());

        let mut gaussian = gaussian_curve(size, 10.0);
        // Apply the output to the gaussian curve
        for value in gaussian.iter_mut() {
            *value *= output
        }

        gaussian
    }

}