use console;
use log::info;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde_json::json;
use std::collections::HashMap;
use std::io::{self, stdin, Write};
use wordle::parse::*;
use wordle::builtin_words::*;
use wordle::config::*;
use wordle::engine::*;
use wordle::game::{next_answer, play_one_round};
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
    parse_config_file(&mut config)?;
    parse_args(&mut config)?;

    let mut game_state = parse_states(&config)?;
    load_state_stats(&mut game_state, &config, &mut stats)?;

    /* Create FINAL and ACCEPTABLE source */
    let mut final_words: Vec<String> = Vec::new();
    let mut acceptable_words: Vec<String> = Vec::new();
    get_strings(&mut final_words, &config.final_set, FINAL)?;
    get_strings(&mut acceptable_words, &config.acceptable_set, ACCEPTABLE)?;

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

        let result = play_one_round(&answer_word, &acceptable_words, &config, &scores)?;
        update_stats(&mut stats, &result);

        // if !is_tty {
        if !result.success {
            println!("FAILED {}", answer_word.to_uppercase());
        } else {
            println!("CORRECT {}", result.guess_count);
        }
        // }
        // println!("SUCCESS: {success}, is_tty: {is_tty}, answer: {}", answer_word.to_uppercase());
        // println!("total: {total_game_cnt}, success: {success_game_cnt}");

        /* print statistics */
        print_statistics(&config, &stats);

        /* Dump json file/state */
        if config.state != "".to_string() {
            let saved_guesses: Vec<String> = result
                .guesses
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
