extern crate env_logger;
#[macro_use]
extern crate rosrust;
#[macro_use]
extern crate serde_derive;

fn main() {
    env_logger::init();

    // Initialize node
    rosrust::init("param_test");

    // Create parameter, go through all methods, and delete it
    let param = rosrust::param("~foo").unwrap();
    ros_info!("Handling ~foo:");
    ros_info!("Exists? {:?}", param.exists()); // false
    param.set(&42u64).unwrap();
    ros_info!("Get: {:?}", param.get::<u64>().unwrap());
    ros_info!("Get raw: {:?}", param.get_raw().unwrap());
    ros_info!("Search: {:?}", param.search().unwrap());
    ros_info!("Exists? {}", param.exists().unwrap());
    param.delete().unwrap();
    ros_info!("Get {:?}", param.get::<u64>().unwrap_err());
    ros_info!("Exists? {}", param.exists().unwrap());

    // Same as before, but don't delete it
    let param = rosrust::param("bar").unwrap();
    ros_info!("Handling bar:");
    param.set(&"string data").unwrap();
    ros_info!("Get: {:?}", param.get::<String>().unwrap());
    ros_info!("Get raw: {:?}", param.get_raw().unwrap());
    ros_info!("Search: {:?}", param.search().unwrap());
    ros_info!("Exists? {}", param.exists().unwrap());

    // Access existing parameter
    let param = rosrust::param("/baz").unwrap();
    ros_info!("Handling /baz:");
    if param.exists().unwrap() {
        ros_info!("Get raw: {:?}", param.get_raw().unwrap());
        ros_info!("Search: {:?}", param.search().unwrap());
    } else {
        ros_info!("Create own parameter /baz with 'rosparam' to observe interaction.");
    }

    // Access command line parameter
    let param = rosrust::param("~privbaz").unwrap();
    ros_info!("Handling ~privbaz:");
    if param.exists().unwrap() {
        ros_info!("Get raw: {:?}", param.get_raw().unwrap());
        ros_info!("Search: {:?}", param.search().unwrap());
    } else {
        ros_info!("Create ~privbaz by passing _privbaz:=value to observe interaction.");
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct TestStruct {
        foo: String,
        bar: i32,
        baz: bool,
    }

    // Create tree and output the end result
    ros_info!("Handling /qux:");
    ros_info!("Setting /qux/alpha");
    rosrust::param("/qux/alpha").unwrap().set(&"meow").unwrap();
    ros_info!("Setting /qux/beta");
    rosrust::param("/qux/beta").unwrap().set(&44).unwrap();
    ros_info!("Setting /qux/gamma/x");
    rosrust::param("/qux/gamma/x").unwrap().set(&3.0).unwrap();
    ros_info!("Setting /qux/gamma/y");
    rosrust::param("/qux/gamma/y").unwrap().set(&2).unwrap();
    ros_info!("Setting /qux/gamma/z");
    rosrust::param("/qux/gamma/z").unwrap().set(&true).unwrap();
    ros_info!("Setting /qux/delta");
    rosrust::param("/qux/delta")
        .unwrap()
        .set(&[1, 2, 3])
        .unwrap();
    rosrust::param("/qux/epsilon")
        .unwrap()
        .set(&TestStruct {
            foo: "x".into(),
            bar: 42,
            baz: false,
        })
        .unwrap();
    ros_info!(
        "Get raw: {:?}",
        rosrust::param("/qux").unwrap().get_raw().unwrap()
    );
}
