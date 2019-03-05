use crossbeam::channel::{unbounded, Receiver, SendError, Sender, TrySendError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[allow(clippy::mutex_atomic)]
pub fn lossy_channel<T>(queue_size: usize) -> (LossySender<T>, LossyReceiver<T>) {
    let (tx, rx) = unbounded();
    let is_open = Arc::new(AtomicBool::new(true));
    let receiver = rx.clone();
    let queue_size = Arc::new(Mutex::new(queue_size));
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
    pub queue_size: Arc<Mutex<usize>>,
}

impl<T> LossySender<T> {
    pub fn try_send(&self, msg: T) -> Result<(), TrySendError<Option<T>>> {
        if !self.is_open.load(Ordering::SeqCst) {
            return Err(TrySendError::Disconnected(Some(msg)));
        }
        self.tx.try_send(Some(msg))?;
        self.remove_extra_data();
        Ok(())
    }

    pub fn close(&self) -> Result<(), SendError<Option<T>>> {
        self.is_open.store(false, Ordering::SeqCst);
        self.tx.send(None)
    }

    fn remove_extra_data(&self) {
        let queue_size: usize = *self.queue_size.lock().expect(FAILED_TO_LOCK);
        while self.rx.len() > queue_size {
            if self.rx.try_recv().is_err() {
                log::error!("Failed to remove excess data from message queue");
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

pub type LossyReceiver<T> = Receiver<Option<T>>;

static FAILED_TO_LOCK: &'static str = "Failed to acquire lock";
