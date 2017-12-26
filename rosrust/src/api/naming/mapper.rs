use std::collections::HashMap;
use super::path::{Buffer, Path, Slice};

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

    pub fn add(&mut self, keys: &[String], value: Buffer) {
        match keys.split_first() {
            None => {
                self.value = Some(value);
            }
            Some((key, child_keys)) => {
                self.children
                    .entry(key.clone())
                    .or_insert_with(Mapper::new)
                    .add(child_keys, value);
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

    static FAILED_TO_MAP: &'static str = "Failed to map";

    #[test]
    fn matches_existing_paths() {
        let mut mapper = Mapper::new();
        let src1 = "/foo/bar".parse::<Buffer>().expect(FAILED_TO_MAP);
        let dst = "/a/b/c".parse::<Buffer>().expect(FAILED_TO_MAP);
        mapper.add(src1.get(), dst);
        let src2 = "/foo/ter".parse::<Buffer>().expect(FAILED_TO_MAP);
        let dst = "/d/e/f".parse::<Buffer>().expect(FAILED_TO_MAP);
        mapper.add(src2.get(), dst);
        assert_eq!(
            "/a/b/c",
            format!("{}", mapper.translate(src1.get()).expect(FAILED_TO_MAP))
        );
        assert_eq!(
            "/d/e/f",
            format!("{}", mapper.translate(src2.get()).expect(FAILED_TO_MAP))
        );
    }

    #[test]
    fn allows_root_path() {
        let mut mapper = Mapper::new();
        let src1 = "/".parse::<Buffer>().expect(FAILED_TO_MAP);
        let dst = "/a/b/c".parse::<Buffer>().expect(FAILED_TO_MAP);
        mapper.add(src1.get(), dst);
        let src2 = "/foo/ter".parse::<Buffer>().expect(FAILED_TO_MAP);
        let dst = "/".parse::<Buffer>().expect(FAILED_TO_MAP);
        mapper.add(src2.get(), dst);
        assert_eq!(
            "/a/b/c",
            format!("{}", mapper.translate(src1.get()).expect(FAILED_TO_MAP))
        );
        assert_eq!(
            "",
            format!("{}", mapper.translate(src2.get()).expect(FAILED_TO_MAP))
        );
    }

    #[test]
    fn fails_missing_paths() {
        let mut mapper = Mapper::new();
        let src1 = "/foo/bar".parse::<Buffer>().expect(FAILED_TO_MAP);
        let dst = "/a/b/c".parse::<Buffer>().expect(FAILED_TO_MAP);
        mapper.add(src1.get(), dst);
        let src2 = "/foo/ter".parse::<Buffer>().expect(FAILED_TO_MAP);
        let dst = "/d/e/f".parse::<Buffer>().expect(FAILED_TO_MAP);
        mapper.add(src2.get(), dst);
        let src3 = "/foo/bla".parse::<Buffer>().expect(FAILED_TO_MAP);
        assert!(mapper.translate(src3.get()).is_none());
    }

    #[test]
    fn allows_to_redefine() {
        let mut mapper = Mapper::new();
        let src = "/foo/bar".parse::<Buffer>().expect(FAILED_TO_MAP);
        let dst1 = "/a/b/c".parse::<Buffer>().expect(FAILED_TO_MAP);
        mapper.add(src.get(), dst1);
        assert_eq!(
            "/a/b/c",
            format!("{}", mapper.translate(src.get()).expect(FAILED_TO_MAP))
        );
        let dst2 = "/d/e/f".parse::<Buffer>().expect(FAILED_TO_MAP);
        mapper.add(src.get(), dst2);
        assert_eq!(
            "/d/e/f",
            format!("{}", mapper.translate(src.get()).expect(FAILED_TO_MAP))
        );
    }
}
