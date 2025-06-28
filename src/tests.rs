use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Send a 20s second long light with a changing color from red to white
#[test]
fn test_sacn_sender() {
    let sender = super::controller::sender::Sender::new_multicast_sender();
    let (s, r) = mpsc::channel::<Vec<u8>>();
    sender.listen(r);

    let signal = vec![1f32; 60];
    let mut color = true;
    for i in 0..20 {
        thread::sleep(Duration::from_secs(1));
        let r = 255;
        let g = if color { 0 } else { 255 };
        let b = if color { 0 } else { 255 };
        let transposed = super::controller::math::transpose(signal.as_slice(), (r, g, b));
        s.send(transposed).unwrap();
        color = !color;
    }

}