use super::*;
use crate::dsp::ExponentialFilter;
use crate::math::gaussian_curve;

use num_traits::Pow;


const GAIN_RISE: f32 = 0.9;
const GAIN_DECAY: f32 = 0.001;
const SMOOTHING_RISE: f32 = 0.4;
const SMOOTHING_DECAY: f32 = 0.1;
const STANDARD_DEVIATION: f32 = 10.0;

pub struct EnergyEffect {
    gain_filter: GainFilter,
    smoothing_filter: ExponentialFilter<f32>
}

impl EnergyEffect {
    
    pub fn new() -> Self {
        EnergyEffect {
            gain_filter: GainFilter::new(0.1, GAIN_RISE, GAIN_DECAY),
            smoothing_filter: ExponentialFilter::new(0.1, SMOOTHING_RISE, SMOOTHING_DECAY),
        }
    }

    fn smoothed_rms(&mut self, data: AudioData) -> f32 {
        let size = data.raw_data.len();
        let energy = data.raw_data
            .iter()
            .map(|it| it.pow(2))
            .sum::<f32>();

        let rms = (energy / size as f32).sqrt();
        let rms = rms / self.gain_filter.update(rms);

        self.smoothing_filter.update(rms)
    }
    
    
}

impl AudioEffect for EnergyEffect {

    fn visualize(&mut self, data: AudioData) -> Vec<f32> {
        let len = data.melbank.len();
        let mut gaussian = gaussian_curve(len, STANDARD_DEVIATION);
        let smoothed_rms = self.smoothed_rms(data);

        // Apply the rms to the gaussian curve
        for value in gaussian.iter_mut() {
            *value *= smoothed_rms
        }

        gaussian
    }

}