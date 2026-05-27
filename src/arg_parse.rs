use crate::config::*;

pub fn parse_args(config: &mut GameConfig) -> Result<(), Box<dyn std::error::Error>> {
  let mut meet_word_argument = 0;
  let mut meet_day_argument = 0;
  let mut meet_seed_argument = 0;
  let mut meet_f_argument = 0;
  let mut meet_accept_argument = 0;
  let mut meet_state_argument = 0;
  let mut meet_config_argument = 0;

  let mut has_word_arg = 0;
  let mut random_mode = 0;
  let mut has_day_arg = 0;
  let mut has_seed_arg = 0;

  for arg in std::env::args() {
    if meet_word_argument == 1 {
      config.word = arg.clone();
      config.has_word_arg = true;
      meet_word_argument = 0;
    } else if meet_day_argument == 1 {
      config.day = arg.clone().parse().unwrap();
      meet_day_argument = 0;
    } else if meet_seed_argument == 1 {
      config.seed = arg.clone().parse().unwrap();
      meet_seed_argument = 0;
    } else if meet_f_argument == 1 {
      config.final_set = arg.clone();
      meet_f_argument = 0;
    } else if meet_accept_argument == 1 {
      config.acceptable_set = arg.clone();
      meet_accept_argument = 0;
    } else if meet_state_argument == 1 {
      config.state = arg.clone();
      meet_state_argument = 0;
    } else if meet_config_argument == 1 {
      config.config = arg.clone();
      meet_config_argument = 0;
    } else if arg == String::from("-w") || arg == String::from("--word") {
      has_word_arg = 1;
      meet_word_argument = 1;
    } else if arg == String::from("-r") || arg == String::from("--random") {
      random_mode = 1;
      config.random = true;
    } else if arg == String::from("-D") || arg == String::from("--difficult") {
      config.difficult = true;
    } else if arg == String::from("-t") || arg == String::from("--stats") {
      config.stats = true;
    } else if arg == String::from("-d") || arg == String::from("--day") {
      has_day_arg = 1;
      meet_day_argument = 1;
    } else if arg == String::from("-s") || arg == String::from("--seed") {
      has_seed_arg = 1;
      meet_seed_argument = 1;
    } else if arg == String::from("-f") || arg == String::from("--final-set") {
      meet_f_argument = 1;
    } else if arg == String::from("-a") || arg == String::from("--acceptable-set") {
      meet_accept_argument = 1;
    } else if arg == String::from("-S") || arg == String::from("--state") {
      meet_state_argument = 1;
    } else if arg == String::from("-c") || arg == String::from("--config") {
      meet_config_argument = 1;
    }
  }

  if meet_word_argument == 1 {
    return Err(Box::from("Error! Missing word argument"));
  }

  if meet_day_argument == 1 {
    return Err(Box::from("Error! Missing day argument"));
  }

  if meet_seed_argument == 1 {
    return Err(Box::from("Error! Missing seed argument"));
  }

  if meet_accept_argument == 1 {
    return Err(Box::from("Error! Missing acceptable-set argument"));
  }

  if meet_f_argument == 1 {
    return Err(Box::from("Error! Missing final-set argument"));
  }

  if meet_state_argument == 1 {
    return Err(Box::from("Error! Missing state argument"));
  }

  if has_word_arg == 1 && (random_mode == 1 || has_day_arg == 1 || has_seed_arg == 1) {
    return Err(Box::from("Error! conflict arguments"));
  }

  if random_mode == 0 && (has_day_arg == 1 || has_seed_arg == 1) {
    return Err(Box::from("Error! seed/day require random mode"));
  }

  Ok(())
}