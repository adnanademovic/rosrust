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
    let mut previous_capture = -1.0;

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

#[allow(dead_code)]
pub fn test_publisher<T: Clone + rosrust::Message>(
    publisher: &rosrust::Publisher<T>,
    message: &T,
    rx: &Receiver<(i8, String)>,
    regex_string: &str,
    attempts: usize,
) {
    let regex = regex::Regex::new(regex_string).unwrap();

    let rate = rosrust::rate(10.0);

    for _ in 0..attempts {
        publisher.send(message.clone()).unwrap();
        rate.sleep();

        for (level, message) in rx.try_iter() {
            println!("Received message at level {}: {}", level, message);
            if level == 2 && regex.is_match(&message) {
                return;
            }
        }
    }

    panic!("Failed to receive logged data on /rosout_agg");
}
