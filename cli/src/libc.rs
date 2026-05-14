use std::io::ErrorKind;
use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::TcpStream};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReqType {
    GET = 0,
    SET = 1,
    DEL = 2,
    EXISTS = 3,
    TOTAL = 4,
}
pub enum Status {
    Ok = 0,
    NotFound = 1,
    Error = 2,
}

pub struct Response {
    pub status: Status,
    pub value: Option<String>,
}

pub fn write_str(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    buf.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    buf.extend_from_slice(bytes);
}

pub async fn read_exact(stream: &mut TcpStream, n: usize) -> std::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; n];
    stream.read_exact(&mut buf).await?;
    Ok(buf)
}

pub async fn read_str(stream: &mut TcpStream) -> std::io::Result<String> {
    let bytes = read_exact(stream, 4).await?;
    let len = u32::from_be_bytes(
        bytes
            .try_into()
            .map_err(|_| std::io::Error::new(ErrorKind::InvalidData, "Invalid length"))?,
    ) as usize;
    let bytes = read_exact(stream, len).await?;
    String::from_utf8(bytes).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))
}

pub async fn send_request(
    stream: &mut TcpStream,
    req: &ReqType,
    key: Option<String>,
    value: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    buf.push(*req as u8);
    if *req == ReqType::TOTAL {
        buf.extend_from_slice(&0u32.to_be_bytes());
    } else if let Some(k) = key {
        write_str(&mut buf, &k);
        match value {
            Some(v) => write_str(&mut buf, &v),
            None => match req {
                ReqType::SET => return Err("Value is required for SET operation".into()),
                _ => (),
            },
        }
    } else {
        return Err("Key is required for this operation".into());
    }

    let _ = stream
        .write_all(&buf)
        .await
        .map_err(|e| println!("{}", e.to_string()))
        .map(|_| ());
    Ok(())
}

pub async fn recv_response(stream: &mut TcpStream) -> std::io::Result<Response> {
    let status = match read_exact(stream, 1).await?[0] {
        0 => Status::Ok,
        1 => Status::NotFound,
        _ => Status::Error,
    };
    let value = match read_str(stream).await? {
        s if s.is_empty() => None,
        s => Some(s),
    };
    Ok(Response { status, value })
}
