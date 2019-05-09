use crate::util::FAILED_TO_LOCK;
use crossbeam::channel::{self, unbounded, Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[allow(clippy::mutex_atomic)]
pub fn lossy_channel<T>(queue_size: usize) -> (LossySender<T>, LossyReceiver<T>) {
    let (data_tx, data_rx) = unbounded();
    let (kill_tx, kill_rx) = unbounded();
    let is_open = Arc::new(AtomicBool::new(true));
    let receiver = LossyReceiver {
        data_rx: data_rx.clone(),
        kill_rx,
    };
    let queue_size = Arc::new(Mutex::new(queue_size));
    let sender = LossySender {
        data_tx,
        data_rx,
        kill_tx,
        is_open,
        queue_size,
    };
    (sender, receiver)
}

#[derive(Clone)]
pub struct LossySender<T> {
    data_tx: Sender<T>,
    data_rx: Receiver<T>,
    kill_tx: Sender<()>,
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

    pub fn close(&self) -> Result<(), channel::SendError<()>> {
        self.is_open.store(false, Ordering::SeqCst);
        self.kill_tx.send(())
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

pub struct LossyReceiver<T> {
    pub data_rx: Receiver<T>,
    pub kill_rx: Receiver<()>,
}

impl<T> Iterator for LossyReceiver<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        channel::select! {
            recv(self.data_rx) -> msg => msg.ok(),
            recv(self.kill_rx) -> _ => None,
        }
    }
}
