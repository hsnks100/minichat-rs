use std::{collections::HashSet, net::SocketAddr};

#[derive(Debug)]
pub struct Room {
    pub room_index: i32,
    pub password: String,
    pub users: HashSet<SocketAddr>,
}

impl Room {
    pub fn new(ch: i32, pw: &str) -> Self {
        Room {
            room_index: ch,
            password: pw.to_string(),
            users: HashSet::new(),
        }
    }
}
