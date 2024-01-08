use tokio::sync::mpsc::UnboundedSender;
use warp::filters::ws::Message;

#[derive(Debug, Clone)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub score: u32,
    pub is_ready: bool,
    pub tx: Option<UnboundedSender<Message>>,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Player {
            x,
            y,
            score: 0,
            is_ready: false,
            tx: None,
        }
    }
}
