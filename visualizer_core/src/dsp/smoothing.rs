use crate::FPS;

/// Exponential filter for the types f32 and Vec<f32>
/// with two individual factors for rise or decay
pub struct ExponentialFilter<T> {
    last: T,
    alpha_rise: f32,
    alpha_decay: f32
}

impl <T> ExponentialFilter<T> {
    /// Create a new exponential filter with two individual factors for rise or decay
    pub fn new(last: T, alpha_rise: f32, alpha_decay: f32) -> ExponentialFilter<T> {
        ExponentialFilter {
            last,
            alpha_rise,
            alpha_decay
        }
    }
}

impl ExponentialFilter<f32> {
    /// Calculate the next smoothed value
    pub fn update(&mut self, value: f32) -> f32 {
        // If the new value is bigger than the last, another factor will be used
        // A faster rise and a shorter decay is usually wanted
        let alpha = if value > self.last { self.alpha_rise } else { self.alpha_decay };

        // Use exponential smoothing for the signal
        // y = a*xt + (1 - a) * xt-1
        self.last = alpha * value + (1.0 - alpha) * self.last;
        self.last
    }

    /// Set the default settings for a gain filter
    pub fn gain_settings() -> Self {
        Self {
            last: 0.1,
            alpha_rise: 0.99,
            alpha_decay: 0.1,
        }
    }
}

impl ExponentialFilter<Vec<f32>> {
    /// Calculate the next smoothed value
    pub fn update(&mut self, values: &mut [f32]) {
        // Dot he same exponential smoothing as the implementation for the f32 value, but now for every value in the vector
        for (last, value) in self.last.iter_mut().zip(values.iter_mut()) {
            // A faster rise and a shorter decay is usually wanted
            let alpha = if *value > *last { self.alpha_rise } else { self.alpha_decay };

            // Use exponential smoothing for the signal
            // y = a*xt + (1 - a) * xt-1
            *last = alpha * *value + (1.0 - alpha) * *last;

            // Replace the input with the output
            *value = *last;
        }
    }

    /// Set the default settings for a smoothing filter
    pub fn smoothing_settings() -> Self {
        Self {
            last: vec![0.0; FPS],
            alpha_rise: 0.99,
            alpha_decay: 0.05,
        }
    }
}
