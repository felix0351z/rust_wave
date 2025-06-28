
use realfft::num_traits::abs;
use crate::controller::math::linspace;

/// Convert the frequency in heart to the mel scale
pub fn heart_to_mel(f: f32) -> f32 {
    2595.0 * (1.0 + (f/700.0)).log10()
}

/// Convert the frequency in mel to the heart scale
pub fn mel_to_heart(m: f32) -> f32 {
    700.0 * (10.0_f32.powf(m/2595.0)  - 1.0)
}

/// Creates a list with all bins with their frequency (in mel) as their value
pub fn get_mel_frequency_bins(n_bins: usize, min_frequency: f32, max_frequency: f32) -> Vec<f32> {
    // Get the min and max frequency in the mel scale
    let min = heart_to_mel(min_frequency);
    let max = heart_to_mel(max_frequency);

    // Calculate the space between the bins
    let delta_mel = abs(max - min) / (n_bins + 1) as f32;

    // With the information about the space between the bins, create a list with all the bins with their frequency (in mel) as value
    // Note that 2 extra bins are needed to create the matrix
    let mut frequency_bins: Vec<f32> = vec![0.0; n_bins + 2];
    for b in 0..n_bins+2 {
        // bn = min + d * n
        frequency_bins[b] = min + delta_mel * b as f32;
    }

    frequency_bins
}

pub fn compute_mel_matrix(n_bins: usize, n_fft_bins: usize, min_freq: f32, max_freq: f32, sample_rate: u16) -> Vec<Vec<f32>> {
    // Get the mel frequency bins
    let bins = get_mel_frequency_bins(n_bins, min_freq, max_freq);

    // Subdivide the frequencies into center, lower and upper bands
    let center_bins = &bins[1..bins.len()-1];
    let lower_edges = &bins[..bins.len()-2];
    let upper_edges = &bins[2..bins.len()];

    // Transform the bands to its hz value
    let center_bins_hz = center_bins.iter()
        .map(|m| mel_to_heart(*m)).collect::<Vec<f32>>();
    let lower_edges_hz = lower_edges.iter()
        .map(|m| mel_to_heart(*m)).collect::<Vec<f32>>();
    let upper_edges_hz = upper_edges.iter()
        .map(|m| mel_to_heart(*m)).collect::<Vec<f32>>();

    // Create a list with all frequency bins from the original fft
    let frequencies = linspace(0.0, (sample_rate / 2) as f32, n_fft_bins);

    // Create the matrix with the mel bins as x and the fft bins as y
    let mut matrix = vec![vec![0.0; n_fft_bins]; n_bins];

    // Go through every mel bin with the center, lower and upper bands
    // These bands are necessary to check if the frequency is in the mel bin and later calculate their weight
    for (((i, center),lower), upper) in center_bins_hz.iter().enumerate().zip(lower_edges_hz).zip(upper_edges_hz) {

        // Go through every fft bin
        for (j, f) in frequencies.iter().enumerate() {
            // Check if the current frequency f is in the left slope of the mel bin
            if *f >= lower && *f <= *center {
                // Calculate the weight of the frequency f
                let weight = (*f - lower) / (*center - lower);
                // Add the weight to the matrix
                matrix[i][j] = weight;
            }

            // Check also if the current frequency f is in the right slope
            if *f >= *center && *f <= upper {
                // Calculate the weight of the frequency f
                let weight = (upper - *f) / (upper - *center);
                // Add the weight to the matrix
                matrix[i][j] = weight;
            }
        }
    }

    matrix
}