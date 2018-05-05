use std::collections::VecDeque;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub fn fork<T: Write + Send + 'static>() -> (TargetList<T>, DataStream) {
    let (streams_sender, streams) = channel();
    let (data_sender, data) = channel();
    let queue_size = Arc::new(AtomicUsize::new(usize::max_value()));
    let queue_size_for_thread = Arc::clone(&queue_size);
    thread::spawn(move || fork_thread::<T>(&streams, data, queue_size_for_thread));
    (
        TargetList(streams_sender),
        DataStream {
            sender: data_sender,
            queue_size,
        },
    )
}

fn fill_with_queued(data: &Receiver<Arc<Vec<u8>>>, queue: &mut VecDeque<Arc<Vec<u8>>>) -> bool {
    if queue.is_empty() {
        match data.recv() {
            Err(_) => return false,
            Ok(item) => queue.push_front(item),
        };
    }
    while let Ok(item) = data.try_recv() {
        queue.push_front(item);
    }
    true
}

fn fork_thread<T: Write + Send + 'static>(
    streams: &Receiver<T>,
    data: Receiver<Arc<Vec<u8>>>,
    queue_size: Arc<AtomicUsize>,
) {
    let mut targets = Vec::new();
    let mut datapoints = VecDeque::new();
    while fill_with_queued(&data, &mut datapoints) {
        datapoints.truncate(queue_size.load(Ordering::Relaxed));
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
    sender: Sender<Arc<Vec<u8>>>,
    queue_size: Arc<AtomicUsize>,
}

impl DataStream {
    pub fn send(&self, data: Arc<Vec<u8>>) -> ForkResult {
        self.sender.send(data).or(Err(()))
    }

    pub fn set_queue_size(&self, queue_size: Option<usize>) {
        self.queue_size
            .store(queue_size.unwrap_or(usize::max_value()), Ordering::Relaxed);
    }
}
