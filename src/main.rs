use console;
use std::io::{self, stdin, Write};
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use log::info;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde_json::json;
use wordle::builtin_words::*;
use wordle::config::*;
use wordle::arg_parse::*;

fn top_n_keys<K: Clone + Ord, V: Ord>(map: &HashMap<K, V>, n: usize) -> Vec<K> {
    let mut pairs: Vec<(&K, &V)> = map.iter().collect();

    pairs.sort_by(|a, b| match b.1.cmp(a.1) {
        std::cmp::Ordering::Equal => a.0.cmp(b.0),
        other => other,
    });

    pairs.into_iter()
      .take(n)
      .map(|(k, _v)| k.clone())
      .collect()
}

fn read_word_list(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let words = content
      .lines()
      .map(|line| line.trim().to_lowercase())
      .collect();

    Ok(words)
}

fn print_result(user_status: &Vec<char>, keyboard_status: &Vec<char>) {
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
    println!("");
}

fn update_keyboard_status(keyboard_status: &mut Vec<char>, user_status: &Vec<char>,
                          guess_chars: &Vec<char>, scores: &HashMap<char, i32>) {
    for i in 0..5 {
        let idx = guess_chars[i] as usize - 'a' as usize;
        if scores.get(&keyboard_status[idx]) < scores.get(&user_status[i]) {
            keyboard_status[idx] = user_status[i];
        }
    }
}

fn make_yellow(user_status: &mut Vec<char>, colored: &mut Vec<i32>, guess_chars: &Vec<char>,
               answer_count: &Vec<i32>) {
    for i in 0..5 {
        let idx = guess_chars[i] as usize - 'a' as usize;
        if user_status[i] != 'G' {
            if answer_count[idx] > 0 && colored[idx] < answer_count[idx] {
                user_status[i] = 'Y';
                colored[idx] += 1;
            }
        }
    }
}

fn make_green(user_status: &mut Vec<char>, colored: &mut Vec<i32>, guess_chars: &Vec<char>,
              answer_chars: &Vec<char>) {
    for i in 0..5 {
        user_status[i] = if guess_chars[i] == answer_chars[i] {
            let idx = guess_chars[i] as usize - 'a' as usize;
            colored[idx] += 1;
            'G'
        } else { 'R' };
    }
}

fn check_difficult(difficult_on: &bool, fail_difficult: &mut i32, total_guessing_count: &i32, last_user_guess_status: &Vec<char>,
                   last_user_guess_word: &Vec<char>, user_status: &Vec<char>, guess_chars: &Vec<char>) {
    if *difficult_on == false {
       return;
    }

    /* Must use all green */
    for i in 0..5 {
        if last_user_guess_status[i] == 'G' && user_status[i] != 'G' {
            // println!("Error at here: 172, i={i}");
            *fail_difficult = 1;
            break;
        }
    }

    /* Must use all yellow */
    /* For a certain char C, the yellow number of C + green number of C >= last time total sum */

    if *total_guessing_count >= 1 {
        let mut last_cnt = vec![0; 26];
        let mut curr_cnt = vec![0; 26];

        for i in 0..5 {
            let idx_1 = last_user_guess_word[i] as usize - 'a' as usize;
            let idx_2 = guess_chars[i] as usize - 'a' as usize;
            last_cnt[idx_1] += if last_user_guess_status[i] == 'Y' || last_user_guess_status[i] == 'G' { 1 } else { 0 };
            curr_cnt[idx_2] += if user_status[i] == 'Y' || user_status[i] == 'G' { 1 } else { 0 };
        }

        // println!("Last user status: {:#?}", last_user_guess_status);
        // println!("Curr user status: {:#?}", user_status);
        // println!("Last: {:#?}", last_cnt);
        // println!("Curr: {:#?}", curr_cnt);

        for i in 0..26 {
            if last_cnt[i] > curr_cnt[i] {
                *fail_difficult = 1;
                break;
            }
        }
    }

    if *fail_difficult == 1 {
        println!("INVALID");
    }
}

fn dump_json_file(
    game_state: &serde_json::Value,
    state_file: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(state_file.as_str())?;
    serde_json::to_writer_pretty(file, &game_state)?;
    Ok(())
}

/// The main function for the Wordle game, implement your own logic here
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let is_tty = atty::is(atty::Stream::Stdout);

    let mut scores = HashMap::new();
    scores.insert('G', 10);
    scores.insert('Y', 5);
    scores.insert('R', 2);
    scores.insert('X', 1);

    let mut success_game_cnt: f32 = 0.0;
    let mut total_game_cnt: f32 = 0.0;
    let mut total_success_guess_try = 0.0;

    let mut guess_frequency: HashMap<String, i32> = HashMap::new();

    let mut config = GameConfig::new(false, false, false,
                                         1, 0, "".to_string(),
                                         "".to_string(), "".to_string(), "".to_string(), false, "".to_string());

    let mut file_config_path = String::new();
    let mut meet_c = false;
    for arg in std::env::args() {
        if meet_c {
            file_config_path = arg.clone();
            meet_c = false;
        }
        if arg == "-c" || arg == "--config" {
            // file_config_path = arg.clone();
            // break;
            meet_c = true;
        }
    }

    if meet_c {
        return Err(Box::from("Error! Missing config file path"));
    }

    if file_config_path != "" {
        let content = std::fs::read_to_string(file_config_path.as_str())?;
        let value: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(random) = value["random"].as_bool() {
            config.random = random;
        }
        if let Some(difficult) = value["difficult"].as_bool() {
            config.difficult = difficult;
        }
        if let Some(stats) = value["stats"].as_bool() {
            config.stats = stats;
        }
        if let Some(day) = value["day"].as_u64() {
            config.day = day as usize;
        }
        if let Some(seed) = value["seed"].as_u64() {
            config.seed = seed;
        }
        if let Some(final_set) = value["final_set"].as_str() {
            config.final_set = final_set.to_string();
        }
        if let Some(acceptable_set) = value["acceptable_set"].as_str() {
            config.acceptable_set = acceptable_set.to_string();
        }
        if let Some(state) = value["state"].as_str() {
            config.state = state.to_string();
        }
        if let Some(word) = value["word"].as_str() {
            config.has_word_arg = true;
            config.word = word.to_string();
        }
    }

    parse_args(&mut config)?;
    // if has_config_file == 1 {
    //     let content = std::fs::read_to_string(config_file.as_str())?;
    //     let config: serde_json::Value = serde_json::from_str(&content)?;
    //
    //     if random_mode == 0 {
    //         todo!();
    //     }
    // }

    let mut game_state = if config.state != "".to_string() {
        let content = std::fs::read_to_string(config.state.as_str())?;
        let value: serde_json::Value = serde_json::from_str(&content)?;
        if value.as_object().map(|object| object.is_empty()).unwrap_or(false) {
            json!({
                "total_rounds": 0,
                "games": []
            })
        } else {
            value
        }
    } else {
        json!({
            "total_rounds": 0,
            "games": []
        })
    };

    if config.state != "".to_string() {
        let games = game_state["games"]
            .as_array()
            .ok_or("Error! invalid state games")?;
        total_game_cnt = games.len() as f32;
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
                *guess_frequency.entry(guess).or_insert(0) += 1;
            }
            if guesses
                .last()
                .and_then(|guess| guess.as_str())
                .map(|guess| guess.to_lowercase() == answer)
                .unwrap_or(false)
            {
                success_game_cnt += 1.0;
                total_success_guess_try += guesses.len() as f64;
            }
        }
    }

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
    for word in final_words.iter() {
        if !acceptable_words.contains(word) {
            return Err(Box::from("Error! FINAL is not a subset of ACCEPTABLE!"));
        }
    }

    /* Init random seed */
    let mut rng = rand::rngs::StdRng::seed_from_u64(config.seed);
    final_words.shuffle(&mut rng);

    loop {
        /* Get answer word */
        let answer_word = if config.random == true {
            if config.day == 0 || config.day > final_words.len() {
                return Err(Box::from("Error! day out of range"));
            }
            let word = String::from(final_words[config.day - 1].clone());
            config.day += 1;
            word
        } else if config.has_word_arg {
            config.word.trim().to_lowercase()
        } else {
            let mut answer_input = String::new();
            if stdin().read_line(&mut answer_input)? == 0 {
                break;
            }
            answer_input.trim().to_lowercase()
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
            if stdin().read_line(&mut user_guess).expect("Fail to readline") == 0 {
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
            check_difficult(&config.difficult, &mut fail_difficult, &total_guessing_count, &last_user_guess_status, &last_user_guess_word,
                            &user_status, &guess_chars);

            /* Failed difficult mode or difficult mode is not on */
            if fail_difficult == 1 {
                continue;
            }

            /* Keyboard status */
            update_keyboard_status(&mut keyboard_status, &user_status, &guess_chars, &scores);

            /* print result */
            print_result(&user_status, &keyboard_status);

            guessing_history.push(user_guess.clone());
            *guess_frequency.entry(user_guess.clone()).or_insert(0) += 1;
            total_guessing_count += 1;

            last_user_guess_status = user_status.clone();
            last_user_guess_word = guess_chars.clone();

            if user_guess == answer_word {
                success = 1;
                total_success_guess_try += total_guessing_count as f64;
                break;
            }
        }

        total_game_cnt += 1.0;
        success_game_cnt += success as f32;

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
        if config.stats == true {
            /* print basic */
            let x = if success_game_cnt == 0.0 { 0.0 } else { total_success_guess_try / success_game_cnt as f64 };
            println!("{success_game_cnt} {} {:.2}", total_game_cnt - success_game_cnt, x);

            /* print word frequency */
            let top_words = top_n_keys(&guess_frequency, 5);
            let mut word_stats: Vec<String> = Vec::new();
            for word in top_words {
                word_stats.push(format!("{} {}", word.to_uppercase(), guess_frequency.get(&word).unwrap()));
            }
            println!("{}", word_stats.join(" "));
        }

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
