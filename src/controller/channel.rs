use std::sync::mpsc;
use std::sync::mpsc::channel;

pub struct Frame {
    pub data: Option<Vec<u8>>,
    pub view: Option<ViewFrame>
}

pub struct ViewFrame {
    pub effect: Vec<f32>,
    pub color: (u8, u8, u8)
}

pub fn new() -> (Sender, Receiver) {
    let (tx_sacn, rx_sacn) = channel::<Vec<u8>>();
    let (tx_view, rx_view) = channel::<ViewFrame>();

    let tx = Sender { tx_sacn, tx_view };
    let rx = Receiver { rx_sacn, rx_view};

    (tx, rx)
}

pub struct Receiver {
    pub rx_sacn: mpsc::Receiver<Vec<u8>>,
    pub rx_view: mpsc::Receiver<ViewFrame>,
}

pub struct Sender {
    tx_sacn: mpsc::Sender<Vec<u8>>,
    tx_view: mpsc::Sender<ViewFrame>,
}

impl Sender {
    pub fn send(&mut self, frame: Frame) {
        if let Some(data) = frame.data {
            self.tx_sacn.send(data).expect("gffgdfgf");
        }

        if let Some(view) = frame.view {
            self.tx_view.send(view).expect("gffgdfgf");
        }
    }
}