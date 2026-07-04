use crate::config::GameConfig;
use serde_json::json;
use std::collections::HashMap;
use crate::game::RoundResult;

pub struct GameStats {
    pub success_game_cnt: f32,
    pub total_game_cnt: f32,
    pub total_success_guess_try: f64,
    pub guess_frequency: HashMap<String, i32>,
}

impl GameStats {
    pub fn new(
        success_game_cnt: f32,
        total_game_cnt: f32,
        total_success_guess_try: f64,
        guess_frequency: HashMap<String, i32>,
    ) -> GameStats {
        GameStats {
            success_game_cnt,
            total_game_cnt,
            total_success_guess_try,
            guess_frequency,
        }
    }

}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            success_game_cnt: 0.0,
            total_game_cnt: 0.0,
            total_success_guess_try: 0.0,
            guess_frequency: Default::default(),
        }
    }
}

pub fn update_stats(stats: &mut GameStats, result: &RoundResult) {
    stats.total_game_cnt += 1.0;
    stats.success_game_cnt += if result.success { 1.0 } else { 0.0 };
    if result.success {
        stats.total_success_guess_try += result.guess_count as f64;
    }
    for guess in result.guesses.iter() {
        *stats.guess_frequency.entry(guess.clone()).or_insert(0) += 1;
    }
}