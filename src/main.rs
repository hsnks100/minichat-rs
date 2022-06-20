mod chat;
mod session;
mod processor;
// use chat;
use anyhow::bail;
use bytes::Bytes;
use processor::process;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
