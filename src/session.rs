use std::{collections::HashMap, net::SocketAddr, sync::Arc, sync::Mutex};

use anyhow::bail;
use tokio::{io, net::TcpStream, sync::mpsc};

/// Shorthand for the transmit half of the message channel.
type Tx = mpsc::UnboundedSender<String>;

/// Shorthand for the receive half of the message channel.
type Rx = mpsc::UnboundedReceiver<String>;

use super::room::*;

#[derive(Debug)]
pub struct User {
    pub user_id: String,
    pub sess_key: SocketAddr,
    pub channel_index: i32,
    pub tx: Tx,
}

impl User {
    pub fn new(id: String, s: SocketAddr, ch: i32, t: Tx) -> Self {
        User {
            user_id: id,
            sess_key: s,
            channel_index: ch,
            tx: t,
        }
    }
}
pub struct Shared {
    pub peers: HashMap<SocketAddr, User>,
    pub channels: HashMap<i32, Room>,
    pub room_num: i32,
    pub last_room_no: i32,
}

impl Shared {
    /// Create a new, empty, instance of `Shared`.
    pub fn new() -> Self {
        Shared {
            peers: HashMap::new(),
            channels: HashMap::new(),
            room_num: 1,
            last_room_no: 3,
        }
    }

    pub fn add_channel(&mut self, password: &str) -> anyhow::Result<i32> {
        let mut ret = 0;
        for n in self.last_room_no + 1..999999 {
            let n2 = n;
            let t = self.channels.get(&n2);
            if let None = t {
                self.last_room_no = n as i32;
                let r = Room::new(n2, password);
                self.channels.insert(n2.clone(), r);
                ret = n2;
                break;
            }
        }
        println!("all channels");
        for i in &self.channels {
            println!("{} -> {:?}", i.0, i.1);
        }
        Ok(ret)
    }
    pub fn add_user(&mut self, user: User) -> anyhow::Result<()> {
        println!("add_user ------------------------- {:?}", self.peers);
        let sk = user.sess_key;
        if let Some(s) = self.peers.insert(sk, user) {
            bail!("can't add key: {}, exist: {:?}", sk, s);
        }
        Ok(())
    }

    pub fn add_user_to_channel(&mut self, ch_index: i32, user: SocketAddr) {
        if let Some(s) = self.peers.get_mut(&user) {
            s.channel_index = ch_index;
        }
        if let Some(s) = self.channels.get_mut(&ch_index) {
            s.users.insert(user);
        }
    }

    // user 를 삭제함.
    pub fn delete_user(&mut self, user: SocketAddr) {
        println!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        for i in self.channels.iter_mut() {
            // i.1.users.iter().enumerate();
            let t = &mut i.1.users;
            t.remove(&user);
        }
        println!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbb222222222");
        self.peers.remove(&user);
    }

    pub fn display(&mut self) {
        for i in &self.channels {
            println!("room {} -> {:?}", i.0, i.1)
        }
        for i in &self.peers {
            println!("peer {} -> {:?}", i.0, i.1)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use tokio::sync::mpsc;

    use crate::chat::ChatPacket;

    use super::{Shared, User};

    #[test]
    fn it_works() -> anyhow::Result<()> {
        {
            let mut send_data1 = [0u8; 12 + 3];
            let mut send_data2 = [0u8; 1025];
            send_data1.fill(0);
            send_data2.fill(0);

            let mut send_packet = ChatPacket::new(4, &format!("{}", 2));
            let send_packet = send_packet.make_bytes();
            send_data1[..send_packet.len()].copy_from_slice(&send_packet);
            // send_data1.copy_from_slice(&send_packet);
            println!("{:?}", send_data1);
            let hello = &b"hello world"[..];
            send_data2[..hello.len()].copy_from_slice(hello);
            println!("{:?}", send_data2);
            let mut v = Vec::new();
            v.append(&mut send_data1.to_vec());
            v.append(&mut send_data2.to_vec());
            println!("v {:?}", v);
        }
        return Ok(());
        let mut t = Shared::new();
        let (tx, rx) = mpsc::unbounded_channel();
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let user = User::new("String".to_string(), socket, 100, tx.clone());
        let ch_index = t.add_channel("abcd")?;
        let sk = user.sess_key.clone();
        t.add_user(user);
        t.add_user_to_channel(ch_index, sk);
        if let Some(s) = t.channels.get(&ch_index) {}
        println!("chindex: {}", ch_index);
        let ch_index = t.add_channel("dfge")?;
        println!("chindex: {}", ch_index);
        {
            let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(126, 0, 0, 1)), 8080);
            let user = User::new("String".to_string(), socket, 100, tx.clone());
            let ch_index = t.add_channel("gggg")?;
            let sk = user.sess_key.clone();
            t.add_user(user);
            t.add_user_to_channel(ch_index, sk);
            if let Some(s) = t.channels.get(&ch_index) {}
        }
        println!("------------------------------------------");
        t.display();
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(126, 0, 0, 1)), 8080);
        t.delete_user(socket);
        println!("------------------------------------------");
        t.display();
        let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        t.delete_user(socket);
        println!("------------------------------------------");
        t.display();
        Ok(())
    }
}
