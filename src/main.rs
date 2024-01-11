mod client_event;
mod game;
mod handlers;
mod player;
mod pong;
mod templates;

use game::Game;
use lazy_static::lazy_static;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use warp::Filter;

type GameStore = Arc<RwLock<HashMap<String, Game>>>;

lazy_static! {
    pub static ref GAME_STORE: GameStore = Arc::new(RwLock::new(HashMap::new()));
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let home_page = warp::path::end().and(warp::fs::file("templates/index.html"));

    let hello_page = warp::path("hello").and(warp::fs::file("templates/hello.html"));

    let game_page = warp::path("games")
        .and(warp::path::param())
        .map(|game_id: String| game_id)
        .and_then(handlers::game_handler);

    let create_game_route = warp::path!("create_game")
        .and(warp::post())
        .and(warp::body::form())
        .map(|body: HashMap<String, String>| body.get("name").unwrap().clone())
        .and_then(handlers::create_game_handler);

    let ws_route = warp::path!("ws" / String / String)
        .map(|game_id: String, player_id: String| (game_id, player_id))
        .and(warp::ws())
        .and_then(handlers::ws_handler);

    let routes = home_page
        .or(hello_page)
        .or(game_page)
        .or(create_game_route)
        .or(ws_route)
        .with(warp::log("pong"));

    let addr = format!("{}:{}", "0.0.0.0", 3030)
        .parse::<SocketAddr>()
        .expect("Invalid address format");

    eprintln!("Pong server running on {:?}", addr);

    warp::serve(routes).run(addr).await;
}
