use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::collections::HashMap;
use std;
use serde_rosmsg::from_slice;
use super::error::{ErrorKind, Result, ResultExt};
use super::header::{decode, encode, match_field};
use super::Message;

pub struct Subscriber {
    data_stream: Sender<Option<Vec<u8>>>,
    publishers_stream: Sender<SocketAddr>,
    pub topic: String,
    pub msg_type: String,
}

impl Subscriber {
    pub fn new<T, F>(caller_id: &str, topic: &str, callback: F) -> Subscriber
    where
        T: Message,
        F: Fn(T) -> () + Send + 'static,
    {
        let (data_tx, data_rx) = channel();
        let (pub_tx, pub_rx) = channel();
        let caller_id = String::from(caller_id);
        let topic_name = String::from(topic);
        let data_stream = data_tx.clone();
        thread::spawn(move || join_connections::<T>(&data_tx, pub_rx, &caller_id, &topic_name));
        thread::spawn(move || handle_data::<T, F>(data_rx, callback));
        Subscriber {
            data_stream: data_stream,
            publishers_stream: pub_tx,
            topic: String::from(topic),
            msg_type: T::msg_type(),
        }
    }

    pub fn connect_to<U: ToSocketAddrs>(&mut self, addresses: U) -> std::io::Result<()> {
        for address in addresses.to_socket_addrs()? {
            // This should never fail, so it's safe to unwrap
            // Failure could only be caused by the join_connections
            // thread not running, which only happens after
            // Subscriber has been deconstructed
            self.publishers_stream
                .send(address)
                .expect("Connected thread died");
        }
        Ok(())
    }
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        if self.data_stream.send(None).is_err() {
            error!(
                "Subscriber data stream to topic '{}' has already been killed",
                self.topic
            );
        }
    }
}

fn handle_data<T, F>(data: Receiver<Option<Vec<u8>>>, callback: F)
where
    T: Message,
    F: Fn(T) -> (),
{
    for buffer_option in data {
        let buffer = match buffer_option {
            Some(v) => v,
            None => break, // Only the Subscriber destructor can send this signal
        };
        match from_slice(&buffer) {
            Ok(value) => callback(value),
            Err(err) => error!("Failed to decode message: {}", err),
        }
    }
}

fn join_connections<T>(
    data_stream: &Sender<Option<Vec<u8>>>,
    publishers: Receiver<SocketAddr>,
    caller_id: &str,
    topic: &str,
) where
    T: Message,
{
    // Ends when publisher sender is destroyed, which happens at Subscriber destruction
    for publisher in publishers {
        let result = join_connection::<T>(data_stream, &publisher, caller_id, topic)
            .chain_err(|| ErrorKind::TopicConnectionFail(topic.into()));
        if let Err(err) = result {
            let info = err.iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<_>>()
                .join("\nCaused by:");
            error!("{}", info);
        }
    }
}

fn join_connection<T>(
    data_stream: &Sender<Option<Vec<u8>>>,
    publisher: &SocketAddr,
    caller_id: &str,
    topic: &str,
) -> Result<()>
where
    T: Message,
{
    let mut stream = TcpStream::connect(publisher)?;
    exchange_headers::<T, _>(&mut stream, caller_id, topic)?;
    let target = data_stream.clone();
    thread::spawn(move || {
        while let Ok(buffer) = package_to_vector(&mut stream) {
            if target.send(Some(buffer)).is_err() {
                // Data receiver has been destroyed after
                // Subscriber destructor's kill signal
                break;
            }
        }
    });
    Ok(())
}

fn write_request<T: Message, U: std::io::Write>(
    mut stream: &mut U,
    caller_id: &str,
    topic: &str,
) -> Result<()> {
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("message_definition"), T::msg_definition());
    fields.insert(String::from("callerid"), String::from(caller_id));
    fields.insert(String::from("topic"), String::from(topic));
    fields.insert(String::from("md5sum"), T::md5sum());
    fields.insert(String::from("type"), T::msg_type());
    encode(&mut stream, &fields)?;
    Ok(())
}

fn read_response<T: Message, U: std::io::Read>(mut stream: &mut U) -> Result<()> {
    let fields = decode(&mut stream)?;
    match_field(&fields, "md5sum", &T::md5sum())?;
    match_field(&fields, "type", &T::msg_type())
}

fn exchange_headers<T, U>(stream: &mut U, caller_id: &str, topic: &str) -> Result<()>
where
    T: Message,
    U: std::io::Write + std::io::Read,
{
    write_request::<T, U>(stream, caller_id, topic)?;
    read_response::<T, U>(stream)
}

#[inline]
fn package_to_vector<R: std::io::Read>(stream: &mut R) -> std::io::Result<Vec<u8>> {
    let length = stream.read_u32::<LittleEndian>()?;
    let mut buffer = length_vector(length)?;
    buffer.resize(length as usize + 4, 0);
    stream.read_exact(&mut buffer[4..])?;
    Ok(buffer)
}

#[inline]
fn length_vector(length: u32) -> std::io::Result<Vec<u8>> {
    let mut cursor = std::io::Cursor::new(Vec::new());
    cursor.write_u32::<LittleEndian>(length)?;
    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std;

    static FAILED_TO_READ_WRITE_VECTOR: &'static str = "Failed to read or write from vector";

    #[test]
    fn length_vector_properly_encodes() {
        let data = length_vector(0x01234567).expect(FAILED_TO_READ_WRITE_VECTOR);
        assert_eq!(data, [0x67, 0x45, 0x23, 0x01]);
    }

    #[test]
    fn package_to_vector_creates_right_buffer_from_reader() {
        let input = [7, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7];
        let data =
            package_to_vector(&mut std::io::Cursor::new(input)).expect(FAILED_TO_READ_WRITE_VECTOR);
        assert_eq!(data, [7, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn package_to_vector_respects_provided_length() {
        let input = [7, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let data =
            package_to_vector(&mut std::io::Cursor::new(input)).expect(FAILED_TO_READ_WRITE_VECTOR);
        assert_eq!(data, [7, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn package_to_vector_fails_if_stream_is_shorter_than_annotated() {
        let input = [7, 0, 0, 0, 1, 2, 3, 4, 5];
        package_to_vector(&mut std::io::Cursor::new(input)).unwrap_err();
    }

    #[test]
    fn package_to_vector_fails_leaves_cursor_at_end_of_reading() {
        let input = [7, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 4, 0, 0, 0, 11, 12, 13, 14];
        let mut cursor = std::io::Cursor::new(input);
        let data = package_to_vector(&mut cursor).expect(FAILED_TO_READ_WRITE_VECTOR);
        assert_eq!(data, [7, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7]);
        let data = package_to_vector(&mut cursor).expect(FAILED_TO_READ_WRITE_VECTOR);
        assert_eq!(data, [4, 0, 0, 0, 11, 12, 13, 14]);
    }
}
