pub use api::naming::error as naming;
pub use rosxmlrpc::error as rosxmlrpc;
pub use tcpros::error as tcpros;
pub use rosxmlrpc::ResponseError;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Nix(::nix::Error);
        FromUTF8(::std::string::FromUtf8Error);
        Response(ResponseError);
    }
    links {
        XmlRpc(rosxmlrpc::Error, rosxmlrpc::ErrorKind);
        Tcpros(tcpros::Error, tcpros::ErrorKind);
        Naming(naming::Error, naming::ErrorKind);
    }
    errors {
        Duplicate(t: String) {
            description("Could not add duplicate")
            display("Could not add duplicate {}", t)
        }
        TimeoutError
    }
}

pub mod api {
    error_chain! {
        errors {
            SystemFail(message: String) {
                description("Failure to handle API call")
                display("Failure to handle API call: {}", message)
            }
            BadData(message: String) {
                description("Bad parameters provided in API call")
                display("Bad parameters provided in API call: {}", message)
            }
        }
    }
}
