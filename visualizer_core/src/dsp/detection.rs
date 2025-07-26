use super::smoothing::ExponentialFilter;


pub struct PeakDetector {
    average_filter: ExponentialFilter<f32>,
    gain_filter: ExponentialFilter<f32>,
    smooth_filter: ExponentialFilter<f32>,
    sensitivity: f32,
    on_peak: bool
}

impl PeakDetector {


    /// Creates a new peak detector object
    ///
    /// accuracy: Defines the type of peaks to detect. From 0.1 to 0.9
    /// A higher value means a faster adjustment to the original signal and a better detection for short and high peaks. (Heavily used in Hip-Hop, Pop).
    /// A lower value means a less adjustment to the original signal and a better detection for long peaks. (Used in Rock or Punk etc.)
    ///
    /// sensitivity: Defines how much the current signal should be higher than the average signal to detect a peak.
    /// Mostly 1.5 times higher or 2 times higher values are used
    ///
    /// gain_decay: Defines how fast the detector adjusts himself the output signal to the actual volume of the signal
    ///
    /// smoothing: Smoothed output signal if wished. Can be None
    pub fn new(
        accuracy: f32,
        sensitivity: f32,
        gain_decay: f32,
        smoothing: (f32, f32),
    ) -> PeakDetector {
        PeakDetector {
            average_filter: ExponentialFilter::new(0.1, 0.1, accuracy),
            gain_filter: ExponentialFilter::new(0.1, 0.9, gain_decay),
            smooth_filter: ExponentialFilter::new(0.1, smoothing.0, smoothing.1),
            sensitivity,
            on_peak: false,
        }
    }

    fn check_begin_and_end(&mut self, value: f32) -> Option<bool> {
        if self.on_peak && value < 0.1 {
            self.on_peak = false;
            return Some(self.on_peak);
        }
        if !self.on_peak && value > 0.1 {
            self.on_peak = true;
            return Some(self.on_peak);
        }

        None
    }

    pub fn update(&mut self, melbank: &[f32]) -> (f32, Option<bool>) {

        // Get the sum of all frequencies together
        let sum = melbank.iter().sum::<f32>();

        let average_value = self.average_filter.update(sum);

        // If the current sum is (sensitivity) times bigger than the average curve, a peak will be delivered.
        let mut output_value = if sum > average_value*self.sensitivity { sum } else { 0.0 };

        // Do a maximum gain update
        let current_gain = self.gain_filter.update(output_value);

        // If the delivered value is two times smaller than the highest sum, the peak is too small and will not be counted
        if output_value < (current_gain / 2.0) { output_value = 0.0 }

        // Gain normalization
        output_value = output_value / current_gain;

        // Apply the smoothing filter
        output_value = self.smooth_filter.update(output_value);

        // If a peak started or ended, notify
        (output_value, self.check_begin_and_end(output_value))
    }



}