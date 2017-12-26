use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};

pub fn iterate(listener: TcpListener, tag: String) -> (Raii, TcpConnectionIterator) {
    let (tx, rx) = channel();
    let killer = Raii { killer: tx.clone() };
    thread::spawn(move || listener_thread(&listener, &tag, &tx));
    (killer, TcpConnectionIterator { listener: rx })
}

fn listener_thread(connections: &TcpListener, tag: &str, out: &Sender<Option<TcpStream>>) {
    for stream in connections.incoming() {
        match stream {
            Ok(stream) => {
                if out.send(Some(stream)).is_err() {
                    break;
                }
            }
            Err(err) => {
                error!("TCP connection failed at {}: {}", tag, err);
            }
        }
    }
}

pub struct TcpConnectionIterator {
    listener: Receiver<Option<TcpStream>>,
}

impl Iterator for TcpConnectionIterator {
    type Item = TcpStream;

    fn next(&mut self) -> Option<Self::Item> {
        self.listener.recv().unwrap_or(None)
    }
}

pub struct Raii {
    killer: Sender<Option<TcpStream>>,
}

impl Drop for Raii {
    fn drop(&mut self) {
        if self.killer.send(None).is_err() {
            error!("TCP connection listener has already been killed");
        }
    }
}
