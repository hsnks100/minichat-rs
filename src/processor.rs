use super::chat::*;
use super::session::User;
use crate::{chat::*, session};
use anyhow::bail;
use bytes::Bytes;
use std::ops::DerefMut;
use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc, sync::Mutex};
use tokio::sync::mpsc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc::UnboundedSender,
};

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

    let (tx, mut rx) = mpsc::unbounded_channel();
    {
        let state2 = state.clone();
        let mut shared = state2.lock().unwrap();
        shared.add_user(User::new("qwe".to_string(), addr, 0, tx))?;
    }
    let mut parser = ChatParser::new();
    let (mut s_reader, mut s_writer) = stream.into_split();
    let mut functor: HashMap<
        i32,
        Box<
            dyn Fn(Arc<Mutex<session::Shared>>, SocketAddr, ChatPacket) -> anyhow::Result<()>
                + Send,
        >,
    > = HashMap::new();
    functor.insert(CHAT_MAKE_CODE, Box::new(&make_room));
    functor.insert(CHAT_ENTER_CODE, Box::new(&enter_room));
    // functor.insert(CHAT_OUT_CODE, Box::new(&out_room));
    functor.insert(MSG_SEND_CODE, Box::new(&send_message));

    tokio::spawn(async move {
        loop {
            println!("SENDER START!!");
            let recv = rx.recv().await;
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
        println!("******** 0step *******, recv bytes: {} then parse", n);
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
    let password = match cp.body.find('\0') {
        Some(s) => &cp.body[0..s],
        None => return Ok(()),
    };
    let room_no = state.add_channel(password)?;
    state.add_user_to_channel(room_no, socketaddr);
    let user = match state.peers.get_mut(&socketaddr) {
        Some(u) => u,
        None => return Ok(()),
    };
    user.channel_index = room_no;
    let sender = user.tx.clone();
    let mut sendPacket = ChatPacket {
        service_code: 1,
        length: 1,
        body: room_no.to_string(),
    };
    let send_packet = sendPacket.make_bytes();
    sender.send(String::from_utf8_lossy(send_packet.as_slice()).to_string())?;
    Ok(())
}

fn send_message(
    state: Arc<Mutex<session::Shared>>,
    socketaddr: SocketAddr,
    cp: ChatPacket,
) -> anyhow::Result<()> {
    println!("send_message: {:?}", cp);
    Ok(())
}

fn enter_room(
    state: Arc<Mutex<session::Shared>>,
    socketaddr: SocketAddr,
    cp: ChatPacket,
) -> anyhow::Result<()> {
    println!("enter room: {:?}", cp);
    if let Some(s) = cp.body.find('\0') {
        let body = &cp.body[0..s];
        let bodies: Vec<&str> = body.split('*').collect();
        if bodies.len() < 2 {
            bail!("can't enter room")
        }
        let room_no = bodies[0].parse::<i32>()?;
        let password = bodies[1];
        println!("room_no: {}, password: {}", room_no, password);
        let mut state = state.lock().unwrap();
        state.add_user_to_channel(room_no, socketaddr);
        let user = match state.peers.get_mut(&socketaddr) {
            Some(u) => u,
            None => return Ok(()),
        };
        let sender = user.tx.clone();
        let mut send_packet = ChatPacket {
            service_code: 2, // 아마도 서버가 클라한테 주는 서비스코드인 듯..
            length: 1,
            body: "3".to_string(),
        };
        let send_packet = send_packet.make_bytes();
        sender.send(String::from_utf8_lossy(send_packet.as_slice()).to_string())?;
        state.display();
    }
    Ok(())
}
