mod chat;
mod processor;
mod room;
mod session;
// use chat;
use anyhow::bail;
use bytes::Bytes;
use processor::process;
use session::User;
use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc, sync::Mutex};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc::UnboundedSender,
};
// async fn enter_room(state:Arc<Mutex<session::Shared>>, cp: ChatPacket, sender: UnboundedSender<String>) {
//     println!("this is enter_room");
// }
// fn send_message(state:Arc<Mutex<session::Shared>>, cp: ChatPacket, sender: UnboundedSender<String>) {
//     println!("this is send_message");
// }
// fn out_room(state:Arc<Mutex<session::Shared>>, cp: ChatPacket, sender: UnboundedSender<String>) {
//     println!("this is out_room");
// }

struct Field {
    pub fff: i32,
}
struct Person {
    pub age: i32,
    pub name: String,
    pub props: HashMap<i32, i32>,
}

impl Person {
    pub fn func1(&mut self) {}
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    {
        let mut state = Person {
            age: 3,
            name: "ss".to_string(),
            props: HashMap::new(),
        };
        let mut prop = &mut state.props;
        prop.insert(2, 3);
        let mut prop2 = &mut state.props;
        prop2.insert(1, 3);
        // let u = prop.get(&3);
    }
    let state = Arc::new(Mutex::new(session::Shared::new()));
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        let (mut socket, addr) = listener.accept().await?;
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            if let Err(e) = process(state, socket, addr).await {
                println!("an error occurred; error = {:?}", e);
            }
        });
    }
}
