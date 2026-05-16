use std::io::ErrorKind;
use tokio::{io::AsyncReadExt, net::TcpStream};

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
