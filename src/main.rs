mod chat;
mod session;
// use chat;
use std::{
    error::Error,
    net::SocketAddr,
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Mutex},
};
use bytes::{Bytes};
async fn process(
    state: Arc<Mutex<session::Shared>>,
    mut stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut buf = [0; 1024];

    let mut peer = session::Peer::new(state.clone(), addr).await?;

    let mut parser = chat::ChatParser::new();

    tokio::spawn(async move {
        loop {
            println!("temp spawn");
            let recv = peer.rx.recv().await;
            let msg = match recv {
                Some(v) => {
                    println!("recv: {:?}", v);
                }
                None => {}
            };
        }
    });
    loop {
        let n = match stream.read(&mut buf).await {
            // socket closed
            Ok(n) if n == 0 => {
                return Ok(());
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
                return Err(Box::new(e));
            }
        };
        let bb = Bytes::copy_from_slice(&buf[0..n]);
        parser.put(bb);
        println!("******** 0step *******, recv! {} then parse", n);
        loop {
            let res = parser.parse();
            match res {
                Ok(v) => {
                    println!("parsed: {:?}", v);
                }, 
                Err(e) => {
                    break;
                }
            }
        }
        println!("******** 1step *******, broadcast!!?");
        let mut state = &*state.lock().await;
        let tx = state.peers.get(&addr);
        match tx {
            Some(some) => {
                some.send("!!!!!!!!!!!!!!".to_string());
            }, 
            None => {
                ()
            }
        }
        if let Err(e) = stream.write_all(&buf[0..n]).await {
            eprintln!("failed to write to socket; err = {:?}", e);
            return Err(Box::new(e));
        }
    }
    // return Ok(());
}
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
