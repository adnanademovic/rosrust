use crossbeam::channel;

pub enum KillMode {
    Sync,
    Async,
}

impl KillMode {
    fn into_channel<T>(self) -> (channel::Sender<T>, channel::Receiver<T>) {
        match self {
            KillMode::Sync => channel::bounded(0),
            KillMode::Async => channel::unbounded(),
        }
    }
}

#[derive(Clone)]
pub struct Sender {
    pub kill_tx: channel::Sender<()>,
}

impl Sender {
    pub fn send(&self) -> Result<(), channel::SendError<()>> {
        self.kill_tx.send(())
    }
}

pub struct Receiver {
    pub kill_rx: channel::Receiver<()>,
}

impl Receiver {
    pub fn try_recv(&self) -> Result<(), channel::TryRecvError> {
        self.kill_rx.try_recv()
    }
}

pub fn channel(kill_mode: KillMode) -> (Sender, Receiver) {
    let (kill_tx, kill_rx) = kill_mode.into_channel();
    (Sender { kill_tx }, Receiver { kill_rx })
}
