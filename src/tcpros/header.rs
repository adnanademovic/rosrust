use byteorder::{LittleEndian, WriteBytesExt};
use rustc_serialize::{Decodable, Encodable};
use std::collections::HashMap;
use std;
use super::{Decoder, Encoder};
use super::error::Error;

fn number_to_result(n: &u8) -> Result<u8, std::io::Error> {
    Ok(*n)
}

pub fn decode(data: Vec<u8>) -> Result<HashMap<String, String>, Error> {
    let vector_length = data.len();
    let mut decoder = Decoder::new(data.iter().map(number_to_result));
    let length = try!(decoder.pop_length()) as usize;
    if length + 4 != vector_length {
        return Err(Error::Truncated);
    }
    let mut result = HashMap::<String, String>::new();
    let mut size_count = 0;
    while length > size_count {
        let point = try!(String::decode(&mut decoder));
        size_count += point.len() + 4;
        let mut point = point.splitn(2, '=');
        let key = try!(point.next().ok_or(Error::UnsupportedData));
        let value = try!(point.next().ok_or(Error::UnsupportedData));
        result.insert(key.to_owned(), value.to_owned());
    }
    Ok(result)
}

pub fn encode(data: HashMap<String, String>) -> Result<Vec<u8>, Error> {
    let mut encoder = Encoder::new();
    for (key, value) in data {
        try!([key, value].join("=").encode(&mut encoder));
    }
    let mut buffer = vec![];
    let mut data = encoder.extract_data();
    buffer.reserve(4 + data.len());
    try!(buffer.write_u32::<LittleEndian>(data.len() as u32));
    buffer.append(&mut data);
    Ok(buffer)
}
