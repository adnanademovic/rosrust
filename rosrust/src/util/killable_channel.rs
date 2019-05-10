use super::kill;
pub use super::kill::KillMode;
use crossbeam::channel;
pub use crossbeam::channel::Sender;

pub struct Receiver<T> {
    pub data_rx: channel::Receiver<T>,
    pub kill_rx: kill::Receiver,
}

impl<T> Iterator for Receiver<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        channel::select! {
            recv(self.data_rx) -> msg => msg.ok(),
            recv(self.kill_rx.kill_rx) -> _ => None,
        }
    }
}

pub type Killer = kill::Sender;

#[allow(dead_code)]
pub enum SendMode {
    Sync,
    Bounded(usize),
    Unbounded,
}

impl SendMode {
    fn into_channel<T>(self) -> (channel::Sender<T>, channel::Receiver<T>) {
        match self {
            SendMode::Sync => channel::bounded(0),
            SendMode::Bounded(queue_size) => channel::bounded(queue_size),
            SendMode::Unbounded => channel::unbounded(),
        }
    }
}

pub fn channel<T>(send_mode: SendMode, kill_mode: KillMode) -> (Killer, Sender<T>, Receiver<T>) {
    let (data_tx, data_rx) = send_mode.into_channel();
    let (kill_tx, kill_rx) = kill::channel(kill_mode);
    (kill_tx, data_tx, Receiver { data_rx, kill_rx })
}
