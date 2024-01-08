use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};

pub enum ClientEventType {
    MoveEvent(MoveEvent),
    ReadyEvent,
}

#[derive(Deserialize)]
pub struct MoveEvent {
    pub y: f32,
}

impl<'de> Deserialize<'de> for ClientEventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: Map<String, Value> = Deserialize::deserialize(deserializer)?;

        let event_type = map.get("event_type").and_then(|v| v.as_str());
        let event_body = map.get("event_body").cloned().unwrap();

        match event_type {
            Some("ready") => Ok(ClientEventType::ReadyEvent),
            Some("move") => {
                let e = serde_json::from_value(event_body).map_err(serde::de::Error::custom)?;
                Ok(ClientEventType::MoveEvent(e))
            }
            _ => Err(serde::de::Error::custom("Unknown event type")),
        }
    }
}

pub fn parse_client_event(json_data: &str) -> Option<ClientEventType> {
    match serde_json::from_str(json_data) {
        Ok(event_type) => Some(event_type),
        Err(err) => {
            log::error!("Failed to parse json: {}. Err: {}", json_data, err);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_ready_event() {
        let json_data = r#"
            {
                "event_type": "ready",
                "event_body": {}
            }
        "#;

        let result = match parse_client_event(json_data) {
            Some(ClientEventType::ReadyEvent) => true,
            _ => panic!("Event type not recognised"),
        };

        assert_eq!(result, true);
    }
}
