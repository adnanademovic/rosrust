use self::path::Path;
use self::mapper::Mapper;
pub use self::error::{Error, ErrorKind};

mod path;
mod mapper;
pub mod error;

pub struct Resolver {
    path: path::Buffer,
    namespace: path::Buffer,
    mapper: Mapper,
}

impl Resolver {
    pub fn new(name: &str) -> Result<Resolver, Error> {
        let path = name.parse::<path::Buffer>()?;
        let namespace = path.parent()?.take();
        Ok(Resolver {
            path: path,
            namespace: namespace,
            mapper: Mapper::new(),
        })
    }

    pub fn map(&mut self, source: &str, destination: &str) -> Result<(), Error> {
        let source = self.resolve(source)?;
        let destination = self.resolve(destination)?;
        self.mapper.add(source.get(), destination);
        Ok(())
    }

    fn resolve(&self, name: &str) -> Result<path::Buffer, Error> {
        let first_char = *name.as_bytes().get(0).ok_or(ErrorKind::EmptyName)?;
        if first_char == b'/' {
            return name.parse();
        }
        Ok(if first_char == b'~' {
            self.path.slice() + (String::from("/") + &name[1..]).parse::<path::Buffer>()?
        } else {
            self.namespace.slice() + (String::from("/") + name).parse::<path::Buffer>()?
        })
    }

    pub fn translate(&self, name: &str) -> Result<String, Error> {
        let path = self.resolve(name)?;
        match self.mapper.translate(path.get()) {
            Some(v) => Ok(format!("{}", v)),
            None => Ok(format!("{}", path)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::path::Path;

    static FAILED_TO_RESOLVE: &'static str = "Failed to resolve";

    #[test]
    fn constructs_from_legal_path() {
        assert!(Resolver::new("/foo").is_ok());
        assert!(Resolver::new("/foo/bar").is_ok());
        assert!(Resolver::new("/f1_aA/Ba02/Xx").is_ok());
        assert!(Resolver::new("").is_err());
        assert!(Resolver::new("a").is_err());
        assert!(Resolver::new("/123/").is_err());
        assert!(Resolver::new("/foo$").is_err());
        assert!(Resolver::new("/_e").is_err());
        assert!(Resolver::new("/a//b").is_err());
    }

    #[test]
    fn rejects_illegal_names() {
        let r = Resolver::new("/some/long/path").expect(FAILED_TO_RESOLVE);
        assert!(r.resolve("/fo$o").is_err());
        assert!(r.resolve("1foo/bar").is_err());
        assert!(r.resolve("#f1_aA/Ba02/Xx").is_err());
    }

    #[test]
    fn resolves_absolute_names() {
        let r = Resolver::new("/some/long/path").expect(FAILED_TO_RESOLVE);
        assert_eq!(
            vec![String::from("foo")],
            r.resolve("/foo").expect(FAILED_TO_RESOLVE).get()
        );
        assert_eq!(
            vec![String::from("foo"), String::from("bar")],
            r.resolve("/foo/bar").expect(FAILED_TO_RESOLVE).get()
        );
        assert_eq!(
            vec![
                String::from("f1_aA"),
                String::from("Ba02"),
                String::from("Xx"),
            ],
            r.resolve("/f1_aA/Ba02/Xx").expect(FAILED_TO_RESOLVE).get()
        );
    }

    #[test]
    fn resolves_relative_names() {
        let r = Resolver::new("/some/long/path").expect(FAILED_TO_RESOLVE);
        assert_eq!(
            vec![
                String::from("some"),
                String::from("long"),
                String::from("foo"),
            ],
            r.resolve("foo").expect(FAILED_TO_RESOLVE).get()
        );
        assert_eq!(
            vec![
                String::from("some"),
                String::from("long"),
                String::from("foo"),
                String::from("bar"),
            ],
            r.resolve("foo/bar").expect(FAILED_TO_RESOLVE).get()
        );
        assert_eq!(
            vec![
                String::from("some"),
                String::from("long"),
                String::from("f1_aA"),
                String::from("Ba02"),
                String::from("Xx"),
            ],
            r.resolve("f1_aA/Ba02/Xx").expect(FAILED_TO_RESOLVE).get()
        );
    }

    #[test]
    fn resolves_private_names() {
        let r = Resolver::new("/some/long/path").expect(FAILED_TO_RESOLVE);
        assert_eq!(
            vec![
                String::from("some"),
                String::from("long"),
                String::from("path"),
                String::from("foo"),
            ],
            r.resolve("~foo").expect(FAILED_TO_RESOLVE).get()
        );
        assert_eq!(
            vec![
                String::from("some"),
                String::from("long"),
                String::from("path"),
                String::from("foo"),
                String::from("bar"),
            ],
            r.resolve("~foo/bar").expect(FAILED_TO_RESOLVE).get()
        );
        assert_eq!(
            vec![
                String::from("some"),
                String::from("long"),
                String::from("path"),
                String::from("f1_aA"),
                String::from("Ba02"),
                String::from("Xx"),
            ],
            r.resolve("~f1_aA/Ba02/Xx").expect(FAILED_TO_RESOLVE).get()
        );
    }

    #[test]
    fn translates_strings() {
        let r = Resolver::new("/some/long/path").expect(FAILED_TO_RESOLVE);
        assert_eq!(
            String::from("/f1_aA/Ba02/Xx"),
            r.translate("/f1_aA/Ba02/Xx").expect(FAILED_TO_RESOLVE)
        );
        assert_eq!(
            String::from("/some/long/f1_aA/Ba02/Xx"),
            r.translate("f1_aA/Ba02/Xx").expect(FAILED_TO_RESOLVE)
        );
        assert_eq!(
            String::from("/some/long/path/f1_aA/Ba02/Xx"),
            r.translate("~f1_aA/Ba02/Xx").expect(FAILED_TO_RESOLVE)
        );
    }

    #[test]
    fn supports_remapping() {
        let mut r = Resolver::new("/some/long/path").expect(FAILED_TO_RESOLVE);
        r.map("a", "/d").expect(FAILED_TO_RESOLVE);
        r.map("~x", "/e").expect(FAILED_TO_RESOLVE);
        r.map("/z", "/f").expect(FAILED_TO_RESOLVE);
        r.map("/a1", "g").expect(FAILED_TO_RESOLVE);
        r.map("a2", "~g").expect(FAILED_TO_RESOLVE);
        assert_eq!(
            String::from("/d"),
            r.translate("/some/long/a").expect(FAILED_TO_RESOLVE)
        );
        assert_eq!(
            String::from("/e"),
            r.translate("path/x").expect(FAILED_TO_RESOLVE)
        );
        assert_eq!(
            String::from("/f"),
            r.translate("/z").expect(FAILED_TO_RESOLVE)
        );
        assert_eq!(
            String::from("/some/long/g"),
            r.translate("/a1").expect(FAILED_TO_RESOLVE)
        );
        assert_eq!(
            String::from("/some/long/path/g"),
            r.translate("/some/long/a2").expect(FAILED_TO_RESOLVE)
        );
        assert_eq!(
            String::from("/some/long/other"),
            r.translate("other").expect(FAILED_TO_RESOLVE)
        );
    }
}
