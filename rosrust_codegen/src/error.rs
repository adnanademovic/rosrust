error_chain::error_chain! {
    foreign_links {
        Regex(::regex::Error);
    }

    errors {
        MessageNotFound(msg: String, folders: String) {
            description("message not found in provided directories")
            display("message {} not found in provided directories\nDirectories:\n{}", msg, folders)
        }
    }
}
