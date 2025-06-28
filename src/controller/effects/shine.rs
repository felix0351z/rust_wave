use std::arch::x86_64::_andn_u64;
use crate::controller::dsp::apply_mel_matrix;
use crate::controller::dsp::detection::PeakDetector;
use crate::controller::effects::{apply_gain_filter, apply_smoothing_filter, AudioData, AudioEffect, Color, GainFilter, SmoothingFilter};
use crate::controller::math::transpose;
use crate::controller::channel::{Frame, ViewFrame};

pub struct ShineEffect {
    gain_filter: GainFilter,
    smooth_filter: SmoothingFilter,
    peak_detector: PeakDetector,
    color: Color
}

const SHINE_FREQ: (f32, f32) = (0.0, 200.0);
const SHINE_SMOOTHING: (f32, f32) = (0.8, 0.15);
const MAIN_COLOR: (u8, u8, u8) = (0, 100, 255);
const SHINE_COLOR: (u8, u8, u8) = (255, 255, 255);
const TRANSITION_TIME: u8 = 3;

impl ShineEffect {
    pub fn new() -> ShineEffect {
        let detector =  PeakDetector::new(
            0.1,
            1.5,
            0.0001,
            (SHINE_SMOOTHING.0, SHINE_SMOOTHING.1),
        );
        let color = Color::new(MAIN_COLOR);

        let mut shine_effect = ShineEffect {
            gain_filter: GainFilter::gain_settings(),
            smooth_filter: SmoothingFilter::smoothing_settings(),
            peak_detector: detector,
            color,
        };

        shine_effect
    }
}

impl ShineEffect {

    fn build_spectrum_animation(&mut self, melbank: &[f32]) -> Vec<f32> {
        let mut buffer = melbank.to_vec();

        apply_gain_filter(&mut buffer, &mut self.gain_filter);
        apply_smoothing_filter(&mut buffer, &mut self.smooth_filter);

        // Reflect the signal in the middle
        let mut out = Vec::from_iter(buffer.iter().cloned().rev());
        out.append(&mut buffer);

        out
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
            self.peak_changed(peak_update)
        }

        out
    }

    fn peak_changed(&mut self, started: bool) {
        let color = if started { SHINE_COLOR } else { MAIN_COLOR };
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

    fn visualize_with_color(&mut self, data: AudioData, _color: (u8, u8, u8)) -> Frame {
        let animation = self.visualize(data);

        // Update the color
        let color = self.color.rgb();
        // Transpose the signal with the color
        let transposed = transpose(animation.as_slice(), color);

        Frame {
            data: transposed,
            view: Some(ViewFrame {
                effect: animation,
                color,
            }),
        }
    }

    fn name(&self) -> &'static str {
        "Shine"
    }

    fn description(&self) -> &'static str {
        ""
    }

    fn amount_melbank_bins(&self, amount_led_bins: usize) -> usize {
        amount_led_bins/2
    }

}