use axum::{
    extract::ConnectInfo,
    routing::{get, post},
    Router,
};
use once_cell::sync::OnceCell;
// use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
extern crate redis;
use redis::{Commands, Connection};

fn instance() -> &'static Arc<Mutex<Connection>> {
    static INSTANCE: OnceCell<Arc<Mutex<Connection>>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let con = client.get_connection().expect("请装一下redis-server");
        return Arc::new(Mutex::new(con));
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/api/report_num", post(handler));
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn root() -> String {
    let c = instance().clone();
    let mut con = c.lock().unwrap();
    let keys: Vec<String> = con.keys("*").unwrap();
    let length = keys.len();
    if length > 0 {
        let values: Vec<String> = con.mget(&keys).unwrap();
        let mut result = String::new();
        // return format!("{:?}", values);
        for i in 0..length {
            result.push_str(format!("{} {}\n", keys[i], values[i]).as_str());
        }
        return result;
    }
    "".to_string()
}

async fn handler(ConnectInfo(addr): ConnectInfo<SocketAddr>, body: String) {
    let ip = addr.ip();
    let card_num: u32 = match body.parse() {
        Ok(v) => v,
        Err(e) => {
            eprintln! {"{}\n{}", e, body};
            return;
        }
    };
    let c = instance().clone();
    let mut con = c.lock().unwrap();
    let _: () = redis::cmd("SET")
        .arg(card_num)
        .arg(ip.to_string())
        .arg("EX")
        .arg("180")
        .query(&mut con)
        .unwrap();
}
