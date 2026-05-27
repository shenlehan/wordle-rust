pub struct GameConfig {
    pub random: bool,
    pub difficult: bool,
    pub stats: bool,
    pub day: usize,
    pub seed: u64,
    pub final_set: String,
    pub acceptable_set: String,
    pub state: String,
    pub word: String,
    pub has_word_arg: bool,
    pub config: String,
}

impl GameConfig {
    pub fn new(
        random: bool,
        difficult: bool,
        stats: bool,
        day: usize,
        seed: u64,
        final_set: String,
        acceptable_set: String,
        state: String,
        word: String,
        has_word_arg: bool,
        config: String,
    ) -> GameConfig {
        GameConfig {
            random,
            difficult,
            stats,
            day,
            seed,
            final_set,
            acceptable_set,
            state,
            word,
            has_word_arg,
            config,
        }
    }

}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            random: false,
            difficult: false,
            stats: false,
            day: 1,
            seed: 0,
            final_set: String::new(),
            acceptable_set: String::new(),
            state: String::new(),
            word: String::new(),
            has_word_arg: false,
            config: String::new(),
        }
    }
}
