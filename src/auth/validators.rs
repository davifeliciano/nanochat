pub fn is_valid_username(username: &str) -> bool {
    username.len() >= 2
        && username.len() <= 32
        && username
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c == '-' || c == '_')
}

pub fn is_valid_password(password: &str) -> bool {
    let alphabetic_count = password.chars().filter(|c| c.is_alphabetic()).count();
    let ascii_digit_count = password.chars().filter(|c| c.is_ascii_digit()).count();
    let other_count = password.len() - alphabetic_count - ascii_digit_count;

    password.len() >= 12
        && password.len() <= 64
        && alphabetic_count > 0
        && ascii_digit_count > 0
        && other_count > 0
}
