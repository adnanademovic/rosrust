use crate::util::killable_channel::{channel, KillMode, Killer, Receiver, SendMode, Sender};
use log::error;
use std::net::{TcpListener, TcpStream};
use std::thread;

pub fn iterate(listener: TcpListener, tag: String) -> (Raii, TcpConnectionIterator) {
    let (killer, tcp_stream_tx, tcp_stream_rx) = channel(SendMode::Unbounded, KillMode::Sync);
    let killer = Raii { killer };
    thread::spawn(move || listener_thread(&listener, &tag, &tcp_stream_tx));
    (killer, tcp_stream_rx)
}

fn listener_thread(connections: &TcpListener, tag: &str, out: &Sender<TcpStream>) {
    for stream in connections.incoming() {
        match stream {
            Ok(stream) => {
                if out.send(stream).is_err() {
                    break;
                }
            }
            Err(err) => {
                error!("TCP connection failed at {}: {}", tag, err);
            }
        }
    }
}

pub type TcpConnectionIterator = Receiver<TcpStream>;

pub struct Raii {
    killer: Killer,
}

impl Drop for Raii {
    fn drop(&mut self) {
        if self.killer.send().is_err() {
            error!("TCP connection listener has already been killed");
        }
    }
}
