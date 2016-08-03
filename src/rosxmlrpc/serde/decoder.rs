use xml;
use rustc_serialize;
use std;

pub struct Decoder {
    tree: std::vec::IntoIter<FlatTree>,
}

impl Decoder {
    pub fn new<T: std::io::Read>(body: T) -> Decoder {
        Decoder { tree: Tree::new(body).map(flatten_tree).unwrap_or(vec![].into_iter()) }
    }

    pub fn peel_named_layer(&mut self, name: &str) -> Result<(), Error> {
        if let Some(FlatTree::Node(key, length)) = self.tree.next() {
            if key == name && length == 1 {
                return Ok(());
            }
        }
        Err(Error::UnsupportedDataFormat)
    }

    fn peel_layer(&mut self) -> Result<(String, usize), Error> {
        if let Some(FlatTree::Node(key, length)) = self.tree.next() {
            if key == "value" && length == 1 {
                if let Some(FlatTree::Node(key, length)) = self.tree.next() {
                    return Ok((key, length));
                }
            }
        }
        Err(Error::UnsupportedDataFormat)
    }

    pub fn peel_response_body(&mut self) -> Result<(), Error> {
        try!(self.peel_named_layer("methodResponse"));
        try!(self.peel_named_layer("params"));
        try!(self.peel_named_layer("param"));
        Ok(())
    }

    pub fn peel_request_body(&mut self) -> Result<(String, usize), Error> {
        if let Some(FlatTree::Node(key, length)) = self.tree.next() {
            if key == "methodCall" && length == 2 {
                try!(self.peel_named_layer("methodName"));
                if let Some(FlatTree::Leaf(method_name)) = self.tree.next() {
                    if let Some(FlatTree::Node(key, children)) = self.tree.next() {
                        if key == "params" {
                            return Ok((method_name, children));
                        }
                    }
                }
            }
        }
        Err(Error::UnsupportedDataFormat)
    }

    pub fn decode_request_parameter<T: rustc_serialize::Decodable>(&mut self) -> Result<T, Error> {
        try!(self.peel_named_layer("param"));
        T::decode(self)
    }
}

impl rustc_serialize::Decoder for Decoder {
    type Error = Error;

    fn read_nil(&mut self) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_usize(&mut self) -> Result<usize, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_u64(&mut self) -> Result<u64, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_u32(&mut self) -> Result<u32, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_u16(&mut self) -> Result<u16, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_u8(&mut self) -> Result<u8, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_isize(&mut self) -> Result<isize, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_i64(&mut self) -> Result<i64, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_i32(&mut self) -> Result<i32, Self::Error> {
        let (key, length) = try!(self.peel_layer());
        if length == 1 && (key == "i4" || key == "int") {
            if let Some(FlatTree::Leaf(value)) = self.tree.next() {
                return Ok(try!(value.parse::<i32>()));
            }
        }
        Err(Error::UnsupportedDataFormat)
    }

    fn read_i16(&mut self) -> Result<i16, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_i8(&mut self) -> Result<i8, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_bool(&mut self) -> Result<bool, Self::Error> {
        let (key, length) = try!(self.peel_layer());
        if length == 1 && key == "boolean" {
            if let Some(FlatTree::Leaf(value)) = self.tree.next() {
                return Ok(try!(value.parse::<i32>()) != 0);
            }
        }
        Err(Error::UnsupportedDataFormat)
    }

    fn read_f64(&mut self) -> Result<f64, Self::Error> {
        let (key, length) = try!(self.peel_layer());
        if length == 1 && key == "double" {
            if let Some(FlatTree::Leaf(value)) = self.tree.next() {
                return Ok(try!(value.parse::<f64>()));
            }
        }
        Err(Error::UnsupportedDataFormat)
    }

    fn read_f32(&mut self) -> Result<f32, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_char(&mut self) -> Result<char, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_str(&mut self) -> Result<String, Self::Error> {
        let (key, length) = try!(self.peel_layer());
        if key == "string" {
            if length == 1 {
                if let Some(FlatTree::Leaf(value)) = self.tree.next() {
                    return Ok(value);
                }
            } else if length == 0 {
                return Ok("".to_owned());
            }
        }
        Err(Error::UnsupportedDataFormat)
    }

    fn read_enum<T, F>(&mut self, _: &str, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_enum_variant<T, F>(&mut self, _: &[&str], _: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, usize) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_enum_variant_arg<T, F>(&mut self, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_enum_struct_variant<T, F>(&mut self, _: &[&str], _: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, usize) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_enum_struct_variant_field<T, F>(&mut self,
                                            _: &str,
                                            _: usize,
                                            _: F)
                                            -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_struct<T, F>(&mut self, _: &str, size: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        self.read_tuple(size, f)
    }

    fn read_struct_field<T, F>(&mut self, _: &str, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        f(self)
    }

    fn read_tuple<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        let (key, length) = try!(self.peel_layer());
        if length == 1 && key == "array" {
            if let Some(FlatTree::Node(key, _)) = self.tree.next() {
                if key == "data" {
                    return f(self);
                }
            }
        }
        Err(Error::UnsupportedDataFormat)
    }

    fn read_tuple_arg<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        f(self)
    }

    fn read_tuple_struct<T, F>(&mut self, _: &str, size: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        self.read_tuple(size, f)
    }

    fn read_tuple_struct_arg<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        f(self)
    }

    fn read_option<T, F>(&mut self, _: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, bool) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_seq<T, F>(&mut self, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self, usize) -> Result<T, Self::Error>
    {
        let (key, length) = try!(self.peel_layer());
        if length == 1 && key == "array" {
            if let Some(FlatTree::Node(key, length)) = self.tree.next() {
                if key == "data" {
                    return f(self, length);
                }
            }
        }
        Err(Error::UnsupportedDataFormat)
    }

    fn read_seq_elt<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        f(self)
    }

    fn read_map<T, F>(&mut self, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self, usize) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_map_elt_key<T, F>(&mut self, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_map_elt_val<T, F>(&mut self, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn error(&mut self, err: &str) -> Self::Error {
        Error::Decoding(err.to_owned())
    }
}

enum Node {
    Open(String),
    Data(String),
    Close(String),
}

enum Tree {
    Leaf(String),
    Node(String, Vec<Tree>),
}

enum FlatTree {
    Leaf(String),
    Node(String, usize),
}

fn flatten_tree(tree: Tree) -> std::vec::IntoIter<FlatTree> {
    match tree {
        Tree::Leaf(value) => vec![FlatTree::Leaf(value)].into_iter(),
        Tree::Node(key, children) => {
            let mut result = vec![FlatTree::Node(key, children.len())];
            for child in children.into_iter().map(flatten_tree) {
                for node in child {
                    result.push(node);
                }
            }
            result.into_iter()
        }
    }
}

impl Tree {
    fn new<T: std::io::Read>(body: T) -> Result<Tree, Error> {
        try!(parse_tree(&mut xml::EventReader::new(body))).ok_or(Error::UnsupportedDataFormat)
    }
}

fn parse_tree<T: std::io::Read>(reader: &mut xml::EventReader<T>) -> Result<Option<Tree>, Error> {
    match try!(next_node(reader)) {
        Node::Close(..) => Ok(None),
        Node::Data(value) => Ok(Some(Tree::Leaf(value))),
        Node::Open(name) => {
            Ok(Some(Tree::Node(name, {
                let mut children = Vec::<Tree>::new();
                while let Some(node) = try!(parse_tree(reader)) {
                    children.push(node);
                }
                children
            })))
        }
    }
}

fn next_node<T: std::io::Read>(reader: &mut xml::EventReader<T>) -> Result<Node, Error> {
    match try!(reader.next()) {
        xml::reader::XmlEvent::StartElement { name, .. } => Ok(Node::Open(name.local_name)),
        xml::reader::XmlEvent::Characters(value) => Ok(Node::Data(value)),
        xml::reader::XmlEvent::EndElement { name } => Ok(Node::Close(name.local_name)),
        _ => next_node(reader),
    }
}

#[derive(Debug)]
pub enum Error {
    XmlRead(xml::reader::Error),
    UnsupportedDataFormat,
    Decoding(String),
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),
}

impl From<xml::reader::Error> for Error {
    fn from(err: xml::reader::Error) -> Error {
        Error::XmlRead(err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Error {
        Error::ParseInt(err)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(err: std::num::ParseFloatError) -> Error {
        Error::ParseFloat(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::XmlRead(ref err) => write!(f, "XML reading error: {}", err),
            Error::UnsupportedDataFormat => write!(f, "Unencodable data provided"),
            Error::Decoding(ref err) => write!(f, "Internal error while decoding: {}", err),
            Error::ParseInt(ref err) => write!(f, "Int parsing error: {}", err),
            Error::ParseFloat(ref err) => write!(f, "Float parsing error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::XmlRead(ref err) => err.description(),
            Error::UnsupportedDataFormat => "Unencodable data provided",
            Error::Decoding(..) => "Internal error while decoding",
            Error::ParseInt(ref err) => err.description(),
            Error::ParseFloat(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::XmlRead(ref err) => Some(err),
            Error::UnsupportedDataFormat => None,
            Error::Decoding(..) => None,
            Error::ParseInt(ref err) => Some(err),
            Error::ParseFloat(ref err) => Some(err),
        }
    }
}
