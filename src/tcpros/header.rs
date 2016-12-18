use byteorder::{LittleEndian, WriteBytesExt};
use rustc_serialize::{Decodable, Encodable};
use std::collections::HashMap;
use std;
use super::{Decoder, Encoder};
use super::error::Error;

pub fn decode<T: std::io::Read>(data: &mut T) -> Result<HashMap<String, String>, Error> {
    let mut decoder = Decoder::new(data);
    let length = decoder.pop_length()? as usize;
    let mut result = HashMap::<String, String>::new();
    let mut size_count = 0;
    while length > size_count {
        let point = String::decode(&mut decoder)?;
        size_count += point.len() + 4;
        let mut point = point.splitn(2, '=');
        let key = point.next().ok_or(Error::UnsupportedData)?;
        let value = point.next().ok_or(Error::UnsupportedData)?;
        result.insert(String::from(key), String::from(value));
    }
    Ok(result)
}

pub fn encode<T: std::io::Write>(data: HashMap<String, String>,
                                 buffer: &mut T)
                                 -> Result<(), Error> {
    let mut encoder = Encoder::new();
    for (key, value) in data {
        [key, value].join("=").encode(&mut encoder)?;
    }
    buffer.write_u32::<LittleEndian>(encoder.len() as u32)?;
    encoder.write_to(buffer).map_err(|v| Error::Io(v))
}
