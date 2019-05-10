use log::error;
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Feedback {
    AcceptNextStream,
    StopAccepting,
}

pub fn iterate<F>(listener: TcpListener, tag: String, handler: F)
where
    F: Fn(TcpStream) -> Feedback + Send + 'static,
{
    thread::spawn(move || listener_thread(&listener, &tag, handler));
}

fn listener_thread<F>(connections: &TcpListener, tag: &str, handler: F)
where
    F: Fn(TcpStream) -> Feedback + Send + 'static,
{
    for stream in connections.incoming() {
        match stream {
            Ok(stream) => match handler(stream) {
                Feedback::AcceptNextStream => {}
                Feedback::StopAccepting => break,
            },
            Err(err) => {
                error!("TCP connection failed at {}: {}", tag, err);
            }
        }
    }
}
