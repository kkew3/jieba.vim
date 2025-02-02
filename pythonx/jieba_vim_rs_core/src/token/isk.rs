//! Categorize characters into words, nonwords and spaces, based on
//! `'iskeyword'` Vim option.

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

#[derive(Debug)]
enum Error {
    Lexer,
    Parser,
}

impl From<LexerError> for Error {
    fn from(_: LexerError) -> Self {
        Error::Lexer
    }
}

impl From<ParseError<Ast>> for Error {
    fn from(_: ParseError<Ast>) -> Self {
        Error::Parser
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

struct IskParser {
    lexer_rules: LexerRules,
    grammar: Grammar<Ast>,
}

impl IskParser {
    fn new() -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar() {
        let parser = IskParser::new();

        assert_eq!(
            parser.parse("48").unwrap(),
            vec![Part::Part(Item::CharSpec(CharSpec::Number(48)))]
        );
        assert_eq!(
            parser.parse("#-43").unwrap(),
            vec![Part::Part(Item::Range(
                CharSpec::Char('#'),
                CharSpec::Number(43)
            ))]
        );
        assert_eq!(
            parser.parse("128-140").unwrap(),
            vec![Part::Part(Item::Range(
                CharSpec::Number(128),
                CharSpec::Number(140)
            ))]
        );
        assert_eq!(
            parser.parse("--57").unwrap(),
            vec![Part::Part(Item::Range(
                CharSpec::Char('-'),
                CharSpec::Number(57)
            ))]
        );
        assert_eq!(
            parser.parse("---").unwrap(),
            vec![Part::Part(Item::Range(
                CharSpec::Char('-'),
                CharSpec::Char('-')
            ))]
        );
        assert_eq!(
            parser.parse("^a-z").unwrap(),
            vec![Part::NegPart(Item::Range(
                CharSpec::Char('a'),
                CharSpec::Char('z')
            ))]
        );
        assert_eq!(
            parser.parse("93-95,^^").unwrap(),
            vec![
                Part::Part(Item::Range(
                    CharSpec::Number(93),
                    CharSpec::Number(95)
                )),
                Part::NegPart(Item::CharSpec(CharSpec::Char('^')))
            ]
        );
        assert_eq!(
            parser.parse("^93-^").unwrap(),
            vec![Part::NegPart(Item::Range(
                CharSpec::Number(93),
                CharSpec::Char('^')
            ))]
        );
        assert_eq!(
            parser.parse("^48-^,,,^").unwrap(),
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
            parser.parse("48-57,93-^").unwrap(),
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
            parser.parse("^a-z,#,^").unwrap(),
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
            parser.parse("^a-z,#,^^").unwrap(),
            vec![
                Part::NegPart(Item::Range(
                    CharSpec::Char('a'),
                    CharSpec::Char('z')
                )),
                Part::Part(Item::CharSpec(CharSpec::Char('#'))),
                Part::NegPart(Item::CharSpec(CharSpec::Char('^')))
            ]
        );
        parser.parse("^-^").unwrap_err();
        assert_eq!(
            parser.parse("@").unwrap(),
            vec![Part::Part(Item::CharSpec(CharSpec::Char('@')))]
        );
        assert_eq!(
            parser.parse("@-@").unwrap(),
            vec![Part::Part(Item::Range(
                CharSpec::Char('@'),
                CharSpec::Char('@')
            ))]
        );
        assert_eq!(
            parser.parse("@-65").unwrap(),
            vec![Part::Part(Item::Range(
                CharSpec::Char('@'),
                CharSpec::Number(65)
            ))]
        );
        assert_eq!(
            parser.parse("_,-,128-140,^#-43").unwrap(),
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
            parser.parse("@,^a-z").unwrap(),
            vec![
                Part::Part(Item::CharSpec(CharSpec::Char('@'))),
                Part::NegPart(Item::Range(
                    CharSpec::Char('a'),
                    CharSpec::Char('z')
                ))
            ]
        );
        assert_eq!(
            parser.parse("a-z,A-Z,@-@").unwrap(),
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
            parser.parse(",").unwrap(),
            vec![Part::Part(Item::CharSpec(CharSpec::Char(',')))]
        );
        assert_eq!(
            parser.parse(",,,").unwrap(),
            vec![
                Part::Part(Item::CharSpec(CharSpec::Char(','))),
                Part::Part(Item::CharSpec(CharSpec::Char(',')))
            ]
        );
        assert_eq!(
            parser.parse("48-57,,,_").unwrap(),
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
            parser.parse("32-~,^,,9").unwrap(),
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
            parser.parse(",,^,,^^,^").unwrap(),
            vec![
                Part::Part(Item::CharSpec(CharSpec::Char(','))),
                Part::NegPart(Item::CharSpec(CharSpec::Char(','))),
                Part::NegPart(Item::CharSpec(CharSpec::Char('^'))),
                Part::Part(Item::CharSpec(CharSpec::Char('^')))
            ]
        );
        assert_eq!(
            parser.parse("^^,,,^,,^").unwrap(),
            vec![
                Part::NegPart(Item::CharSpec(CharSpec::Char('^'))),
                Part::Part(Item::CharSpec(CharSpec::Char(','))),
                Part::NegPart(Item::CharSpec(CharSpec::Char(','))),
                Part::Part(Item::CharSpec(CharSpec::Char('^')))
            ]
        );
        // 'iskeyword' value for vim help.
        assert_eq!(
            parser.parse(r#"!-~,^*,^|,^",192-255"#).unwrap(),
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
            parser.parse("a-z,A-Z,48-57,_,.,-,>").unwrap(),
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
}
