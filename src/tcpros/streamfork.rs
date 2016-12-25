use std::io::Write;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use super::encoder::Encoder;

pub fn fork<T: Write + Send + 'static>() -> (TargetList<T>, DataStream) {
    let (streams_sender, streams) = channel();
    let (data_sender, data) = channel();
    thread::spawn(move || fork_thread::<T>(streams, data));
    (TargetList(streams_sender), DataStream(data_sender))
}

fn fork_thread<T: Write + Send + 'static>(streams: Receiver<T>, data: Receiver<Encoder>) {
    let mut targets = Vec::new();
    while let Ok(encoder) = data.recv() {
        while let Ok(target) = streams.try_recv() {
            targets.push(target);
        }
        targets = targets.into_iter()
            .filter_map(|mut target| {
                match encoder.write_to(&mut target) {
                    Ok(()) => Some(target),
                    Err(_) => None,
                }
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
pub struct DataStream(Sender<Encoder>);

impl DataStream {
    pub fn send(&self, data: Encoder) -> ForkResult {
        self.0.send(data).or(Err(()))
    }
}
