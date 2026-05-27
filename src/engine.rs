use std::collections::HashMap;

pub fn update_keyboard_status(
    keyboard_status: &mut Vec<char>,
    user_status: &Vec<char>,
    guess_chars: &Vec<char>,
    scores: &HashMap<char, i32>,
) {
    for i in 0..5 {
        let idx = guess_chars[i] as usize - 'a' as usize;
        if scores.get(&keyboard_status[idx]) < scores.get(&user_status[i]) {
            keyboard_status[idx] = user_status[i];
        }
    }
}

pub fn make_yellow(
    user_status: &mut Vec<char>,
    colored: &mut Vec<i32>,
    guess_chars: &Vec<char>,
    answer_count: &Vec<i32>,
) {
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

pub fn make_green(
    user_status: &mut Vec<char>,
    colored: &mut Vec<i32>,
    guess_chars: &Vec<char>,
    answer_chars: &Vec<char>,
) {
    for i in 0..5 {
        user_status[i] = if guess_chars[i] == answer_chars[i] {
            let idx = guess_chars[i] as usize - 'a' as usize;
            colored[idx] += 1;
            'G'
        } else {
            'R'
        };
    }
}

pub fn check_difficult(
    difficult_on: &bool,
    fail_difficult: &mut i32,
    total_guessing_count: &i32,
    last_user_guess_status: &Vec<char>,
    last_user_guess_word: &Vec<char>,
    user_status: &Vec<char>,
    guess_chars: &Vec<char>,
) {
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
            last_cnt[idx_1] +=
                if last_user_guess_status[i] == 'Y' || last_user_guess_status[i] == 'G' {
                    1
                } else {
                    0
                };
            curr_cnt[idx_2] += if user_status[i] == 'Y' || user_status[i] == 'G' {
                1
            } else {
                0
            };
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
