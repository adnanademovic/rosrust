# rosrust

[![MIT Licensed](https://img.shields.io/crates/l/rosrust.svg?maxAge=3600)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/rosrust.svg?maxAge=3600)](https://crates.io/crates/rosrust)

**rosrust** is a pure Rust implementation of a [ROS](http://www.ros.org/) client library.

## Implementation

**rosrust** is under development to implement all features of a ROS Client Library by fulfilling [ROS requirements for implementing client libraries](http://wiki.ros.org/Implementing%20Client%20Libraries) which are given in a more detailed form in the [technical overview](http://wiki.ros.org/ROS/Technical%20Overview).

The implementation will start by supporting minimal features. The optional feature of object representation of message types will be a priority, due to the nature of Rust.

Currently implemented features:
* ROS Master API
* Basic ROS Parameter Server API
* Stubs and several methods of ROS Slave API
* Interface for getting/setting parameters with known type
* TCPROS publisher/subscriber basics

The package will be available on [cargo](https://crates.io/) once any features have been added.

Integration with [catkin](http://www.ros.org/wiki/catkin) will be handled once a satisfying set of features has been implemented.

## License

**rosrust** is distributed under the MIT license.
