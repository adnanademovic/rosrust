#![deny(warnings)]

mod msg;

fn main() {
    println!("{}", msg::visualization_msgs::ImageMarker::default().id);
}
