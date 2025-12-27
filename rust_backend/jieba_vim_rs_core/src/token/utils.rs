use super::tokenize::{Token, TokenLike};

/// Get the index of the token in `tokens` that covers `col`. Return `None` if
/// `col` is to the right of the last token.
pub fn index_tokens(tokens: &[Token], col: usize) -> Option<usize> {
    use std::cmp::Ordering;
    tokens
        .binary_search_by(|tok| {
            if col < tok.first_char() {
                Ordering::Greater
            } else if col >= tok.last_char1() {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .ok()
}

/// Try to convert `c` to an ASCII. If failed, give back `c`.
pub fn ascii_or(c: char) -> Option<u8> {
    if c as u32 <= u8::MAX as u32 {
        Some(c as u8)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::index_tokens;

    #[test]
    fn test_index_tokens() {
        assert_eq!(index_tokens(&[], 0), None);
    }
}
