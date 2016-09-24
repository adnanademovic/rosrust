use byteorder::{LittleEndian, ReadBytesExt};
use rustc_serialize::Decodable;
use std::net::{TcpStream, ToSocketAddrs};
use std::io::{Read, Write};
use std;
use super::error::Error;
use super::header::{encode, decode};
use super::message::RosMessage;
use super::decoder::Decoder;

pub trait SubscriberCallback {
    type N: RosMessage + Decodable;

    fn receive(&mut self, message: Self::N);
}

pub struct Subscriber<T: SubscriberCallback> {
    stream: Decoder<std::io::Bytes<TcpStream>>,
    callback: T,
}

impl<T> Subscriber<T>
    where T: SubscriberCallback
{
    pub fn new<U>(address: U,
                  callback: T,
                  caller_id: &str,
                  topic: &str)
                  -> Result<Subscriber<T>, Error>
        where U: ToSocketAddrs
    {
        let mut stream = try!(TcpStream::connect(address));
        {
            let mut fields = std::collections::HashMap::<String, String>::new();
            fields.insert("message_definition".to_owned(), T::N::msg_definition());
            fields.insert("callerid".to_owned(), caller_id.to_owned());
            fields.insert("topic".to_owned(), topic.to_owned());
            fields.insert("md5sum".to_owned(), T::N::md5sum());
            fields.insert("type".to_owned(), T::N::msg_type());

            let fields = try!(encode(fields));

            try!(stream.write_all(&fields));
        }
        {
            let mut bytes = [0u8; 4];
            try!(stream.read_exact(&mut bytes));
            let mut reader = std::io::Cursor::new(bytes);
            let data_length = try!(reader.read_u32::<LittleEndian>());
            let mut payload = vec![0u8; data_length as usize];
            try!(stream.read_exact(&mut payload));
            let data = bytes.iter().chain(payload.iter()).cloned().collect();
            let fields = try!(decode(data));
            if fields.get("md5sum") != Some(&T::N::md5sum()) {
                return Err(Error::Mismatch);
            }
            if fields.get("type") != Some(&T::N::msg_type()) {
                return Err(Error::Mismatch);
            }
        }

        Ok(Subscriber {
            stream: Decoder::new(stream.bytes()),
            callback: callback,
        })
    }

    pub fn spin(&mut self) -> Result<(), Error> {
        loop {
            self.callback.receive(try!(T::N::decode(&mut self.stream)));
        }
    }
}
