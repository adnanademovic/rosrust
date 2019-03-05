use crossbeam::channel::{unbounded, Receiver, SendError, Sender, TrySendError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn lossy_channel<T>(queue_size: usize) -> (LossySender<T>, LossyReceiver<T>) {
    let (tx, rx) = unbounded();
    let is_open = Arc::new(AtomicBool::new(true));
    let receiver = rx.clone();
    let sender = LossySender {
        tx,
        rx,
        is_open,
        queue_size,
    };
    (sender, receiver)
}

#[derive(Clone)]
pub struct LossySender<T> {
    tx: Sender<Option<T>>,
    rx: Receiver<Option<T>>,
    is_open: Arc<AtomicBool>,
    queue_size: usize,
}

impl<T> LossySender<T> {
    pub fn try_send(&self, msg: T) -> Result<(), TrySendError<Option<T>>> {
        if !self.is_open.load(Ordering::SeqCst) {
            return Err(TrySendError::Disconnected(Some(msg)));
        }
        self.tx.try_send(Some(msg))?;
        self.remove_extra_data()
    }

    pub fn close(&self) -> Result<(), SendError<Option<T>>> {
        self.is_open.store(false, Ordering::SeqCst);
        self.tx.send(None)
    }

    fn remove_extra_data(&self) -> Result<(), TrySendError<Option<T>>> {
        if self.queue_size == 0 {
            return Ok(());
        }
        while self.rx.len() > self.queue_size {
            if self.rx.try_recv().is_err() {
                log::error!("Failed to remove excess data from message queue");
            }
        }
        Ok(())
    }
}

pub type LossyReceiver<T> = Receiver<Option<T>>;
