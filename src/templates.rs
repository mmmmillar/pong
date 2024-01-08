use askama::Template;

use crate::{
    game::{PLAYER_1, PLAYER_2, SCALING_FACTOR},
    pong::{
        BALL_RADIUS, BALL_START, P1_START, P2_START, PLAYER_HEIGHT, PLAYER_WIDTH, PONG_HEIGHT,
        PONG_WIDTH,
    },
};

#[derive(Template)]
#[template(path = "game.html")]
pub struct GameTemplate {
    game_id: String,
    player_id: String,
    game_width: f32,
    game_height: f32,
    player_width: f32,
    player_height: f32,
    ball_radius: f32,
    player_start_x: f32,
    player_start_y: f32,
    opponent_start_x: f32,
    opponent_start_y: f32,
    ball_start_x: f32,
    ball_start_y: f32,
}

impl GameTemplate {
    pub fn new(game_id: String, player_id: String) -> Self {
        let player = match player_id.as_str() {
            PLAYER_1 => (P1_START.0 * SCALING_FACTOR, P1_START.1 * SCALING_FACTOR),
            PLAYER_2 => (P2_START.0 * SCALING_FACTOR, P2_START.1 * SCALING_FACTOR),
            _ => panic!("no such player"),
        };

        let opponent = match player_id.as_str() {
            PLAYER_1 => (P2_START.0 * SCALING_FACTOR, P2_START.1 * SCALING_FACTOR),
            PLAYER_2 => (P1_START.0 * SCALING_FACTOR, P1_START.1 * SCALING_FACTOR),
            _ => panic!("no such player"),
        };

        GameTemplate {
            game_id,
            player_id,
            game_width: PONG_WIDTH * SCALING_FACTOR,
            game_height: PONG_HEIGHT * SCALING_FACTOR,
            player_width: PLAYER_WIDTH * SCALING_FACTOR,
            player_height: PLAYER_HEIGHT * SCALING_FACTOR,
            ball_radius: BALL_RADIUS * SCALING_FACTOR,
            player_start_x: player.0,
            player_start_y: player.1,
            opponent_start_x: opponent.0,
            opponent_start_y: opponent.1,
            ball_start_x: BALL_START.0 * SCALING_FACTOR,
            ball_start_y: BALL_START.1 * SCALING_FACTOR,
        }
    }
}
