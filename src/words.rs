pub fn get_strings(words: &mut Vec<String>, source: &String, default_source: &[&str])
-> Result<(), Box<dyn std::error::Error>> {
    if source != "" {
        *words = read_word_list(source.as_str())?;
        words.sort();
    } else {
        *words = default_source.iter().map(|w| w.to_string()).collect();
    }
    Ok(())
}

pub fn read_word_list(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let words = content
        .lines()
        .map(|line| line.trim().to_lowercase())
        .collect();

    Ok(words)
}
