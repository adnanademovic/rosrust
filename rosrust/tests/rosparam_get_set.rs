use lazy_static::lazy_static;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::process::Command;
use std::str::from_utf8;
use yaml_rust::YamlLoader;

mod util;

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Clone, Debug, PartialEq, Eq)]
struct Struct1 {
    pub param1: String,
    pub param2: u32,
}

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Clone, Debug, PartialEq, Eq)]
struct Struct2 {
    pub param3: Vec<bool>,
    pub param4: BTreeMap<String, u32>,
}

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Clone, Debug, PartialEq, Eq)]
struct Struct3 {
    pub param5: Struct1,
    pub param6: Struct2,
}

fn global_init() -> util::ChildProcessTerminator {
    let roscore = util::run_roscore_for(util::TestVariant::RosparamGetSet);
    rosrust::init("rosparam_tester");
    roscore
}

lazy_static! {
    static ref ROS_CORE: Option<util::ChildProcessTerminator> = Some(global_init());
}

fn setup() {
    assert!(ROS_CORE.is_some());
}

#[test]
fn simple_loopback_works() {
    setup();
    let parameter = rosrust::param("test1").unwrap();
    parameter.set(&String::from("foo")).unwrap();
    assert_eq!("foo", parameter.get::<String>().unwrap());
}

#[test]
fn types_are_checked() {
    setup();
    let parameter = rosrust::param("test2").unwrap();
    parameter.set(&String::from("foo")).unwrap();
    assert!(parameter.get::<bool>().is_err());
}

fn test_param_case<'de, T>(key: &str, value: &T)
where
    T: serde::Deserialize<'de> + serde::Serialize + Debug + Eq,
{
    let parameter = rosrust::param(key).unwrap();
    assert!(parameter.exists().unwrap());
    assert_eq!(value, &parameter.get::<T>().unwrap());
}

fn test_param_cases<'de, T>(key: &str, value: &T)
where
    T: serde::Deserialize<'de> + serde::Serialize + Debug + Eq,
{
    for prefix in &["test3", "test3yaml", "test3partial"] {
        test_param_case(&format!("/{}{}", prefix, key), value);
    }
}

fn test_param_case_via_rosparam(key: &str, value: &str) {
    let desired = YamlLoader::load_from_str(value).unwrap();
    let output = Command::new("rosparam")
        .arg("get")
        .arg(key)
        .output()
        .unwrap();
    assert!(output.status.success());
    let output_value = from_utf8(&output.stdout).unwrap();
    let rosparam_provided = YamlLoader::load_from_str(output_value).unwrap();
    assert!(
        rosparam_provided == desired,
        "rosparam get {} unexpectedly returned: {}",
        key,
        output_value
    );
}

fn test_param_cases_via_rosparam(key: &str, value: &str) {
    for prefix in &["test3", "test3yaml", "test3partial"] {
        test_param_case_via_rosparam(&format!("/{}{}", prefix, key), value);
    }
}

fn rosparam_set_key_value(key: &str, value: &str) {
    assert!(Command::new("rosparam")
        .arg("set")
        .arg(key)
        .arg(value)
        .output()
        .unwrap()
        .status
        .success());
}

#[test]
fn structures_create_subtrees() {
    setup();
    let parameter1 = rosrust::param("test3").unwrap();
    let mut param4 = BTreeMap::new();
    param4.insert("bar".into(), 100);
    param4.insert("baz".into(), 200);
    let data = Struct3 {
        param5: Struct1 {
            param1: String::from("foo"),
            param2: 42,
        },
        param6: Struct2 {
            param3: vec![true, false, true, true, false, false],
            param4,
        },
    };

    // Set /test3 to actual structure
    parameter1.set(&data).unwrap();

    // Set /test3yaml to yaml representation of the structure via rosparam
    rosparam_set_key_value("test3yaml","{param5: {param1: foo, param2: '42'}, param6: {param3: [true, false, true, true, false, false], param4: {bar: '100', baz: '200'}}}");

    // Set /test3partial field by field via rosparam
    rosparam_set_key_value("test3partial/param5/param1", "foo");
    rosparam_set_key_value("test3partial/param5/param2", "'42'");
    rosparam_set_key_value(
        "test3partial/param6/param3",
        "[true, false, true, true, false, false]",
    );
    rosparam_set_key_value("test3partial/param6/param4/bar", "'100'");
    rosparam_set_key_value("test3partial/param6/param4/baz", "'200'");

    let parameter_set = rosrust::parameters()
        .unwrap()
        .into_iter()
        .collect::<BTreeSet<String>>();
    for prefix in &["test3", "test3yaml", "test3partial"] {
        assert!(parameter_set.contains(&format!("/{}/param5/param1", prefix)));
        assert!(parameter_set.contains(&format!("/{}/param5/param2", prefix)));
        assert!(parameter_set.contains(&format!("/{}/param6/param3", prefix)));
        assert!(parameter_set.contains(&format!("/{}/param6/param4/bar", prefix)));
        assert!(parameter_set.contains(&format!("/{}/param6/param4/baz", prefix)));
    }

    test_param_cases("", &data);
    test_param_cases("/param5", &data.param5);
    test_param_cases("/param6", &data.param6);
    test_param_cases("/param5/param1", &data.param5.param1);
    test_param_cases("/param5/param2", &data.param5.param2);
    test_param_cases("/param6/param3", &data.param6.param3);
    test_param_cases("/param6/param4", &data.param6.param4);
    test_param_cases("/param6/param4/bar", &data.param6.param4["bar"]);
    test_param_cases("/param6/param4/baz", &data.param6.param4["baz"]);

    test_param_cases_via_rosparam("/param5/param1", "foo");
    test_param_cases_via_rosparam("/param5/param2", "'42'");
    test_param_cases_via_rosparam("/param5", "{param1: foo, param2: '42'}");
    test_param_cases_via_rosparam("/param6/param3", "[true, false, true, true, false, false]");
    test_param_cases_via_rosparam("/param6/param4", "{bar: '100', baz: '200'}");
    test_param_cases_via_rosparam("/param6/param4/bar", "'100'");
    test_param_cases_via_rosparam("/param6/param4/baz", "'200'");
    test_param_cases_via_rosparam(
        "/param6",
        "{param3: [true, false, true, true, false, false], param4: {bar: '100', baz: '200'}}",
    );
    test_param_cases_via_rosparam("", "{param5: {param1: foo, param2: '42'}, param6: {param3: [true, false, true, true, false, false], param4: {bar: '100', baz: '200'}}}");
}
