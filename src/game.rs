use std::collections::HashMap;
use std::io::stdin;
use crate::config::GameConfig;
use crate::engine::{check_difficult, make_green, make_yellow, update_keyboard_status};
use crate::utilities::print_result;

pub struct RoundResult {
    pub success: bool,
    pub guess_count: i32,
    pub guesses: Vec<String>,
}

impl Default for RoundResult {
    fn default() -> Self {
       Self {
           success: false,
           guess_count: 0,
           guesses: Vec::new()
       }
    }
}

pub fn next_answer(
    config: &mut GameConfig,
    final_words: &Vec<String>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
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
            return Ok(None);
        }
        answer_input.trim().to_lowercase()
    };

    Ok(Some(answer_word))
}

pub fn play_one_round(
    answer_word: &str,
    acceptable_words: &Vec<String>,
    config: &GameConfig,
    scores: &HashMap<char, i32>,
) -> Result<RoundResult, Box<dyn std::error::Error>> {
    let mut total_guessing_count = 0;
    let mut guessing_history: Vec<String> = Vec::new();
    let mut success = false;

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
        total_guessing_count += 1;

        last_user_guess_status = user_status.clone();
        last_user_guess_word = guess_chars.clone();

        if user_guess == answer_word {
            success = true;
            break;
        }
    }

    Ok(RoundResult {
        success,
        guess_count: total_guessing_count,
        guesses: guessing_history,
    })
}
