error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }
    links {
        Decoder(self::decoder::Error, self::decoder::ErrorKind);
        Encoder(self::encoder::Error, self::encoder::ErrorKind);
    }
    errors {
        ServiceConnectionFail(service: String, uri: String) {
            description("Failed to connect to service")
            display("Failed to connect with client to service {} at uri {}", service, uri)
        }
        TopicConnectionFail(topic:String) {
            description("Failed to connect to topic")
            display("Failed to connect to topic '{}'", topic)
        }
        HeaderMismatch(field: String, expected: String, actual: String) {
            description("Data field within header mismatched")
            display("Data field '{}' within header mismatched. Expected: '{}' Actual: '{}'",
                    field, expected, actual)
        }
        HeaderMissingField(field: String) {
            description("Data field within header missing")
            display("Data field '{}' within header missing", field)
        }
        MessageTypeMismatch(expected: String, actual: String) {
            description("Cannot publish with multiple message types")
            display("Cannot publish '{}' data on '{}' publisher", actual, expected)
        }
        ServiceResponseInterruption {
            description("Data stream interrupted while reading service response")
            display("Data stream interrupted while reading service response")
        }
        ServiceResponseUnknown {
            description("Unknown error caused service response to panic")
            display("Unknown error caused service response to panic")
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
