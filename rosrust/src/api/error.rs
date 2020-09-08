pub use crate::api::naming::error as naming;
pub use crate::rosxmlrpc::error as rosxmlrpc;
pub use crate::rosxmlrpc::ResponseError;
pub use crate::tcpros::error as tcpros;

error_chain::error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Nix(::nix::Error);
        FromUTF8(::std::string::FromUtf8Error);
        Response(ResponseError);
        SigintOverride(::ctrlc::Error);
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
        MismatchedType(topic: String, actual_type: String, attempted_type:String) {
            description("Attempted to connect to topic with wrong message type")
            display("Attempted to connect to {} topic '{}' with message type {}", actual_type, topic, attempted_type)
        }
        MultipleInitialization {
            description("Cannot initialize multiple nodes")
            display("Cannot initialize multiple nodes")
        }
        TimeoutError
        BadYamlData(details: String) {
            description("Bad YAML data provided")
            display("Bad YAML data provided: {}", details)
        }
        CannotResolveName(name: String) {
            description("Failed to resolve name")
            display("Failed to resolve name: {}", name)
        }
        CommunicationIssue(details: String) {
            description("Failure in communication with ROS API")
            display("Failure in communication with ROS API: {}", details)
        }
    }
}

pub mod api {
    error_chain::error_chain! {
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
