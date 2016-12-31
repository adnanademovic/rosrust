use std;

#[derive(Debug)]
pub enum Error {
    IllegalPath,
    MappingSourceExists,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::error::Error;
        write!(f, "Naming error: {}", self.description())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IllegalPath => "Illegal ROS path specified",
            Error::MappingSourceExists => "Path is already mapped to another path",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}
