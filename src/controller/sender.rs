use sacn::packet::ACN_SDT_MULTICAST_PORT;
use sacn::source::SacnSource;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use log::error;
use std::sync::mpsc::Receiver;
use crate::ControllerError::SacnError;

/// Default Universe the data is to be sent on
const UNIVERSE: u16 = 1;

type Result<T> = std::result::Result<T, crate::ControllerError>;

pub struct Sender {
    source: Arc<Mutex<SacnSource>>,
    universe: u16,
    /*dst_ip: Option<SocketAddr>,
    sync_universe: Option<u16>,
    priority: Option<u8>*/
}

impl Sender {

    /// Create a new Sacn sender with Multicast enabled.
    pub fn new_multicast_sender() -> Sender {
        let addr = SocketAddr::new(IpAddr::V4("0.0.0.0".parse().unwrap()), ACN_SDT_MULTICAST_PORT + 1);
        let mut src = SacnSource::with_ip("Source", addr)
            .expect("Sacn source can't be bound to local ip address");

        src.register_universe(UNIVERSE).expect("Can't register default universe");
        Sender {
            source: Arc::new(Mutex::new(src)),
            universe: UNIVERSE, // Use default universe at the beginning
            /*dst_ip: None, // Set to None to use multicast
            sync_universe: None, // Do not synchronize
            priority: None,*/
        }
    }

    /// Listen to the receiver and send all incoming data over sacn
    pub fn listen(&self, receiver: Receiver<Vec<u8>>) {
        // Prepare values for the send-thread
        let source = self.source.clone();
        let universe = self.universe;

        // Spawn a new thread to send the data
        thread::spawn(move || {
            loop {
                if let Ok(data) = receiver.recv() {
                    // Only send effects
                    Self::send(&source, data.as_slice(), universe);
                } else {
                    break; // Break the loop if the channel disconnected
                }
            }
        });
    }

    /// Change the universe of the sender
    pub fn set_universe(&mut self, universe: u16) -> Result<()> {
        // Lock the source
        let mut source = self.source.lock().expect("Thread failed while holding the lock");

        // Register the new universe, if successful disconnect from old universe and update
        match source.register_universe(universe) {
            Ok(_) => {
                match source.terminate_stream(self.universe, 0) {
                    Ok(_) => {
                        Ok(self.universe = universe)
                    }
                    Err(err) => { Err(SacnError(err)) }
                }
            }
            Err(err) => { Err(SacnError(err)) }
        }
    }

    /// Get the currently used universe
    fn get_universe(&self) -> u16 {
        self.universe
    }

    fn send(source: &Arc<Mutex<SacnSource>>, data: &[u8], universe: u16, /*priority: Option<u8>, dst_ip: Option<SocketAddr>, sync_universe: Option<u16>*/)  {
        // Lock the source and send the data
        let mut source = source.lock().expect("Thread failed while holding the lock");
        match source.send(&[universe], data, None, None, None) {
            Ok(_) => {},
            Err(error) => {
                error!("Error occurred while sending sacn data: {}", error);
            }
        }
    }


}
