use axum::{
    extract::ConnectInfo,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
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
        .route("/lib/axios.js", get(axios_js))
        .route("/lib/index.css", get(index_css))
        .route("/lib/index.js", get(index_js))
        .route("/lib/vue.js", get(vue_js))
        .route("/api/getinfo", get(json))
        .route("/api/report_num", post(handler));
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn root() -> Html<&'static str> {
    Html(include_str!("../assets/index.html"))
}

async fn axios_js() -> Html<&'static str> {
    Html(include_str!("../assets/lib/axios.js"))
}

async fn index_css() -> Html<&'static str> {
    Html(include_str!("../assets/lib/index.css"))
}

async fn index_js() -> Html<&'static str> {
    Html(include_str!("../assets/lib/index.js"))
}

async fn vue_js() -> Html<&'static str> {
    Html(include_str!("../assets/lib/vue.js"))
}

#[derive(Serialize, Deserialize)]
struct Item {
    name: String,
    ip: String,
}

async fn json() -> Json<Vec<Item>> {
    let c = instance().clone();
    let mut con = c.lock().unwrap();
    let keys: Vec<String> = con.keys("*").unwrap();
    let length = keys.len();
    let mut result = Vec::new();
    if length > 0 {
        let values: Vec<String> = con.mget(&keys).unwrap();
        for i in 0..length {
            // result.push_str(format!("{} {}\n", keys[i], values[i]).as_str());
            result.push(Item {
                name: keys[i].clone(),
                ip: values[i].clone(),
            });
        }
    }
    Json(result)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json() {
        let mut is = Vec::new();
        is.push(Item {
            name: "802".to_string(),
            ip: "10.111.0.66".to_string(),
        });
        is.push(Item {
            name: "803".to_string(),
            ip: "10.111.0.67".to_string(),
        });
        is.push(Item {
            name: "804".to_string(),
            ip: "10.111.0.68".to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&is).unwrap());
    }
}
