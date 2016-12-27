use self::path::Path;

mod path;

pub struct Resolver {
    path: path::Buffer,
    namespace: path::Buffer,
}

impl Resolver {
    pub fn new(name: &str) -> Result<Resolver, ()> {
        let path = name.parse::<path::Buffer>()?;
        let namespace = path.parent().ok_or(())?.take();
        Ok(Resolver {
            path: path,
            namespace: namespace,
        })
    }

    pub fn resolve(&self, name: &str) -> Result<path::Buffer, ()> {
        let first_char = *name.as_bytes().get(0).ok_or(())?;
        if first_char == b'/' {
            return name.parse();
        }
        Ok(if first_char == b'~' {
            self.path.slice() + (String::from("/") + &name[1..]).parse::<path::Buffer>()?
        } else {
            self.namespace.slice() + (String::from("/") + name).parse::<path::Buffer>()?
        })
    }

    pub fn translate(&self, name: &str) -> Result<String, ()> {
        Ok(format!("{}", self.resolve(name)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::path::Path;

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
        let r = Resolver::new("/some/long/path").unwrap();
        assert!(r.resolve("/fo$o").is_err());
        assert!(r.resolve("1foo/bar").is_err());
        assert!(r.resolve("#f1_aA/Ba02/Xx").is_err());
    }

    #[test]
    fn resolves_absolute_names() {
        let r = Resolver::new("/some/long/path").unwrap();
        assert_eq!(vec![String::from("foo")], r.resolve("/foo").unwrap().get());
        assert_eq!(vec![String::from("foo"), String::from("bar")],
                   r.resolve("/foo/bar").unwrap().get());
        assert_eq!(vec![String::from("f1_aA"), String::from("Ba02"), String::from("Xx")],
                   r.resolve("/f1_aA/Ba02/Xx").unwrap().get());
    }

    #[test]
    fn resolves_relative_names() {
        let r = Resolver::new("/some/long/path").unwrap();
        assert_eq!(vec![String::from("some"), String::from("long"), String::from("foo")],
                   r.resolve("foo").unwrap().get());
        assert_eq!(vec![String::from("some"),
                        String::from("long"),
                        String::from("foo"),
                        String::from("bar")],
                   r.resolve("foo/bar").unwrap().get());
        assert_eq!(vec![String::from("some"),
                        String::from("long"),
                        String::from("f1_aA"),
                        String::from("Ba02"),
                        String::from("Xx")],
                   r.resolve("f1_aA/Ba02/Xx").unwrap().get());
    }

    #[test]
    fn resolves_private_names() {
        let r = Resolver::new("/some/long/path").unwrap();
        assert_eq!(vec![String::from("some"),
                        String::from("long"),
                        String::from("path"),
                        String::from("foo")],
                   r.resolve("~foo").unwrap().get());
        assert_eq!(vec![String::from("some"),
                        String::from("long"),
                        String::from("path"),
                        String::from("foo"),
                        String::from("bar")],
                   r.resolve("~foo/bar").unwrap().get());
        assert_eq!(vec![String::from("some"),
                        String::from("long"),
                        String::from("path"),
                        String::from("f1_aA"),
                        String::from("Ba02"),
                        String::from("Xx")],
                   r.resolve("~f1_aA/Ba02/Xx").unwrap().get());
    }

    #[test]
    fn translates_strings() {
        let r = Resolver::new("/some/long/path").unwrap();
        assert_eq!(String::from("/f1_aA/Ba02/Xx"),
                   r.translate("/f1_aA/Ba02/Xx").unwrap());
        assert_eq!(String::from("/some/long/f1_aA/Ba02/Xx"),
                   r.translate("f1_aA/Ba02/Xx").unwrap());
        assert_eq!(String::from("/some/long/path/f1_aA/Ba02/Xx"),
                   r.translate("~f1_aA/Ba02/Xx").unwrap());
    }
}
