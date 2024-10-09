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
// use axum::extract::connect_info::ConnectInfo;
use http::HeaderValue;
use openai_realtime_proxy::Proxy;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/v1/realtime", get(ws_handler));

    let addr = SocketAddr::from((
        [0, 0, 0, 0],
        env::var("PORT").map_or(Ok(8000), |p| p.parse()).unwrap(),
    ));
    let listener = TcpListener::bind(&addr).await.unwrap();

    println!("listening on http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap()
}

// #[debug_handler]
async fn ws_handler(ws: WebSocketUpgrade, headers: HeaderMap) -> impl IntoResponse {
    // check for authentication/access/etc. here
    let addr = "";
    let auth = headers.get("authorization");
    let key = &env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY env var not set.");
    let hdr_auth = HeaderValue::from_str(&( "Bearer ".to_string() + key)).unwrap();
    println!("auth: {auth:?} hdr_auth: {hdr_auth:?}");
    if auth != Some(&hdr_auth) {
        println!("unauthorized {addr}");
        Response::builder().status(StatusCode::FORBIDDEN).body(Body::from("Unauthorized")).unwrap()
    } else {
        println!("authorized {addr}");
        let proxy = Proxy::new(env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY env var not set."));
        ws.on_upgrade(|socket| proxy.handle(socket))
    }
}
