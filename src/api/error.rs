pub use api::naming::error as naming;
pub use rosxmlrpc::error as rosxmlrpc;
pub use tcpros::error as tcpros;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Nix(::nix::Error);
        FromUTF8(::std::string::FromUtf8Error);
    }
    links {
        XmlRpc(rosxmlrpc::Error, rosxmlrpc::ErrorKind);
        Tcpros(tcpros::Error, tcpros::ErrorKind);
        Naming(naming::Error, naming::ErrorKind);
        Master(self::master::Error, self::master::ErrorKind);
    }
    errors {
        Duplicate(t: String) {
            description("Could not add duplicate")
            display("Could not add duplicate {}", t)
        }
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

pub mod master {
    use xml_rpc;

    error_chain! {
        links {
            XmlRpcErr(xml_rpc::error::Error, xml_rpc::error::ErrorKind);
            XmlRpc(super::rosxmlrpc::Error, super::rosxmlrpc::ErrorKind);
            Api(::error::api::Error, ::error::api::ErrorKind);
        }
        errors {
            Fault(fault: xml_rpc::Fault) {
                description("Queried XML-RPC server returned a fault")
                display("Fault #{}: {}", fault.code, fault.message)
            }
        }
    }
}
