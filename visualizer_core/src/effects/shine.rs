use super::*;
use crate::dsp::{PeakDetector, apply_mel_matrix};
use crate::math::{transpose, Flip};
use crate::stream::channel::{Frame, ViewFrame};

pub struct ShineEffect {
    gain_filter: GainFilter,
    smooth_filter: SmoothingFilter,
    peak_detector: PeakDetector,
    color: Color
}

const SHINE_FREQ: (f32, f32) = (0.0, 200.0);
const SHINE_SMOOTHING: (f32, f32) = (0.8, 0.1);
const SHINE_COLOR: [u8; 3] = [255; 3];
const TRANSITION_TIME: u8 = 3;

impl ShineEffect {
    pub fn new() -> ShineEffect {
        let detector =  PeakDetector::new(
            0.1,
            1.5,
            0.0001,
            (SHINE_SMOOTHING.0, SHINE_SMOOTHING.1),
        );
        let color = Color::new(SHINE_COLOR);

        ShineEffect {
            gain_filter: GainFilter::gain_settings(),
            smooth_filter: SmoothingFilter::smoothing_settings(),
            peak_detector: detector,
            color,
        }
    }
}

impl ShineEffect {

    fn build_spectrum_animation(&mut self, melbank: &[f32]) -> Vec<f32> {
        let mut buffer = melbank.to_vec();

        apply_gain_filter(&mut buffer, &mut self.gain_filter);
        apply_smoothing_filter(&mut buffer, &mut self.smooth_filter);

        // Reflect the signal in the middle
        [buffer.clone_flip(), buffer].concat()
    }

    fn build_shine_animation(&mut self, data: &AudioData) -> Vec<f32> {
        let melbank = apply_mel_matrix(data.power_spectrum, SHINE_FREQ.0, SHINE_FREQ.1, 60, data.sample_rate);

        let (peak_value, peak_update) = self.peak_detector.update(melbank.as_slice());

        let mut out = vec![1.0f32; data.settings.n_bins];
        for x in out.iter_mut() {
            *x = *x * peak_value;
        }

        // Update if a peak started, or ended
        if let Some(peak_update) = peak_update {
            self.peak_changed(data.color, peak_update)
        }

        out
    }

    fn peak_changed(&mut self, default_color: [u8; 3],  started: bool) {
        let color = if started { SHINE_COLOR } else { default_color };
        let time = if started { TRANSITION_TIME*2 } else { TRANSITION_TIME };

        self.color.change_color(color);
        self.color.change_transition_time(time)
    }



}

impl AudioEffect for ShineEffect {

    fn visualize(&mut self, data: AudioData) -> Vec<f32> {

        let mut main_animation = self.build_spectrum_animation(data.melbank);
        let shine_animation = self.build_shine_animation(&data);

        // Take the stronger animation
        for (x_main, x_shine) in main_animation.iter_mut().zip(shine_animation.iter()) {
            if x_shine > x_main {
                *x_main = *x_shine;
            }
        }

        main_animation
    }

    fn transpose_animation(&mut self, data: AudioData) -> Frame {
        let animation = self.visualize(data);

        // Update the color
        let color = self.color.rgb();
        // Transpose the signal with the color
        let transposed = transpose(animation.as_slice(), color);

        Frame {
            data: Some(transposed),
            view: Some(ViewFrame {
                effect: animation,
                color,
            }),
        }
    }

    fn amount_melbank_bins(&self, amount_led_bins: usize) -> usize {
        amount_led_bins/2
    }

}