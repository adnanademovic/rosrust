use byteorder::{LittleEndian, ReadBytesExt};
use rustc_serialize::Decodable;
use std::net::{TcpStream, ToSocketAddrs};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std;
use super::error::Error;
use super::header::{encode, decode};
use super::Message;
use super::decoder::Decoder;

pub struct Subscriber<T>
    where T: Message + Decodable + Send + 'static
{
    rx: mpsc::Receiver<T>,
}

impl<T> Subscriber<T>
    where T: Message + Decodable + Send + 'static
{
    pub fn new<U>(address: U, caller_id: &str, topic: &str) -> Result<Subscriber<T>, Error>
        where U: ToSocketAddrs
    {
        let mut stream = TcpStream::connect(address)?;
        {
            let mut fields = std::collections::HashMap::<String, String>::new();
            fields.insert("message_definition".to_owned(), T::msg_definition());
            fields.insert("callerid".to_owned(), caller_id.to_owned());
            fields.insert("topic".to_owned(), topic.to_owned());
            fields.insert("md5sum".to_owned(), T::md5sum());
            fields.insert("type".to_owned(), T::msg_type());

            let fields = encode(fields)?;

            stream.write_all(&fields)?;
        }
        {
            let mut bytes = [0u8; 4];
            stream.read_exact(&mut bytes)?;
            let mut reader = std::io::Cursor::new(bytes);
            let data_length = reader.read_u32::<LittleEndian>()?;
            let mut payload = vec![0u8; data_length as usize];
            stream.read_exact(&mut payload)?;
            let data = bytes.iter().chain(payload.iter()).cloned().collect();
            let fields = decode(data)?;
            if fields.get("md5sum") != Some(&T::md5sum()) {
                return Err(Error::Mismatch);
            }
            if fields.get("type") != Some(&T::msg_type()) {
                return Err(Error::Mismatch);
            }
        }

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || spin_subscriber(stream, tx));

        Ok(Subscriber { rx: rx })
    }
}

fn spin_subscriber<T>(stream: TcpStream, tx: mpsc::Sender<T>) -> Result<(), Error>
    where T: Decodable
{
    let mut stream = Decoder::new(stream);
    while let Ok(()) = tx.send(T::decode(&mut stream)?) {
    }
    Ok(())
}

impl<T> std::iter::Iterator for Subscriber<T>
    where T: Message + Decodable + Send + 'static
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.rx.recv().ok()
    }
}
