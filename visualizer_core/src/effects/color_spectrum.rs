use super::*;
use crate::math::{transpose, Flip};
use crate::stream::channel::Frame;

const COLOR_LOW: [u8; 3] = [255, 0, 0];
const COLOR_MIDDLE: [u8; 3] = [0, 255, 0];
const COLOR_HIGH: [u8; 3] = [0, 0, 255];

pub struct ColorSpectrumEffect {
    gain_filter: GainFilter,
    smooth_filter: SmoothingFilter,
}

impl ColorSpectrumEffect {

    pub fn new() -> ColorSpectrumEffect {
        ColorSpectrumEffect {
            gain_filter: GainFilter::gain_settings(),
            smooth_filter: SmoothingFilter::smoothing_settings(),
        }
    }

    pub fn animate_color_spectrum(&mut self, data: AudioData) -> Vec<u8> {
        let mut buffer = data.melbank.to_vec();
        apply_gain_filter(&mut buffer, &mut self.gain_filter);
        apply_smoothing_filter(&mut buffer, &mut self.smooth_filter);

        let chunk_len = buffer.len() / 3;
        let c: Vec<&[f32]> = (0..3)
            .map(|i| &buffer[i * chunk_len..(i+1) * chunk_len])
            .collect();

        let low = [c[0], c[0], c[0]].concat();
        let middle = [c[1], c[1], c[1]].concat();
        let high = [c[2], c[2], c[2]].concat();

        let low = transpose([low.clone_flip(), low].concat().as_slice(), COLOR_LOW);
        let mut middle = transpose([middle.clone_flip(), middle].concat().as_slice(), COLOR_MIDDLE);
        let high = transpose([high.clone_flip(), high].concat().as_slice(), COLOR_HIGH);

        for (v_low, v_middle) in low.iter().zip(middle.iter_mut()) {
            if *v_low > *v_middle {
                *v_middle = *v_low;
            }
        }
        for (v_middle, v_high) in middle.iter_mut().zip(high.iter()) {
            if *v_high > *v_middle {
                *v_middle = *v_high;
            }
        }

        middle
    }

}


impl AudioEffect for ColorSpectrumEffect {
    fn visualize(&mut self, _: AudioData) -> Vec<f32> {
        unimplemented!()
    }

    fn transpose_animation(&mut self, data: AudioData) -> Frame {
        let animation = self.animate_color_spectrum(data);

        Frame {
            data: Some(animation),
            view: None,
        }
    }

    fn amount_melbank_bins(&self, led_amount: usize) -> usize {
        led_amount/2
    }

    fn disable_color_wheel(&self) -> bool {
        true
    }
}