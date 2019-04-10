use crate::util::lossy_channel::{lossy_channel, LossyReceiver, LossySender};
use crossbeam::channel::{self, unbounded, Receiver, Sender};
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

pub fn fork<T: Write + Send + 'static>(queue_size: usize) -> (TargetList<T>, DataStream) {
    let (streams_sender, streams) = unbounded();
    let (data_sender, data) = lossy_channel(queue_size);

    let mut fork_thread = ForkThread::new();
    let target_count = fork_thread.clone_target_count();

    thread::spawn(move || fork_thread.run(&streams, &data));

    (
        TargetList(streams_sender),
        DataStream {
            sender: data_sender,
            target_count,
        },
    )
}

struct ForkThread<T: Write + Send + 'static> {
    targets: Vec<T>,
    target_count: Arc<AtomicUsize>,
}

impl<T: Write + Send + 'static> ForkThread<T> {
    pub fn new() -> Self {
        Self {
            targets: vec![],
            target_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn clone_target_count(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.target_count)
    }

    fn publish_buffer_and_prune_targets(&mut self, buffer: &[u8]) {
        let mut dropped_targets = vec![];
        for (idx, target) in self.targets.iter_mut().enumerate() {
            if target.write_all(&buffer).is_err() {
                dropped_targets.push(idx);
            }
        }

        if !dropped_targets.is_empty() {
            // We reverse the order, to remove bigger indices first.
            for idx in dropped_targets.into_iter().rev() {
                self.targets.swap_remove(idx);
            }

            self.target_count
                .store(self.targets.len(), Ordering::SeqCst);
        }
    }

    fn add_target(&mut self, target: T) {
        self.targets.push(target);
        self.target_count
            .store(self.targets.len(), Ordering::SeqCst);
    }

    fn step(
        &mut self,
        streams: &Receiver<T>,
        data: &LossyReceiver<Arc<Vec<u8>>>,
    ) -> Result<(), channel::RecvError> {
        channel::select! {
            recv(data) -> msg => {
                let buffer = msg?.ok_or(channel::RecvError)?;
                self.publish_buffer_and_prune_targets(&buffer);
            }
            recv(streams) -> target => {
                self.add_target(target?);
            }
        }
        Ok(())
    }

    pub fn run(&mut self, streams: &Receiver<T>, data: &LossyReceiver<Arc<Vec<u8>>>) {
        while self.step(streams, data).is_ok() {}
    }
}

pub type ForkResult = Result<(), ()>;

pub struct TargetList<T: Write + Send + 'static>(Sender<T>);

impl<T: Write + Send + 'static> TargetList<T> {
    pub fn add(&self, stream: T) -> ForkResult {
        self.0.send(stream).or(Err(()))
    }
}

#[derive(Clone)]
pub struct DataStream {
    sender: LossySender<Arc<Vec<u8>>>,
    target_count: Arc<AtomicUsize>,
}

impl DataStream {
    pub fn send(&self, data: Arc<Vec<u8>>) -> ForkResult {
        self.sender.try_send(data).or(Err(()))
    }

    #[inline]
    pub fn get_target_count(&self) -> usize {
        self.target_count.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn set_queue_size(&self, queue_size: usize) {
        self.sender.set_queue_size(queue_size);
    }

    #[inline]
    pub fn set_queue_size_max(&self, queue_size: usize) {
        self.sender.set_queue_size_max(queue_size);
    }
}
