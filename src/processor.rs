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

const ROOM_MAKE_SUCCESS: i32 = 1;
const ROOM_MAKE_FAIL: i32 = 2;
const ROOM_ENTER_SUCCESS: i32 = 3;
const ROOM_ENTER_FAIL: i32 = 4;
const MSG_SEND_SUCCESS: i32 = 5;
const MSG_SEND_FAIL: i32 = 6;
const ROOM_EXIT: i32 = 8;
const ROOM_OUT: i32 = 9;

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
                None => {
                    println!("recv is none, writer is closing.");
                    return;
                }
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
        let n = s_reader.read(&mut buf).await;
        let n = match n {
            Ok(s) => s,
            Err(e) => {
                let mut state = state.lock().unwrap();
                state.delete_user(addr);
                println!("process is end");
                return Ok(());
            }
        };
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
    state.add_user_to_channel(room_no, socketaddr)?;
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

// socketaddr 방에 있는 모든 tx 에게 채팅 날려주면 댐.
fn send_message(
    state: Arc<Mutex<session::Shared>>,
    socketaddr: SocketAddr,
    cp: ChatPacket,
) -> anyhow::Result<()> {
    println!("send_message: {:?}", cp);
    let message = match cp.body.find('\0') {
        Some(s) => &cp.body[..s],
        None => return Ok(()),
    };
    let mut state = state.lock().unwrap();
    let user = match state.peers.get_mut(&socketaddr) {
        Some(u) => u,
        None => return Ok(()),
    };
    let room_no = user.channel_index;
    let channel = match state.channels.get(&room_no) {
        Some(s) => s,
        None => return Ok(()),
    };
    let mut cnt = 0;
    for i in &channel.users {
        cnt += 1;
    }
    println!("방 참여자 수: {}", cnt);
    for i in &channel.users {
        let targetUser = match state.peers.get(&i) {
            Some(s) => s,
            None => return Ok(()),
        };

        let mut send_data1 = [0u8; 12 + 3];
        let mut send_data2 = [0u8; 1025];
        send_data1.fill(0);
        send_data2.fill(0);

        let mut send_packet = ChatPacket::new(MSG_SEND_CODE, &format!("{}", 2));
        send_packet.length = 3 + 1025;
        let send_packet = send_packet.make_bytes();
        send_data1[..send_packet.len()].copy_from_slice(&send_packet);
        println!("{:?}", send_data1);
        send_data2[..message.len()].copy_from_slice(message.as_bytes());
        println!("{:?}", send_data2);
        let mut v = Vec::new();
        v.append(&mut send_data1.to_vec());
        v.append(&mut send_data2.to_vec());
        let sender = targetUser.tx.clone();

        sender.send(String::from_utf8_lossy(v.as_slice()).to_string())?;
        // 보내는 패킷의 구조
        // %04d&%06d&%d 먼저 담고 1024 + 1 + 12 + 3
        // [12] 위치에서 1025 개만큼 메시지를 쏨 (아마도 널 종료))]
        // MSG_SIZE
        // sender.send(String::from_utf8_lossy(send_packet.as_slice()).to_string())?;
        // targetUser.tx.send(message)
    }
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
        let user = match state.peers.get_mut(&socketaddr) {
            Some(u) => u,
            None => return Ok(()),
        };
        let sender = user.tx.clone();
        match state.add_user_to_channel(room_no, socketaddr) {
            Ok(s) => {}
            Err(e) => {
                let send_packet = ChatPacket::new(2, &ROOM_ENTER_FAIL.to_string()).make_bytes();
                sender.send(String::from_utf8_lossy(send_packet.as_slice()).to_string())?;
                return Err(e);
            }
        }
        let send_packet = ChatPacket::new(2, &ROOM_ENTER_SUCCESS.to_string()).make_bytes();
        sender.send(String::from_utf8_lossy(send_packet.as_slice()).to_string())?;
        state.display();
    }
    Ok(())
}
