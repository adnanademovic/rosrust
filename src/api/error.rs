pub use ::api::naming::error as naming;
pub use ::rosxmlrpc::error as rosxmlrpc;
pub use ::tcpros::error as tcpros;

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
        Protocol(t: String) {
            description("Error within ROS protocols")
            display("Error within ROS protocols: {}", t)
        }
        Critical(t: String) {
            description("Critical error")
            display("Critical error: {}", t)
        }
    }
}

pub mod api {
    error_chain! {
        errors {
            Fail(message: String) {
                description("Failure to handle API call")
                display("Failure to handle API call: {}", message)
            }
            Error(message: String) {
                description("Error while handling API call")
                display("Error while handling API call: {}", message)
            }
        }
    }
}

pub mod master {
    error_chain! {
        links {
            XmlRpc(super::rosxmlrpc::Error, super::rosxmlrpc::ErrorKind);
            Api(::error::api::Error, ::error::api::ErrorKind);
        }
    }
}
