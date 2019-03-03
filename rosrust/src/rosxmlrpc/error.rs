#![allow(deprecated)]
error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Utf8(::std::string::FromUtf8Error);
        ForeignXmlRpc(xml_rpc::error::Error);
    }

    errors {
        TopicConnectionError(topic: String) {
            description("Could not connect to topic")
            display("Could not connect to {}", topic)
        }
    }
}
