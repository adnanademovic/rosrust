use super::naming;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Nix(::nix::Error);
        FromUTF8(::std::string::FromUtf8Error);
    }
    links {
        XmlRpc(::rosxmlrpc::error::Error, ::rosxmlrpc::error::ErrorKind);
        Tcpros(::tcpros::error::Error, ::tcpros::error::ErrorKind);
        Naming(naming::error::Error, naming::error::ErrorKind);
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
            XmlRpc(::rosxmlrpc::error::Error, ::rosxmlrpc::error::ErrorKind);
            Api(super::api::Error, super::api::ErrorKind);
        }
    }
}
