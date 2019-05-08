mod msg;

fn main() {
    let message = msg::visualization_msgs::ImageMarker::default();
    println!("{:?} {:?}", message.id, message.type_);
}
