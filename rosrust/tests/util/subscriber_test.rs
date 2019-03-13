use crossbeam::channel::Receiver;

#[allow(dead_code)]
pub fn test_subscriber(rx: Receiver<String>, regex_string: &str, counts: bool, checks: usize) {
    test_subscriber_detailed(rx, regex_string, counts, checks, true)
}

#[allow(dead_code)]
pub fn test_subscriber_detailed(
    rx: Receiver<String>,
    regex_string: &str,
    counts: bool,
    checks: usize,
    force_match: bool,
) {
    let regex = regex::Regex::new(regex_string).unwrap();

    let mut checks_to_perform = checks;
    let mut previous_capture = 0.0;

    while checks_to_perform > 0 {
        let data = rx.recv().unwrap();
        println!("Handling: {}", data);
        let captures_option = regex.captures(&data);
        let captures = if force_match {
            captures_option.unwrap()
        } else {
            match captures_option {
                None => continue,
                Some(value) => value,
            }
        };
        if counts {
            let newest_capture = captures[1].parse::<f64>().unwrap();
            assert!(newest_capture > previous_capture);
            previous_capture = newest_capture;
        }
        checks_to_perform -= 1;
    }
}
