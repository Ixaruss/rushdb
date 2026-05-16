use std::io::{Error, ErrorKind};

use crate::{
    serevrtypes::{ReqType, Request, Response, Status},
    str::{read_exact, read_str, write_str},
};
use tokio::{io::AsyncWriteExt, net::TcpStream};

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

pub async fn recv_request(stream: &mut TcpStream) -> std::io::Result<Request> {
    let op_byte = read_exact(stream, 1).await?[0];
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
