use std::{env, net::SocketAddr};

use axum::{
    extract::WebSocketUpgrade,
    response::IntoResponse,
    routing::get,
    Router,
    http::StatusCode,
};
use axum::http::header::HeaderMap;
use axum::http::Response;
use axum::body::Body;
//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;
// use http::HeaderValue;
use openai_realtime_proxy::Proxy;
use tokio::net::TcpListener;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use toml;
// use axum::debug_handler;
use axum_macros::debug_handler;

#[derive(Debug, Deserialize)]
struct Config {
    keys: Vec<String>,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/v1/realtime", get(ws_handler));

    let addr = SocketAddr::from((
        [0, 0, 0, 0],
        env::var("PORT").map_or(Ok(8000), |p| p.parse()).unwrap(),
    ));
    let listener = TcpListener::bind(&addr).await.unwrap();

    println!("listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap()
}

#[debug_handler]
async fn ws_handler(ws: WebSocketUpgrade, headers: HeaderMap, ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    // check for authentication/access/etc. here
    let auth = headers.get("authorization");
    let mut authed = false;

    let split: Vec<&str> = auth.unwrap().to_str().unwrap().split(' ').collect();

    if (split.len() != 2) || (split[0] != "Bearer") {
        authed = false;
    } else {
        let mut file = File::open("config.toml").unwrap();
        let mut config_string = String::new();
        file.read_to_string(&mut config_string).unwrap();

        // Parse the string to a Config
        let config: Config = toml::from_str(&config_string).unwrap();

        // Check if the key is present
        authed = config.keys.iter().any(|key| *key == split[1]);
    }

    if !authed {
        println!("unauthorized {addr}");
        Response::builder().status(StatusCode::FORBIDDEN).body(Body::from("Unauthorized")).unwrap()
    } else {
        println!("authorized {addr}");
        let proxy = Proxy::new(env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY env var not set."));
        ws.on_upgrade(|socket| proxy.handle(socket))
    }
}
