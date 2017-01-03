# rosrust

[![MIT Licensed](https://img.shields.io/crates/l/rosrust.svg?maxAge=3600)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/rosrust.svg?maxAge=3600)](https://crates.io/crates/rosrust)
[![Build Status](https://travis-ci.org/adnanademovic/rosrust.svg?branch=master)](https://travis-ci.org/adnanademovic/rosrust)

**rosrust** is a pure Rust implementation of a [ROS](http://www.ros.org/) client library.

## Implementation

**rosrust** is under development to implement all features of a ROS Client Library by fulfilling [ROS requirements for implementing client libraries](http://wiki.ros.org/Implementing%20Client%20Libraries) which are given in a more detailed form in the [technical overview](http://wiki.ros.org/ROS/Technical%20Overview).

Currently implemented features:
* ROS Master API, Parameter Server API and most of ROS Slave API (excluding 2 ambiguous methods)
* Interface for handling parameters
* Publisher, Subscriber, Client, Service fully working (until packet snooping for another undocumented feature is needed)
* Simple bindings for messages, but ROS integration for generating them has yet to be done, and will be part of the catkin integration
* Manual remapping is provided, and CLI parameter remapping is coming!

There are still quite some features to be implemented for this to become a stable API (for the library user's interface to stop changing). The most important ones are listed as Issues by the repository owner.

Integration with [catkin](http://www.ros.org/wiki/catkin) will be handled once a satisfying set of features has been implemented.

## Examples

The API is far from being stable, but this is the current desired functionality.

Most of the examples are followed by an infinite loop. ROS shutdown signal checking is coming!

### Messages

Currently we don't have any support for message generation by ROS itself, because a 100% decision of how we'll integrate generated files hasn't been made. There is a prototype in the works, but in the meantime messages have to be manually generated. Looking from the bright side - at least you'll know what constitutes a ROS message.

Let's demonstrate the [`std_msgs/UInt64.msg`](https://github.com/ros/std_msgs/blob/groovy-devel/msg/UInt64.msg) message.

```rust
#[derive(Debug,RustcEncodable,RustcDecodable)]
struct UInt64 {
    data: u64,
}

impl rosrust::Message for UInt64 {
    fn msg_definition() -> String {
        String::from("uint64 data\n")
    }

    fn md5sum() -> String {
        String::from("1b2a79973e8bf53d7b53acb71299cb57")
    }

    fn msg_type() -> String {
        String::from("std_msgs/UInt64")
    }
}
```

Services are not too different, so let's show the [`rospy_tutorials/AddTwoInts.srv`](https://github.com/ros/ros_tutorials/blob/kinetic-devel/rospy_tutorials/srv/AddTwoInts.srv) service, from [the `rospy` Service/Client tutorial](http://wiki.ros.org/rospy_tutorials/Tutorials/WritingServiceClient).

```rust
#[derive(Debug,RustcEncodable,RustcDecodable)]
struct AddTwoIntsReq {
    a: i64,
    b: i64,
}

#[derive(Debug,RustcEncodable,RustcDecodable)]
struct AddTwoIntsRes {
    sum: i64,
}

#[derive(Debug,RustcEncodable,RustcDecodable)]
struct AddTwoInts {}

impl rosrust::Service for AddTwoInts {
    type Request = AddTwoIntsReq;
    type Response = AddTwoIntsRes;
}

impl rosrust::Message for AddTwoInts {
    fn msg_definition() -> String {
        String::from("")
    }

    fn md5sum() -> String {
        String::from("6a2e34150c00229791cc89ff309fff21")
    }

    fn msg_type() -> String {
        String::from("rospy_tutorials/AddTwoInts")
    }
}
```

This will all be unimportant information once message generation is done, but it's useful to know that ROS Messages and Services will just be normal structures. It's important to note that all of this depends on `rustc-serialize`.

### Publishing to topic

If we wanted to publish a defined message (let's use the previously described `UInt64`) to topic `some_topic` approximately every second, we can do it in the following way.

```rust
extern crate rosrust;
use rosrust::Ros;
use std::{thread, time};

fn main() {
    let mut ros = Ros::new("node_name").unwrap();
    let mut publisher = ros.publish::<Uint64>("some_topic").unwrap();
    loop {
        thread::sleep(time::Duration::from_secs(1);
        publisher.send(UInt64 { data: 42 }).unwrap();
    }
}
```

### Subscribing to topic

If we wanted to subscribe to an `std_msgs/UInt64` topic `some_topic`, we just declare a callback. An alternative extra interface with iterators is being considered, but for now this is the only option.

```rust
extern crate rosrust;
use rosrust::Ros;
use std::{thread, time};

fn main() {
    let mut ros = Ros::new("node_name").unwrap();
    ros.subscribe("some_topic", |v: UInt64| println!("{}", v.data)).unwrap();
    loop {
        thread::sleep(time::Duration::from_secs(100);
    }
}
```

### Creating a service

Creating a service is the easiest out of all the options. Just define a callback for each request. Let's make our own `AddTwoInts` service on the topic `/add_two_ints`.

```rust
extern crate rosrust;
use rosrust::Ros;
use std::{thread, time};

fn main() {
    let mut ros = Ros::new("node_name").unwrap();
    ros.service::<AddTwoInts>("/add_two_ints",
        |req| Ok(AddTwoIntsRes { sum: req.a + req.b }) ).unwrap();
    loop {
        thread::sleep(time::Duration::from_secs(100);
    }
}
```

### Creating a client

Clients can handle requests synchronously and asynchronously. The sync method behaves like a function, while the async approach is via a callback. The async consumes the passed parameter, since we're passing the parameter between threads, and it's more common for users to pass and drop a parameter, so this being the default prevents needless cloning. Let's call requests from the `AddTwoInts` service on the topic `/add_two_ints`.

```rust
extern crate rosrust;
use rosrust::Ros;
use std::{thread, time};

fn main() {
    let mut ros = Ros::new("node_name").unwrap();
    let client = ros.client::<AddTwoInts>("/add_two_ints").unwrap();
    loop {
        // Sync approach
        println!("5 + 7 = {}",
            client.req(&AddTwoIntsReq { a: 5, b: 7 }).unwrap().unwrap().sum);
        // Async approach
        client.req_callback(AddTwoIntsReq { a: 5, b: 7 },
            |result| println!("12 + 4 = {}", result.unwrap().unwrap().sum));
        thread::sleep(time::Duration::from_secs(1);
    }
}
```

### Parameters

```rust
extern crate rosrust;
use rosrust::Ros;

fn main() {
    let ros = Ros::new("node_name").unwrap();
    let param = ros.param("~cow").unwrap(); // access /node_name/cow parameter
    println!("Exists? {:?}", param.exists()); // false
    param.set(&UInt64 { data: 42 });
    println!("Get {}", param.get::<UInt64>().unwrap().data); // 42
    // XmlRpcValue representing any parameter
    println!("Get raw {:?}", param.get_raw().unwrap());
    println!("Search {:?}", param.search().unwrap()); // /node_name/cow
    println!("Exists? {}", param.exists().unwrap()); // true
    param.delete().unwrap()
    println!("Get {:?}", param.get::<UInt64>().unwrap_err()); // fails to find
    println!("Exists? {}", param.exists()); //false
}
```

## License

**rosrust** is distributed under the MIT license.
