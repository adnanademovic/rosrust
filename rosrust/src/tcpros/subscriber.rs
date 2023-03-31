use super::error::{ErrorKind, Result, ResultExt};
use super::header::{decode, encode, match_field};
use super::{Message, Topic};
use crate::rosmsg::RosMsg;
use crate::util::lossy_channel::{lossy_channel, LossyReceiver, LossySender};
use crate::SubscriptionHandler;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crossbeam::channel::{bounded, select, Receiver, Sender, TrySendError};
use log::error;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::thread;

enum DataStreamConnectionChange {
    Connect(
        usize,
        LossySender<MessageInfo>,
        Sender<HashMap<String, String>>,
    ),
    Disconnect(usize),
}

pub struct SubscriberRosConnection {
    next_data_stream_id: usize,
    data_stream_tx: Sender<DataStreamConnectionChange>,
    publishers_stream: Sender<SocketAddr>,
    topic: Topic,
    pub connected_ids: BTreeSet<usize>,
    pub connected_publishers: BTreeSet<String>,
}

impl SubscriberRosConnection {
    pub fn new(
        caller_id: &str,
        topic: &str,
        msg_definition: String,
        msg_type: String,
        md5sum: String,
    ) -> SubscriberRosConnection {
        let subscriber_connection_queue_size = 8;
        let (data_stream_tx, data_stream_rx) = bounded(subscriber_connection_queue_size);
        let publisher_connection_queue_size = 8;
        let (pub_tx, pub_rx) = bounded(publisher_connection_queue_size);
        let caller_id = String::from(caller_id);
        let topic_name = String::from(topic);
        thread::spawn({
            let msg_type = msg_type.clone();
            let md5sum = md5sum.clone();
            move || {
                join_connections(
                    data_stream_rx,
                    pub_rx,
                    &caller_id,
                    &topic_name,
                    &msg_definition,
                    &md5sum,
                    &msg_type,
                )
            }
        });
        let topic = Topic {
            name: String::from(topic),
            msg_type,
            md5sum,
        };
        SubscriberRosConnection {
            next_data_stream_id: 1,
            data_stream_tx,
            publishers_stream: pub_tx,
            topic,
            connected_ids: BTreeSet::new(),
            connected_publishers: BTreeSet::new(),
        }
    }

    // TODO: allow synchronous handling for subscribers
    // This creates a new thread to call on_message. Next API change should
    // allow subscribing with either callback or inline handler of the queue.
    // The queue is lossy, so it wouldn't be blocking.
    pub fn add_subscriber<T, H>(&mut self, queue_size: usize, handler: H) -> usize
    where
        T: Message,
        H: SubscriptionHandler<T>,
    {
        let data_stream_id = self.next_data_stream_id;
        self.connected_ids.insert(data_stream_id);
        self.next_data_stream_id += 1;
        let (data_tx, data_rx) = lossy_channel(queue_size);
        let (connection_tx, connection_rx) = bounded(8);
        if self
            .data_stream_tx
            .send(DataStreamConnectionChange::Connect(
                data_stream_id,
                data_tx,
                connection_tx,
            ))
            .is_err()
        {
            // TODO: we might want to panic here
            error!("Subscriber failed to connect to data stream");
        }
        thread::spawn(move || handle_data::<T, H>(data_rx, connection_rx, handler));
        data_stream_id
    }

    pub fn remove_subscriber(&mut self, id: usize) {
        self.connected_ids.remove(&id);
        if self
            .data_stream_tx
            .send(DataStreamConnectionChange::Disconnect(id))
            .is_err()
        {
            // TODO: we might want to panic here
            error!("Subscriber failed to disconnect from data stream");
        }
    }

    pub fn has_subscribers(&self) -> bool {
        !self.connected_ids.is_empty()
    }

    #[inline]
    pub fn publisher_count(&self) -> usize {
        self.connected_publishers.len()
    }

    #[inline]
    pub fn publisher_uris(&self) -> Vec<String> {
        self.connected_publishers.iter().cloned().collect()
    }

    #[allow(clippy::useless_conversion)]
    pub fn connect_to<U: ToSocketAddrs>(
        &mut self,
        publisher: &str,
        addresses: U,
    ) -> std::io::Result<()> {
        for address in addresses.to_socket_addrs()? {
            // This should never fail, so it's safe to unwrap
            // Failure could only be caused by the join_connections
            // thread not running, which only happens after
            // Subscriber has been deconstructed
            self.publishers_stream
                .send(address)
                .expect("Connected thread died");
        }
        self.connected_publishers.insert(publisher.to_owned());
        Ok(())
    }

    pub fn is_connected_to(&self, publisher: &str) -> bool {
        self.connected_publishers.contains(publisher)
    }

    pub fn limit_publishers_to(&mut self, publishers: &BTreeSet<String>) {
        let difference: Vec<String> = self
            .connected_publishers
            .difference(publishers)
            .cloned()
            .collect();
        for item in difference {
            self.connected_publishers.remove(&item);
        }
    }

    pub fn get_topic(&self) -> &Topic {
        &self.topic
    }
}

fn handle_data<T, H>(
    data: LossyReceiver<MessageInfo>,
    connections: Receiver<HashMap<String, String>>,
    mut handler: H,
) where
    T: Message,
    H: SubscriptionHandler<T>,
{
    loop {
        select! {
            recv(data.kill_rx.kill_rx) -> _ => break,
            recv(data.data_rx) -> msg => match msg {
                Err(_) => break,
                Ok(buffer) => match RosMsg::decode_slice(&buffer.data) {
                    Ok(value) => handler.message(value, &buffer.caller_id),
                    Err(err) => error!("Failed to decode message: {}", err),
                },
            },
            recv(connections) -> msg => match msg {
                Err(_) => break,
                Ok(conn) => handler.connection(conn),
            },
        }
    }
}

fn join_connections(
    subscribers: Receiver<DataStreamConnectionChange>,
    publishers: Receiver<SocketAddr>,
    caller_id: &str,
    topic: &str,
    msg_definition: &str,
    md5sum: &str,
    msg_type: &str,
) {
    type Sub = (LossySender<MessageInfo>, Sender<HashMap<String, String>>);
    let mut subs: BTreeMap<usize, Sub> = BTreeMap::new();
    let mut existing_headers: Vec<HashMap<String, String>> = Vec::new();

    let (data_tx, data_rx): (Sender<MessageInfo>, Receiver<MessageInfo>) = bounded(8);

    // Ends when subscriber or publisher sender is destroyed, which happens at Subscriber destruction
    loop {
        select! {
            recv(data_rx) -> msg => {
                match msg {
                    Err(_) => break,
                    Ok(v) => for sub in subs.values() {
                        if sub.0.try_send(v.clone()).is_err() {
                            error!("Failed to send data to subscriber");
                        }
                    }
                }
            }
            recv(subscribers) -> msg => {
                match msg {
                    Err(_) => break,
                    Ok(DataStreamConnectionChange::Connect(id, data, conn)) => {
                        for header in &existing_headers {
                            if conn.send(header.clone()).is_err() {
                                error!("Failed to send connection info for subscriber");
                            };
                        }
                        subs.insert(id, (data, conn));
                    }
                    Ok(DataStreamConnectionChange::Disconnect(id)) => {
                        if let Some((mut data, _)) = subs.remove(&id) {
                            if data.close().is_err() {
                                error!("Subscriber data stream to topic has already been killed");
                            }
                        }
                    }
                }
            }
            recv(publishers) -> msg => {
                match msg {
                    Err(_) => break,
                    Ok(publisher) => {
                        let result = join_connection(
                            &data_tx,
                            &publisher,
                            caller_id,
                            topic,
                            msg_definition,
                            md5sum,
                            msg_type,
                        )
                        .chain_err(|| ErrorKind::TopicConnectionFail(topic.into()));
                        match result {
                            Ok(headers) => {
                                for sub in subs.values() {
                                    if sub.1.send(headers.clone()).is_err() {
                                        error!("Failed to send connection info for subscriber");
                                    }
                                }
                                existing_headers.push(headers);
                            }
                            Err(err) => {
                                let info = err
                                    .iter()
                                    .map(|v| format!("{}", v))
                                    .collect::<Vec<_>>()
                                    .join("\nCaused by:");
                                error!("{}", info);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn join_connection(
    data_stream: &Sender<MessageInfo>,
    publisher: &SocketAddr,
    caller_id: &str,
    topic: &str,
    msg_definition: &str,
    md5sum: &str,
    msg_type: &str,
) -> Result<HashMap<String, String>> {
    let mut stream = TcpStream::connect(publisher)?;
    let headers = exchange_headers::<_>(
        &mut stream,
        caller_id,
        topic,
        msg_definition,
        md5sum,
        msg_type,
    )?;
    let pub_caller_id = headers.get("callerid").cloned();
    let target = data_stream.clone();
    thread::spawn(move || {
        let pub_caller_id = Arc::new(pub_caller_id.unwrap_or_default());
        while let Ok(buffer) = package_to_vector(&mut stream) {
            if let Err(TrySendError::Disconnected(_)) =
                target.try_send(MessageInfo::new(Arc::clone(&pub_caller_id), buffer))
            {
                // Data receiver has been destroyed after
                // Subscriber destructor's kill signal
                break;
            }
        }
    });
    Ok(headers)
}

fn write_request<U: std::io::Write>(
    mut stream: &mut U,
    caller_id: &str,
    topic: &str,
    msg_definition: &str,
    md5sum: &str,
    msg_type: &str,
) -> Result<()> {
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("message_definition"), msg_definition.into());
    fields.insert(String::from("callerid"), caller_id.into());
    fields.insert(String::from("topic"), topic.into());
    fields.insert(String::from("md5sum"), md5sum.into());
    fields.insert(String::from("type"), msg_type.into());
    encode(&mut stream, &fields)?;
    Ok(())
}

fn read_response<U: std::io::Read>(
    mut stream: &mut U,
    md5sum: &str,
    msg_type: &str,
) -> Result<HashMap<String, String>> {
    let fields = decode(&mut stream)?;
    if md5sum != "*" {
        match_field(&fields, "md5sum", md5sum)?;
    }
    if msg_type != "*" {
        match_field(&fields, "type", msg_type)?;
    }
    Ok(fields)
}

fn exchange_headers<U>(
    stream: &mut U,
    caller_id: &str,
    topic: &str,
    msg_definition: &str,
    md5sum: &str,
    msg_type: &str,
) -> Result<HashMap<String, String>>
where
    U: std::io::Write + std::io::Read,
{
    write_request::<U>(stream, caller_id, topic, msg_definition, md5sum, msg_type)?;
    read_response::<U>(stream, md5sum, msg_type)
}

#[inline]
fn package_to_vector<R: std::io::Read>(stream: &mut R) -> std::io::Result<Vec<u8>> {
    let length = stream.read_u32::<LittleEndian>()?;
    let u32_size = std::mem::size_of::<u32>();
    let num_bytes = length as usize + u32_size;

    // Allocate memory of the proper size for the incoming message. We
    // do not initialize the memory to zero here (as would be safe)
    // because it is expensive and ultimately unnecessary. We know the
    // length of the message and if the length is incorrect, the
    // stream reading functions will bail with an Error rather than
    // leaving memory uninitialized.
    let mut out = Vec::<u8>::with_capacity(num_bytes);

    let out_ptr = out.as_mut_ptr();
    // Read length from stream.
    std::io::Cursor::new(unsafe { std::slice::from_raw_parts_mut(out_ptr as *mut u8, u32_size) })
        .write_u32::<LittleEndian>(length)?;

    // Read data from stream.
    let read_buf = unsafe { std::slice::from_raw_parts_mut(out_ptr as *mut u8, num_bytes) };
    stream.read_exact(&mut read_buf[u32_size..])?;

    // Don't drop the original Vec which has size==0 and instead use
    // its memory to initialize a new Vec with size == capacity == num_bytes.
    std::mem::forget(out);

    // Return the new, now full and "safely" initialized.
    Ok(unsafe { Vec::from_raw_parts(out_ptr, num_bytes, num_bytes) })
}

#[derive(Clone)]
struct MessageInfo {
    caller_id: Arc<String>,
    data: Vec<u8>,
}

impl MessageInfo {
    fn new(caller_id: Arc<String>, data: Vec<u8>) -> Self {
        Self { caller_id, data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static FAILED_TO_READ_WRITE_VECTOR: &str = "Failed to read or write from vector";

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
