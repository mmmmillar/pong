use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::{sync::mpsc::UnboundedSender, time::sleep};
use warp::filters::ws::Message;

use crate::{
    player::Player,
    pong::{Pong, P1_START, P2_START},
    GAME_STORE,
};

pub const PLAYER_1: &str = "p1";
pub const PLAYER_2: &str = "p2";
pub const SCALING_FACTOR: f32 = 8.0;

#[derive(Serialize, Deserialize)]
struct StepResult {
    ball_x: f32,
    ball_y: f32,
    left_wall_contact: bool,
    right_wall_contact: bool,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub id: String,
    players: HashMap<String, Player>,
}

impl Game {
    pub fn new(id: String) -> Self {
        Game {
            id,
            players: HashMap::new(),
        }
    }

    pub fn num_players(&self) -> usize {
        self.players.len()
    }

    pub fn add_player(&mut self) -> &str {
        if self.players.get(PLAYER_1).is_none() {
            self.players
                .insert(String::from(PLAYER_1), Player::new(P1_START.0, P1_START.1));
            PLAYER_1
        } else if self.players.get(PLAYER_2).is_none() {
            self.players
                .insert(String::from(PLAYER_2), Player::new(P2_START.0, P2_START.1));
            PLAYER_2
        } else {
            panic!("Too many players")
        }
    }

    pub fn set_player_tx(&mut self, player_id: &str, tx: UnboundedSender<Message>) {
        if let Some(mut player) = self.players.get(player_id).cloned() {
            player.tx = Some(tx);
            self.players.insert(player_id.into(), player);
        }
    }

    pub fn set_player_ready(&mut self, player_id: &str) {
        if let Some(mut player) = self.players.get(player_id).cloned() {
            player.is_ready = true;
            self.players.insert(player_id.into(), player);
        }
    }

    pub fn update_player_pos(&mut self, player_id: &str, y: f32) {
        if let Some(mut player) = self.players.get(player_id).cloned() {
            player.y = y;
            log::info!("{} pos: {},{}", player_id, player.x, y);
            self.players.insert(player_id.into(), player);
        }
    }

    pub fn inc_player_score(&mut self, player_id: &str) {
        if let Some(mut player) = self.players.get(player_id).cloned() {
            player.score += 1;
            self.players.insert(player_id.into(), player);
        }
    }

    pub fn both_players_ready(&self) -> bool {
        self.players.get(PLAYER_1).is_some_and(|p| p.is_ready)
            && self.players.get(PLAYER_2).is_some_and(|p| p.is_ready)
    }

    pub fn get_player(&self, player_id: &str) -> &Player {
        if let Some(player) = self.players.get(player_id) {
            return player;
        }
        panic!("no such player")
    }

    fn step(&self, pong: &mut Pong) -> StepResult {
        let (x, y, left_wall_contact, right_wall_contact) =
            pong.next(Some((self.get_player("p1").y, self.get_player("p2").y)));

        StepResult {
            ball_x: x,
            ball_y: y,
            left_wall_contact,
            right_wall_contact,
        }
    }

    pub fn start(&self) {
        tokio::spawn(execute(self.id.clone()));
    }
}

async fn execute(id: String) {
    let mut p = Pong::new(6.0);

    loop {
        let mut games_write = GAME_STORE.write().await;
        let game = games_write.get_mut(&id).unwrap();

        let step_result = game.step(&mut p);

        send_update_screen(&game, &step_result);
        sleep(Duration::from_millis(15)).await;
        if step_result.left_wall_contact {
            game.inc_player_score(PLAYER_2);
            game.update_player_pos(PLAYER_1, P1_START.1);
            game.update_player_pos(PLAYER_2, P2_START.1);
            send_end_point(&game);
            p = Pong::new(6.0);
        } else if step_result.right_wall_contact {
            game.inc_player_score(PLAYER_1);
            game.update_player_pos(PLAYER_1, P1_START.1);
            game.update_player_pos(PLAYER_2, P2_START.1);
            send_end_point(&game);
            p = Pong::new(6.0);
        }
    }
}

fn send_end_point(game: &Game) {
    let p1_score = game.players.get(PLAYER_1).unwrap().score;
    let p2_score = game.players.get(PLAYER_2).unwrap().score;

    game.players.iter().for_each(|(_, player)| {
        player
            .clone()
            .tx
            .unwrap()
            .send(Message::text(
                json!({
                    "event_type": "end_point",
                    "event_body": {
                        "p1_score": p1_score,
                        "p2_score": p2_score,
                    }

                })
                .to_string(),
            ))
            .unwrap();
    });
}

fn send_update_screen(game: &Game, step_result: &StepResult) {
    game.players.iter().for_each(|(id, player)| {
        let opponent_id = match id.as_str() {
            PLAYER_1 => PLAYER_2,
            PLAYER_2 => PLAYER_1,
            _ => panic!("bad player id"),
        };
        let opponent = game.players.get(opponent_id).unwrap();

        player
            .clone()
            .tx
            .unwrap()
            .send(Message::text(
                json!({
                    "event_type": "update_screen",
                    "event_body": {
                        "ball_x": step_result.ball_x * SCALING_FACTOR,
                        "ball_y": step_result.ball_y * SCALING_FACTOR,
                        "opponent_y": opponent.y * SCALING_FACTOR
                    }

                })
                .to_string(),
            ))
            .unwrap();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_players_ready_true_when_both_ready() {
        let mut game = Game::new("123".into());
        game.add_player();
        game.add_player();
        game.set_player_ready("p1");
        game.set_player_ready("p2");

        assert_eq!(game.both_players_ready(), true)
    }
}
