error_chain!{
    errors {
        XmlRpcReading(t:String) {
            description("Issue while reading XML-RPC data")
            display("Issue while reading XML-RPC data, for {}", t)
        }
        Decoding(t: String) {
            description("Issue while decoding data structure")
            display("Issue while decoding data structure, within {}", t)
        }
        UnsupportedDataType(t: String) {
            description("Datatype is not supported")
            display("Datatype is not supported, issue within {}", t)
        }
        MismatchedDataFormat(t: String) {
            description("Provided XML-RPC tree does not match target format")
            display("Provided XML-RPC tree does not match target format, {} was expected", t)
        }
    }
}
