use std::collections::HashMap;
use super::path::{Buffer, Slice, Path};
use super::Error;

pub struct Mapper {
    children: HashMap<String, Mapper>,
    value: Option<Buffer>,
}

impl Mapper {
    pub fn new() -> Mapper {
        Mapper {
            children: HashMap::new(),
            value: None,
        }
    }

    pub fn add(&mut self, keys: &[String], value: Buffer) -> Result<(), Error> {
        match keys.split_first() {
            None => {
                if self.value.is_some() {
                    Err(Error::MappingSourceExists)
                } else {
                    self.value = Some(value);
                    Ok(())
                }
            }
            Some((key, child_keys)) => {
                self.children
                    .entry(key.clone())
                    .or_insert(Mapper::new())
                    .add(child_keys, value)
            }
        }
    }

    pub fn translate(&self, keys: &[String]) -> Option<Slice> {
        match keys.split_first() {
            None => self.value.as_ref().map(|v| v.slice()),
            Some((key, child_keys)) => self.children.get(key).and_then(|v| v.translate(child_keys)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::path::{Buffer, Path};

    #[test]
    fn matches_existing_paths() {
        let mut mapper = Mapper::new();
        let src1 = "/foo/bar".parse::<Buffer>().unwrap();
        let dst = "/a/b/c".parse::<Buffer>().unwrap();
        mapper.add(src1.get(), dst).unwrap();
        let src2 = "/foo/ter".parse::<Buffer>().unwrap();
        let dst = "/d/e/f".parse::<Buffer>().unwrap();
        mapper.add(src2.get(), dst).unwrap();
        assert_eq!("/a/b/c",
                   format!("{}", mapper.translate(src1.get()).unwrap()));
        assert_eq!("/d/e/f",
                   format!("{}", mapper.translate(src2.get()).unwrap()));
    }

    #[test]
    fn fails_missing_paths() {
        let mut mapper = Mapper::new();
        let src1 = "/foo/bar".parse::<Buffer>().unwrap();
        let dst = "/a/b/c".parse::<Buffer>().unwrap();
        mapper.add(src1.get(), dst).unwrap();
        let src2 = "/foo/ter".parse::<Buffer>().unwrap();
        let dst = "/d/e/f".parse::<Buffer>().unwrap();
        mapper.add(src2.get(), dst).unwrap();
        let src3 = "/foo/bla".parse::<Buffer>().unwrap();
        assert!(mapper.translate(src3.get()).is_none());
    }

    #[test]
    fn refuses_to_redefine() {
        let mut mapper = Mapper::new();
        let src = "/foo/bar".parse::<Buffer>().unwrap();
        let dst = "/a/b/c".parse::<Buffer>().unwrap();
        assert!(mapper.add(src.get(), dst.clone()).is_ok());
        assert!(mapper.add(src.get(), dst.clone()).is_err());
        assert!(mapper.add(src.get(), dst.clone()).is_err());
    }
}
