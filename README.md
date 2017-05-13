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

There is a demo project at [OTL/rosrust_tutorial](https://github.com/OTL/rosrust_tutorial), showing everything that needs to be done (including a tiny tiny `build.rs` script) to get `rosrust` running.

Most of the examples are followed by an infinite loop. ROS shutdown signal checking is coming!

### Messages

Message generation is done at build time. In the `build.rs` script, all you need to add is:

```rust
#[macro_use]
extern crate rosrust;
// If you wanted std_msgs/String, sensor_msgs/Imu
// and all the message types used by them, like geometry_msgs/Vector3
rosmsg_main!("std_msgs/String", "sensor_msgs/Imu");
```

You need to depend on `rosrust`, `serde`, and `serde_derive`, and have `rosrust` as a build dependency. After that, add this to your main file:

```rust
#[macro_use]
extern crate rosrust;
rosmsg_include!();
```

This will include all the generated structures, and at them to the `msg` namespace. Thus, to create a new `sensor_msgs/Imu`, you call `msg::sensor_msgs::Imu::new()`.

All of the structures implement debug writing, so you can easily inspect their contents.

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
        thread::sleep(time::Duration::from_secs(1));
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
        thread::sleep(time::Duration::from_secs(100));
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

Clients can handle requests synchronously and asynchronously. The sync method behaves like a function, while the async approach is via reading data afterwards. The async consumes the passed parameter, since we're passing the parameter between threads, and it's more common for users to pass and drop a parameter, so this being the default prevents needless cloning. Let's call requests from the `AddTwoInts` service on the topic `/add_two_ints`.

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
        let retval = client.req_async(AddTwoIntsReq { a: 5, b: 7 }),
        println!("12 + 4 = {}", retval.read().unwrap().unwrap().sum));
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
