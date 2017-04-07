#[derive(Serialize,Deserialize,Debug)]
pub struct Time {
    pub sec: i32,
    pub nsec: i32,
}

impl Time {
    pub fn new() -> Time {
        Time { sec: 0, nsec: 0 }
    }
}

#[derive(Serialize,Deserialize,Debug)]
pub struct Duration {
    pub sec: i32,
    pub nsec: i32,
}

impl Duration {
    pub fn new() -> Duration {
        Duration { sec: 0, nsec: 0 }
    }
}
