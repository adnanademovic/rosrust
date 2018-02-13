error_chain! {
    foreign_links {
        Io(::std::io::Error);
        SerdeRosmsg(::serde_rosmsg::error::Error);
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

#[inline]
fn is_closed_connection(err: &::std::io::Error) -> bool {
    use std::io::ErrorKind as IoErrorKind;
    match err.kind() {
        IoErrorKind::BrokenPipe | IoErrorKind::ConnectionReset => true,
        _ => false,
    }
}

impl Error {
    pub fn is_closed_connection(&self) -> bool {
        match *self.kind() {
            ErrorKind::SerdeRosmsg(ref serde_err) => match *serde_err.kind() {
                ::serde_rosmsg::error::ErrorKind::Io(ref io_err) => is_closed_connection(io_err),
                _ => false,
            },
            ErrorKind::Io(ref io_err) => is_closed_connection(io_err),
            _ => false,
        }
    }
}
