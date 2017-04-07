#[derive(Serialize,Deserialize,Debug)]
pub struct Time {
    pub sec: i32,
    pub nsec: i32,
}

#[derive(Serialize,Deserialize,Debug)]
pub struct Duration {
    pub sec: i32,
    pub nsec: i32,
}
