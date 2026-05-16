#[derive(Clone, Copy, Debug, PartialEq)]
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

pub struct Response {
    pub status: Status,
    pub value: Option<String>,
}

pub struct Request {
    pub op: ReqType,
    pub key: String,
    pub value: Option<String>, // only for SET
}
