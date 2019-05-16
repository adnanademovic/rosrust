use crate::util::lossy_channel::{lossy_channel, LossyReceiver, LossySender};
use crate::util::FAILED_TO_LOCK;
use crossbeam::channel::{self, unbounded, Receiver, Sender};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn fork<T: Write + Send + 'static>(queue_size: usize) -> (TargetList<T>, DataStream) {
    let (streams_sender, streams) = unbounded();
    let (data_sender, data) = lossy_channel(queue_size);

    let mut fork_thread = ForkThread::new();
    let target_names = Arc::clone(&fork_thread.target_names);

    thread::spawn(move || fork_thread.run(&streams, &data));

    (
        TargetList(streams_sender),
        DataStream {
            sender: data_sender,
            target_names,
        },
    )
}

struct ForkThread<T: Write + Send + 'static> {
    targets: Vec<SubscriberInfo<T>>,
    target_names: Arc<Mutex<TargetNames>>,
}

impl<T: Write + Send + 'static> ForkThread<T> {
    pub fn new() -> Self {
        Self {
            targets: vec![],
            target_names: Arc::new(Mutex::new(TargetNames {
                targets: Vec::new(),
            })),
        }
    }

    fn publish_buffer_and_prune_targets(&mut self, buffer: &[u8]) {
        let mut dropped_targets = vec![];
        for (idx, target) in self.targets.iter_mut().enumerate() {
            if target.stream.write_all(&buffer).is_err() {
                dropped_targets.push(idx);
            }
        }

        if !dropped_targets.is_empty() {
            // We reverse the order, to remove bigger indices first.
            for idx in dropped_targets.into_iter().rev() {
                self.targets.swap_remove(idx);
            }
            self.update_target_names();
        }
    }

    fn add_target(&mut self, target: SubscriberInfo<T>) {
        self.targets.push(target);
        self.update_target_names();
    }

    fn update_target_names(&self) {
        let targets = self
            .targets
            .iter()
            .map(|target| target.caller_id.clone())
            .collect();
        *self.target_names.lock().expect(FAILED_TO_LOCK) = TargetNames { targets };
    }

    fn step(
        &mut self,
        streams: &Receiver<SubscriberInfo<T>>,
        data: &LossyReceiver<Arc<Vec<u8>>>,
    ) -> Result<(), channel::RecvError> {
        channel::select! {
            recv(data.kill_rx.kill_rx) -> msg => {
                return msg.and(Err(channel::RecvError));
            }
            recv(data.data_rx) -> msg => {
                self.publish_buffer_and_prune_targets(&msg?);
            }
            recv(streams) -> target => {
                self.add_target(target?);
            }
        }
        Ok(())
    }

    pub fn run(
        &mut self,
        streams: &Receiver<SubscriberInfo<T>>,
        data: &LossyReceiver<Arc<Vec<u8>>>,
    ) {
        while self.step(streams, data).is_ok() {}
    }
}

pub type ForkResult = Result<(), ()>;

pub struct TargetList<T: Write + Send + 'static>(Sender<SubscriberInfo<T>>);

impl<T: Write + Send + 'static> TargetList<T> {
    pub fn add(&self, caller_id: String, stream: T) -> ForkResult {
        self.0
            .send(SubscriberInfo { caller_id, stream })
            .or(Err(()))
    }
}

struct SubscriberInfo<T> {
    caller_id: String,
    stream: T,
}

#[derive(Clone)]
pub struct DataStream {
    sender: LossySender<Arc<Vec<u8>>>,
    target_names: Arc<Mutex<TargetNames>>,
}

impl DataStream {
    pub fn send(&self, data: Arc<Vec<u8>>) -> ForkResult {
        self.sender.try_send(data).or(Err(()))
    }

    #[inline]
    pub fn target_count(&self) -> usize {
        self.target_names.lock().expect(FAILED_TO_LOCK).count()
    }

    #[inline]
    pub fn target_names(&self) -> Vec<String> {
        self.target_names.lock().expect(FAILED_TO_LOCK).names()
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

#[derive(Debug)]
pub struct TargetNames {
    targets: Vec<String>,
}

impl TargetNames {
    #[inline]
    pub fn count(&self) -> usize {
        self.targets.len()
    }

    #[inline]
    pub fn names(&self) -> Vec<String> {
        self.targets.clone()
    }
}
