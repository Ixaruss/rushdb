use crate::conf::Config;
use crate::engine::Store;
use crate::persistence::{load, save};
use common::{
    io::{recv_request, send_response},
    serevrtypes::{ReqType, Response, Status},
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

pub async fn server_init() -> std::io::Result<()> {
    let config = Config::load(Some(String::from("./conf.toml")));
    let state: Arc<RwLock<Store>> = Arc::new(RwLock::new(load(config.clone())));

    let m = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(m.clone()).await?;
    println!("listening on {}", m);
    let local = Arc::clone(&state);

    // tokio::spawn(async move {
    //     let interval = config.server.save_interval;
    //     let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval));
    //     loop {
    //         interval.tick().await;
    //         let snapshot = {
    //             let store = local.read().await;
    //             store.db.clone()
    //         };
    //         save(&snapshot, config.clone()).ok();
    //         println!("saved a snapshot!!!")
    //     }
    // });

    loop {
        let (mut stream, addr) = listener.accept().await?;
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            loop {
                match recv_request(&mut stream).await {
                    Ok(req) => {
                        let res = match req.op {
                            ReqType::GET => {
                                let store = state.read().await;
                                match store.db.get(&req.key) {
                                    Some(v) => Response {
                                        status: Status::Ok,
                                        value: Some(v.clone()),
                                    },
                                    None => Response {
                                        status: Status::NotFound,
                                        value: None,
                                    },
                                }
                            }
                            ReqType::SET => {
                                let mut store = state.write().await;
                                store.db.insert(req.key, req.value.unwrap());
                                Response {
                                    status: Status::Ok,
                                    value: None,
                                }
                            }
                            ReqType::DEL => {
                                let mut store = state.write().await;
                                store.db.remove(&req.key);
                                Response {
                                    status: Status::Ok,
                                    value: None,
                                }
                            }
                            ReqType::EXISTS => {
                                let store = state.read().await;
                                let v = store.db.contains_key(&req.key);
                                Response {
                                    status: Status::Ok,
                                    value: Some(v.to_string()),
                                }
                            }
                            ReqType::TOTAL => {
                                let store = state.read().await;
                                let n = store.db.len();
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
