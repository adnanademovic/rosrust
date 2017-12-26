use std::{self, env};

pub fn master() -> String {
    if let Some(v) = find_with_prefix("__master:=") {
        return v;
    }
    env::var("ROS_MASTER_URI").unwrap_or_else(|_| String::from("http://localhost:11311/"))
}

pub fn hostname() -> String {
    if let Some(v) = find_with_prefix("__hostname:=") {
        return v;
    }
    if let Some(v) = find_with_prefix("__ip:=") {
        return v;
    }
    if let Ok(v) = env::var("ROS_HOSTNAME") {
        return v;
    }
    if let Ok(v) = env::var("ROS_IP") {
        return v;
    }
    system_hostname()
}

pub fn namespace() -> String {
    if let Some(v) = find_with_prefix("__ns:=") {
        return v;
    }
    env::var("ROS_NAMESPACE").unwrap_or_default()
}

pub fn name(default: &str) -> String {
    find_with_prefix("__name:=").unwrap_or_else(|| String::from(default))
}

pub fn mappings() -> Vec<(String, String)> {
    args()
        .skip(1)
        .filter(|v| !v.starts_with('_'))
        .map(|v| v.split(":=").map(String::from).collect::<Vec<String>>())
        .filter(|v| v.len() == 2)
        .map(|v| v.into_iter())
        .map(|mut v| {
            (
                v.next().expect(UNEXPECTED_EMPTY_ARRAY),
                v.next().expect(UNEXPECTED_EMPTY_ARRAY),
            )
        })
        .collect()
}

pub fn params() -> Vec<(String, String)> {
    args()
        .skip(1)
        .filter(|v| v.starts_with('_'))
        .filter(|v| !v.starts_with("__"))
        .map(|v| v.splitn(2, ":=").map(String::from).collect::<Vec<String>>())
        .filter(|v| v.len() == 2)
        .map(|v| v.into_iter())
        .map(|mut v| {
            (
                v.next()
                    .expect(UNEXPECTED_EMPTY_ARRAY)
                    .replacen('_', "~", 1),
                v.next().expect(UNEXPECTED_EMPTY_ARRAY),
            )
        })
        .collect()
}

pub fn get_unused_args() -> Vec<String> {
    args()
        .enumerate()
        .filter_map(|(idx, v)| {
            if idx == 0 || !v.contains(":=") {
                Some(v)
            } else {
                None
            }
        })
        .collect()
}

fn find_with_prefix(prefix: &str) -> Option<String> {
    args()
        .skip(1)
        .find(|v| v.starts_with(prefix))
        .map(|v| String::from(v.trim_left_matches(prefix)))
}

#[cfg(not(test))]
fn system_hostname() -> String {
    use nix::unistd::gethostname;
    let mut hostname = [0u8; 256];
    gethostname(&mut hostname)
        .expect("Hostname is either unavailable or too long to fit into buffer");
    let hostname = hostname
        .into_iter()
        .take_while(|&v| *v != 0u8)
        .cloned()
        .collect::<Vec<_>>();
    String::from_utf8(hostname).expect("Hostname is not legal UTF-8")
}

#[cfg(test)]
fn system_hostname() -> String {
    String::from("myhostname")
}

#[cfg(not(test))]
#[inline]
fn args() -> std::env::Args {
    env::args()
}

#[cfg(test)]
#[inline]
fn args() -> std::vec::IntoIter<String> {
    tests::args_mock()
}

static UNEXPECTED_EMPTY_ARRAY: &'static str = "Popping failure from this array is impossible";

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::{self, env};

    static FAILED_TO_LOCK: &'static str = "Failed to acquire lock";

    lazy_static!{
        static ref DATA: Mutex<Vec<String>> = Mutex::new(Vec::new());
        static ref TESTCASE: Mutex<()> = Mutex::new(());
    }

    pub fn args_mock() -> std::vec::IntoIter<String> {
        DATA.lock().expect(FAILED_TO_LOCK).clone().into_iter()
    }

    fn set_args(args: &Vec<&str>) {
        let mut data = DATA.lock().expect(FAILED_TO_LOCK);
        data.clear();
        data.push(String::from("IGNORE"));
        for &arg in args {
            data.push(String::from(arg));
        }
    }

    #[test]
    #[allow(unused_variables)]
    fn mappings_default_to_empty_vector() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        assert_eq!(Vec::<(String, String)>::new(), mappings());
    }

    #[test]
    #[allow(unused_variables)]
    fn mappings_maps_good_and_ignores_everything_else() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        assert_eq!(Vec::<(String, String)>::new(), mappings());
        set_args(&vec![
            "a:=x",
            "b=e",
            "/c:=d",
            "e:=/f_g",
            "__name:=something",
            "_param:=something",
            "a:=b:=c",
            "~oo_e:=/ab_c",
            "/x_y:=~i",
        ]);
        assert_eq!(
            vec![
                (String::from("a"), String::from("x")),
                (String::from("/c"), String::from("d")),
                (String::from("e"), String::from("/f_g")),
                (String::from("~oo_e"), String::from("/ab_c")),
                (String::from("/x_y"), String::from("~i")),
            ],
            mappings()
        );
    }

    #[test]
    #[allow(unused_variables)]
    fn params_default_to_empty_vector() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        assert_eq!(Vec::<(String, String)>::new(), params());
    }

    #[test]
    #[allow(unused_variables)]
    fn params_maps_good_and_ignores_everything_else() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        assert_eq!(Vec::<(String, String)>::new(), params());
        set_args(&vec![
            "a:=x",
            "b=e",
            "/c:=d",
            "e:=/f_g",
            "__name:=something",
            "_param:=something",
            "a:=b:=c",
            "~oo_e:=/ab_c",
            "_foo:=123:=456",
            "/x_y:=~i",
        ]);
        assert_eq!(
            vec![
                (String::from("~param"), String::from("something")),
                (String::from("~foo"), String::from("123:=456")),
            ],
            params()
        );
    }

    #[test]
    #[allow(unused_variables)]
    fn get_unused_args_gets_everything_without_equal_sign() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        assert_eq!(vec![String::from("IGNORE")], get_unused_args());
        set_args(&vec![
            "a:=x",
            "b=e",
            "/c:=d",
            "this",
            "e:=/f_g",
            "__name:=something",
            "_param:=something",
            "a:=b:=c",
            "foo",
            "~oo_e:=/ab_c",
            "_foo:=123:=456",
            "bar=:baz",
            "/x_y:=~i",
        ]);
        assert_eq!(
            vec![
                String::from("IGNORE"),
                String::from("b=e"),
                String::from("this"),
                String::from("foo"),
                String::from("bar=:baz"),
            ],
            get_unused_args()
        );
    }

    #[test]
    #[allow(unused_variables)]
    fn name_uses_passed_value_by_default() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        assert_eq!(String::from("myname"), name("myname"));
        set_args(&vec!["unimportant", "also_unimportant"]);
        assert_eq!(String::from("othername"), name("othername"));
    }

    #[test]
    #[allow(unused_variables)]
    fn name_uses_argument_when_provided() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        assert_eq!(String::from("myname"), name("myname"));
        set_args(&vec!["__name:=othername"]);
        assert_eq!(String::from("othername"), name("myname"));
    }

    #[test]
    #[allow(unused_variables)]
    fn namespace_uses_empty_string_by_default() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_NAMESPACE");
        assert_eq!(String::from(""), namespace());
        set_args(&vec!["unimportant", "also_unimportant"]);
        assert_eq!(String::from(""), namespace());
    }

    #[test]
    #[allow(unused_variables)]
    fn namespace_uses_environment_when_passed() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_NAMESPACE");
        assert_eq!(String::from(""), namespace());
        env::set_var("ROS_NAMESPACE", "/myns");
        assert_eq!(String::from("/myns"), namespace());
    }

    #[test]
    #[allow(unused_variables)]
    fn namespace_uses_argument_when_passed() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_NAMESPACE");
        assert_eq!(String::from(""), namespace());
        set_args(&vec!["__ns:=/myns"]);
        assert_eq!(String::from("/myns"), namespace());
    }

    #[test]
    #[allow(unused_variables)]
    fn namespace_prioritizes_argument_when_both_passed() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_NAMESPACE");
        assert_eq!(String::from(""), namespace());
        env::set_var("ROS_NAMESPACE", "/myns1");
        set_args(&vec!["__ns:=/myns2"]);
        assert_eq!(String::from("/myns2"), namespace());
    }

    #[test]
    #[allow(unused_variables)]
    fn master_uses_default_uri_by_default() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_MASTER_URI");
        assert_eq!(String::from("http://localhost:11311/"), master());
        set_args(&vec!["unimportant", "also_unimportant"]);
        assert_eq!(String::from("http://localhost:11311/"), master());
    }

    #[test]
    #[allow(unused_variables)]
    fn master_uses_environment_when_passed() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_MASTER_URI");
        assert_eq!(String::from("http://localhost:11311/"), master());
        env::set_var("ROS_MASTER_URI", "http://somebody:21212/");
        assert_eq!(String::from("http://somebody:21212/"), master());
    }

    #[test]
    #[allow(unused_variables)]
    fn master_uses_argument_when_passed() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_MASTER_URI");
        assert_eq!(String::from("http://localhost:11311/"), master());
        set_args(&vec!["__master:=http://somebody:21212/"]);
        assert_eq!(String::from("http://somebody:21212/"), master());
    }

    #[test]
    #[allow(unused_variables)]
    fn master_prioritizes_argument_when_both_passed() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_MASTER_URI");
        assert_eq!(String::from("http://localhost:11311/"), master());
        env::set_var("ROS_MASTER_URI", "http://somebody1:21212/");
        set_args(&vec!["__master:=http://somebody2:21212/"]);
        assert_eq!(String::from("http://somebody2:21212/"), master());
    }

    #[test]
    #[allow(unused_variables)]
    fn hostname_uses_default_uri_by_default() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_HOSTNAME");
        env::remove_var("ROS_IP");
        assert_eq!(String::from("myhostname"), hostname());
        set_args(&vec!["unimportant", "also_unimportant"]);
        assert_eq!(String::from("myhostname"), hostname());
    }

    #[test]
    #[allow(unused_variables)]
    fn hostname_uses_hostname_environment_when_passed() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_HOSTNAME");
        env::remove_var("ROS_IP");
        assert_eq!(String::from("myhostname"), hostname());
        env::set_var("ROS_HOSTNAME", "host");
        assert_eq!(String::from("host"), hostname());
    }

    #[test]
    #[allow(unused_variables)]
    fn hostname_uses_ip_environment_when_passed() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_HOSTNAME");
        env::remove_var("ROS_IP");
        assert_eq!(String::from("myhostname"), hostname());
        env::set_var("ROS_IP", "192.168.0.1");
        assert_eq!(String::from("192.168.0.1"), hostname());
    }

    #[test]
    #[allow(unused_variables)]
    fn hostname_prioritizes_hostname_over_ip_environment_when_passed() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_HOSTNAME");
        env::remove_var("ROS_IP");
        assert_eq!(String::from("myhostname"), hostname());
        env::set_var("ROS_HOSTNAME", "host");
        env::set_var("ROS_IP", "192.168.0.1");
        assert_eq!(String::from("host"), hostname());
    }

    #[test]
    #[allow(unused_variables)]
    fn hostname_uses_hostname_argument_when_passed() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_HOSTNAME");
        env::remove_var("ROS_IP");
        assert_eq!(String::from("myhostname"), hostname());
        set_args(&vec!["__hostname:=host"]);
        assert_eq!(String::from("host"), hostname());
    }

    #[test]
    #[allow(unused_variables)]
    fn hostname_uses_ip_argument_when_passed() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_HOSTNAME");
        env::remove_var("ROS_IP");
        assert_eq!(String::from("myhostname"), hostname());
        set_args(&vec!["__ip:=192.168.0.1"]);
        assert_eq!(String::from("192.168.0.1"), hostname());
    }

    #[test]
    #[allow(unused_variables)]
    fn hostname_prioritizes_hostname_over_ip_argument_when_passed() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_HOSTNAME");
        env::remove_var("ROS_IP");
        assert_eq!(String::from("myhostname"), hostname());
        set_args(&vec!["__hostname:=host", "__ip:=192.168.0.1"]);
        assert_eq!(String::from("host"), hostname());
    }

    #[test]
    #[allow(unused_variables)]
    fn hostname_prioritizes_argument_when_both_passed() {
        let testcase = TESTCASE.lock().expect(FAILED_TO_LOCK);
        set_args(&vec![]);
        env::remove_var("ROS_HOSTNAME");
        env::remove_var("ROS_IP");
        assert_eq!(String::from("myhostname"), hostname());
        env::set_var("ROS_HOSTNAME", "host");
        env::set_var("ROS_IP", "192.168.0.1");
        set_args(&vec!["__hostname:=host2", "__ip:=127.0.0.1"]);
        assert_eq!(String::from("host2"), hostname());
    }

}
