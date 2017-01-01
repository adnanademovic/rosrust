error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }
    errors {
        UnsupportedDataType(t: String) {
            description("Datatype is not supported")
            display("Datatype is not supported, issue within {}", t)
        }
        Mismatch {
            description("Mismatch within data")
            display("Mismatch within data")
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
