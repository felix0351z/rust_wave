use super::*;

pub struct SpectrumEffect {
    gain_filter: GainFilter,
    smooth_filter: SmoothingFilter,
}

impl SpectrumEffect {
    pub fn new() -> SpectrumEffect {
        SpectrumEffect {
            gain_filter: GainFilter::gain_settings(),
            smooth_filter: SmoothingFilter::smoothing_settings(),
        }
    }
}

impl AudioEffect for SpectrumEffect {

    fn visualize(&mut self, data: AudioData) -> Vec<f32> {
        let mut buffer = data.melbank.to_vec();

        apply_gain_filter(&mut buffer, &mut self.gain_filter);
        apply_smoothing_filter(&mut buffer, &mut self.smooth_filter);

        // Reflect the signal in the middle
        let mut out = Vec::from_iter(buffer.iter().cloned().rev());
        out.append(&mut buffer);

        out
    }

    fn amount_melbank_bins(&self, amount_led_bins: usize) -> usize {
        amount_led_bins/2
    }

}