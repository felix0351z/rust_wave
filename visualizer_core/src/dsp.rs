use std::sync::{Arc, Mutex};

use hann_rs::get_hann_window;
use realfft::num_traits::{Pow};
use realfft::RealFftPlanner;

use super::stream;
use super::effects::AudioData;
use super::math::array_product;

// Modules
mod melbank;
mod smoothing;
mod detection;

// Re-export all utilities for the effects
pub use melbank::compute_mel_matrix;
pub use smoothing::ExponentialFilter;
pub use detection::PeakDetector;

type Buffer = Arc<Mutex<stream::InnerStream>>;

/// Entry point for the raw input signal from the sound card
pub fn tick(data: &[f32], buffer: Buffer) {
    // Create a vector with the data of this frame and the last frame
    // This is necessary to prevent data loss during the windowing for the fft
    let mut input = vec![0.0; data.len()*2];
    input[data.len()..].copy_from_slice(data);

    if let Ok(mut buffer) = buffer.lock() {
        // Initialize the frame, if it was not already
        if buffer.last_frame.is_empty() { buffer.last_frame = vec![0.0; data.len()]; }

        // Check if the input data of the frame is correct
        if data.len() != buffer.last_frame.len() { return; }

        // Save the last frame to the input
        input[..data.len()].
            copy_from_slice(&buffer.last_frame);
        // Set the last frame new
        buffer.last_frame.copy_from_slice(data);
    }

    // Apply a pre-emphasis filter on the input signal
    let mut filtered = pre_emphasis(input.as_mut_slice());
    // Apply the threshold filter
    threshold_filter(filtered.as_mut_slice());

    // Get the hanning window
    let hann_window = get_hann_window(filtered.len()).expect("Wrong window length");
    let mut windowed = array_product(&filtered, &hann_window).expect("Wrong window length");

    // Process the fft
    let magnitude = fft(&mut windowed);
    // Calculate the power frames
    let power_frames = magnitude.iter()
        .map(|it| it.pow(2))
        .collect::<Vec<f32>>();


    if let Ok(mut buffer) = buffer.try_lock() {
        // Convert the fft frame to a melbank frame
        let melbank = apply_mel_matrix(
            &power_frames,
            buffer.settings.min_frequency as f32,
            buffer.settings.max_frequency as f32,
            buffer.effect.amount_melbank_bins(buffer.settings.n_bins),
            buffer.sample_rate
        );

        let data = AudioData {
            melbank: melbank.as_slice(),
            power_spectrum: power_frames.as_slice(),
            raw_data: input.as_slice(),
            settings: buffer.settings,
            sample_rate: buffer.sample_rate,
            color: buffer.color
        };

        let out = buffer.effect.transpose_animation(data);

        // Send the data
        buffer.sender.send(out);
    }
}

const PRE_EMPHASIS_CONST: f32 = 0.9;

fn pre_emphasis(x: &[f32]) -> Vec<f32> {
    let mut y = Vec::<f32>::new();

    // y(t) = x(t) - a*x(t-1)
    y.push(x[0]);
    for i in 1..x.len() {
        y.push(x[i] - PRE_EMPHASIS_CONST*x[i-1]);
    }

    y
}

const THRESHOLD: f32 = 0.0002;
fn threshold_filter(x: &mut [f32]) {
    if let Some(max) = x.iter().max_by(|x, y| x.partial_cmp(y).unwrap()) {
        if *max <= THRESHOLD {
            x.iter_mut().for_each(|v| *v = 0.0);
        }

    }

}

fn fft(filtered: &mut [f32]) -> Vec<f32> {
    // Prepare for the fft
    let mut planner = RealFftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(filtered.len());

    // Run the fft and get the spectrum
    let mut spectrum = fft.make_output_vec();
    fft.process(filtered, &mut spectrum).expect("fft process failed. Is the buffer length correct?");
    let amplitude = &spectrum[0..(spectrum.len()/2)];

    // Calculate the magnitude of the fft spectrum
    amplitude.iter().map(|it| {
        it.norm()
    }).collect::<Vec<f32>>()
}

pub fn apply_mel_matrix(fft: &[f32], min_freq: f32, max_freq: f32, bins: usize,  sample_rate: u32) -> Vec<f32> {
    let matrix = melbank::compute_mel_matrix(bins, fft.len(), min_freq, max_freq, sample_rate as u16);

    // Create a vector with the length of the matrix size
    let mut calculated = vec![0.0f32; matrix.len()];

    // go through every mel bin
    for (i, bin) in matrix.iter().enumerate() {

        // and multiply the actual data with the pre-constructed mel bin
        calculated[i] = bin.iter().zip(fft.iter())
            .map(|(a, b)| *a * *b)
            .sum::<f32>();

    }

    calculated
}
