//! Figure out which letters are words, based on `'iskeyword'` Vim option.
//! Why to use context-free grammar parser when the option value is regular?
//! Because it's not possible to capture repetition of the same group, while at
//! the same time, iterative partial matching produces ambiguity.

use std::rc::Rc;

use santiago::grammar::Grammar;
use santiago::lexer::{Lexeme, Lexer, LexerError, LexerRules, NextLexeme};
use santiago::parser::ParseError;

santiago::def!(NUMBER, "[0-9]+");
santiago::def!(SINGLE_CHAR_NO_CARET, r"[ -/:-\]_-~]");
santiago::def!(SINGLE_CHAR, "[ -/:-~]");

fn push_and_take<'a>(lexer: &mut Lexer<'a>, state: &'a str) -> NextLexeme {
    lexer.push_state(state);
    lexer.take()
}

fn pop_and_take(lexer: &mut Lexer<'_>) -> NextLexeme {
    lexer.pop_state();
    lexer.take()
}

fn pop_and_skip(lexer: &mut Lexer<'_>) -> NextLexeme {
    lexer.pop_state();
    lexer.skip()
}

fn pop_push_and_take<'a>(lexer: &mut Lexer<'a>, state: &'a str) -> NextLexeme {
    lexer.pop_state();
    lexer.push_state(state);
    lexer.take()
}

fn lexer_rules() -> LexerRules {
    santiago::lexer_rules!(
        "DEFAULT" | "NUMBER" = pattern NUMBER!() =>
            |lexer| push_and_take(lexer, "ITEM_CHARSPEC");
        "DEFAULT" | "SINGLE_CHAR_NO_CARET" = pattern SINGLE_CHAR_NO_CARET!() =>
            |lexer| push_and_take(lexer, "ITEM_CHARSPEC");
        "ITEM_CHARSPEC" | "PART_SEP" = string "," =>
            pop_and_take;
        "ITEM_CHARSPEC" | "" = string "" =>
            pop_and_skip;
        "ITEM_CHARSPEC" | "RANGE_SEP" = string "-" =>
            |lexer| pop_push_and_take(lexer, "ITEM_RANGE");
        "ITEM_RANGE" | "NUMBER" = pattern NUMBER!() =>
            |lexer| pop_push_and_take(lexer, "ITEM_CHARSPEC_RHS");
        "ITEM_RANGE" | "SINGLE_CHAR" = pattern SINGLE_CHAR!() =>
            |lexer| pop_push_and_take(lexer, "ITEM_CHARSPEC_RHS");
        "ITEM_CHARSPEC_RHS" | "PART_SEP" = string "," =>
            pop_and_take;
        "ITEM_CHARSPEC_RHS" | "" = string "" =>
            pop_and_skip;
        "DEFAULT" | "CARET" = string "^" =>
            |lexer| push_and_take(lexer, "ITEM_CHARSPEC_CARET");
        "ITEM_CHARSPEC_CARET" | "NUMBER" = pattern NUMBER!() =>
            |lexer| pop_push_and_take(lexer, "NEG_ITEM_CHARSPEC");
        "ITEM_CHARSPEC_CARET" | "SINGLE_CHAR" = pattern SINGLE_CHAR!() =>
            |lexer| pop_push_and_take(lexer, "NEG_ITEM_CHARSPEC");
        "ITEM_CHARSPEC_CARET"  | "" = string "" =>
            pop_and_skip;
        "NEG_ITEM_CHARSPEC" | "PART_SEP" = string "," =>
            pop_and_take;
        "NEG_ITEM_CHARSPEC" | "" = string "" =>
            pop_and_skip;
        "NEG_ITEM_CHARSPEC" | "RANGE_SEP" = string "-" =>
            |lexer| pop_push_and_take(lexer, "NEG_ITEM_RANGE");
        "NEG_ITEM_RANGE" | "NUMBER" = pattern NUMBER!() =>
            |lexer| pop_push_and_take(lexer, "NEG_ITEM_CHARSPEC_RHS");
        "NEG_ITEM_RANGE" | "SINGLE_CHAR" = pattern SINGLE_CHAR!() =>
            |lexer| pop_push_and_take(lexer, "NEG_ITEM_CHARSPEC_RHS");
        "NEG_ITEM_CHARSPEC_RHS" | "PART_SEP" = string "," =>
            pop_and_take;
        "NEG_ITEM_CHARSPEC_RHS" | "" = string "" =>
            pop_and_skip;
    )
}

#[derive(Debug, PartialEq, Eq)]
enum CharSpec {
    Number(u8),
    Char(char),
}

#[derive(Debug, PartialEq, Eq)]
enum Item {
    CharSpec(CharSpec),
    Range(CharSpec, CharSpec),
}

#[derive(Debug, PartialEq, Eq)]
enum Part {
    Part(Item),
    NegPart(Item),
}

enum Ast {
    PartSep,
    RangeSep,
    Number(u8),
    SingleChar(char),
    Parts { parts: Box<Ast>, part: Part },
    PartsTerm,
    Part(Item),
    NegPart(Item),
    CharSpecItem(CharSpec),
    RangeItem(CharSpec, CharSpec),
}

fn vec_to_tuple1<T>(mut v: Vec<T>) -> T {
    assert_eq!(v.len(), 1);
    v.pop().unwrap()
}

fn vec_to_tuple2<T>(mut v: Vec<T>) -> (T, T) {
    assert_eq!(v.len(), 2);
    let b = v.pop().unwrap();
    let a = v.pop().unwrap();
    (a, b)
}

fn vec_to_tuple3<T>(mut v: Vec<T>) -> (T, T, T) {
    assert_eq!(v.len(), 3);
    let c = v.pop().unwrap();
    let b = v.pop().unwrap();
    let a = v.pop().unwrap();
    (a, b, c)
}

fn ast_char_spec_to_char_spec(char_spec: Ast) -> CharSpec {
    match char_spec {
        Ast::Number(value) => CharSpec::Number(value),
        Ast::SingleChar(value) => CharSpec::Char(value),
        _ => unreachable!(),
    }
}

fn ast_item_to_item(item: Ast) -> Item {
    match item {
        Ast::CharSpecItem(char_spec) => Item::CharSpec(char_spec),
        Ast::RangeItem(lhs, rhs) => Item::Range(lhs, rhs),
        _ => unreachable!(),
    }
}

fn ast_part_to_part(part: Ast) -> Part {
    match part {
        Ast::Part(item) => Part::Part(item),
        Ast::NegPart(neg_item) => Part::NegPart(neg_item),
        _ => unreachable!(),
    }
}

fn parse_lexemes_number(lexemes: &[&Rc<Lexeme>]) -> Ast {
    let value = str::parse(&lexemes[0].raw).unwrap();
    Ast::Number(value)
}

fn parse_lexemes_single_char(lexemes: &[&Rc<Lexeme>]) -> Ast {
    Ast::SingleChar(lexemes[0].raw.chars().next().unwrap())
}

fn grammar() -> Grammar<Ast> {
    santiago::grammar!(
        "sent" => rules "parts" "last_part" =>
            |trees| {
                let (parts, last_part) = vec_to_tuple2(trees);
                Ast::Parts {
                    parts: Box::new(parts),
                    part: ast_part_to_part(last_part),
                }
            };
        "parts" => empty =>
            |_| Ast::PartsTerm;
        "parts" => rules "parts" "part" "part_sep" =>
            |trees| {
                let (parts, part, _) = vec_to_tuple3(trees);
                Ast::Parts {
                    parts: Box::new(parts),
                    part: ast_part_to_part(part),
                }
            };
        "part" => rules "item" =>
            |trees| {
                let item = vec_to_tuple1(trees);
                Ast::Part(ast_item_to_item(item))
            };
        "part" => rules "caret" "neg_item" =>
            |trees| {
                let (_, neg_item) = vec_to_tuple2(trees);
                Ast::NegPart(ast_item_to_item(neg_item))
            };
        "item" => rules "char_spec" =>
            |trees| {
                let char_spec = vec_to_tuple1(trees);
                Ast::CharSpecItem(ast_char_spec_to_char_spec(char_spec))
            };
        "item" => rules "range" =>
            vec_to_tuple1;
        // "char_spec" => rules "number";
        "char_spec" => lexemes "NUMBER" =>
            parse_lexemes_number;
        // "char_spec" => rules "single_char_no_caret";
        "char_spec" => lexemes "SINGLE_CHAR_NO_CARET" =>
            parse_lexemes_single_char;
        "range" => rules "char_spec" "range_sep" "char_spec_rhs" =>
            |trees| {
                let (lhs, _, rhs) = vec_to_tuple3(trees);
                let lhs = ast_char_spec_to_char_spec(lhs);
                let rhs = ast_char_spec_to_char_spec(rhs);
                Ast::RangeItem(lhs, rhs)
            };
        // "char_spec_rhs" => rules "number";
        "char_spec_rhs" => lexemes "NUMBER" =>
            parse_lexemes_number;
        // "char_spec_rhs" => rules "single_char";
        "char_spec_rhs" => lexemes "SINGLE_CHAR" =>
            parse_lexemes_single_char;
        // "char_spec_rhs" => rules "single_char_no_caret";
        "char_spec_rhs" => lexemes "SINGLE_CHAR_NO_CARET" =>
            parse_lexemes_single_char;
        // "char_spec_rhs" => rules "caret";
        "char_spec_rhs" => lexemes "CARET" =>
            |_| Ast::SingleChar('^');
        "neg_item" => rules "char_spec_rhs" =>
            |trees| {
                let char_spec = vec_to_tuple1(trees);
                Ast::CharSpecItem(ast_char_spec_to_char_spec(char_spec))
            };
        "neg_item" => rules "neg_range" =>
            vec_to_tuple1;
        "neg_range" => rules "char_spec_rhs" "range_sep" "char_spec_rhs" =>
            |trees| {
                let (lhs, _, rhs) = vec_to_tuple3(trees);
                let lhs = ast_char_spec_to_char_spec(lhs);
                let rhs = ast_char_spec_to_char_spec(rhs);
                Ast::RangeItem(lhs, rhs)
            };
        "last_part" => rules "last_item" =>
            |trees| {
                let item = vec_to_tuple1(trees);
                Ast::Part(ast_item_to_item(item))
            };
        "last_part" => rules "caret" "neg_item" =>
            |trees| {
                let (_, neg_item) = vec_to_tuple2(trees);
                Ast::NegPart(ast_item_to_item(neg_item))
            };
        "last_item" => rules "char_spec_rhs" =>
            |trees| {
                let char_spec = vec_to_tuple1(trees);
                Ast::CharSpecItem(ast_char_spec_to_char_spec(char_spec))
            };
        "last_item" => rules "range" =>
            vec_to_tuple1;

        "part_sep" => lexemes "PART_SEP" =>
            |_| Ast::PartSep;
        "range_sep" => lexemes "RANGE_SEP" =>
            |_| Ast::RangeSep;
        "number" => lexemes "NUMBER" =>
            parse_lexemes_number;
        "caret" => lexemes "CARET" =>
            |_| Ast::SingleChar('^');
        "single_char_no_caret" => lexemes "SINGLE_CHAR_NO_CARET" =>
            parse_lexemes_single_char;
        "single_char" => lexemes "SINGLE_CHAR" =>
            parse_lexemes_single_char;
    )
}

/// `'iskeyword'` option parsing error.
#[derive(Debug)]
pub enum Error {
    Lexer(LexerError),
    Parser(ParseError<Ast>),
}

impl From<LexerError> for Error {
    fn from(value: LexerError) -> Self {
        Error::Lexer(value)
    }
}

impl From<ParseError<Ast>> for Error {
    fn from(value: ParseError<Ast>) -> Self {
        Error::Parser(value)
    }
}

fn ast_parts_to_parts(parts: Ast) -> Vec<Part> {
    let mut parts = parts;
    let mut part_vec = Vec::new();
    while let Ast::Parts { parts: inner, part } = parts {
        part_vec.push(part);
        parts = *inner;
    }
    part_vec.reverse();
    part_vec
}

/// Parser for `'iskeyword'` option values.
pub struct IskParser {
    lexer_rules: LexerRules,
    grammar: Grammar<Ast>,
}

impl IskParser {
    pub fn new() -> Self {
        Self {
            lexer_rules: lexer_rules(),
            grammar: grammar(),
        }
    }

    fn parse(&self, value: &str) -> Result<Vec<Part>, Error> {
        let lexemes = santiago::lexer::lex(&self.lexer_rules, value)?;
        let parse_trees = santiago::parser::parse(&self.grammar, &lexemes)?;
        let ast = parse_trees[0].as_abstract_syntax_tree();
        Ok(ast_parts_to_parts(ast))
    }
}

trait RetainMut<T> {
    fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool;
}

impl<T> RetainMut<T> for Vec<T> {
    fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        self.dedup_by(|a, _| !f(a));
        if !self.is_empty() {
            if !f(self.first_mut().unwrap()) {
                self.remove(0);
            }
        }
    }
}

/// Types where each value has at most one successor and at most one
/// predecessor.
trait BoundedChain: Ord + Sized {
    /// Return the successor of `self`, if exists.
    fn succ(&self) -> Option<Self>;
    /// Return the predecessor of `self`, if exists.
    fn pred(&self) -> Option<Self>;
}

impl BoundedChain for u8 {
    fn succ(&self) -> Option<Self> {
        self.checked_add(1)
    }

    fn pred(&self) -> Option<Self> {
        self.checked_sub(1)
    }
}

/// Set defined by doubly inclusive intervals. When there's no interval, the
/// set is an empty set.
struct Intervals<T>(Vec<(T, T)>);

impl<T> Default for Intervals<T> {
    /// Construct an empty set.
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> Intervals<T> {
    fn push(&mut self, interval: (T, T)) {
        self.0.push(interval);
    }

    fn append(&mut self, mut intervals: Vec<(T, T)>) {
        self.0.append(&mut intervals);
    }
}

impl<T: Ord> Intervals<T> {
    fn contains(&self, value: &T) -> bool {
        self.0.iter().any(|(a, b)| a <= value && value <= b)
    }
}

impl<T: Ord + Copy> Intervals<T> {
    /// Merge overlapping intervals.
    fn merge(&mut self) {
        self.0.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        self.0.dedup_by(|a, b| {
            if a.0 <= b.1 {
                b.1 = std::cmp::max(a.1, b.1);
                true
            } else {
                false
            }
        });
    }
}

impl<T: BoundedChain + Copy> Intervals<T> {
    /// Remove the interval `r` from the union of `self`.
    fn remove(&mut self, r: &(T, T)) {
        let mut new_intervals = Vec::new();
        self.0.retain_mut(|a| {
            if a.0 == a.1 {
                r.1 < a.0 || r.0 > a.1
            } else {
                if r.1 < a.0 || r.0 > a.1 {
                    true
                } else if r.1 == a.0 {
                    // Since a.1 > a.0 == r.1, the successor of r.1 must exist.
                    a.0 = r.1.succ().unwrap();
                    true
                } else if r.0 == a.1 {
                    // Since a.0 < a.1 == r.0, the predecessor of r.0 must exist.
                    a.1 = r.0.pred().unwrap();
                    true
                } else if r.0 <= a.0 && r.1 > a.0 && r.1 < a.1 {
                    // Since a.1 > r.1, the successor of r.1 must exist.
                    a.0 = r.1.succ().unwrap();
                    true
                } else if r.0 > a.0 /* && r.1 > a.0 */ && r.1 < a.1 {
                    // Since r.1 < a.1, the successor of r.1 must exist.
                    new_intervals.push((r.1.succ().unwrap(), a.1));
                    // Since a.0 < r.0, the predecessor of r.0 must exist.
                    a.1 = r.0.pred().unwrap();
                    true
                } else if r.1 >= a.1 && r.0 > a.0 && r.0 < a.1 {
                    // Since a.0 < r.0, the predecessor of r.0 must exist.
                    a.1 = r.0.pred().unwrap();
                    true
                } else if r.1 >= a.1 && r.0 <= a.0 {
                    false
                } else if r.1 == a.1 && r.0 > a.0 {
                    // Since a.0 < r.0, the predecessor of r.0 must exist.
                    a.1 = r.0.pred().unwrap();
                    true
                } else if r.0 == a.0 && r.1 < a.1 {
                    // Since r.1 < a.1, the successor of r.1 must exist.
                    a.0 = r.1.succ().unwrap();
                    true
                } else {
                    unreachable!()
                }
            }
        });
        self.0.append(&mut new_intervals);
    }
}

/// Represents the '@'. When interpreted as a char, it's '@'. When interpreted
/// as an ASCII range, it's `a-z,A-Z,192-255`, per
/// https://vimhelp.org/options.txt.html#%27isfname%27:
///
/// > Normally these are the characters a to z and A to Z, plus accented
///   characters.
struct AtSymbol;

impl From<AtSymbol> for u8 {
    fn from(_: AtSymbol) -> Self {
        64
    }
}

impl From<AtSymbol> for Vec<(u8, u8)> {
    fn from(_: AtSymbol) -> Self {
        vec![(65, 90), (97, 122), (192, 255)]
    }
}

impl TryFrom<CharSpec> for u8 {
    type Error = AtSymbol;

    /// Convert `value` to `char`, treating '@' as a special case.
    fn try_from(value: CharSpec) -> Result<Self, Self::Error> {
        match value {
            CharSpec::Number(num) => Ok(num.into()),
            CharSpec::Char(ch) => {
                if ch == '@' {
                    Err(AtSymbol)
                } else {
                    if ch as u32 <= u8::MAX as u32 {
                        Ok(ch as u8)
                    } else {
                        panic!("CharSpec holds non-ASCII char: {}", ch)
                    }
                }
            }
        }
    }
}

impl TryFrom<Item> for (u8, u8) {
    type Error = AtSymbol;

    /// Convert `value` to `(u8, u8)`, treating '@' outside a range as a
    /// special case.
    fn try_from(value: Item) -> Result<Self, Self::Error> {
        match value {
            Item::CharSpec(cs) => u8::try_from(cs).map(|ch| (ch, ch)),
            Item::Range(lhs, rhs) => {
                let lhs = lhs.try_into().unwrap_or_else(Into::into);
                let rhs = rhs.try_into().unwrap_or_else(Into::into);
                Ok((lhs, rhs))
            }
        }
    }
}

/// Predicate for whether an ASCII or unicode is a word.
pub struct WordPredicate {
    /// Set of ASCII characters defined by doubly inclusive intervals. When
    /// there's no interval, the set is an empty set.
    ascii_set: Intervals<u8>,
    /// True if '@' is included.
    include_alphabetic: bool,
}

impl WordPredicate {
    fn new() -> Self {
        Self {
            ascii_set: Intervals::default(),
            include_alphabetic: false,
        }
    }

    fn add_part(&mut self, part: Part) {
        match part {
            Part::Part(item) => match item.try_into() {
                Ok(interval) => {
                    self.ascii_set.push(interval);
                    self.ascii_set.merge();
                }
                Err(at) => {
                    let at_intervals: Vec<_> = at.into();
                    self.ascii_set.append(at_intervals);
                    self.ascii_set.merge();
                    self.include_alphabetic = true;
                }
            },
            Part::NegPart(item) => match item.try_into() {
                Ok(interval) => self.ascii_set.remove(&interval),
                Err(at) => {
                    let at_intervals: Vec<_> = at.into();
                    for interval in at_intervals.iter() {
                        self.ascii_set.remove(interval);
                    }
                    self.include_alphabetic = false;
                }
            },
        }
    }

    /// Try to construct a `WordPredicate` from `'iskeyword'` option value.
    pub fn try_from_isk(
        isk_parser: &IskParser,
        value: &str,
    ) -> Result<Self, Error> {
        let mut wp = Self::new();
        for part in isk_parser.parse(value)? {
            wp.add_part(part);
        }
        Ok(wp)
    }

    /// Check if `ascii` is a word. Panics if `ascii` cannot be converted to
    /// u8.
    pub fn is_word(&self, ascii: char) -> bool {
        if ascii as u32 <= u8::MAX as u32 {
            let ascii = ascii as u8;
            self.ascii_set.contains(&ascii)
        } else {
            panic!("char is not ascii: {}", ascii);
        }
    }

    /// Check if a unicode alphabet like 汉字 is a word.
    pub fn is_unicode_alphabet_word(&self) -> bool {
        self.include_alphabetic
    }
}

#[cfg(test)]
mod tests {
    use jieba_vim_rs_test::assert_elapsed::AssertElapsed;

    use super::*;

    fn parse_isk_test(
        parser: &IskParser,
        value: &str,
    ) -> Result<Vec<Part>, Error> {
        let timing = AssertElapsed::tic(5);
        let parts = parser.parse(value);
        timing.toc();
        parts
    }

    #[test]
    fn test_grammar() {
        let timing = AssertElapsed::tic(5);
        let parser = IskParser::new();
        timing.toc();

        assert_eq!(
            parse_isk_test(&parser, "48").unwrap(),
            vec![Part::Part(Item::CharSpec(CharSpec::Number(48)))]
        );
        assert_eq!(
            parse_isk_test(&parser, "#-43").unwrap(),
            vec![Part::Part(Item::Range(
                CharSpec::Char('#'),
                CharSpec::Number(43)
            ))]
        );
        assert_eq!(
            parse_isk_test(&parser, "128-140").unwrap(),
            vec![Part::Part(Item::Range(
                CharSpec::Number(128),
                CharSpec::Number(140)
            ))]
        );
        assert_eq!(
            parse_isk_test(&parser, "--57").unwrap(),
            vec![Part::Part(Item::Range(
                CharSpec::Char('-'),
                CharSpec::Number(57)
            ))]
        );
        assert_eq!(
            parse_isk_test(&parser, "---").unwrap(),
            vec![Part::Part(Item::Range(
                CharSpec::Char('-'),
                CharSpec::Char('-')
            ))]
        );
        assert_eq!(
            parse_isk_test(&parser, "^a-z").unwrap(),
            vec![Part::NegPart(Item::Range(
                CharSpec::Char('a'),
                CharSpec::Char('z')
            ))]
        );
        assert_eq!(
            parse_isk_test(&parser, "93-95,^^").unwrap(),
            vec![
                Part::Part(Item::Range(
                    CharSpec::Number(93),
                    CharSpec::Number(95)
                )),
                Part::NegPart(Item::CharSpec(CharSpec::Char('^')))
            ]
        );
        assert_eq!(
            parse_isk_test(&parser, "^93-^").unwrap(),
            vec![Part::NegPart(Item::Range(
                CharSpec::Number(93),
                CharSpec::Char('^')
            ))]
        );
        assert_eq!(
            parse_isk_test(&parser, "^48-^,,,^").unwrap(),
            vec![
                Part::NegPart(Item::Range(
                    CharSpec::Number(48),
                    CharSpec::Char('^')
                )),
                Part::Part(Item::CharSpec(CharSpec::Char(','))),
                Part::Part(Item::CharSpec(CharSpec::Char('^')))
            ]
        );
        assert_eq!(
            parse_isk_test(&parser, "48-57,93-^").unwrap(),
            vec![
                Part::Part(Item::Range(
                    CharSpec::Number(48),
                    CharSpec::Number(57)
                )),
                Part::Part(Item::Range(
                    CharSpec::Number(93),
                    CharSpec::Char('^')
                ))
            ]
        );
        assert_eq!(
            parse_isk_test(&parser, "^a-z,#,^").unwrap(),
            vec![
                Part::NegPart(Item::Range(
                    CharSpec::Char('a'),
                    CharSpec::Char('z')
                )),
                Part::Part(Item::CharSpec(CharSpec::Char('#'))),
                Part::Part(Item::CharSpec(CharSpec::Char('^')))
            ]
        );
        assert_eq!(
            parse_isk_test(&parser, "^a-z,#,^^").unwrap(),
            vec![
                Part::NegPart(Item::Range(
                    CharSpec::Char('a'),
                    CharSpec::Char('z')
                )),
                Part::Part(Item::CharSpec(CharSpec::Char('#'))),
                Part::NegPart(Item::CharSpec(CharSpec::Char('^')))
            ]
        );
        parse_isk_test(&parser, "^-^").unwrap_err();
        assert_eq!(
            parse_isk_test(&parser, "@").unwrap(),
            vec![Part::Part(Item::CharSpec(CharSpec::Char('@')))]
        );
        assert_eq!(
            parse_isk_test(&parser, "@-@").unwrap(),
            vec![Part::Part(Item::Range(
                CharSpec::Char('@'),
                CharSpec::Char('@')
            ))]
        );
        assert_eq!(
            parse_isk_test(&parser, "@-65").unwrap(),
            vec![Part::Part(Item::Range(
                CharSpec::Char('@'),
                CharSpec::Number(65)
            ))]
        );
        assert_eq!(
            parse_isk_test(&parser, "_,-,128-140,^#-43").unwrap(),
            vec![
                Part::Part(Item::CharSpec(CharSpec::Char('_'))),
                Part::Part(Item::CharSpec(CharSpec::Char('-'))),
                Part::Part(Item::Range(
                    CharSpec::Number(128),
                    CharSpec::Number(140)
                )),
                Part::NegPart(Item::Range(
                    CharSpec::Char('#'),
                    CharSpec::Number(43)
                ))
            ]
        );
        assert_eq!(
            parse_isk_test(&parser, "@,^a-z").unwrap(),
            vec![
                Part::Part(Item::CharSpec(CharSpec::Char('@'))),
                Part::NegPart(Item::Range(
                    CharSpec::Char('a'),
                    CharSpec::Char('z')
                ))
            ]
        );
        assert_eq!(
            parse_isk_test(&parser, "a-z,A-Z,@-@").unwrap(),
            vec![
                Part::Part(Item::Range(
                    CharSpec::Char('a'),
                    CharSpec::Char('z')
                )),
                Part::Part(Item::Range(
                    CharSpec::Char('A'),
                    CharSpec::Char('Z')
                )),
                Part::Part(Item::Range(
                    CharSpec::Char('@'),
                    CharSpec::Char('@')
                ))
            ]
        );
        assert_eq!(
            parse_isk_test(&parser, ",").unwrap(),
            vec![Part::Part(Item::CharSpec(CharSpec::Char(',')))]
        );
        assert_eq!(
            parse_isk_test(&parser, ",,,").unwrap(),
            vec![
                Part::Part(Item::CharSpec(CharSpec::Char(','))),
                Part::Part(Item::CharSpec(CharSpec::Char(',')))
            ]
        );
        assert_eq!(
            parse_isk_test(&parser, "48-57,,,_").unwrap(),
            vec![
                Part::Part(Item::Range(
                    CharSpec::Number(48),
                    CharSpec::Number(57)
                )),
                Part::Part(Item::CharSpec(CharSpec::Char(','))),
                Part::Part(Item::CharSpec(CharSpec::Char('_')))
            ]
        );
        assert_eq!(
            parse_isk_test(&parser, "32-~,^,,9").unwrap(),
            vec![
                Part::Part(Item::Range(
                    CharSpec::Number(32),
                    CharSpec::Char('~')
                )),
                Part::NegPart(Item::CharSpec(CharSpec::Char(','))),
                Part::Part(Item::CharSpec(CharSpec::Number(9)))
            ]
        );
        assert_eq!(
            parse_isk_test(&parser, ",,^,,^^,^").unwrap(),
            vec![
                Part::Part(Item::CharSpec(CharSpec::Char(','))),
                Part::NegPart(Item::CharSpec(CharSpec::Char(','))),
                Part::NegPart(Item::CharSpec(CharSpec::Char('^'))),
                Part::Part(Item::CharSpec(CharSpec::Char('^')))
            ]
        );
        assert_eq!(
            parse_isk_test(&parser, "^^,,,^,,^").unwrap(),
            vec![
                Part::NegPart(Item::CharSpec(CharSpec::Char('^'))),
                Part::Part(Item::CharSpec(CharSpec::Char(','))),
                Part::NegPart(Item::CharSpec(CharSpec::Char(','))),
                Part::Part(Item::CharSpec(CharSpec::Char('^')))
            ]
        );
        assert_eq!(
            parse_isk_test(&parser, "^^-^").unwrap(),
            vec![Part::NegPart(Item::Range(
                CharSpec::Char('^'),
                CharSpec::Char('^')
            ))]
        );
        assert_eq!(
            parse_isk_test(&parser, "^").unwrap(),
            vec![Part::Part(Item::CharSpec(CharSpec::Char('^')))]
        );
        assert_eq!(
            parse_isk_test(&parser, "^^").unwrap(),
            vec![Part::NegPart(Item::CharSpec(CharSpec::Char('^')))]
        );
        // 'iskeyword' default for Win32.
        assert_eq!(
            parse_isk_test(&parser, "@,48-57,_,128-167,224-235").unwrap(),
            vec![
                Part::Part(Item::CharSpec(CharSpec::Char('@'))),
                Part::Part(Item::Range(
                    CharSpec::Number(48),
                    CharSpec::Number(57)
                )),
                Part::Part(Item::CharSpec(CharSpec::Char('_'))),
                Part::Part(Item::Range(
                    CharSpec::Number(128),
                    CharSpec::Number(167)
                )),
                Part::Part(Item::Range(
                    CharSpec::Number(224),
                    CharSpec::Number(235)
                )),
            ]
        );
        // 'iskeyword' value for vim help.
        assert_eq!(
            parse_isk_test(&parser, r#"!-~,^*,^|,^",192-255"#).unwrap(),
            vec![
                Part::Part(Item::Range(
                    CharSpec::Char('!'),
                    CharSpec::Char('~')
                )),
                Part::NegPart(Item::CharSpec(CharSpec::Char('*'))),
                Part::NegPart(Item::CharSpec(CharSpec::Char('|'))),
                Part::NegPart(Item::CharSpec(CharSpec::Char('"'))),
                Part::Part(Item::Range(
                    CharSpec::Number(192),
                    CharSpec::Number(255)
                ))
            ]
        );
        // Recommended 'iskeyword' value for C.
        assert_eq!(
            parse_isk_test(&parser, "a-z,A-Z,48-57,_,.,-,>").unwrap(),
            vec![
                Part::Part(Item::Range(
                    CharSpec::Char('a'),
                    CharSpec::Char('z')
                )),
                Part::Part(Item::Range(
                    CharSpec::Char('A'),
                    CharSpec::Char('Z')
                )),
                Part::Part(Item::Range(
                    CharSpec::Number(48),
                    CharSpec::Number(57)
                )),
                Part::Part(Item::CharSpec(CharSpec::Char('_'))),
                Part::Part(Item::CharSpec(CharSpec::Char('.'))),
                Part::Part(Item::CharSpec(CharSpec::Char('-'))),
                Part::Part(Item::CharSpec(CharSpec::Char('>')))
            ]
        );
    }

    fn merge_intervals_test<T: Ord + Copy>(
        intervals: Vec<(T, T)>,
    ) -> Vec<(T, T)> {
        let mut intervals = Intervals(intervals);
        intervals.merge();
        intervals.0
    }

    #[test]
    fn test_merge_intervals() {
        let itvls = vec![(1, 5), (6, 7)];
        assert_eq!(merge_intervals_test(itvls), vec![(1, 5), (6, 7)]);

        let itvls = vec![(1, 5), (5, 7)];
        assert_eq!(merge_intervals_test(itvls), vec![(1, 7)]);

        let itvls = vec![(1, 5), (3, 7)];
        assert_eq!(merge_intervals_test(itvls), vec![(1, 7)]);

        let itvls = vec![(1, 5), (0, 7)];
        assert_eq!(merge_intervals_test(itvls), vec![(0, 7)]);

        let itvls = vec![(1, 5), (3, 4)];
        assert_eq!(merge_intervals_test(itvls), vec![(1, 5)]);

        let itvls = vec![(1, 5), (1, 5)];
        assert_eq!(merge_intervals_test(itvls), vec![(1, 5)]);

        let itvls = vec![(1, 5), (0, 3)];
        assert_eq!(merge_intervals_test(itvls), vec![(0, 5)]);

        let itvls = vec![(1, 5), (0, 1)];
        assert_eq!(merge_intervals_test(itvls), vec![(0, 5)]);

        let itvls = vec![(1, 5), (0, 0)];
        assert_eq!(merge_intervals_test(itvls), vec![(0, 0), (1, 5)]);

        let itvls = vec![(1, 5), (7, 10), (6, 7)];
        assert_eq!(merge_intervals_test(itvls), vec![(1, 5), (6, 10)]);

        let itvls = vec![(1, 5), (7, 10), (3, 8)];
        assert_eq!(merge_intervals_test(itvls), vec![(1, 10)]);

        let itvls = vec![(1, 5), (8, 10), (15, 17), (7, 11)];
        assert_eq!(
            merge_intervals_test(itvls),
            vec![(1, 5), (7, 11), (15, 17)]
        );

        let itvls = vec![(1, 5), (8, 10), (15, 17), (5, 8)];
        assert_eq!(merge_intervals_test(itvls), vec![(1, 10), (15, 17)]);

        let itvls = vec![(1, 5), (8, 10), (15, 17), (2, 8), (9, 20)];
        assert_eq!(merge_intervals_test(itvls), vec![(1, 20)]);
    }

    #[test]
    fn test_retain_mut() {
        fn rule(e: &mut i32) -> bool {
            if e != &0 {
                *e *= 2;
                true
            } else {
                false
            }
        }

        let mut v = vec![];
        v.retain_mut(rule);
        assert!(v.is_empty());

        let mut v = vec![0];
        v.retain_mut(rule);
        assert!(v.is_empty());

        let mut v = vec![2];
        v.retain_mut(rule);
        assert_eq!(v, vec![4]);

        let mut v = vec![2, 0];
        v.retain_mut(rule);
        assert_eq!(v, vec![4]);

        let mut v = vec![0, 2];
        v.retain_mut(rule);
        assert_eq!(v, vec![4]);

        let mut v = vec![0, 2, 0];
        v.retain_mut(rule);
        assert_eq!(v, vec![4]);

        let mut v = vec![2, 3, 0, 0, 1, 0, 4, 5, 6, 0];
        v.retain_mut(rule);
        assert_eq!(v, vec![4, 6, 2, 8, 10, 12]);

        let mut v = vec![0, 0, 2, 3, 0, 0, 1, 0, 4, 5, 6, 0];
        v.retain_mut(rule);
        assert_eq!(v, vec![4, 6, 2, 8, 10, 12]);
    }

    fn remove_interval_test<T: BoundedChain + Copy>(
        intervals: Vec<(T, T)>,
        r: &(T, T),
    ) -> Vec<(T, T)> {
        let mut intervals = Intervals(intervals);
        intervals.remove(r);
        intervals.0
    }

    #[test]
    fn test_remove_interval() {
        assert_eq!(remove_interval_test(vec![(5, 10)], &(0, 3)), vec![(5, 10)]);
        assert_eq!(remove_interval_test(vec![(5, 10)], &(0, 5)), vec![(6, 10)]);
        assert_eq!(remove_interval_test(vec![(5, 10)], &(0, 6)), vec![(7, 10)]);
        assert!(remove_interval_test(vec![(5, 10)], &(0, 10)).is_empty());
        assert!(remove_interval_test(vec![(5, 10)], &(0, 13)).is_empty());
        assert_eq!(remove_interval_test(vec![(5, 10)], &(5, 5)), vec![(6, 10)]);
        assert_eq!(remove_interval_test(vec![(5, 10)], &(5, 6)), vec![(7, 10)]);
        assert!(remove_interval_test(vec![(5, 10)], &(5, 10)).is_empty());
        assert_eq!(
            remove_interval_test(vec![(5, 10)], &(6, 6)),
            vec![(5, 5), (7, 10)]
        );
        assert_eq!(
            remove_interval_test(vec![(5, 10)], &(6, 8)),
            vec![(5, 5), (9, 10)]
        );
        assert_eq!(remove_interval_test(vec![(5, 10)], &(6, 10)), vec![(5, 5)]);
        assert_eq!(remove_interval_test(vec![(5, 10)], &(6, 13)), vec![(5, 5)]);
        assert_eq!(
            remove_interval_test(vec![(5, 10)], &(10, 10)),
            vec![(5, 9)]
        );
        assert_eq!(
            remove_interval_test(vec![(5, 10)], &(10, 12)),
            vec![(5, 9)]
        );
        assert_eq!(
            remove_interval_test(vec![(5, 10)], &(11, 13)),
            vec![(5, 10)]
        );
        assert_eq!(remove_interval_test(vec![(5, 5)], &(0, 3)), vec![(5, 5)]);
        assert!(remove_interval_test(vec![(5, 5)], &(0, 5)).is_empty());
        assert!(remove_interval_test(vec![(5, 5)], &(0, 6)).is_empty());
        assert!(remove_interval_test(vec![(5, 5)], &(5, 5)).is_empty());
        assert!(remove_interval_test(vec![(5, 5)], &(5, 7)).is_empty());
        assert_eq!(remove_interval_test(vec![(5, 5)], &(7, 8)), vec![(5, 5)]);
    }
}
