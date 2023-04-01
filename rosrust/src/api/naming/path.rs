use super::error::{Error, ErrorKind};
use error_chain::bail;
use std::fmt;
use std::ops::{Add, AddAssign};

pub trait Path {
    fn get(&self) -> &[String];

    fn take(self) -> Buffer;

    fn slice(&self) -> Slice {
        Slice { chain: self.get() }
    }

    fn parent(&self) -> Result<Slice, Error> {
        match self.get().split_last() {
            Some(v) => Ok(Slice { chain: v.1 }),
            None => Err(ErrorKind::MissingParent.into()),
        }
    }
}

#[derive(Clone, Debug)]
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
        Buffer {
            chain: Vec::from(self.chain),
        }
    }
}

impl<'a, T: Path> Add<T> for Slice<'a> {
    type Output = Buffer;

    fn add(self, rhs: T) -> Self::Output {
        self.take() + rhs
    }
}

#[derive(Clone, Debug)]
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
        self.chain.extend_from_slice(rhs.get());
    }
}

impl std::str::FromStr for Buffer {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_end_matches('/');
        let words = if let Some(first_char) = s.bytes().next() {
            if !is_legal_first_char(first_char) {
                bail!(ErrorKind::IllegalFirstCharacter(s.into()));
            }
            let mut word_iter = s.split('/');
            match word_iter.next() {
                Some("") => {}
                Some(_) | None => bail!(ErrorKind::LeadingSlashMissing(s.into())),
            }
            word_iter
                .map(process_name)
                .collect::<Result<Vec<String>, Error>>()?
        } else {
            Vec::new()
        };
        Ok(Buffer { chain: words })
    }
}

fn process_name(name: &str) -> Result<String, Error> {
    if name.is_empty() {
        bail!(ErrorKind::EmptyName)
    }

    let mut bytes = name.bytes();
    if !bytes.all(is_legal_char) {
        bail!(ErrorKind::IllegalCharacter(name.into()));
    }
    Ok(String::from(name))
}

fn is_legal_first_char(v: u8) -> bool {
    v.is_ascii_uppercase() || v.is_ascii_lowercase() || v == b'/' || v == b'~' || v.is_ascii_digit()
}

fn is_legal_char(v: u8) -> bool {
    is_legal_first_char(v) || v == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    static FAILED_TO_HANDLE: &str = "Failed to handle";

    #[test]
    fn names_are_legal() {
        assert!("/foo".parse::<Buffer>().is_ok());
        assert!("/foo/bar".parse::<Buffer>().is_ok());
        assert!("/f1_aA/Ba02/Xx".parse::<Buffer>().is_ok());
        assert!("/asdf/".parse::<Buffer>().is_ok());
        assert!("/".parse::<Buffer>().is_ok());
        assert!("".parse::<Buffer>().is_ok());
        assert!("/123/".parse::<Buffer>().is_ok());
        assert!("/_e".parse::<Buffer>().is_ok());

        assert!("a".parse::<Buffer>().is_err());
        assert!("/foo$".parse::<Buffer>().is_err());
        assert!("/a//b".parse::<Buffer>().is_err());
    }

    #[test]
    fn names_are_parsed() {
        assert_eq!(
            vec![String::from("foo")],
            "/foo".parse::<Buffer>().expect(FAILED_TO_HANDLE).get()
        );
        assert_eq!(
            vec![String::from("foo"), String::from("bar")],
            "/foo/bar".parse::<Buffer>().expect(FAILED_TO_HANDLE).get()
        );
        assert_eq!(
            vec![
                String::from("f1_aA"),
                String::from("Ba02"),
                String::from("Xx"),
            ],
            "/f1_aA/Ba02/Xx"
                .parse::<Buffer>()
                .expect(FAILED_TO_HANDLE)
                .get()
        );
    }

    #[test]
    fn is_formatted() {
        assert_eq!(
            "/foo",
            format!("{}", "/foo".parse::<Buffer>().expect(FAILED_TO_HANDLE))
        );
        assert_eq!(
            "/foo/bar",
            format!("{}", "/foo/bar".parse::<Buffer>().expect(FAILED_TO_HANDLE))
        );
        assert_eq!(
            "/f1_aA/Ba02/Xx",
            format!(
                "{}",
                "/f1_aA/Ba02/Xx".parse::<Buffer>().expect(FAILED_TO_HANDLE)
            )
        );
    }

    #[test]
    fn parents_are_handled() {
        assert_eq!(
            "",
            format!(
                "{}",
                "/foo"
                    .parse::<Buffer>()
                    .expect(FAILED_TO_HANDLE)
                    .parent()
                    .expect(FAILED_TO_HANDLE)
            )
        );
        assert_eq!(
            "/foo",
            format!(
                "{}",
                "/foo/bar"
                    .parse::<Buffer>()
                    .expect(FAILED_TO_HANDLE)
                    .parent()
                    .expect(FAILED_TO_HANDLE)
            )
        );
        assert_eq!(
            "/f1_aA/Ba02",
            format!(
                "{}",
                "/f1_aA/Ba02/Xx"
                    .parse::<Buffer>()
                    .expect(FAILED_TO_HANDLE)
                    .parent()
                    .expect(FAILED_TO_HANDLE)
            )
        );
        assert!("/"
            .parse::<Buffer>()
            .expect(FAILED_TO_HANDLE)
            .parent()
            .is_err());
        assert!("/foo"
            .parse::<Buffer>()
            .expect(FAILED_TO_HANDLE)
            .parent()
            .expect(FAILED_TO_HANDLE)
            .parent()
            .is_err());
    }

    #[test]
    fn addition_works() {
        let part1 = "/foo".parse::<Buffer>().expect(FAILED_TO_HANDLE);
        let part2 = "/B4r/x".parse::<Buffer>().expect(FAILED_TO_HANDLE);
        let part3 = "/bA_z".parse::<Buffer>().expect(FAILED_TO_HANDLE);
        assert_eq!("/foo/B4r/x", format!("{}", part1.slice() + part2.slice()));
        assert_eq!(
            "/B4r/x/foo/foo",
            format!("{}", part2.slice() + part1.slice() + part1.slice())
        );
        assert_eq!("/foo/B4r/x/bA_z", format!("{}", part1 + part2 + part3));
    }
}
