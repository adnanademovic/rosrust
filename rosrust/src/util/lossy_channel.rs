use crate::util::killable_channel::{channel, KillMode, Killer, Receiver, SendMode, Sender};
use crate::util::FAILED_TO_LOCK;
use crossbeam::channel;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[allow(clippy::mutex_atomic)]
pub fn lossy_channel<T>(queue_size: usize) -> (LossySender<T>, LossyReceiver<T>) {
    let (killer, data_tx, receiver) = channel(SendMode::Unbounded, KillMode::Async);
    let is_open = Arc::new(AtomicBool::new(true));
    let queue_size = Arc::new(Mutex::new(queue_size));
    let sender = LossySender {
        data_tx,
        data_rx: receiver.data_rx.clone(),
        killer,
        is_open,
        queue_size,
    };
    (sender, receiver)
}

#[derive(Clone)]
pub struct LossySender<T> {
    data_tx: Sender<T>,
    data_rx: channel::Receiver<T>,
    killer: Killer,
    is_open: Arc<AtomicBool>,
    pub queue_size: Arc<Mutex<usize>>,
}

impl<T> LossySender<T> {
    pub fn try_send(&self, msg: T) -> Result<(), channel::TrySendError<T>> {
        if !self.is_open.load(Ordering::SeqCst) {
            return Err(channel::TrySendError::Disconnected(msg));
        }
        self.data_tx.try_send(msg)?;
        self.remove_extra_data();
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), channel::SendError<()>> {
        self.is_open.store(false, Ordering::SeqCst);
        self.killer.send()
    }

    fn remove_extra_data(&self) {
        let queue_size: usize = *self.queue_size.lock().expect(FAILED_TO_LOCK);
        while self.data_rx.len() > queue_size {
            if self.data_rx.try_recv().is_err() {
                log::error!("Failed to remove excess data from message queue");
                break;
            }
        }
    }

    pub fn set_queue_size(&self, queue_size: usize) {
        *self.queue_size.lock().expect(FAILED_TO_LOCK) = queue_size;
    }

    pub fn set_queue_size_max(&self, queue_size: usize) {
        let mut current_size = self.queue_size.lock().expect(FAILED_TO_LOCK);
        *current_size = current_size.max(queue_size);
    }
}

pub type LossyReceiver<T> = Receiver<T>;
