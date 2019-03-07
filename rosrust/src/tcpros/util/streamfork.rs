use crate::util::lossy_channel::{lossy_channel, LossyReceiver, LossySender};
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::collections::VecDeque;
use std::io::Write;
use std::sync::Arc;
use std::thread;

pub fn fork<T: Write + Send + 'static>(queue_size: usize) -> (TargetList<T>, DataStream) {
    let (streams_sender, streams) = unbounded();
    let (data_sender, data) = lossy_channel(queue_size);
    thread::spawn(move || fork_thread::<T>(&streams, &data));
    (
        TargetList(streams_sender),
        DataStream {
            sender: data_sender,
        },
    )
}

fn fill_with_queued<T>(data: &LossyReceiver<T>, queue: &mut VecDeque<T>) -> bool {
    if queue.is_empty() {
        match data.recv() {
            Err(_) | Ok(None) => return false,
            Ok(Some(value)) => queue.push_front(value),
        };
    }
    while let Ok(item) = data.try_recv() {
        match item {
            Some(value) => queue.push_front(value),
            None => return false,
        }
    }
    true
}

fn fork_thread<T: Write + Send + 'static>(
    streams: &Receiver<T>,
    data: &LossyReceiver<Arc<Vec<u8>>>,
) {
    let mut targets = Vec::new();
    let mut datapoints = VecDeque::new();
    let mut sender_is_open = true;
    while sender_is_open {
        sender_is_open = fill_with_queued(data, &mut datapoints);

        let buffer = match datapoints.pop_back() {
            Some(v) => v,
            None => continue,
        };
        while let Ok(target) = streams.try_recv() {
            targets.push(target);
        }
        targets = targets
            .into_iter()
            .filter_map(|mut target| match target.write_all(&buffer) {
                Ok(()) => Some(target),
                Err(_) => None,
            })
            .collect()
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
}

impl DataStream {
    pub fn send(&self, data: Arc<Vec<u8>>) -> ForkResult {
        self.sender.try_send(data).or(Err(()))
    }

    pub fn set_queue_size(&self, queue_size: usize) {
        self.sender.set_queue_size(queue_size);
    }

    pub fn set_queue_size_max(&self, queue_size: usize) {
        self.sender.set_queue_size_max(queue_size);
    }
}
