use std;
use std::ops::{Add, AddAssign};
use std::fmt;

pub trait Path {
    fn get(&self) -> &[String];

    fn take(self) -> Buffer;

    fn slice(&self) -> Slice {
        Slice { chain: self.get() }
    }

    fn parent(&self) -> Option<Slice> {
        self.get().split_last().map(|v| Slice { chain: v.1 })
    }
}

#[derive(Clone,Debug)]
pub struct Slice<'a> {
    chain: &'a [String],
}

impl<'a> fmt::Display for Slice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for item in self.get() {
            write!(f, "/{}", item)?;
        }
        Ok(())
    }
}

impl<'a> Path for Slice<'a> {
    fn get(&self) -> &[String] {
        self.chain
    }

    fn take(self) -> Buffer {
        Buffer { chain: Vec::from(self.chain) }
    }
}

impl<'a, T: Path> Add<T> for Slice<'a> {
    type Output = Buffer;

    fn add(self, rhs: T) -> Self::Output {
        self.take() + rhs
    }
}

#[derive(Clone,Debug)]
pub struct Buffer {
    chain: Vec<String>,
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for item in self.get() {
            write!(f, "/{}", item)?;
        }
        Ok(())
    }
}

impl Path for Buffer {
    fn get(&self) -> &[String] {
        &self.chain
    }

    fn take(self) -> Buffer {
        self
    }
}

impl<T: Path> Add<T> for Buffer {
    type Output = Buffer;

    fn add(mut self, rhs: T) -> Self::Output {
        self += rhs;
        self
    }
}

impl<T: Path> AddAssign<T> for Buffer {
    fn add_assign(&mut self, rhs: T) {
        self.chain.extend_from_slice(&mut rhs.get());
    }
}

impl std::str::FromStr for Buffer {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // ^(/[a-zA-Z][0-9a-zA-Z_]*)$

        let mut word_iter = s.split('/');
        if let Some("") = word_iter.next() {
            let words = word_iter.map(|v| String::from(v)).collect::<Vec<String>>();
            if words.len() > 0 {
                if words.iter().all(is_legal_name) {
                    return Ok(Buffer { chain: words });
                }
            }
        }
        Err(())
    }
}

fn is_legal_name(v: &String) -> bool {
    let mut bytes = v.bytes();
    let first_char = match bytes.next() {
        Some(v) => v,
        None => return false,
    };
    is_legal_first_char(first_char) && bytes.all(is_legal_char)
}

fn is_legal_first_char(v: u8) -> bool {
    v >= b'A' && v <= b'Z' || v >= b'a' && v <= b'z'
}

fn is_legal_char(v: u8) -> bool {
    is_legal_first_char(v) || v >= b'0' && v <= b'9' || v == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_legal() {
        assert!("/foo".parse::<Buffer>().is_ok());
        assert!("/foo/bar".parse::<Buffer>().is_ok());
        assert!("/f1_aA/Ba02/Xx".parse::<Buffer>().is_ok());
        assert!("".parse::<Buffer>().is_err());
        assert!("a".parse::<Buffer>().is_err());
        assert!("/123/".parse::<Buffer>().is_err());
        assert!("/foo$".parse::<Buffer>().is_err());
        assert!("/_e".parse::<Buffer>().is_err());
        assert!("/a//b".parse::<Buffer>().is_err());
    }

    #[test]
    fn names_are_parsed() {
        assert_eq!(vec![String::from("foo")],
                   "/foo".parse::<Buffer>().unwrap().get());
        assert_eq!(vec![String::from("foo"), String::from("bar")],
                   "/foo/bar".parse::<Buffer>().unwrap().get());
        assert_eq!(vec![String::from("f1_aA"), String::from("Ba02"), String::from("Xx")],
                   "/f1_aA/Ba02/Xx".parse::<Buffer>().unwrap().get());
    }

    #[test]
    fn is_formatted() {
        assert_eq!("/foo", format!("{}", "/foo".parse::<Buffer>().unwrap()));
        assert_eq!("/foo/bar",
                   format!("{}", "/foo/bar".parse::<Buffer>().unwrap()));
        assert_eq!("/f1_aA/Ba02/Xx",
                   format!("{}", "/f1_aA/Ba02/Xx".parse::<Buffer>().unwrap()));
    }

    #[test]
    fn parents_are_handled() {
        assert_eq!("",
                   format!("{}", "/foo".parse::<Buffer>().unwrap().parent().unwrap()));
        assert_eq!("/foo",
                   format!("{}",
                           "/foo/bar".parse::<Buffer>().unwrap().parent().unwrap()));
        assert_eq!("/f1_aA/Ba02",
                   format!("{}",
                           "/f1_aA/Ba02/Xx".parse::<Buffer>().unwrap().parent().unwrap()));
        assert!("/foo".parse::<Buffer>().unwrap().parent().unwrap().parent().is_none());
    }

    #[test]
    fn addition_works() {
        let foo = "/foo".parse::<Buffer>().unwrap();
        let bar = "/B4r/x".parse::<Buffer>().unwrap();
        let baz = "/bA_z".parse::<Buffer>().unwrap();
        assert_eq!("/foo/B4r/x", format!("{}", foo.slice() + bar.slice()));
        assert_eq!("/B4r/x/foo/foo",
                   format!("{}", bar.slice() + foo.slice() + foo.slice()));
        assert_eq!("/foo/B4r/x/bA_z", format!("{}", foo + bar + baz));
    }
}
