use crate::punc;
use const_format::concatcp;
use jieba_rs::Jieba;
use once_cell::sync::Lazy;
use pyo3::prelude::*;
use pyo3::types::PySequence;
use regex::Regex;
use std::io::BufReader;

/// Types of tokens.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum TokenType {
    /// Whitespace characters.
    Space,
    /// Chinese punctuations.
    Punc,
    /// Other non-word characters.
    NonWord,
    /// Everything else (including alphanum).
    Hans,
}

impl From<&str> for TokenType {
    fn from(value: &str) -> Self {
        static PAT_SPACE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\s+").unwrap());
        static PAT_NONWORD: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"\W+").unwrap());
        const PUNC_CLASS: &str = concatcp!("[", punc::PUNCTUATION, "]");
        static PAT_PUNC: Lazy<Regex> =
            Lazy::new(|| Regex::new(PUNC_CLASS).unwrap());

        if value.is_empty() || PAT_SPACE.is_match(value) {
            TokenType::Space
        } else if PAT_PUNC.is_match(value) {
            TokenType::Punc
        } else if PAT_NONWORD.is_match(value) {
            TokenType::NonWord
        } else {
            TokenType::Hans
        }
    }
}

impl TokenType {
    #[inline]
    fn is_space(&self) -> bool {
        match self {
            TokenType::Space => true,
            _ => false,
        }
    }
}

/// Each token is parsed as a `ParsedToken`. If `start_byte_index` is larger
/// than `end_byte_index`, it means that the underlying token is an empty
/// string.
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedToken {
    /// Byte index of the first character of the token.
    start_byte_index: usize,
    /// Byte index of the last character of the token.
    end_byte_index: usize,
    /// Type of the token.
    token_type: TokenType,
}

/// Parse a list of tokens into a list of `ParsedToken`s. We assume that no
/// token provided is empty.
fn parse_tokens(tokens: &[&str]) -> Vec<ParsedToken> {
    let mut cum_byte_index: usize = 0;
    tokens
        .iter()
        .map(|tok| {
            let start_byte_index = cum_byte_index;
            let token_type = TokenType::from(*tok);
            cum_byte_index += tok.len();
            let end_byte_index =
                cum_byte_index - tok.chars().last().unwrap().len_utf8();
            ParsedToken {
                start_byte_index,
                end_byte_index,
                token_type,
            }
        })
        .collect()
}

/// Merge and transform `elements` according to `rule_func` by pushing them
/// iteratively to a stack.
///
/// `rule_func` should be a function that takes `e1` and `e2` as arguments and
/// returns a list of transformed elements or an `Err` that gives back
/// `(e1, e2)`. `e1` will be `None` when `e2` is the first element of
/// `elements`.
fn stack_merge<T, F>(elements: Vec<T>, mut rule_func: F) -> Vec<T>
where
    F: FnMut(Option<T>, T) -> Result<Vec<T>, (Option<T>, T)>,
{
    let mut stack: Vec<T> = vec![];
    for pt in elements {
        match rule_func(stack.pop(), pt) {
            Err((None, pt)) => stack.push(pt),
            Err((Some(popped), pt)) => {
                stack.push(popped);
                stack.push(pt);
            }
            Ok(mut new_elements) => {
                stack.append(&mut new_elements);
            }
        }
    }
    stack
}

fn insert_implicit_space_rule(
    parsed_token1: Option<ParsedToken>,
    parsed_token2: ParsedToken,
) -> Result<Vec<ParsedToken>, (Option<ParsedToken>, ParsedToken)> {
    // We assume that `pt.start_byte_index` is greater than zero.
    #[inline]
    fn generate_implicit_space_in_between(pt: &ParsedToken) -> ParsedToken {
        let start_byte_index = pt.start_byte_index;
        ParsedToken {
            start_byte_index,
            end_byte_index: start_byte_index - 1,
            token_type: TokenType::Space,
        }
    }

    use TokenType::*;

    match parsed_token1 {
        None => Err((parsed_token1, parsed_token2)),
        Some(parsed_token1) => {
            let need_insert_implicit_space =
                match (parsed_token1.token_type, parsed_token2.token_type) {
                    (Hans, Hans) => true,
                    (Hans, _) => false,
                    (Punc, Space) => false,
                    (Punc, _) => true,
                    (Space, _) | (NonWord, _) => false,
                };
            if need_insert_implicit_space {
                let imp_space =
                    generate_implicit_space_in_between(&parsed_token2);
                Ok(vec![parsed_token1, imp_space, parsed_token2])
            } else {
                Err((Some(parsed_token1), parsed_token2))
            }
        }
    }
}

/// Return the token index at which the byte index `bi` lies. The returned
/// value is guaranteed in range `0..parsed_tokens.len()`. Panic if the token
/// index is not found.
fn index_tokens(parsed_tokens: &[ParsedToken], bi: usize) -> usize {
    // The reason to traverse `parsed_tokens` in reverse order is that we need
    // to index after all implicit space tokens.
    match parsed_tokens
        .iter()
        .rposition(|pt| pt.start_byte_index <= bi)
    {
        None => panic!(
            "Token index of byte index `{}` not found in parsed token `{:?}`",
            bi, parsed_tokens
        ),
        Some(ti) => ti,
    }
}

pub fn index_last_start_of_word(parsed_tokens: &[ParsedToken]) -> Option<usize> {
    if parsed_tokens.is_empty() {
        Some(0)
    } else {
        parsed_tokens
            .iter()
            .rev()
            .find_map(|pt| match pt.token_type {
                TokenType::Space => None,
                _ => Some(pt.start_byte_index),
            })
    }
}

pub fn index_prev_start_of_word(
    parsed_tokens: &[ParsedToken],
    col: usize,
) -> Option<usize> {
    if parsed_tokens.is_empty() {
        None
    } else {
        let ti = index_tokens(parsed_tokens, col);
        let take_first =
            if col == parsed_tokens.get(ti).unwrap().start_byte_index {
                ti
            } else {
                ti + 1
            };
        parsed_tokens.iter().take(take_first).rev().find_map(|pt| {
            match pt.token_type {
                TokenType::Space => None,
                _ => Some(pt.start_byte_index),
            }
        })
    }
}

#[allow(non_snake_case)]
pub fn index_last_start_of_WORD(parsed_tokens: &[ParsedToken]) -> Option<usize> {
    if parsed_tokens.is_empty() {
        Some(0)
    } else {
        parsed_tokens
            .iter()
            .rev()
            .skip_while(|pt| pt.token_type.is_space())
            .map_while(|pt| match pt.token_type {
                TokenType::Space => None,
                _ => Some(pt.start_byte_index),
            })
            .last()
    }
}

#[allow(non_snake_case)]
pub fn index_prev_start_of_WORD(
    parsed_tokens: &[ParsedToken],
    col: usize,
) -> Option<usize> {
    if parsed_tokens.is_empty() {
        None
    } else {
        let ti = index_tokens(parsed_tokens, col);
        let take_first =
            if col == parsed_tokens.get(ti).unwrap().start_byte_index {
                ti
            } else {
                ti + 1
            };
        parsed_tokens
            .iter()
            .take(take_first)
            .rev()
            .skip_while(|pt| pt.token_type.is_space())
            .map_while(|pt| match pt.token_type {
                TokenType::Space => None,
                _ => Some(pt.start_byte_index),
            })
            .last()
    }
}

pub fn index_last_end_of_word(parsed_tokens: &[ParsedToken]) -> Option<usize> {
    if parsed_tokens.is_empty() {
        Some(0)
    } else {
        parsed_tokens
            .iter()
            .rev()
            .find_map(|pt| match pt.token_type {
                TokenType::Space => None,
                _ => Some(pt.end_byte_index),
            })
    }
}

pub fn index_prev_end_of_word(
    parsed_tokens: &[ParsedToken],
    col: usize,
) -> Option<usize> {
    if parsed_tokens.is_empty() {
        None
    } else {
        let ti = index_tokens(parsed_tokens, col);
        let take_first = ti;
        parsed_tokens.iter().take(take_first).rev().find_map(|pt| {
            match pt.token_type {
                TokenType::Space => None,
                _ => Some(pt.end_byte_index),
            }
        })
    }
}

#[allow(non_snake_case)]
pub fn index_last_end_of_WORD(parsed_tokens: &[ParsedToken]) -> Option<usize> {
    index_last_end_of_word(parsed_tokens)
}

#[allow(non_snake_case)]
pub fn index_prev_end_of_WORD(
    parsed_tokens: &[ParsedToken],
    col: usize,
) -> Option<usize> {
    if parsed_tokens.is_empty() {
        None
    } else {
        let ti = index_tokens(parsed_tokens, col);
        let take_first = ti + 1;
        parsed_tokens
            .iter()
            .take(take_first)
            .rev()
            .skip_while(|pt| !pt.token_type.is_space())
            .skip_while(|pt| pt.token_type.is_space())
            .find_map(|pt| match pt.token_type {
                TokenType::Space => None,
                _ => Some(pt.end_byte_index),
            })
    }
}

pub fn index_first_start_of_word(parsed_tokens: &[ParsedToken]) -> Option<usize> {
    if parsed_tokens.is_empty() {
        Some(0)
    } else {
        parsed_tokens.iter().find_map(|pt| match pt.token_type {
            TokenType::Space => None,
            _ => Some(pt.start_byte_index),
        })
    }
}

pub fn index_next_start_of_word(
    parsed_tokens: &[ParsedToken],
    col: usize,
) -> Option<usize> {
    if parsed_tokens.is_empty() {
        None
    } else {
        let ti = index_tokens(parsed_tokens, col);
        let skip_first = ti + 1;
        parsed_tokens.iter().skip(skip_first).find_map(|pt| {
            match pt.token_type {
                TokenType::Space => None,
                _ => Some(pt.start_byte_index),
            }
        })
    }
}

#[allow(non_snake_case)]
pub fn index_first_start_of_WORD(parsed_tokens: &[ParsedToken]) -> Option<usize> {
    index_first_start_of_word(parsed_tokens)
}

#[allow(non_snake_case)]
pub fn index_next_start_of_WORD(
    parsed_tokens: &[ParsedToken],
    col: usize,
) -> Option<usize> {
    if parsed_tokens.is_empty() {
        None
    } else {
        let ti = index_tokens(parsed_tokens, col);
        let skip_first = ti;
        parsed_tokens
            .iter()
            .skip(skip_first)
            .skip_while(|pt| !pt.token_type.is_space())
            .skip_while(|pt| pt.token_type.is_space())
            .find_map(|pt| match pt.token_type {
                TokenType::Space => None,
                _ => Some(pt.start_byte_index),
            })
    }
}

pub fn index_first_end_of_word(parsed_tokens: &[ParsedToken]) -> Option<usize> {
    if parsed_tokens.is_empty() {
        Some(0)
    } else {
        parsed_tokens.iter().find_map(|pt| match pt.token_type {
            TokenType::Space => None,
            _ => Some(pt.end_byte_index),
        })
    }
}

pub fn index_next_end_of_word(
    parsed_tokens: &[ParsedToken],
    col: usize,
) -> Option<usize> {
    if parsed_tokens.is_empty() {
        None
    } else {
        let ti = index_tokens(parsed_tokens, col);
        let skip_first = if col == parsed_tokens.get(ti).unwrap().end_byte_index
        {
            ti + 1
        } else {
            ti
        };
        parsed_tokens.iter().skip(skip_first).find_map(|pt| {
            match pt.token_type {
                TokenType::Space => None,
                _ => Some(pt.end_byte_index),
            }
        })
    }
}

#[allow(non_snake_case)]
pub fn index_first_end_of_WORD(parsed_tokens: &[ParsedToken]) -> Option<usize> {
    if parsed_tokens.is_empty() {
        Some(0)
    } else {
        parsed_tokens
            .iter()
            .skip_while(|pt| pt.token_type.is_space())
            .map_while(|pt| match pt.token_type {
                TokenType::Space => None,
                _ => Some(pt.end_byte_index),
            })
            .last()
    }
}

#[allow(non_snake_case)]
pub fn index_next_end_of_WORD(
    parsed_tokens: &[ParsedToken],
    col: usize,
) -> Option<usize> {
    if parsed_tokens.is_empty() {
        None
    } else {
        let ti = index_tokens(parsed_tokens, col);
        let skip_first = if col == parsed_tokens.get(ti).unwrap().end_byte_index
        {
            ti + 1
        } else {
            ti
        };
        parsed_tokens
            .iter()
            .skip(skip_first)
            .skip_while(|pt| pt.token_type.is_space())
            .map_while(|pt| match pt.token_type {
                TokenType::Space => None,
                _ => Some(pt.end_byte_index),
            })
            .last()
    }
}

#[inline]
fn read_buffer_cut_and_parse(
    jieba: &Jieba,
    buffer: &Bound<'_, PySequence>,
    index: usize,
) -> PyResult<Vec<ParsedToken>> {
    Ok(parse_tokens(
        &jieba.cut(buffer.get_item(index)?.extract::<&str>()?, true),
    ))
}

/// Given the index function invoked on the first attempt
/// (`primary_index_func`), the index function invoked on later attempts
/// (`secondary_index_func`), whether the two index function go backward or not
/// (`backward`), current buffer, and current cursor position (row, col),
/// return the new cursor position. Current buffer is expected to be a
/// list-like object.
pub fn navigate<F1, F2>(
    primary_index_func: F1,
    mut secondary_index_func: F2,
    backward: bool,
    buffer: &Bound<'_, PySequence>,
    cursor_pos: (usize, usize),
) -> PyResult<(usize, usize)>
where
    F1: FnOnce(&[ParsedToken], usize) -> Option<usize>,
    F2: FnMut(&[ParsedToken]) -> Option<usize>,
{
    static JIEBA: Lazy<Jieba> = Lazy::new(|| {
        let mut dict = BufReader::new(crate::DICT.as_bytes());
        Jieba::with_dict(&mut dict).unwrap()
    });

    let final_col_default = |parsed_tokens: &[ParsedToken]| {
        if parsed_tokens.is_empty() {
            0
        } else if backward {
            parsed_tokens.first().unwrap().start_byte_index
        } else {
            parsed_tokens.last().unwrap().end_byte_index
        }
    };

    let sentinel_row = if backward { 1 } else { buffer.len()? };
    let step_row = if backward {
        |row: &mut usize| {
            *row -= 1;
        }
    } else {
        |row: &mut usize| {
            *row += 1;
        }
    };
    let (mut row, col) = cursor_pos;
    if row == sentinel_row {
        let parsed_tokens = read_buffer_cut_and_parse(&JIEBA, buffer, row - 1)?;
        let parsed_tokens =
            stack_merge(parsed_tokens, insert_implicit_space_rule);
        let col = primary_index_func(&parsed_tokens, col);
        let col = col.unwrap_or_else(|| final_col_default(&parsed_tokens));
        return Ok((row, col));
    }
    let parsed_tokens = read_buffer_cut_and_parse(&JIEBA, buffer, row - 1)?;
    let parsed_tokens = stack_merge(parsed_tokens, insert_implicit_space_rule);
    let col = primary_index_func(&parsed_tokens, col);
    if let Some(col) = col {
        return Ok((row, col));
    }
    // `row` must be at least one step from `sentinel_row`, because the case
    // where `row == sentinel_row` has been handled before.
    step_row(&mut row);
    while row != sentinel_row {
        let parsed_tokens = read_buffer_cut_and_parse(&JIEBA, buffer, row - 1)?;
        let parsed_tokens =
            stack_merge(parsed_tokens, insert_implicit_space_rule);
        let col = secondary_index_func(&parsed_tokens);
        if let Some(col) = col {
            return Ok((row, col));
        }
        step_row(&mut row);
    }
    let parsed_tokens = read_buffer_cut_and_parse(&JIEBA, buffer, row - 1)?;
    let parsed_tokens = stack_merge(parsed_tokens, insert_implicit_space_rule);
    let col = secondary_index_func(&parsed_tokens);
    let col = col.unwrap_or_else(|| final_col_default(&parsed_tokens));
    Ok((row, col))
}

#[cfg(test)]
mod tests {
    use super::{
        index_first_end_of_WORD, index_first_end_of_word,
        index_first_start_of_WORD, index_first_start_of_word,
        index_last_end_of_WORD, index_last_end_of_word,
        index_last_start_of_WORD, index_last_start_of_word,
        index_next_end_of_WORD, index_next_end_of_word,
        index_next_start_of_WORD, index_next_start_of_word,
        index_prev_end_of_WORD, index_prev_end_of_word,
        index_prev_start_of_WORD, index_prev_start_of_word,
        insert_implicit_space_rule, parse_tokens, stack_merge, ParsedToken,
        TokenType,
    };

    macro_rules! new_pt {
        ($i:literal, $j:literal, hans) => {
            ParsedToken {
                start_byte_index: $i,
                end_byte_index: $j,
                token_type: TokenType::Hans,
            }
        };
        ($i:literal, $j:literal, punc) => {
            ParsedToken {
                start_byte_index: $i,
                end_byte_index: $j,
                token_type: TokenType::Punc,
            }
        };
        ($i:literal, $j:literal, space) => {
            ParsedToken {
                start_byte_index: $i,
                end_byte_index: $j,
                token_type: TokenType::Space,
            }
        };
        ($i:literal, $j:literal, non_word) => {
            ParsedToken {
                start_byte_index: $i,
                end_byte_index: $j,
                token_type: TokenType::NonWord,
            }
        };
    }

    #[test]
    fn test_parse_tokens() {
        let tokens =
            vec!["Pixelmator", "-", "Pro", " ", "在", "设计", "，", "完全"];
        assert_eq!(
            parse_tokens(&tokens),
            vec![
                new_pt!(0, 9, hans),
                new_pt!(10, 10, non_word),
                new_pt!(11, 13, hans),
                new_pt!(14, 14, space),
                new_pt!(15, 15, hans),
                new_pt!(18, 21, hans),
                new_pt!(24, 24, punc),
                new_pt!(27, 30, hans),
            ]
        )
    }

    #[test]
    fn test_stack_merge() {
        fn rule(
            e1: Option<i32>,
            e2: i32,
        ) -> Result<Vec<i32>, (Option<i32>, i32)> {
            match e1 {
                None => Err((e1, e2)),
                Some(e1) => {
                    if e2 % 2 == 1 {
                        Ok(vec![e1, 999, e2 + 10])
                    } else {
                        Err((Some(e1), e2))
                    }
                }
            }
        }

        assert_eq!(
            stack_merge(vec![0, 1, 2, 3], rule),
            vec![0, 999, 11, 2, 999, 13]
        );
    }

    #[test]
    fn test_index_last_start_of_word() {
        let pt = vec![];
        assert_eq!(index_last_start_of_word(&pt), Some(0));

        let pt = vec![ParsedToken {
            start_byte_index: 0,
            end_byte_index: 2,
            token_type: TokenType::Space,
        }];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_word(&pt), None);

        let pt = vec![new_pt!(0, 2, space), new_pt!(3, 4, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_word(&pt), Some(3));

        let pt = vec![new_pt!(0, 1, hans), new_pt!(2, 3, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_word(&pt), Some(2));

        let pt = vec![
            new_pt!(0, 1, hans),
            new_pt!(2, 3, hans),
            new_pt!(4, 4, punc),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_word(&pt), Some(4));

        let pt = vec![
            new_pt!(0, 3, hans),
            new_pt!(4, 4, hans),
            new_pt!(7, 10, punc),
            new_pt!(13, 17, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_word(&pt), Some(7));

        let pt = vec![new_pt!(0, 0, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_word(&pt), Some(3));
    }

    #[test]
    fn test_index_prev_start_of_word() {
        let pt = vec![];
        assert_eq!(index_prev_start_of_word(&pt, 0), None);

        let pt = vec![new_pt!(0, 3, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_start_of_word(&pt, 0), None);
        assert_eq!(index_prev_start_of_word(&pt, 1), Some(0));
        assert_eq!(index_prev_start_of_word(&pt, 3), Some(0));

        let pt = vec![new_pt!(0, 3, hans), new_pt!(4, 5, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_start_of_word(&pt, 0), None);
        assert_eq!(index_prev_start_of_word(&pt, 1), Some(0));
        assert_eq!(index_prev_start_of_word(&pt, 3), Some(0));
        assert_eq!(index_prev_start_of_word(&pt, 4), Some(0));
        assert_eq!(index_prev_start_of_word(&pt, 5), Some(4));

        let pt = vec![new_pt!(0, 3, hans), new_pt!(6, 9, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_start_of_word(&pt, 0), None);
        assert_eq!(index_prev_start_of_word(&pt, 1), Some(0));
        assert_eq!(index_prev_start_of_word(&pt, 3), Some(0));
        assert_eq!(index_prev_start_of_word(&pt, 4), Some(0));
        assert_eq!(index_prev_start_of_word(&pt, 6), Some(0));
        assert_eq!(index_prev_start_of_word(&pt, 7), Some(6));
        assert_eq!(index_prev_start_of_word(&pt, 9), Some(6));

        let pt = vec![
            new_pt!(0, 3, hans),
            new_pt!(4, 4, punc),
            new_pt!(5, 6, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_start_of_word(&pt, 5), Some(4));
        assert_eq!(index_prev_start_of_word(&pt, 6), Some(4));

        let pt = vec![
            new_pt!(0, 1, space),
            new_pt!(2, 2, hans),
            new_pt!(5, 5, punc),
            new_pt!(8, 9, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_start_of_word(&pt, 0), None);
        assert_eq!(index_prev_start_of_word(&pt, 1), None);
        assert_eq!(index_prev_start_of_word(&pt, 2), None);
        assert_eq!(index_prev_start_of_word(&pt, 3), Some(2));
        assert_eq!(index_prev_start_of_word(&pt, 4), Some(2));
        assert_eq!(index_prev_start_of_word(&pt, 5), Some(2));
        assert_eq!(index_prev_start_of_word(&pt, 6), Some(5));
        assert_eq!(index_prev_start_of_word(&pt, 8), Some(5));
        assert_eq!(index_prev_start_of_word(&pt, 9), Some(5));

        let pt = vec![new_pt!(0, 0, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_start_of_word(&pt, 3), Some(0));
        assert_eq!(index_prev_start_of_word(&pt, 4), Some(3));
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_index_last_start_of_WORD() {
        let pt = vec![];
        assert_eq!(index_last_start_of_WORD(&pt), Some(0));

        let pt = vec![new_pt!(0, 1, hans), new_pt!(2, 3, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_WORD(&pt), Some(2));

        let pt = vec![
            new_pt!(0, 3, hans),
            new_pt!(4, 4, hans),
            new_pt!(7, 10, punc),
            new_pt!(13, 17, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_WORD(&pt), Some(4));

        let pt = vec![
            new_pt!(0, 3, hans),
            new_pt!(4, 4, punc),
            new_pt!(7, 10, hans),
            new_pt!(13, 17, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_WORD(&pt), Some(7));

        let pt = vec![new_pt!(0, 1, space), new_pt!(2, 5, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_WORD(&pt), Some(2));

        let pt = vec![
            new_pt!(0, 1, space),
            new_pt!(2, 5, hans),
            new_pt!(8, 8, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_WORD(&pt), Some(2));

        let pt = vec![new_pt!(0, 0, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_WORD(&pt), Some(3));

        let pt = vec![
            new_pt!(0, 1, hans),
            new_pt!(2, 2, punc),
            new_pt!(5, 5, non_word),
            new_pt!(6, 6, punc),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_WORD(&pt), Some(5));

        let pt = vec![
            new_pt!(0, 1, hans),
            new_pt!(2, 2, hans),
            new_pt!(5, 5, non_word),
            new_pt!(6, 7, hans),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_WORD(&pt), Some(2));

        let pt = vec![
            new_pt!(0, 1, hans),
            new_pt!(2, 2, hans),
            new_pt!(5, 5, non_word),
            new_pt!(6, 7, punc),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_WORD(&pt), Some(2));

        let pt = vec![
            new_pt!(0, 1, hans),
            new_pt!(2, 2, punc),
            new_pt!(5, 5, non_word),
            new_pt!(6, 7, hans),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_start_of_WORD(&pt), Some(5));
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_index_prev_start_of_WORD() {
        let pt = vec![];
        assert_eq!(index_prev_start_of_WORD(&pt, 0), None);

        let pt = vec![
            new_pt!(0, 1, space),
            new_pt!(2, 2, hans),
            new_pt!(5, 5, punc),
            new_pt!(8, 9, space),
            new_pt!(10, 10, punc),
            new_pt!(13, 13, hans),
            new_pt!(16, 16, hans),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_start_of_WORD(&pt, 0), None);
        assert_eq!(index_prev_start_of_WORD(&pt, 1), None);
        assert_eq!(index_prev_start_of_WORD(&pt, 2), None);
        assert_eq!(index_prev_start_of_WORD(&pt, 3), Some(2));
        assert_eq!(index_prev_start_of_WORD(&pt, 5), Some(2));
        assert_eq!(index_prev_start_of_WORD(&pt, 6), Some(2));
        assert_eq!(index_prev_start_of_WORD(&pt, 7), Some(2));
        assert_eq!(index_prev_start_of_WORD(&pt, 9), Some(2));
        assert_eq!(index_prev_start_of_WORD(&pt, 10), Some(2));
        assert_eq!(index_prev_start_of_WORD(&pt, 11), Some(10));
        assert_eq!(index_prev_start_of_WORD(&pt, 13), Some(10));
        assert_eq!(index_prev_start_of_WORD(&pt, 14), Some(13));
        assert_eq!(index_prev_start_of_WORD(&pt, 15), Some(13));
        assert_eq!(index_prev_start_of_WORD(&pt, 16), Some(13));
        assert_eq!(index_prev_start_of_WORD(&pt, 17), Some(16));

        let pt = vec![new_pt!(0, 0, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_start_of_WORD(&pt, 3), Some(0));
        assert_eq!(index_prev_start_of_WORD(&pt, 4), Some(3));
    }

    #[test]
    fn test_index_last_end_of_word() {
        let pt = vec![];
        assert_eq!(index_last_end_of_word(&pt), Some(0));

        let pt = vec![new_pt!(0, 2, space)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_end_of_word(&pt), None);

        let pt = vec![new_pt!(0, 2, space), new_pt!(3, 4, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_end_of_word(&pt), Some(4));

        let pt = vec![new_pt!(0, 1, hans), new_pt!(2, 3, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_end_of_word(&pt), Some(3));

        let pt = vec![
            new_pt!(0, 1, hans),
            new_pt!(2, 3, hans),
            new_pt!(4, 4, punc),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_end_of_word(&pt), Some(4));

        let pt = vec![
            new_pt!(0, 3, hans),
            new_pt!(4, 4, hans),
            new_pt!(7, 10, punc),
            new_pt!(13, 17, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_end_of_word(&pt), Some(10));

        let pt = vec![new_pt!(0, 0, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_end_of_word(&pt), Some(3));
    }

    #[test]
    fn test_index_prev_end_of_word() {
        let pt = vec![];
        assert_eq!(index_prev_end_of_word(&pt, 0), None);

        let pt = vec![new_pt!(0, 3, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_end_of_word(&pt, 0), None);
        assert_eq!(index_prev_end_of_word(&pt, 1), None);
        assert_eq!(index_prev_end_of_word(&pt, 3), None);

        let pt = vec![new_pt!(0, 3, hans), new_pt!(4, 5, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_end_of_word(&pt, 0), None);
        assert_eq!(index_prev_end_of_word(&pt, 1), None);
        assert_eq!(index_prev_end_of_word(&pt, 3), None);
        assert_eq!(index_prev_end_of_word(&pt, 4), Some(3));
        assert_eq!(index_prev_end_of_word(&pt, 5), Some(3));

        let pt = vec![new_pt!(0, 3, hans), new_pt!(6, 9, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_end_of_word(&pt, 0), None);
        assert_eq!(index_prev_end_of_word(&pt, 1), None);
        assert_eq!(index_prev_end_of_word(&pt, 3), None);
        assert_eq!(index_prev_end_of_word(&pt, 4), None);
        assert_eq!(index_prev_end_of_word(&pt, 5), None);
        assert_eq!(index_prev_end_of_word(&pt, 6), Some(3));
        assert_eq!(index_prev_end_of_word(&pt, 7), Some(3));
        assert_eq!(index_prev_end_of_word(&pt, 9), Some(3));

        let pt = vec![
            new_pt!(0, 3, hans),
            new_pt!(4, 4, punc),
            new_pt!(5, 6, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_end_of_word(&pt, 5), Some(4));
        assert_eq!(index_prev_end_of_word(&pt, 6), Some(4));

        let pt = vec![
            new_pt!(0, 1, space),
            new_pt!(2, 2, hans),
            new_pt!(5, 6, punc),
            new_pt!(8, 9, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_end_of_word(&pt, 0), None);
        assert_eq!(index_prev_end_of_word(&pt, 1), None);
        assert_eq!(index_prev_end_of_word(&pt, 2), None);
        assert_eq!(index_prev_end_of_word(&pt, 3), None);
        assert_eq!(index_prev_end_of_word(&pt, 4), None);
        assert_eq!(index_prev_end_of_word(&pt, 5), Some(2));
        assert_eq!(index_prev_end_of_word(&pt, 6), Some(2));
        assert_eq!(index_prev_end_of_word(&pt, 7), Some(2));
        assert_eq!(index_prev_end_of_word(&pt, 8), Some(6));
        assert_eq!(index_prev_end_of_word(&pt, 9), Some(6));

        let pt = vec![new_pt!(0, 0, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_end_of_word(&pt, 4), Some(0));
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_index_last_end_of_WORD() {
        let pt = vec![];
        assert_eq!(index_last_end_of_WORD(&pt), Some(0));

        let pt = vec![new_pt!(0, 1, hans), new_pt!(2, 3, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_end_of_WORD(&pt), Some(3));

        let pt = vec![
            new_pt!(0, 3, hans),
            new_pt!(4, 4, hans),
            new_pt!(7, 10, punc),
            new_pt!(13, 17, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_end_of_WORD(&pt), Some(10));

        let pt = vec![
            new_pt!(0, 3, hans),
            new_pt!(4, 4, punc),
            new_pt!(7, 10, hans),
            new_pt!(13, 17, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_end_of_WORD(&pt), Some(10));

        let pt = vec![new_pt!(0, 1, space), new_pt!(2, 5, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_end_of_WORD(&pt), Some(5));

        let pt = vec![
            new_pt!(0, 1, space),
            new_pt!(2, 5, hans),
            new_pt!(8, 8, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_end_of_WORD(&pt), Some(5));

        let pt = vec![new_pt!(0, 0, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_last_end_of_WORD(&pt), Some(3));
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_index_prev_end_of_WORD() {
        let pt = vec![];
        assert_eq!(index_prev_end_of_WORD(&pt, 0), None);

        let pt = vec![
            new_pt!(0, 1, space),
            new_pt!(2, 3, hans),
            new_pt!(5, 5, punc),
            new_pt!(8, 9, space),
            new_pt!(10, 10, punc),
            new_pt!(13, 13, hans),
            new_pt!(16, 16, hans),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_end_of_WORD(&pt, 0), None);
        assert_eq!(index_prev_end_of_WORD(&pt, 1), None);
        assert_eq!(index_prev_end_of_WORD(&pt, 2), None);
        assert_eq!(index_prev_end_of_WORD(&pt, 3), None);
        assert_eq!(index_prev_end_of_WORD(&pt, 5), None);
        assert_eq!(index_prev_end_of_WORD(&pt, 6), None);
        assert_eq!(index_prev_end_of_WORD(&pt, 7), None);
        assert_eq!(index_prev_end_of_WORD(&pt, 8), Some(5));
        assert_eq!(index_prev_end_of_WORD(&pt, 9), Some(5));
        assert_eq!(index_prev_end_of_WORD(&pt, 10), Some(5));
        assert_eq!(index_prev_end_of_WORD(&pt, 11), Some(5));
        assert_eq!(index_prev_end_of_WORD(&pt, 13), Some(10));
        assert_eq!(index_prev_end_of_WORD(&pt, 14), Some(10));
        assert_eq!(index_prev_end_of_WORD(&pt, 15), Some(10));
        assert_eq!(index_prev_end_of_WORD(&pt, 16), Some(13));
        assert_eq!(index_prev_end_of_WORD(&pt, 17), Some(13));

        let pt = vec![new_pt!(0, 0, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_prev_end_of_WORD(&pt, 4), Some(0));
    }

    #[test]
    fn test_index_first_start_of_word() {
        let pt = vec![];
        assert_eq!(index_first_start_of_word(&pt), Some(0));

        let pt = vec![new_pt!(0, 1, hans), new_pt!(2, 3, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_first_start_of_word(&pt), Some(0));

        let pt = vec![
            new_pt!(0, 2, space),
            new_pt!(3, 6, punc),
            new_pt!(9, 12, hans),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_first_start_of_word(&pt), Some(3));

        let pt = vec![new_pt!(0, 0, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_first_start_of_word(&pt), Some(0));
    }

    #[test]
    fn test_index_next_start_of_word() {
        let pt = vec![];
        assert_eq!(index_next_start_of_word(&pt, 0), None);

        let pt = vec![new_pt!(0, 3, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_start_of_word(&pt, 0), None);
        assert_eq!(index_next_start_of_word(&pt, 1), None);
        assert_eq!(index_next_start_of_word(&pt, 3), None);

        let pt = vec![new_pt!(0, 3, hans), new_pt!(4, 5, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_start_of_word(&pt, 0), Some(4));
        assert_eq!(index_next_start_of_word(&pt, 1), Some(4));
        assert_eq!(index_next_start_of_word(&pt, 3), Some(4));
        assert_eq!(index_next_start_of_word(&pt, 4), None);
        assert_eq!(index_next_start_of_word(&pt, 5), None);

        let pt = vec![new_pt!(0, 3, hans), new_pt!(6, 9, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_start_of_word(&pt, 0), Some(6));
        assert_eq!(index_next_start_of_word(&pt, 1), Some(6));
        assert_eq!(index_next_start_of_word(&pt, 3), Some(6));
        assert_eq!(index_next_start_of_word(&pt, 4), Some(6));
        assert_eq!(index_next_start_of_word(&pt, 5), Some(6));
        assert_eq!(index_next_start_of_word(&pt, 6), None);
        assert_eq!(index_next_start_of_word(&pt, 7), None);
        assert_eq!(index_next_start_of_word(&pt, 9), None);

        let pt = vec![
            new_pt!(0, 2, space),
            new_pt!(3, 6, hans),
            new_pt!(7, 7, punc),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_start_of_word(&pt, 0), Some(3));
        assert_eq!(index_next_start_of_word(&pt, 2), Some(3));
        assert_eq!(index_next_start_of_word(&pt, 3), Some(7));
        assert_eq!(index_next_start_of_word(&pt, 6), Some(7));
        assert_eq!(index_next_start_of_word(&pt, 7), None);

        let pt = vec![
            new_pt!(0, 1, space),
            new_pt!(2, 3, punc),
            new_pt!(5, 6, hans),
            new_pt!(8, 9, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_start_of_word(&pt, 0), Some(2));
        assert_eq!(index_next_start_of_word(&pt, 1), Some(2));
        assert_eq!(index_next_start_of_word(&pt, 2), Some(5));
        assert_eq!(index_next_start_of_word(&pt, 3), Some(5));
        assert_eq!(index_next_start_of_word(&pt, 4), Some(5));
        assert_eq!(index_next_start_of_word(&pt, 5), None);
        assert_eq!(index_next_start_of_word(&pt, 6), None);
        assert_eq!(index_next_start_of_word(&pt, 7), None);
        assert_eq!(index_next_start_of_word(&pt, 8), None);
        assert_eq!(index_next_start_of_word(&pt, 9), None);

        let pt = vec![new_pt!(0, 0, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_start_of_word(&pt, 0), Some(3));
        assert_eq!(index_next_start_of_word(&pt, 1), Some(3));
        assert_eq!(index_next_start_of_word(&pt, 2), Some(3));
        assert_eq!(index_next_start_of_word(&pt, 3), None);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_index_first_start_of_WORD() {
        let pt = vec![];
        assert_eq!(index_first_start_of_WORD(&pt), Some(0));

        let pt = vec![new_pt!(0, 1, hans), new_pt!(2, 3, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_first_start_of_WORD(&pt), Some(0));

        let pt = vec![
            new_pt!(0, 2, space),
            new_pt!(3, 6, punc),
            new_pt!(9, 12, hans),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_first_start_of_WORD(&pt), Some(3));

        let pt = vec![new_pt!(0, 0, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_first_start_of_WORD(&pt), Some(0));
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_index_next_start_of_WORD() {
        let pt = vec![];
        assert_eq!(index_next_start_of_WORD(&pt, 0), None);

        let pt = vec![new_pt!(0, 3, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_start_of_WORD(&pt, 0), None);
        assert_eq!(index_next_start_of_WORD(&pt, 1), None);
        assert_eq!(index_next_start_of_WORD(&pt, 3), None);

        let pt = vec![new_pt!(0, 3, hans), new_pt!(4, 5, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_start_of_WORD(&pt, 0), Some(4));
        assert_eq!(index_next_start_of_WORD(&pt, 1), Some(4));
        assert_eq!(index_next_start_of_WORD(&pt, 3), Some(4));
        assert_eq!(index_next_start_of_WORD(&pt, 4), None);
        assert_eq!(index_next_start_of_WORD(&pt, 5), None);

        let pt = vec![new_pt!(0, 3, hans), new_pt!(6, 9, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_start_of_WORD(&pt, 0), Some(6));
        assert_eq!(index_next_start_of_WORD(&pt, 1), Some(6));
        assert_eq!(index_next_start_of_WORD(&pt, 3), Some(6));
        assert_eq!(index_next_start_of_WORD(&pt, 4), Some(6));
        assert_eq!(index_next_start_of_WORD(&pt, 5), Some(6));
        assert_eq!(index_next_start_of_WORD(&pt, 6), None);
        assert_eq!(index_next_start_of_WORD(&pt, 7), None);
        assert_eq!(index_next_start_of_WORD(&pt, 9), None);

        let pt = vec![
            new_pt!(0, 2, space),
            new_pt!(3, 6, hans),
            new_pt!(7, 7, punc),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_start_of_WORD(&pt, 0), Some(3));
        assert_eq!(index_next_start_of_WORD(&pt, 2), Some(3));
        assert_eq!(index_next_start_of_WORD(&pt, 3), None);
        assert_eq!(index_next_start_of_WORD(&pt, 6), None);
        assert_eq!(index_next_start_of_WORD(&pt, 7), None);

        let pt = vec![
            new_pt!(0, 1, space),
            new_pt!(2, 3, punc),
            new_pt!(5, 6, hans),
            new_pt!(8, 9, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_start_of_WORD(&pt, 0), Some(2));
        assert_eq!(index_next_start_of_WORD(&pt, 1), Some(2));
        assert_eq!(index_next_start_of_WORD(&pt, 2), Some(5));
        assert_eq!(index_next_start_of_WORD(&pt, 3), Some(5));
        assert_eq!(index_next_start_of_WORD(&pt, 4), Some(5));
        assert_eq!(index_next_start_of_WORD(&pt, 5), None);
        assert_eq!(index_next_start_of_WORD(&pt, 6), None);
        assert_eq!(index_next_start_of_WORD(&pt, 7), None);
        assert_eq!(index_next_start_of_WORD(&pt, 8), None);
        assert_eq!(index_next_start_of_WORD(&pt, 9), None);

        let pt = vec![new_pt!(0, 0, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_start_of_WORD(&pt, 0), Some(3));
        assert_eq!(index_next_start_of_WORD(&pt, 1), Some(3));
        assert_eq!(index_next_start_of_WORD(&pt, 2), Some(3));
        assert_eq!(index_next_start_of_WORD(&pt, 3), None);
    }

    #[test]
    fn test_index_first_end_of_word() {
        let pt = vec![];
        assert_eq!(index_first_end_of_word(&pt), Some(0));

        let pt = vec![new_pt!(0, 1, hans), new_pt!(2, 3, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_first_end_of_word(&pt), Some(1));

        let pt = vec![
            new_pt!(0, 2, space),
            new_pt!(3, 6, punc),
            new_pt!(9, 12, hans),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_first_end_of_word(&pt), Some(6));

        let pt = vec![new_pt!(0, 1, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_first_end_of_word(&pt), Some(1));
    }

    #[test]
    fn test_index_next_end_of_word() {
        let pt = vec![];
        assert_eq!(index_next_end_of_word(&pt, 0), None);

        let pt = vec![new_pt!(0, 3, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_end_of_word(&pt, 0), Some(3));
        assert_eq!(index_next_end_of_word(&pt, 1), Some(3));
        assert_eq!(index_next_end_of_word(&pt, 2), Some(3));
        assert_eq!(index_next_end_of_word(&pt, 3), None);

        let pt = vec![new_pt!(0, 3, hans), new_pt!(6, 9, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_end_of_word(&pt, 0), Some(3));
        assert_eq!(index_next_end_of_word(&pt, 1), Some(3));
        assert_eq!(index_next_end_of_word(&pt, 3), Some(9));
        assert_eq!(index_next_end_of_word(&pt, 6), Some(9));
        assert_eq!(index_next_end_of_word(&pt, 7), Some(9));
        assert_eq!(index_next_end_of_word(&pt, 8), Some(9));
        assert_eq!(index_next_end_of_word(&pt, 9), None);

        let pt = vec![
            new_pt!(0, 2, space),
            new_pt!(3, 6, hans),
            new_pt!(7, 7, punc),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_end_of_word(&pt, 0), Some(6));
        assert_eq!(index_next_end_of_word(&pt, 2), Some(6));
        assert_eq!(index_next_end_of_word(&pt, 3), Some(6));
        assert_eq!(index_next_end_of_word(&pt, 6), Some(7));
        assert_eq!(index_next_end_of_word(&pt, 7), None);

        let pt = vec![
            new_pt!(0, 1, space),
            new_pt!(2, 3, punc),
            new_pt!(5, 6, hans),
            new_pt!(8, 9, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_end_of_word(&pt, 0), Some(3));
        assert_eq!(index_next_end_of_word(&pt, 1), Some(3));
        assert_eq!(index_next_end_of_word(&pt, 2), Some(3));
        assert_eq!(index_next_end_of_word(&pt, 3), Some(6));
        assert_eq!(index_next_end_of_word(&pt, 5), Some(6));
        assert_eq!(index_next_end_of_word(&pt, 6), None);
        assert_eq!(index_next_end_of_word(&pt, 8), None);
        assert_eq!(index_next_end_of_word(&pt, 9), None);

        let pt = vec![new_pt!(0, 0, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_end_of_word(&pt, 0), Some(3));
        assert_eq!(index_next_end_of_word(&pt, 3), None);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_index_first_end_of_WORD() {
        let pt = vec![];
        assert_eq!(index_first_end_of_WORD(&pt), Some(0));

        let pt = vec![new_pt!(0, 1, hans), new_pt!(2, 3, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_first_end_of_WORD(&pt), Some(1));

        let pt = vec![
            new_pt!(0, 2, space),
            new_pt!(3, 6, punc),
            new_pt!(9, 12, hans),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_first_end_of_WORD(&pt), Some(6));

        let pt = vec![
            new_pt!(0, 2, space),
            new_pt!(3, 6, hans),
            new_pt!(9, 12, punc),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_first_end_of_WORD(&pt), Some(12));

        let pt = vec![new_pt!(0, 1, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_first_end_of_WORD(&pt), Some(1));
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_index_next_end_of_WORD() {
        let pt = vec![];
        assert_eq!(index_next_end_of_WORD(&pt, 0), None);

        let pt = vec![new_pt!(0, 3, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_end_of_WORD(&pt, 0), Some(3));
        assert_eq!(index_next_end_of_WORD(&pt, 1), Some(3));
        assert_eq!(index_next_end_of_WORD(&pt, 2), Some(3));
        assert_eq!(index_next_end_of_WORD(&pt, 3), None);

        let pt = vec![new_pt!(0, 3, hans), new_pt!(6, 9, hans)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_end_of_WORD(&pt, 0), Some(3));
        assert_eq!(index_next_end_of_WORD(&pt, 1), Some(3));
        assert_eq!(index_next_end_of_WORD(&pt, 3), Some(9));
        assert_eq!(index_next_end_of_WORD(&pt, 6), Some(9));
        assert_eq!(index_next_end_of_WORD(&pt, 7), Some(9));
        assert_eq!(index_next_end_of_WORD(&pt, 8), Some(9));
        assert_eq!(index_next_end_of_WORD(&pt, 9), None);

        let pt = vec![
            new_pt!(0, 2, space),
            new_pt!(3, 6, hans),
            new_pt!(7, 7, punc),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_end_of_WORD(&pt, 0), Some(7));
        assert_eq!(index_next_end_of_WORD(&pt, 2), Some(7));
        assert_eq!(index_next_end_of_WORD(&pt, 3), Some(7));
        assert_eq!(index_next_end_of_WORD(&pt, 6), Some(7));
        assert_eq!(index_next_end_of_WORD(&pt, 7), None);

        let pt = vec![
            new_pt!(0, 1, space),
            new_pt!(2, 3, punc),
            new_pt!(5, 6, hans),
            new_pt!(8, 9, space),
        ];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_end_of_WORD(&pt, 0), Some(3));
        assert_eq!(index_next_end_of_WORD(&pt, 1), Some(3));
        assert_eq!(index_next_end_of_WORD(&pt, 2), Some(3));
        assert_eq!(index_next_end_of_WORD(&pt, 3), Some(6));
        assert_eq!(index_next_end_of_WORD(&pt, 5), Some(6));
        assert_eq!(index_next_end_of_WORD(&pt, 6), None);
        assert_eq!(index_next_end_of_WORD(&pt, 8), None);
        assert_eq!(index_next_end_of_WORD(&pt, 9), None);

        let pt = vec![new_pt!(0, 0, punc), new_pt!(3, 3, punc)];
        let pt = stack_merge(pt, insert_implicit_space_rule);
        assert_eq!(index_next_end_of_WORD(&pt, 0), Some(3));
        assert_eq!(index_next_end_of_WORD(&pt, 3), None);
    }
}
