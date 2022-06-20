use crate::{chat::*, session};
use anyhow::bail;
use bytes::Bytes;
use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc, sync::Mutex};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc::UnboundedSender,
};
use std::ops::DerefMut;



const CHAT_MAKE_CODE: i32 = 1;
const CHAT_ENTER_CODE: i32 = 2;
const MSG_SEND_CODE: i32 = 3;
const CHAT_OUT_CODE: i32 = 4;
const KEEPALIVE_CODE: i32 = 5;
pub async fn process(
    state: Arc<Mutex<session::Shared>>,
    mut stream: TcpStream,
    addr: SocketAddr,
) -> anyhow::Result<()> {
    let mut buf = [0; 1024];
    let mut peer = session::Peer::new(state.clone(), addr).await?;
    let mut parser = ChatParser::new();
    let (mut s_reader, mut s_writer) = stream.into_split();
    let mut functor: HashMap<
        i32,
        Box<
            dyn Fn(
                    Arc<Mutex<session::Shared>>,
                    SocketAddr,
                    ChatPacket,
                ) -> anyhow::Result<()>
                + Send,
        >,
    > = HashMap::new();
    functor.insert(CHAT_MAKE_CODE, Box::new(&make_room));
    // functor.insert(CHAT_ENTER_CODE, Box::new(&enter_room));
    // functor.insert(CHAT_OUT_CODE, Box::new(&out_room));
    // functor.insert(MSG_SEND_CODE, Box::new(&send_message));
    tokio::spawn(async move {
        loop {
            println!("SENDER START!!");
            let recv = peer.rx.recv().await;
            let recv = match recv {
                Some(v) => v,
                None => "".to_string(),
            };
            if recv != "" {
                let r = s_writer.write(recv.as_bytes()).await;
                match r {
                    Ok(r) => {}
                    Err(e) => {
                        println!("write error {:#?}", e);
                        break;
                    }
                }
            }
        }
    });
    loop {
        // 실제로 받는 부분
        let n = s_reader.read(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }
        let bb = Bytes::copy_from_slice(&buf[0..n]);
        parser.put(bb);
        println!("******** 0step *******, recv! {} then parse", n);
        loop {
            let res = parser.parse();
            match res {
                Ok(v) => {
                    println!("parsed: {:?}", v);
                    let g = functor.get(&v.service_code);
                    let g = match g {
                        Some(s) => s,
                        None => {
                            println!("not support service code");
                            continue;
                        }
                    };
                    g(state.clone(), addr, v);
                }
                Err(e) => {
                    println!("parse error: {:#?}", e);
                    break;
                }
            }
        }
        println!("******** 1step *******, broadcast!!?");
        let lock = state.lock().unwrap();
        let state = lock;
        let tx = state.peers.get(&addr);
        let tx = match tx {
            Some(some) => some,
            None => {
                continue;
            }
        };
        // tx.send("messag!!!!!!!!!!!".to_string())?;
    }
    // return Ok(());
}

fn make_room(
    state: Arc<Mutex<session::Shared>>,
    socketaddr: SocketAddr,
    cp: ChatPacket,
) -> anyhow::Result<()> {
    let mut state = state.lock().unwrap();
    let sender = state.peers.get(&socketaddr);
    if let Some(sender) = sender {
        let password = cp.body;
        let sender = sender.clone();
        state.room_num += 1;
        println!("room num: {}", state.room_num);
        println!("this is make_room password: {}", password);

        let mut sendPacket = ChatPacket{service_code:1, length: 1, body: "5".to_string()};
        let sendPacket = sendPacket.make_bytes();
        sender.send(String::from_utf8_lossy(sendPacket.as_slice()).to_string());
    }
    Ok(())
}