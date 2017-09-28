error_chain!{
    foreign_links {
        Regex(::regex::Error);
    }
}
