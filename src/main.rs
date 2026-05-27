use console;
use log::info;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, stdin, Write};
use wordle::parse::*;
use wordle::builtin_words::*;
use wordle::config::*;
use wordle::engine::*;
use wordle::game::next_answer;
use wordle::stats::*;
use wordle::utilities::*;
use wordle::words::*;
use wordle::state::*;

/// The main function for the Wordle game, implement your own logic here
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let is_tty = atty::is(atty::Stream::Stdout);

    let mut scores = HashMap::new();
    scores.insert('G', 10);
    scores.insert('Y', 5);
    scores.insert('R', 2);
    scores.insert('X', 1);

    let mut stats = GameStats::default();
    let mut config = GameConfig::default();

    /* Try to parse config file */
    let mut config_file_path = String::new();
    let mut meet_c = false;
    for arg in std::env::args() {
        if meet_c {
            config_file_path = arg.clone();
            meet_c = false;
        }
        if arg == "-c" || arg == "--config" {
            meet_c = true;
        }
    }

    if meet_c {
        return Err(Box::from("Error! Missing config file path"));
    }

    parse_config_file(&config_file_path, &mut config)?;
    parse_args(&mut config)?;

    let mut game_state = parse_states(&config)?;
    load_state_stats(&mut game_state, &config, &mut stats)?;

    /* Create FINAL and ACCEPTABLE source */
    let mut final_words: Vec<String> = Vec::new();
    if config.final_set != "".to_string() {
        final_words = read_word_list(config.final_set.as_str())?;
        final_words.sort();
    } else {
        final_words = FINAL.iter().map(|w| w.to_string()).collect();
    }

    let mut acceptable_words: Vec<String> = Vec::new();
    if config.acceptable_set != "".to_string() {
        acceptable_words = read_word_list(config.acceptable_set.as_str())?;
        acceptable_words.sort();
    } else {
        acceptable_words = ACCEPTABLE.iter().map(|w| w.to_string()).collect();
    }

    /* Check FINAL & ACCEPTABLE coherence */
    check_final_accept_cohe(&final_words, &acceptable_words)?;

    /* Init random seed */
    let mut rng = rand::rngs::StdRng::seed_from_u64(config.seed);
    final_words.shuffle(&mut rng);

    loop {
        /* Get answer word */
        let answer_word = match next_answer(&mut config, &final_words)? {
            Some(answer_word) => answer_word,
            None => break,
        };

        let mut total_guessing_count = 0;
        let mut guessing_history: Vec<String> = Vec::new();
        let mut success = 0;

        let mut keyboard_status: Vec<char> = vec!['X'; 26];

        let mut last_user_guess_status = vec!['X'; 5];
        let mut last_user_guess_word = vec!['X'; 5];
        let answer_chars: Vec<char> = answer_word.chars().collect();
        let mut answer_count: Vec<i32> = vec![0; 26];
        for c in answer_chars.iter() {
            answer_count[*c as usize - 'a' as usize] += 1;
        }

        while total_guessing_count < 6 {
            let mut user_guess = String::new();
            if stdin()
                .read_line(&mut user_guess)
                .expect("Fail to readline")
                == 0
            {
                break;
            }
            user_guess = user_guess.trim().to_lowercase();
            if user_guess.len() != 5 || !acceptable_words.contains(&user_guess) {
                println!("INVALID");
                continue;
            }

            let guess_chars: Vec<char> = user_guess.chars().collect();
            let mut colored: Vec<i32> = vec![0; 26];
            /* Update keyboard status */
            let mut user_status: Vec<char> = vec!['X'; 5];

            /* Green */
            make_green(&mut user_status, &mut colored, &guess_chars, &answer_chars);

            /* Yellow */
            make_yellow(&mut user_status, &mut colored, &guess_chars, &answer_count);

            let mut fail_difficult = 0;
            check_difficult(
                &config.difficult,
                &mut fail_difficult,
                &total_guessing_count,
                &last_user_guess_status,
                &last_user_guess_word,
                &user_status,
                &guess_chars,
            );

            /* Failed difficult mode or difficult mode is not on */
            if fail_difficult == 1 {
                continue;
            }

            /* Keyboard status */
            update_keyboard_status(&mut keyboard_status, &user_status, &guess_chars, &scores);

            /* print result */
            print_result(&user_status, &keyboard_status);

            guessing_history.push(user_guess.clone());
            *stats.guess_frequency.entry(user_guess.clone()).or_insert(0) += 1;
            total_guessing_count += 1;

            last_user_guess_status = user_status.clone();
            last_user_guess_word = guess_chars.clone();

            if user_guess == answer_word {
                success = 1;
                stats.total_success_guess_try += total_guessing_count as f64;
                break;
            }
        }

        stats.total_game_cnt += 1.0;
        stats.success_game_cnt += success as f32;

        // if !is_tty {
        if success == 0 {
            println!("FAILED {}", answer_word.to_uppercase());
        } else {
            println!("CORRECT {total_guessing_count}");
        }
        // }
        // println!("SUCCESS: {success}, is_tty: {is_tty}, answer: {}", answer_word.to_uppercase());
        // println!("total: {total_game_cnt}, success: {success_game_cnt}");

        /* print statistics */
        print_statistics(&config, &stats);

        /* Dump json file/state */
        if config.state != "".to_string() {
            let saved_guesses: Vec<String> = guessing_history
                .iter()
                .map(|guess| guess.to_uppercase())
                .collect();
            let game_record = json!({
                "answer": answer_word.to_uppercase(),
                "guesses": saved_guesses
            });

            let rounds = {
                let games = game_state["games"]
                    .as_array_mut()
                    .ok_or("Error! invalid state games")?;
                games.push(game_record);
                games.len()
            };
            game_state["total_rounds"] = json!(rounds);

            dump_json_file(&game_state, &config.state)?;
        }

        if config.has_word_arg {
            break;
        } else {
            let mut yn = String::new();
            let result = io::stdin().read_line(&mut yn);
            match result {
                Err(_) => panic!("Fail to read line!"),
                Ok(0) => break,
                Ok(_) => {
                    if yn.trim() == "N" {
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}
