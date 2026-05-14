use std::io::{Error, ErrorKind};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Clone, Copy, Debug)]
pub enum ReqType {
    GET = 0,
    SET = 1,
    DEL = 2,
    EXISTS = 3,
    TOTAL = 4,
}
#[derive(Clone)]
pub enum Status {
    Ok = 0,
    NotFound = 1,
    Error = 2,
}
pub struct Request {
    pub op: ReqType,
    pub key: String,
    pub value: Option<String>, // only for SET
}
#[derive(Clone)]
pub struct Response {
    pub status: Status,
    pub value: Option<String>,
}

pub fn write_str(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    buf.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    buf.extend_from_slice(bytes);
}

pub async fn read_exact(stream: &mut TcpStream, n: usize) -> Result<Vec<u8>, Error> {
    let mut buf = vec![0u8; n];
    let r = stream.read_exact(&mut buf).await;

    match r {
        Ok(_) => Ok(buf),
        Err(e) => Err(e),
    }
}

pub async fn read_str(stream: &mut TcpStream) -> std::io::Result<String> {
    let len = u32::from_be_bytes(read_exact(stream, 4).await?.try_into().unwrap()) as usize;
    let bytes = read_exact(stream, len).await?;
    String::from_utf8(bytes).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))
}
pub async fn recv_request(stream: &mut TcpStream) -> std::io::Result<Request> {
    let op_byte = read_exact(stream, 1).await?[0];
    println!("type: {:?},", op_byte);
    let op = match op_byte {
        0 => ReqType::GET,
        1 => ReqType::SET,
        2 => ReqType::DEL,
        3 => ReqType::EXISTS,
        4 => ReqType::TOTAL,
        _ => return Err(std::io::Error::new(ErrorKind::InvalidData, "unknown op")),
    };
    let key = read_str(stream).await?;
    let value = match op {
        ReqType::SET => Some(read_str(stream).await?),
        _ => None,
    };
    Ok(Request { op, key, value })
}

pub async fn send_response(stream: &mut TcpStream, res: &Response) -> std::io::Result<()> {
    let mut buf = vec![res.status.clone() as u8];
    match &res.value {
        Some(v) => write_str(&mut buf, v),
        None => buf.extend_from_slice(&0u32.to_be_bytes()),
    }
    if stream.write_all(&buf).await.is_err() {
        Err(Error::new(ErrorKind::Other, "something went wrong"))
    } else {
        Ok(())
    }
}
