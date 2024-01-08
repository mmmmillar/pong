use askama::Template;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use std::str::FromStr;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::{filters::ws::Ws, http::Uri, reject::Rejection, reply::Reply};

use crate::{
    client_event::{parse_client_event, ClientEventType},
    game::{Game, SCALING_FACTOR},
    templates, GAME_STORE,
};

pub async fn create_game_handler(name: String) -> Result<impl Reply, Rejection> {
    let id: String = Uuid::new_v4().to_string();

    // is this concurrent-friendly?
    GAME_STORE
        .write()
        .await
        .insert(id.clone(), Game::new(id.clone()));

    Ok(warp::redirect::see_other(
        Uri::from_str(&format!("/games/{}", id)).unwrap(),
    ))
}

pub async fn game_handler(game_id: String) -> Result<impl Reply, Rejection> {
    let mut games_write = GAME_STORE.write().await;
    if let Some(mut game) = games_write.get(&game_id).cloned() {
        if game.num_players() >= 2 {
            // need better reply
            eprintln!("game already full");
            return Err(warp::reject::not_found());
        }

        let player_id = String::from(game.add_player());
        games_write.insert(game_id.clone(), game.clone());
        eprintln!("games {:?}", games_write);

        Ok(warp::reply::html(
            templates::GameTemplate::new(game_id, player_id)
                .render()
                .unwrap(),
        ))
    } else {
        Err(warp::reject::not_found())
    }
}

pub async fn ws_handler(
    (game_id, player_id): (String, String),
    ws: Ws,
) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(|socket| async move {
        let (mut ws_tx, mut ws_rx) = socket.split();
        let (tx, rx) = unbounded_channel();
        let mut rx = UnboundedReceiverStream::new(rx);

        {
            let mut games_write = GAME_STORE.write().await;
            if let Some(mut game) = games_write.get(&game_id).cloned() {
                game.set_player_tx(&player_id, tx);
                games_write.insert(game_id.clone(), game);
            }
        }

        // spawn broadcast task
        tokio::spawn(async move {
            while let Some(message) = rx.next().await {
                ws_tx
                    .send(message)
                    .unwrap_or_else(|e| {
                        eprintln!("websocket send error: {}", e);
                    })
                    .await;
            }
        });

        while let Some(result) = ws_rx.next().await {
            if let Ok(msg) = result {
                match parse_client_event(msg.to_str().unwrap()) {
                    Some(ClientEventType::ReadyEvent) => {
                        let mut games_write = GAME_STORE.write().await;
                        if let Some(mut game) = games_write.get(&game_id).cloned() {
                            game.set_player_ready(&player_id);
                            games_write.insert(game_id.clone(), game.clone());
                            if game.both_players_ready() {
                                eprintln!("starting game");
                                game.start()
                            }
                        }
                    }
                    Some(ClientEventType::MoveEvent(event)) => {
                        let mut games_write = GAME_STORE.write().await;
                        if let Some(mut game) = games_write.get(&game_id).cloned() {
                            game.update_player_pos(&player_id, event.y / SCALING_FACTOR);
                            games_write.insert(game_id.clone(), game);
                        }
                    }
                    None => {
                        log::info!("Event type not recognised");
                    }
                };
            } else {
                log::info!("HELLO");
            }

            // Send the message to the broadcast task
            // tx.send(msg).unwrap();
        }

        // clients.write().await.retain(|client| client.id != id);
        eprintln!("disconnected")
    }))
}
