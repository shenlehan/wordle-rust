use crate::config::GameConfig;
use crate::stats::GameStats;
use serde_json::json;

pub fn parse_states(
    config: &GameConfig,
) -> Result<serde_json::value::Value, Box<dyn std::error::Error>> {
    if config.state != "".to_string() {
        let content = std::fs::read_to_string(config.state.as_str())?;
        let value: serde_json::Value = serde_json::from_str(&content)?;
        if value
            .as_object()
            .map(|object| object.is_empty())
            .unwrap_or(false)
        {
            Ok(json!({
                "total_rounds": 0,
                "games": []
            }))
        } else {
            Ok(value)
        }
    } else {
        Ok(json!({
            "total_rounds": 0,
            "games": []
        }))
    }
}


