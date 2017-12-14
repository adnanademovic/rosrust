extern crate env_logger;
extern crate rosrust;
#[macro_use]
extern crate serde_derive;

use rosrust::Ros;

fn main() {
    env_logger::init().unwrap();

    let ros = Ros::new("param_test").unwrap();

    // Create parameter, go through all methods, and delete it
    let param = ros.param("~foo").unwrap();
    println!("Handling ~foo:");
    println!("Exists? {:?}", param.exists()); // false
    param.set(&42u64).unwrap();
    println!("Get: {:?}", param.get::<u64>().unwrap());
    println!("Get raw: {:?}", param.get_raw().unwrap());
    println!("Search: {:?}", param.search().unwrap());
    println!("Exists? {}", param.exists().unwrap());
    param.delete().unwrap();
    println!("Get {:?}", param.get::<u64>().unwrap_err());
    println!("Exists? {}", param.exists().unwrap());

    // Same as before, but don't delete it
    let param = ros.param("bar").unwrap();
    println!("Handling bar:");
    param.set(&"string data").unwrap();
    println!("Get: {:?}", param.get::<String>().unwrap());
    println!("Get raw: {:?}", param.get_raw().unwrap());
    println!("Search: {:?}", param.search().unwrap());
    println!("Exists? {}", param.exists().unwrap());

    // Access existing parameter
    let param = ros.param("/baz").unwrap();
    println!("Handling /baz:");
    if param.exists().unwrap() {
        println!("Get raw: {:?}", param.get_raw().unwrap());
        println!("Search: {:?}", param.search().unwrap());
    } else {
        println!("Create own parameter /baz with 'rosparam' to observe interaction.");
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct TestStruct {
        foo: String,
        bar: i32,
        baz: bool,
    }

    // Create tree and output the end result
    println!("Handling /qux:");
    println!("Setting /qux/alpha");
    ros.param("/qux/alpha").unwrap().set(&"meow").unwrap();
    println!("Setting /qux/beta");
    ros.param("/qux/beta").unwrap().set(&44).unwrap();
    println!("Setting /qux/gamma/x");
    ros.param("/qux/gamma/x").unwrap().set(&3.0).unwrap();
    println!("Setting /qux/gamma/y");
    ros.param("/qux/gamma/y").unwrap().set(&2).unwrap();
    println!("Setting /qux/gamma/z");
    ros.param("/qux/gamma/z").unwrap().set(&true).unwrap();
    println!("Setting /qux/delta");
    ros.param("/qux/delta").unwrap().set(&[1, 2, 3]).unwrap();
    ros.param("/qux/epsilon")
        .unwrap()
        .set(&TestStruct {
            foo: "x".into(),
            bar: 42,
            baz: false,
        })
        .unwrap();
    println!(
        "Get raw: {:?}",
        ros.param("/qux").unwrap().get_raw().unwrap()
    );
}
