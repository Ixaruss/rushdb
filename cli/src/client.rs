use crate::libc::{ReqType, recv_response, send_request};
use tokio::net::TcpStream;

pub async fn query(
    key: Option<String>,
    value: Option<String>,
    rtype: ReqType,
    stream: &mut TcpStream,
) -> Option<String> {
    send_request(stream, &rtype, key, value).await.ok()?;
    let resp = recv_response(stream).await.ok()?;
    return resp.value;
}
