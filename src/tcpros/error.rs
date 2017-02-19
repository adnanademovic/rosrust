error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }
    links {
        SerdeRosmsg(::serde_rosmsg::error::Error, ::serde_rosmsg::error::ErrorKind);
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
