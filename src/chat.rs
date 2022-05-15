use std::io::prelude::*;

use bytebuffer::ByteBuffer;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
#[derive(Debug)]
pub struct ChatPacket {
    service_code: i32,
    length: i32,
    body: String,
}

pub struct ChatParser {
    pub service_code: i32,
    pub length: i32,
    pub body: String,
    pub buffer: BytesMut,
    // buffer: BufReader<[u80]>,
}

impl ChatParser {
    pub fn new() -> Self{
        return ChatParser{
            service_code: 0, 
            length: 0,
            body: "".to_string(),
            buffer: BytesMut::new(),
        }
    }
    pub fn put(&mut self, data: Bytes) {
        self.buffer.put(&data.to_vec()[..]);
    }
    pub fn parse(&mut self) -> Result<ChatPacket, ()> {
        // 0001&000004&abcd
        if self.buffer.len() >= 12 {
            let b = &self.buffer[..];
            let service_code = String::from_utf8(b[0..4].to_vec());
            let service_code = match service_code {
                Ok(v) => v,
                Err(e) => {
                    return Err(());
                }
            };
            let service_code = match service_code.parse::<i32>() {
                Ok(v) => v,
                Err(e) => {
                    return Err(());
                }
            };
            let length = String::from_utf8(b[5..11].to_vec());
            let length = match length {
                Ok(v) => v,
                Err(e) => {
                    return Err(());
                }
            };
            let length = match length.parse::<i32>() {
                Ok(v) => v,
                Err(e) => {
                    return Err(());
                }
            };
            println!("service code: {:?}", service_code);
            println!("length: {:?}", length);
            if (self.buffer.len()) as u32 >= 12 + length as u32 {
                let l = 12 + length;
                let mut reader = (&mut self.buffer).reader();
                let mut vec = vec![0; l as usize];
                reader.read(&mut vec);
                println!("whole string: {:?}", vec);
                let mut cp = ChatPacket {
                    length: 0,
                    body: String::new(),
                    service_code: 0,
                };
                cp.service_code = service_code;
                cp.length = length;

                let t = String::from_utf8(vec[12..].to_vec());
                cp.body = match t {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(());
                    }
                };
                return Ok(cp);
            }
        }
        return Err(());
    }
}
