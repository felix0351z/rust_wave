use crate::controller::channel::{Frame, ViewFrame};
use crate::controller::dsp::smoothing::ExponentialFilter;
use crate::controller::math::transpose;
use crate::controller::stream::Settings;

pub mod melbank;
pub mod spectrum;
pub mod shine;
pub mod fft;
pub mod color_spectrum;
pub mod energy;

type GainFilter = ExponentialFilter<f32>;
type SmoothingFilter = ExponentialFilter<Vec<f32>>;

pub struct AudioData<'a> {
    pub(crate) melbank: &'a[f32],
    pub(crate) power_spectrum: &'a [f32],
    pub(crate) raw_data: &'a [f32],
    pub(crate) settings: Settings,
    pub(crate) sample_rate: u32,
    pub color: [u8; 3],
}


/// Color Object to paint effects and proceed color changes.
pub struct Color {
    /// The current color
    color: [u8; 3],
    transition_time: u8,
    /// The current amount of frames which are needed to finish a color change
    transition_step: Option<u8>,
    /// The transition color values, which will be added to the current color to proceed a color change
    transition_color: Option<[i16; 3]>,
}

impl Color {

    /// The default time how many frames a color change needs to take.
    const DEFAULT_TRANSITION_TIME: u8 = 20;

    /// Create a new Color-object in the RGB-Format
    pub fn new(rgb: [u8; 3]) -> Self {
        Color {
            color: rgb,
            transition_time: Self::DEFAULT_TRANSITION_TIME,
            transition_step: None,
            transition_color: None,
        }
    }

    pub fn white() -> Self { Self::new([255; 3]) }

    /// Get the current color in the RGB-Format
    pub fn rgb(&mut self) ->[u8; 3] {
        // If there is any transition going on
        if let Some(transition_step) = self.transition_step {
            // Show if the transition reached the end and reset
            if transition_step == 0 {
                self.transition_step = None;
                self.transition_color = None;
            }
            // Update the transition
            if let Some(transition_color) = self.transition_color {
                let r = self.color[0] as i16 + transition_color[0];
                let g = self.color[1] as i16 + transition_color[1];
                let b = self.color[2] as i16 + transition_color[2];

                self.color = [r as u8, g as u8, b as u8];
                // Update step
                self.transition_step = Some(transition_step-1);
            }

        }

        self.color
    }

    /// Do a color change
    pub fn change_color(&mut self, rgb: [u8; 3]) {
        let step_r = (rgb[0] as i16 - self.color[0] as i16) / self.transition_time as i16;
        let step_g = rgb[1] as i16 - self.color[1] as i16 / self.transition_time as i16;
        let step_b = rgb[2] as i16 - self.color[2] as i16 / self.transition_time as i16;

        self.transition_step = Some(self.transition_time);
        self.transition_color = Some([step_r, step_g, step_b]);
    }

    pub fn change_transition_time(&mut self, transition_time: u8) {
        self.transition_time = transition_time;
    }

}

pub trait AudioEffect: Send + 'static {

    fn visualize(&mut self, data: AudioData) -> Vec<f32>;

    /// If the effect produces a color on its own, the color selector should be disabled.
    fn transpose_animation(&mut self, data: AudioData) -> Frame {
        let color = data.color;
        let x = self.visualize(data);
        let transposed = transpose(x.as_slice(), color);

        Frame {
            data: Some(transposed),
            view: Some(ViewFrame {
                effect: x,
                color,
            }),
        }
    }

    fn amount_melbank_bins(&self, led_amount: usize) -> usize { led_amount }

    fn disable_color_wheel(&self) -> bool { false }

}

pub struct EffectDescription {
    pub name: &'static str,
    pub factory: fn() -> Box<dyn AudioEffect>
}


fn apply_gain_filter(buffer: &mut [f32], filter: &mut GainFilter) {
    // Apply the gain filter
    if let Some(max) = buffer.iter().max_by(|x, y| x.partial_cmp(y).unwrap()) {
        // Get the max of the frame and update the gain filter
        let max = filter.update(*max);
        // Then apply the gain for every value in the frame
        buffer.iter_mut()
            .for_each(|val| *val /= max);
    }
}

fn apply_smoothing_filter(buffer: &mut [f32], filter: &mut SmoothingFilter) {
    // Apply the smoothing filter
    filter.update(buffer);
}

#[macro_export]
macro_rules! register_effects {
    ( $( $name:literal => $constructor:path ),* $(,)? ) => {
        vec![
            $(
                EffectDescription {
                    name: $name,
                    factory: || Box::new($constructor()),
                }
            ),*
        ]
    };
}