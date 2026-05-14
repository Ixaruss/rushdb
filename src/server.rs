use crate::conf::Config;
use crate::engine::Store;
use crate::libr::{ReqType, Response, Status, recv_request, send_response};
use crate::persistence::{load, save};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

const SAVE_INTERVAL: u64 = 60;
const ADDRESS: &str = "0.0.0.0";
const PORT: u16 = 6080;

pub async fn server_init() -> std::io::Result<()> {
    let config = Config::load(Some(String::from("./conf.toml")));
    let state: Arc<Mutex<Store>> = Arc::new(Mutex::new(load()));
    let add = if config.server.host.is_empty() {
        ADDRESS.to_string()
    } else {
        config.clone().server.host
    };
    let port = config.server.port | PORT;
    let m = format!("{}:{}", add, port);
    let listener = TcpListener::bind(m.clone()).await?;
    println!("listening on {}", m);
    let local = Arc::clone(&state);

    tokio::spawn(async move {
        let interval = config.server.save_interval | SAVE_INTERVAL;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval));
        loop {
            interval.tick().await;
            let store = local.lock().await;
            save(&store, Some(config.clone())).ok();
            println!("saved a snapshot!!!")
        }
    });
    loop {
        let (mut stream, addr) = listener.accept().await?;
        println!("accepted connection from: {}", addr);
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            loop {
                match recv_request(&mut stream).await {
                    Ok(req) => {
                        let mut store = state.lock().await;
                        let res = match req.op {
                            ReqType::GET => match store.db.get(&req.key) {
                                Some(v) => {
                                    println!("value from store: {v}");
                                    Response {
                                        status: Status::Ok,
                                        value: Some(v.clone()),
                                    }
                                }
                                None => Response {
                                    status: Status::NotFound,
                                    value: None,
                                },
                            },
                            ReqType::SET => {
                                store.db.insert(req.key, req.value.unwrap());
                                Response {
                                    status: Status::Ok,
                                    value: None,
                                }
                            }
                            ReqType::DEL => {
                                store.db.remove(&req.key);
                                Response {
                                    status: Status::Ok,
                                    value: None,
                                }
                            }
                            ReqType::EXISTS => {
                                let v = store.db.contains_key(&req.key);
                                Response {
                                    status: Status::Ok,
                                    value: Some(v.to_string()),
                                }
                            }
                            ReqType::TOTAL => {
                                let n = store.db.len();
                                println!("toal: {n}");
                                Response {
                                    status: Status::Ok,
                                    value: Some(n.to_string()),
                                }
                            }
                        };
                        send_response(&mut stream, &res).await.ok();
                    }
                    Err(_) => break,
                }
            }
        });
    }
}
