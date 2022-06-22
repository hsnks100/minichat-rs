use std::{
    borrow::Borrow,
    io::{self, prelude::*},
};

use anyhow::{bail, Result};

use bytebuffer::ByteBuffer;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
#[derive(Debug)]
pub struct ChatPacket {
    pub service_code: i32,
    pub length: i32,
    pub body: String,
}

impl ChatPacket {
    pub fn make_bytes(&mut self) -> Vec<u8> {
        let t = format!("{:04}&{:06}&{}", self.service_code, self.length, self.body);
        return t.as_bytes().to_vec();
    }
    pub fn new(sc: i32, body: &str) -> Self {
        let mut ret = ChatPacket {
            service_code: sc,
            length: 0,
            body: body.to_string(),
        };
        ret.length = body.len() as i32;
        ret
    }
}

pub struct ChatParser {
    pub service_code: i32,
    pub length: i32,
    pub body: String,
    pub buffer: BytesMut,
}

impl ChatParser {
    pub fn new() -> Self {
        return ChatParser {
            service_code: 0,
            length: 0,
            body: "".to_string(),
            buffer: BytesMut::new(),
        };
    }
    pub fn put(&mut self, data: Bytes) {
        self.buffer.put(&data.to_vec()[..]);
    }
    pub fn parse(&mut self) -> anyhow::Result<ChatPacket> {
        // 0001&000004&abcd
        // 두번 째 & 까지 들어왔으면
        if self.buffer.len() >= 12 {
            let b = &self.buffer[..];
            // 길이 간보고
            let service_code = String::from_utf8(b[0..4].to_vec())?.parse::<i32>()?;
            let length = String::from_utf8(b[5..11].to_vec())?.parse::<i32>()?;
            println!("service code: {:?}", service_code);
            println!("length: {:?}", length);
            if (self.buffer.len()) as u32 >= 12 + length as u32 {
                let l = 12 + length;
                let mut reader = (&mut self.buffer).reader();
                let mut vec = vec![0; l as usize];
                reader.read(&mut vec)?;
                println!("whole string: {:?}", vec);
                let mut cp = ChatPacket {
                    length: 0,
                    body: String::new(),
                    service_code: 0,
                };
                cp.service_code = service_code;
                cp.length = length;

                let t = String::from_utf8_lossy(&vec[12..]);
                cp.body = t.to_string();
                return Ok(cp);
            }
        }
        bail!("yet")
    }
}

struct MakeRoomC2S {
    password: [u8; 101],
}
