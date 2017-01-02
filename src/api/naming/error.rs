error_chain! {
    errors {
        IllegalPath {
            description("Illegal ROS path specified")
            display("Illegal ROS path specified")
        }
        MappingSourceExists {
            description("Path is already mapped to another path")
            display("Path is already mapped to another path")
        }
    }
}
