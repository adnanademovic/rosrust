# rosrust

[![MIT Licensed](https://img.shields.io/crates/l/rosrust.svg?maxAge=3600)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/rosrust.svg?maxAge=3600)](https://crates.io/crates/rosrust)
[![Build Status](https://travis-ci.org/adnanademovic/rosrust.svg?branch=master)](https://travis-ci.org/adnanademovic/rosrust)

**rosrust** is a pure Rust implementation of a [ROS](http://www.ros.org/) client library.

## Usage

The crate heavily uses `serde` for message generation macros, so adding it to the project is advised.
The following set of dependencies is needed for the full set of features:

```toml
[dependencies]
rosrust = "0.6.4"
rosrust_codegen = "0.6.4"
serde = "1.0.25"
serde_derive = "1.0.25"

[build-dependencies]
rosrust_codegen = "0.6.4"
```

The build dependency is used for message generation.

## Implementation

**rosrust** is almost there with implementing all features of a ROS Client Library by fulfilling [ROS requirements for implementing client libraries](http://wiki.ros.org/Implementing%20Client%20Libraries) which are given in a more detailed form in the [technical overview](http://wiki.ros.org/ROS/Technical%20Overview).

Mostly the missing features are related to extra tooling, like `sensor_msgs/PointCloud2` and `sensor_msgs/Image` encoding/decoding, TF tree handling, and other things that are external libraries in `roscpp` and `rospy` as well.

The API is very close to the desired final look.

Integration with [catkin](http://www.ros.org/wiki/catkin) will be handled once a satisfying set of features has been implemented.

## Examples

The API is close to reaching its final form.

There are multiple examples in the [examples folder](https://github.com/adnanademovic/rosrust/tree/master/examples). The publisher/subscriber and service/client examples are designed to closely immitate the `roscpp` tutorial.

## Features

### Message Generation

Message generation is done at build time. If you have ROS installed and sourced in your shell session, you will not need to do any extra setup for this to work.

To generate messages, create a `build.rs` script in the same folder as your `Cargo.toml` file with the following content:

```rust
#[macro_use]
extern crate rosrust_codegen;

// If you wanted
// * messages: std_msgs/String, sensor_msgs/Imu
// * services: roscpp_tutorials/TwoInts
// * and all the message types used by them, like geometry_msgs/Vector3
rosmsg_main!("std_msgs/String", "sensor_msgs/Imu", "roscpp_tutorials/TwoInts");
```

In your main file all you need to add at the top is:

```rust
#[macro_use]
extern crate rosrust;
#[macro_use]
extern crate rosrust_codegen;

rosmsg_include!();
```

This will include all the generated structures, and add them to the `msg` namespace. Thus, to create a new `sensor_msgs/Imu`, you call `msg::sensor_msgs::Imu::default()`. All fields are always public, so you can initialize structures as literals.

All of the structures implement debug writing, so you can easily inspect their contents.

### Publishing to Topic

If we wanted to publish a defined message (let's use `std_msgs/String`) to topic `chatter` ten times a second, we can do it in the following way.

```rust
#[macro_use]
extern crate rosrust;
#[macro_use]
extern crate rosrust_codegen;

rosmsg_include!();

fn main() {
    // Initialize node
    rosrust::init("talker");

    // Create publisher
    let mut chatter_pub = rosrust::publish("chatter").unwrap();

    let mut count = 0;

    // Create object that maintains 10Hz between sleep requests
    let mut rate = rosrust::rate(10.0);

    // Breaks when a shutdown signal is sent
    while rosrust::is_ok() {
        // Create string message
        let mut msg = msg::std_msgs::String::default();
        msg.data = format!("hello world {}", count);

        // Send string message to topic via publisher
        chatter_pub.send(msg).unwrap();

        // Sleep to maintain 10Hz rate
        rate.sleep();

        count += 1;
    }
}
```

### Subscribing to Topic

If we wanted to subscribe to an `std_msgs/UInt64` topic `some_topic`, we just declare a callback. An alternative extra interface with iterators is being considered, but for now this is the only option.

The constructor creates an object, which represents the subscriber lifetime.
Upon the destruction of this object, the topic is unsubscribed as well.

```rust
#[macro_use]
extern crate rosrust;
#[macro_use]
extern crate rosrust_codegen;

rosmsg_include!();

fn main() {
    // Initialize node
    rosrust::init("listener");

    // Create subscriber
    // The subscriber is stopped when the returned object is destroyed
    let _subscriber_raii = rosrust::subscribe("chatter", |v: msg::std_msgs::UInt64| {
        // Callback for handling received messages
        ros_info!("Received: {}", v.data);
    }).unwrap();

    // Block the thread until a shutdown signal is received
    rosrust::spin();
}
```

### Creating a Service

Creating a service is the easiest out of all the options. Just define a callback for each request. Let's use the `roscpp_tutorials/AddTwoInts` service on the topic `/add_two_ints`.

```rust
#[macro_use]
extern crate rosrust;
#[macro_use]
extern crate rosrust_codegen;

rosmsg_include!();

fn main() {
    // Initialize node
    rosrust::init("add_two_ints_server");

    // Create service
    // The service is stopped when the returned object is destroyed
    let _service_raii =
        rosrust::service::<msg::roscpp_tutorials::TwoInts, _>("add_two_ints", move |req| {
            // Callback for handling requests
            let sum = req.a + req.b;

            // Log each request
            ros_info!("{} + {} = {}", req.a, req.b, sum);

            Ok(msg::roscpp_tutorials::TwoIntsRes { sum })
        }).unwrap();

    // Block the thread until a shutdown signal is received
    rosrust::spin();
}

```

### Creating a Client

Clients can handle requests synchronously and asynchronously. The sync method behaves like a function, while the async approach is via reading data afterwards. The async consumes the passed parameter, since we're passing the parameter between threads. It's more common for users to pass and drop a parameter, so this being the default prevents needless cloning.

Let's call requests from the `AddTwoInts` service on the topic `/add_two_ints`.
The numbers shall be provided as command line arguments.

```rust
#[macro_use]
extern crate rosrust;
#[macro_use]
extern crate rosrust_codegen;

use std::{env, time};

fn main() {
    env_logger::init();

    // Fetch args that are not meant for rosrust
    let args: Vec<_> = rosrust::args();

    if args.len() != 3 {
        eprintln!("usage: client X Y");
        return;
    }

    let a = args[1].parse::<i64>().unwrap();
    let b = args[2].parse::<i64>().unwrap();

    // Initialize node
    rosrust::init("add_two_ints_client");

    // Wait ten seconds for the service to appear
    rosrust::wait_for_service("add_two_ints", Some(time::Duration::from_secs(10))).unwrap();

    // Create client for the service
    let client = rosrust::client::<msg::roscpp_tutorials::TwoInts>("add_two_ints").unwrap();

    // Synchronous call that blocks the thread until a response is received
    ros_info!(
        "{} + {} = {}",
        a,
        b,
        client
            .req(&msg::roscpp_tutorials::TwoIntsReq { a, b })
            .unwrap()
            .unwrap()
            .sum
    );

    // Asynchronous call that can be resolved later on
    let retval = client.req_async(msg::roscpp_tutorials::TwoIntsReq { a, b });
    ros_info!("{} + {} = {}", a, b, retval.read().unwrap().unwrap().sum);
}

```

### Parameters

There are a lot of methods provided, so we'll just give a taste of all of them here. Get requests return results, so you can use `unwrap_or` to handle defaults.

```rust
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
    ros_info!("Get with default: {:?}", param.get::<u64>().unwrap_or(44u64));
    ros_info!("Exists? {}", param.exists().unwrap());
}
```

### Logging

Logging is provided through macros `log_debug!()`, `log_info!()`, `log_warn!()`, `log_error!()`, `log_fatal!()`.

Setting verbosity levels and throttled logging have yet to come!

### Command Line Remaps

Similar to `rospy` and `roscpp`, you can use the command line to remap topics and private parameters. Private parameters should be provided in a YAML format.

For more information, look at the [official wiki](http://wiki.ros.org/Remapping%20Arguments), since the attempt was to 100% immitate this interface.

You can get a vector of the leftover command line argument strings with `rosrust::args()`, allowing easy argument parsing. This includes the first argument, the application name.

## License

**rosrust** is distributed under the MIT license.
