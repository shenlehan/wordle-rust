use console;
use std::io::{self, stdin, Write};
use std::collections::HashMap;
use std::hash::Hash;
use crate::builtin_words::ACCEPTABLE;

mod builtin_words;

fn parse_args(meet_word_argument: &mut i32, random_mode: &mut i32, word_argument: &mut String,
              difficult_on: &mut i32, print_stats: &mut i32) {
    for arg in std::env::args() {
        if *meet_word_argument == 1 {
            *word_argument = arg.clone();
            *meet_word_argument = 0;
        } else if arg == String::from("-w") || arg == String::from("--word") {
            *meet_word_argument = 1;
        } else if arg == String::from("-r") || arg == String::from("--random") {
            *random_mode = 1;
        } else if arg == String::from("-D") || arg == String::from("--difficult") {
            *difficult_on = 1;
        } else if arg == String::from("-t") || arg == String::from("--stats") {
            *print_stats = 1;
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
    let mut meet_word_argument = 0;
    let mut random_mode = 0;
    let mut word_argument = String::new();
    let mut keyboard_status: Vec<char> = vec!['X'; 26];
    let mut difficult_on = 0;
    let mut print_stats = 0;

    let mut scores = HashMap::new();
    scores.insert('G', 10);
    scores.insert('Y', 5);
    scores.insert('R', 2);
    scores.insert('X', 1);

    parse_args(&mut meet_word_argument, &mut random_mode, &mut word_argument, &mut difficult_on,
               &mut print_stats);

    if meet_word_argument == 1 {
        return Err(Box::from("Error! missing word argument"));
    }

    let mut answer_word = String::new();
    if random_mode == 1 {
        // println!("{}", word_argument);
        // todo!();
    } else {
        if word_argument == "" {
            stdin().read_line(&mut word_argument).expect("Fail to read line");
        }
        answer_word = word_argument.trim().to_lowercase();
    }

    let mut total_guessing_count = 0;
    let mut guessing_history : Vec<String> = Vec::new();
    let mut success = 0;

    let answer_chars: Vec<char> = answer_word.chars().collect();
    let mut answer_count: Vec<i32> = vec![0; 26];
    for c in answer_chars.iter() {
        answer_count[*c as usize - 'a' as usize] += 1;
    }

    while total_guessing_count < 6 {
        let mut user_guess = String::new();
        stdin().read_line(&mut user_guess).expect("Fail to readline");
        user_guess = user_guess.trim().to_lowercase();
        if user_guess.len() != 5 || !ACCEPTABLE.contains(&user_guess.as_str()){
            println!("INVALID");
            continue;
        }

        guessing_history.push(user_guess.clone());
        total_guessing_count += 1;

        let guess_chars: Vec<char> = user_guess.chars().collect();
        let mut colored: Vec<i32> = vec![0; 26];
        /* Update keyboard status */
        let mut user_status: Vec<char> = vec!['X'; 5];

        /* Green */
        make_green(&mut user_status, &mut colored, &guess_chars, &answer_chars);

        /* Yellow */
        make_yellow(&mut user_status, &mut colored, &guess_chars, &answer_count);

        /* Keyboard status */
        update_keyboard_status(&mut keyboard_status, &user_status, &guess_chars, &scores);

        print_result(&user_status, &keyboard_status);

        if user_guess == answer_word {
            success = 1;
            break;
        }
    }

    if !is_tty {
        if success == 0 {
            println!("FAILED {}", answer_word.to_uppercase());
        } else {
            println!("CORRECT {total_guessing_count}");
        }
    }
    Ok(())
}
