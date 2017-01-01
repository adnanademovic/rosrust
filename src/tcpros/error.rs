error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }
    links {
        Decoder(super::decoder::error::Error, super::decoder::error::ErrorKind);
        Encoder(super::encoder::error::Error, super::encoder::error::ErrorKind);
    }
    errors {
        Mismatch {
            description("Mismatch within data")
            display("Mismatch within data")
        }
    }
}
