use crate::config::GameConfig;
use crate::stats::GameStats;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Write;

pub fn top_n_keys<K: Clone + Ord, V: Ord>(map: &HashMap<K, V>, n: usize) -> Vec<K> {
    let mut pairs: Vec<(&K, &V)> = map.iter().collect();

    pairs.sort_by(|a, b| match b.1.cmp(a.1) {
        std::cmp::Ordering::Equal => a.0.cmp(b.0),
        other => other,
    });

    pairs.into_iter().take(n).map(|(k, _v)| k.clone()).collect()
}

pub fn print_result(user_status: &Vec<char>, keyboard_status: &Vec<char>) {
    for c in user_status.iter() {
        print!("{c}");
        io::stdout().flush().unwrap();
    }
    print!(" ");
    io::stdout().flush().unwrap();
    for c in keyboard_status.iter() {
        print!("{c}");
        io::stdout().flush().unwrap();
    }
    println!();
}

pub fn print_statistics(config: &GameConfig, stats: &GameStats) {
    if config.stats == false {
        return;
    }
    /* print basic */
    let x = if stats.success_game_cnt == 0.0 {
        0.0
    } else {
        stats.total_success_guess_try / stats.success_game_cnt as f64
    };
    println!(
        "{} {} {:.2}",
        stats.success_game_cnt,
        stats.total_game_cnt - stats.success_game_cnt,
        x
    );

    /* print word frequency */
    let top_words = top_n_keys(&stats.guess_frequency, 5);
    let mut word_stats: Vec<String> = Vec::new();
    for word in top_words {
        word_stats.push(format!(
            "{} {}",
            word.to_uppercase(),
            stats.guess_frequency.get(&word).unwrap()
        ));
    }
    println!("{}", word_stats.join(" "));
}

pub fn dump_json_file(
    game_state: &serde_json::Value,
    state_file: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(state_file.as_str())?;
    serde_json::to_writer_pretty(file, &game_state)?;
    Ok(())
}

pub fn load_state_stats(
    game_state: &mut serde_json::Value,
    config: &GameConfig,
    stats: &mut GameStats,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.state == "".to_string() {
        return Ok(());
    }

    let games = game_state["games"]
        .as_array()
        .ok_or("Error! invalid state games")?;
    stats.total_game_cnt = games.len() as f32;
    for game in games {
        let answer = game["answer"]
            .as_str()
            .ok_or("Error! invalid state answer")?
            .to_lowercase();
        let guesses = game["guesses"]
            .as_array()
            .ok_or("Error! invalid state guesses")?;
        for guess in guesses {
            let guess = guess
                .as_str()
                .ok_or("Error! invalid state guess")?
                .to_lowercase();
            *stats.guess_frequency.entry(guess).or_insert(0) += 1;
        }
        if guesses
            .last()
            .and_then(|guess| guess.as_str())
            .map(|guess| guess.to_lowercase() == answer)
            .unwrap_or(false)
        {
            stats.success_game_cnt += 1.0;
            stats.total_success_guess_try += guesses.len() as f64;
        }
    }

    Ok(())
}

pub fn check_final_accept_cohe(
    final_words: &Vec<String>,
    acceptable_words: &Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    for word in final_words.iter() {
        if !acceptable_words.contains(word) {
            return Err(Box::from("Error! FINAL is not a subset of ACCEPTABLE!"));
        }
    }
    Ok(())
}
