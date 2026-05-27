pub fn read_word_list(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let words = content
        .lines()
        .map(|line| line.trim().to_lowercase())
        .collect();

    Ok(words)
}
