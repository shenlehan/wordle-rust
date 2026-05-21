use console;
use std::io::{self, stdin, Write};
use std::collections::HashMap;
use std::hash::Hash;
use log::info;
use crate::builtin_words::{ACCEPTABLE, FINAL};
use rand::seq::SliceRandom;
use rand::SeedableRng;

mod builtin_words;

fn top_n_keys<K: Clone, V: Ord>(map: &HashMap<K, V>, n: usize) -> Vec<K> {
    let mut pairs: Vec<(&K, &V)> = map.iter().collect();

    pairs.sort_by(|a, b| b.1.cmp(a.1));

    pairs.into_iter()
      .take(n)
      .map(|(k, _v)| k.clone())
      .collect()
}

fn parse_args(meet_word_argument: &mut i32, random_mode: &mut i32, word_argument: &mut String,
              difficult_on: &mut i32, print_stats: &mut i32, meet_day_argument: &mut i32, day: &mut usize,
              meet_seed_argument: &mut i32, seed: &mut u64) {
    for arg in std::env::args() {
        if *meet_word_argument == 1 {
            *word_argument = arg.clone();
            *meet_word_argument = 0;
        } else if *meet_day_argument == 1 {
            *day = arg.clone().parse().unwrap();
            *meet_day_argument = 0;
        } else if *meet_seed_argument == 1 {
            *seed = arg.clone().parse().unwrap();
            *meet_seed_argument = 0;
        } else if arg == String::from("-w") || arg == String::from("--word") {
            *meet_word_argument = 1;
        } else if arg == String::from("-r") || arg == String::from("--random") {
            *random_mode = 1;
        } else if arg == String::from("-D") || arg == String::from("--difficult") {
            *difficult_on = 1;
        } else if arg == String::from("-t") || arg == String::from("--stats") {
            *print_stats = 1;
        } else if arg == String::from("-d") || arg == String::from("--day") {
            *meet_day_argument = 1;
        } else if arg == String::from("-s") || arg == String::from("--seed") {
            *meet_seed_argument = 1;
        }
    }
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

fn check_difficult(difficult_on: &i32, fail_difficult: &mut i32, total_guessing_count: &i32, last_user_guess_status: &Vec<char>,
                   last_user_guess_word: &Vec<char>, user_status: &Vec<char>, guess_chars: &Vec<char>) {
    if *difficult_on == 0 {
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


/// The main function for the Wordle game, implement your own logic here
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let is_tty = atty::is(atty::Stream::Stdout);

    // if is_tty {
    //     println!(
    //         "I am in a tty. Please print {}!",
    //         console::style("colorful characters").bold().blink().blue()
    //     );
    // } else {
    //     println!("I am not in a tty. Please print according to test requirements!");
    // }
    //
    // if is_tty {
    //     print!("{}", console::style("Your name: ").bold().red());
    //     io::stdout().flush().unwrap();
    // }
    // let mut line = String::new();
    // io::stdin().read_line(&mut line)?;
    // println!("Welcome to wordle, {}!", line.trim());

    // example: print arguments
    // print!("Command line arguments: ");
    // for arg in std::env::args() {
    //     print!("{} ", arg);
    // }
    // println!("");
    // TODO: parse the arguments in `args`

    let mut scores = HashMap::new();
    scores.insert('G', 10);
    scores.insert('Y', 5);
    scores.insert('R', 2);
    scores.insert('X', 1);

    let mut success_game_cnt: f32 = 0.0;
    let mut total_game_cnt: f32 = 0.0;
    let mut total_success_guess_try = 0.0;

    let mut guess_frequency: HashMap<String, i32> = HashMap::new();

    while true {
        let mut meet_word_argument = 0;
        let mut random_mode = 0;
        let mut word_argument = String::new();
        let mut keyboard_status: Vec<char> = vec!['X'; 26];
        let mut difficult_on = 0;
        let mut print_stats = 0;
        let mut meet_day_argument = 0;
        let mut day: usize = 0;
        let mut meet_seed_argument = 0;
        let mut seed: u64 = 0;
        let mut need_input_answer = 0;

        parse_args(&mut meet_word_argument, &mut random_mode, &mut word_argument, &mut difficult_on,
                   &mut print_stats, &mut meet_day_argument, &mut day, &mut meet_seed_argument, &mut seed);

        if meet_word_argument == 1 {
            return Err(Box::from("Error! missing word argument"));
        }

        if meet_day_argument == 1 {
            return Err(Box::from("Error! Missing day argument"));
        }

        /* Init random seed */
        let mut words = FINAL.to_vec();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        words.shuffle(&mut rng);

        /* Get answer word */
        let mut answer_word = String::new();
        if random_mode == 1 {
            answer_word = String::from(words[day - 1]);
        } else {
            if word_argument == "" {
                stdin().read_line(&mut word_argument).expect("Fail to read line");
                need_input_answer = 1;
            }
            answer_word = word_argument.trim().to_lowercase();
        }

        let mut total_guessing_count = 0;
        let mut guessing_history: Vec<String> = Vec::new();
        let mut success = 0;

        let mut last_user_guess_status = vec!['X'; 5];
        let mut last_user_guess_word = vec!['X'; 5];
        let answer_chars: Vec<char> = answer_word.chars().collect();
        let mut answer_count: Vec<i32> = vec![0; 26];
        for c in answer_chars.iter() {
            answer_count[*c as usize - 'a' as usize] += 1;
        }

        while total_guessing_count < 6 {
            let mut user_guess = String::new();
            stdin().read_line(&mut user_guess).expect("Fail to readline");
            user_guess = user_guess.trim().to_lowercase();
            if user_guess.len() != 5 || !ACCEPTABLE.contains(&user_guess.as_str()) {
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
            check_difficult(&difficult_on, &mut fail_difficult, &total_guessing_count, &last_user_guess_status, &last_user_guess_word,
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
        if print_stats == 1 {
            /* print basic */
            let x = if success_game_cnt == 0.0 { 0.0 } else { total_success_guess_try / success_game_cnt as f64 };
            println!("{success_game_cnt} {} {:.2}", total_game_cnt - success_game_cnt, x);

            /* print word frequency */
            let top_words = top_n_keys(&guess_frequency, 5);
            for word in top_words {
                print!("{} {} ", word.to_uppercase(), guess_frequency.get(&word).unwrap());
                io::stdout().flush().unwrap();
            }
            println!("");
        }

        if need_input_answer == 1 {
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
