error_chain! {
    errors {
        IllegalCharacter(name: String) {
            description("Illegal character")
            display("Illegal character in '{}' - limited to letters, numbers and underscores", name)
        }
        IllegalFirstCharacter(name: String) {
            description("Illegal first character")
            display("Illegal first character in '{}' - limited to letters", name)
        }
        EmptyName {
            description("Name in path is empty")
            display("Name in path is empty")
        }
        LeadingSlashMissing(path: String) {
            description("Leading slash is missing")
            display("Leading slash is missing in path {}", path)
        }
        MissingParent {
            description("Path has no parent due to being root path")
            display("Path has no parent due to being root path")
        }
    }
}
