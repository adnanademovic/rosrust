error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }
    links {
        Decoder(self::decoder::Error, self::decoder::ErrorKind);
        Encoder(self::encoder::Error, self::encoder::ErrorKind);
    }
    errors {
        Mismatch {
            description("Mismatch within data")
            display("Mismatch within data")
        }
    }
}


pub mod decoder {
    error_chain! {
        errors {
            UnsupportedDataType(t: String) {
                description("Datatype is not decodable")
                display("Datatype is not decodable, issue within {}", t)
            }
            FailedToDecode(t: String) {
                description("Failed to decode")
                display("Failed to decode {}", t)
            }
            EndOfBuffer {
                description("Reached end of memory buffer")
                display("Reached end of memory buffer while reading data")
            }
        }
    }
}

pub mod encoder {
    error_chain! {
        errors {
            UnsupportedDataType(t: String) {
                description("Datatype is not encodable")
                display("Datatype is not encodable, issue within {}", t)
            }
        }
    }
}
