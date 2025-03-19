pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut start = 0;

    while start < text.len() {
        let end = (start + chunk_size).min(text.len());
        result.push(text[start..end].to_string());
        start += chunk_size;
    }

    result
}
