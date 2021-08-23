error_chain::error_chain! {
    errors {
        MessageNotFound(msg: String, folders: String) {
            description("message not found in provided directories")
            display("message {} not found in provided directories\nDirectories:\n{}", msg, folders)
        }
        PackageNameInvalid(package: String) {
            description("referenced package does not have a valid name. Look at ROS REP 144 for more details.")
            display("package '{}' does not have a valid name. Look at ROS REP 144 for more details.", package)
        }
    }
}
