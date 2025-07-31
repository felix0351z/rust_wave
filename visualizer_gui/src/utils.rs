use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use egui_plot::PlotPoints;
use visualizer_core::StreamFrame;

/// Custom stream reader to always cache the latest frame from the original stream.
/// This ensures that the UI-Thread does not need to wait to receive any updates.
pub struct StreamReader {
    frame_mutex: Arc<Mutex<Option<StreamFrame>>>,
    thread: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>
}
impl StreamReader {


    /// Create an ew StreamReader object.
    /// Use the start() method to listen to an active stream.
    pub fn new() -> StreamReader {
        StreamReader {
            frame_mutex:  Arc::new(Mutex::new(None)),
            thread: None,
            running: Arc::new(AtomicBool::new(false)) }
    }

    /// Listen to a receiver from the audio stream.
    /// If the reader is already listening, the receiver of the older stream will be dropped.
    pub fn start(&mut self, rx: Receiver<StreamFrame>) {
        let guard = self.frame_mutex.clone();
        if let Some(thread) = self.thread.take() {
            // Close the old thread before starting the new one
            self.running.store(true, SeqCst);
            let _ = thread.join();
        }

        self.running.store(true, SeqCst);
        let running = self.running.clone();
        thread::spawn(move || {
            while running.load(SeqCst) {
                match rx.recv() {
                    // Update the last frame
                    Ok(frame_update) => {
                        *guard.lock().unwrap() = Some(frame_update);
                    }
                    // Sender closed
                    Err(_) => { break; }
                };
            }
        });
    }

    /// Get the last received frame.
    pub fn lock_frame(&self) -> MutexGuard<Option<StreamFrame>> {
        self.frame_mutex.lock().expect("Failed to lock mutex. Has the thread panicked?")
    }

}

pub trait MapToPlotPoints {
    fn to_plot_points(&self, logarithmic_scale: bool) -> PlotPoints<'static>;
}

impl MapToPlotPoints for Vec<f32> {

    /// Map the vector to egui_plots' PlotPoints object.
    /// If logarithmic_scale is set to true, the data will be logarithmically aligned along the X-Axis.
    fn to_plot_points(&self, logarithmic_scale: bool) -> PlotPoints<'static> {
        // Map the values to a list of plot points
        // Use the iterator as x and the vec as y
        (0..self.len())
            .map(|i| {
                let x = if logarithmic_scale { (i as f64).log10() } else { i as f64 };
                let y = self[i] as f64;
                [x, y] })
            .collect()
    }
}