use std::io::stdin;
use crate::config::GameConfig;

struct RoundResult {
    success: bool,
    guess_count: i32,
    guesses: Vec<String>,
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
