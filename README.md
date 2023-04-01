# rosrust

[![MIT Licensed](https://img.shields.io/crates/l/rosrust.svg?maxAge=3600)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/rosrust.svg?maxAge=3600)](https://crates.io/crates/rosrust)
[![Build Status](https://travis-ci.org/adnanademovic/rosrust.svg?branch=master)](https://travis-ci.org/adnanademovic/rosrust)

**rosrust** is a pure Rust implementation of a [ROS](http://www.ros.org/) client library.

## Usage

For all the key features, it is enough to depend on the crate itself. It's highly recommended to depend on `rosrust_msg` as well, as it provides bindings for message generation.

The following dependencies are recommended to use the crate:

```toml
[dependencies]
rosrust = "0.9"
rosrust_msg = "0.1"
```

If using Rust 2015 edition, just depend on the library with macro usage, using:

```rust
#[macro_use]
extern crate rosrust;
```

Examples are written using Rust 2018, as it's the expected edition to use now.

## Implementation

**rosrust** is almost there with implementing all features of a ROS Client Library by fulfilling [ROS requirements for implementing client libraries](http://wiki.ros.org/Implementing%20Client%20Libraries) which are given in a more detailed form in the [technical overview](http://wiki.ros.org/ROS/Technical%20Overview).

Mostly the missing features are related to extra tooling, like `sensor_msgs/PointCloud2` and `sensor_msgs/Image` encoding/decoding, TF tree handling, and other things that are external libraries in `roscpp` and `rospy` as well.

The API is very close to the desired final look.

Integration with [catkin](http://www.ros.org/wiki/catkin) will be handled once a satisfying set of features has been implemented.

## Examples

The API is close to reaching its final form.

There are multiple examples in the [examples folder](https://github.com/adnanademovic/rosrust/tree/master/examples/examples). The publisher/subscriber and service/client examples are designed to closely imitate the `roscpp` tutorial.

## Features

### Message Generation

Message generation can be done automatically by depending on `rosrust_msg`, or manually using `rosrust::rosmsg_include`.

The preferred way is automatic, as it allows interop between dependencies that use messages and your crate.

If you do not have ROS installed, then the message generation utilizes the `ROSRUST_MSG_PATH` environment variable, which is a colon separated list of directories to search.
These directories should have the structure `<ROSRUST_MSG_PATH>/<anything>/<package>/msg/<message>` or `<ROSRUST_MSG_PATH>/<anything>/<package>/srv/<service>`. 

#### Automatic

For automatic message generation just depend on `rosrust_msg`, with the version specified at the top of this document.

After that you'll be able to generate a `sensor_msgs/Imu` message object by using `rosrust_msg::sensor_msgs::Imu::default()`. All fields are always public, so you can initialize structures as literals.

#### Manual

Message generation is done at build time. If you have ROS installed and sourced in your shell session, you will not need to do any extra setup for this to work.

To generate messages, create a module for messages. Using something like a `msg.rs` file in your project root results in importing similar to `roscpp` and `rospy`. The file only needs one line:

```rust
// If you wanted
// * messages: std_msgs/String, sensor_msgs/Imu
// * services: roscpp_tutorials/TwoInts
// * and all the message types used by them, like geometry_msgs/Vector3
rosrust::rosmsg_include!(std_msgs/String,sensor_msgs/Imu,roscpp_tutorials/TwoInts);
```

Just add this file to your project and you're done.

If you have put this in a `src/msg.rs` file, this will include all the generated structures, and add them to the `msg` namespace. Thus, to create a new `sensor_msgs/Imu`, you call `msg::sensor_msgs::Imu::default()`. All fields are always public, so you can initialize structures as literals.

### Publishing to Topic

If we wanted to publish a defined message (let's use `std_msgs/String`) to topic `chatter` ten times a second, we can do it in the following way.

```rust
fn main() {
    // Initialize node
    rosrust::init("talker");

    // Create publisher
    let chatter_pub = rosrust::publish("chatter", 100).unwrap();

    let mut count = 0;

    // Create object that maintains 10Hz between sleep requests
    let rate = rosrust::rate(10.0);

    // Breaks when a shutdown signal is sent
    while rosrust::is_ok() {
        // Create string message
        let mut msg = rosrust_msg::std_msgs::String::default();
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
fn main() {
    // Initialize node
    rosrust::init("listener");

    // Create subscriber
    // The subscriber is stopped when the returned object is destroyed
    let _subscriber_raii = rosrust::subscribe("chatter", 100, |v: rosrust_msg::std_msgs::UInt64| {
        // Callback for handling received messages
        rosrust::ros_info!("Received: {}", v.data);
    }).unwrap();

    // Block the thread until a shutdown signal is received
    rosrust::spin();
}
```

### Creating a Service

Creating a service is the easiest out of all the options. Just define a callback for each request. Let's use the `roscpp_tutorials/AddTwoInts` service on the topic `/add_two_ints`.

```rust
fn main() {
    // Initialize node
    rosrust::init("add_two_ints_server");

    // Create service
    // The service is stopped when the returned object is destroyed
    let _service_raii =
        rosrust::service::<rosrust_msg::roscpp_tutorials::TwoInts, _>("add_two_ints", move |req| {
            // Callback for handling requests
            let sum = req.a + req.b;

            // Log each request
            rosrust::ros_info!("{} + {} = {}", req.a, req.b, sum);

            Ok(rosrust_msg::roscpp_tutorials::TwoIntsRes { sum })
        }).unwrap();

    // Block the thread until a shutdown signal is received
    rosrust::spin();
}

```

### Creating a Client

Clients can handle requests synchronously and asynchronously. The sync method behaves like a function, while the async approach is via reading data afterwards. The async consumes the passed parameter, since we're passing the parameter between threads. It's more common for users to pass and drop a parameter, so this being the default prevents needless cloning.

Let's call requests from the `AddTwoInts` service on the topic `/add_two_ints`.
The numbers shall be provided as command line arguments.

We're also depending on `env_logger` here to log `ros_info` messages to the standard output.

```rust
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
    let client = rosrust::client::<rosrust_msg::roscpp_tutorials::TwoInts>("add_two_ints").unwrap();

    // Synchronous call that blocks the thread until a response is received
    ros_info!(
        "{} + {} = {}",
        a,
        b,
        client
            .req(&rosrust_msg::roscpp_tutorials::TwoIntsReq { a, b })
            .unwrap()
            .unwrap()
            .sum
    );

    // Asynchronous call that can be resolved later on
    let retval = client.req_async(rosrust_msg::roscpp_tutorials::TwoIntsReq { a, b });
    rosrust::ros_info!("{} + {} = {}", a, b, retval.read().unwrap().unwrap().sum);
}

```

### Parameters

There are a lot of methods provided, so we'll just give a taste of all of them here. Get requests return results, so you can use `unwrap_or` to handle defaults.

We're also depending on `env_logger` here to log `ros_info` messages to the standard output.

```rust
fn main() {
    env_logger::init();

    // Initialize node
    rosrust::init("param_test");

    // Create parameter, go through all methods, and delete it
    let param = rosrust::param("~foo").unwrap();
    rosrust::ros_info!("Handling ~foo:");
    rosrust::ros_info!("Exists? {:?}", param.exists()); // false
    param.set(&42u64).unwrap();
    rosrust::ros_info!("Get: {:?}", param.get::<u64>().unwrap());
    rosrust::ros_info!("Get raw: {:?}", param.get_raw().unwrap());
    rosrust::ros_info!("Search: {:?}", param.search().unwrap());
    rosrust::ros_info!("Exists? {}", param.exists().unwrap());
    param.delete().unwrap();
    rosrust::ros_info!("Get {:?}", param.get::<u64>().unwrap_err());
    rosrust::ros_info!("Get with default: {:?}", param.get::<u64>().unwrap_or(44u64));
    rosrust::ros_info!("Exists? {}", param.exists().unwrap());
}
```

### Logging

Logging is provided through macros `ros_debug!()`, `ros_info!()`, `ros_warn!()`, `ros_error!()`, `ros_fatal!()`.

Throttled logging options ara available too.

### Command Line Remaps

Similar to `rospy` and `roscpp`, you can use the command line to remap topics and private parameters. Private parameters should be provided in a YAML format.

For more information, look at the [official wiki](http://wiki.ros.org/Remapping%20Arguments), since the attempt was to 100% immitate this interface.

You can get a vector of the leftover command line argument strings with `rosrust::args()`, allowing easy argument parsing. This includes the first argument, the application name.

## License

**rosrust** is distributed under the MIT license.
