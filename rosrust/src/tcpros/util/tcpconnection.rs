use crossbeam::channel::{bounded, select, unbounded, Receiver, Sender};
use log::error;
use std::net::{TcpListener, TcpStream};
use std::thread;

pub fn iterate(listener: TcpListener, tag: String) -> (Raii, TcpConnectionIterator) {
    let (tcp_stream_tx, tcp_stream_rx) = unbounded();
    let (kill_tx, kill_rx) = bounded(0);
    let killer = Raii { kill_tx };
    thread::spawn(move || listener_thread(&listener, &tag, &tcp_stream_tx));
    (
        killer,
        TcpConnectionIterator {
            tcp_stream_rx,
            kill_rx,
        },
    )
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

pub struct TcpConnectionIterator {
    tcp_stream_rx: Receiver<TcpStream>,
    kill_rx: Receiver<()>,
}

impl Iterator for TcpConnectionIterator {
    type Item = TcpStream;

    fn next(&mut self) -> Option<Self::Item> {
        select! {
            recv(self.tcp_stream_rx) -> msg => msg.ok(),
            recv(self.kill_rx) -> _ => None,
        }
    }
}

pub struct Raii {
    kill_tx: Sender<()>,
}

impl Drop for Raii {
    fn drop(&mut self) {
        let send_result = self.kill_tx.send(());
        if send_result.is_err() {
            error!("TCP connection listener has already been killed");
        }
    }
}
