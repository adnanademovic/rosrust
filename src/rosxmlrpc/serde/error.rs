error_chain!{
    errors {
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
