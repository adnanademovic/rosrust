use crossbeam::channel::Receiver;

#[allow(dead_code)]
pub fn test_subscriber(rx: Receiver<String>, regex_string: &str, counts: bool, checks: usize) {
    let regex = regex::Regex::new(regex_string).unwrap();

    let mut checks_to_perform = checks;
    let mut previous_capture = 0.0;

    while checks_to_perform > 0 {
        let data = rx.recv().unwrap();
        let captures = regex.captures(&data).unwrap();
        if counts {
            let newest_capture = captures[1].parse::<f64>().unwrap();
            assert!(newest_capture > previous_capture);
            previous_capture = newest_capture;
        }
        checks_to_perform -= 1;
    }
}
