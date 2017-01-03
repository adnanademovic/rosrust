pub use ::rosxmlrpc::serde::error as serde;

error_chain! {
    foreign_links {
        Fmt(::hyper::error::Error);
        Io(::std::io::Error);
        Utf8(::std::string::FromUtf8Error);
    }
    links {
        Serde(serde::Error, serde::ErrorKind);
    }
}
