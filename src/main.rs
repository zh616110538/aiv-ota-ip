use axum::{
    extract::ConnectInfo,
    routing::{get, post},
    Router,
};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};

fn instance() -> &'static Arc<Mutex<HashMap<u32, IpAddr>>> {
    static INSTANCE: OnceCell<Arc<Mutex<HashMap<u32, IpAddr>>>> = OnceCell::new();
    INSTANCE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
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
    format!("{:#?}", instance().clone().lock().unwrap())
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
    if let Some(v) = c.lock().unwrap().insert(card_num, ip) {
        if v != ip {
            eprintln!("conflict card num {} between {} and {}", card_num, ip, v);
        }
    };
}
