pub fn clean_input(input: &str) -> &str {
    input.trim().trim_start_matches('\u{feff}')
}
