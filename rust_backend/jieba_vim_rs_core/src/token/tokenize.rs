// Copyright 2025 Kaiwen Wu. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

//! This module defines the tokens, and the tokenizer.

use crate::utils;

use super::JiebaPlaceholder;
use super::char::{self, CharType, NonWordCharType, WordCharType};
use super::isk::WordPredicate;

/// The tokenizer.
pub struct Tokenizer<C> {
    word_predicate: WordPredicate,
    jieba: C,
}

impl<C> Tokenizer<C> {
    /// Create a new tokenizer from `'iskeyword'` option value.
    pub fn try_new<P: TryInto<WordPredicate>>(
        jieba: C,
        word_predicate: P,
    ) -> Result<Self, P::Error> {
        Ok(Self {
            word_predicate: word_predicate.try_into()?,
            jieba,
        })
    }

    /// Create a new tokenizer from `'iskeyword'` option value. Panics if the
    /// option value is invalid.
    #[cfg(test)]
    pub(crate) fn new<P: TryInto<WordPredicate>>(
        jieba: C,
        word_predicate: P,
    ) -> Self
    where
        P::Error: std::fmt::Debug,
    {
        Self {
            word_predicate: word_predicate.try_into().unwrap(),
            jieba,
        }
    }

    pub fn try_set_word_predicate<P: TryInto<WordPredicate>>(
        &mut self,
        word_predicate: P,
    ) -> Result<(), P::Error> {
        self.word_predicate = word_predicate.try_into()?;
        Ok(())
    }

    pub fn get_word_predicate_mut(&mut self) -> &mut WordPredicate {
        &mut self.word_predicate
    }
}

impl<C: JiebaPlaceholder> JiebaPlaceholder for Tokenizer<C> {
    fn cut_hmm<'a>(&self, sentence: &'a str) -> Vec<&'a str> {
        self.jieba.cut_hmm(sentence)
    }
}

/// Any type that can be thought of as a token.
pub trait TokenLike {
    /// The byte position of the first character in the token.
    fn first_char(&self) -> usize;

    /// The byte position of the last character in the token.
    fn last_char(&self) -> usize;

    /// The byte position of the end of the last character in the token.
    fn last_char1(&self) -> usize;
}

fn get_token<'a, T: TokenLike>(line: &'a str, token: &T) -> &'a str {
    &line[token.first_char()..token.last_char1()]
}

/// The column location of a token in a line.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Col {
    /// The byte offset of the first char in the token.
    start_byte_index: usize,
    /// The byte offset of the last char in the token.
    incl_end_byte_index: usize,
    /// [`Col::incl_end_byte_index`] plus the byte length of the last char in
    /// utf-8.
    excl_end_byte_index: usize,
}

/// Implement [`TokenLike`] for a type (first argument) given its [`Col`]
/// variable (second argument), which is assumed to be named "col".
///
/// Example:
///
/// ```ignore
/// struct Token { col: Col };
/// impl_token_like_from_col!(Token);
/// ```
macro_rules! impl_token_like_from_col {
    ($cls:ident) => {
        impl TokenLike for $cls {
            fn first_char(&self) -> usize {
                self.col.start_byte_index
            }

            fn last_char(&self) -> usize {
                self.col.incl_end_byte_index
            }

            fn last_char1(&self) -> usize {
                self.col.excl_end_byte_index
            }
        }
    };
}

/// A Char token.
#[derive(Debug)]
struct CharToken {
    col: Col,
    ty: CharType,
}

impl_token_like_from_col!(CharToken);

impl<C> Tokenizer<C> {
    fn new_char_token(&self, ch: char, start_byte_index: usize) -> CharToken {
        CharToken {
            col: Col {
                start_byte_index,
                incl_end_byte_index: start_byte_index,
                excl_end_byte_index: start_byte_index + ch.len_utf8(),
            },
            ty: char::categorize_char(ch, &self.word_predicate),
        }
    }

    /// The string `line` should not contain the end-of-line character. Return
    /// a vec of [`CharToken`]s. An empty returned vec signifies that the
    /// `line` is empty. Normally `byte_offset` should be zero.
    fn parse_str_into_chars(
        &self,
        line: &str,
        byte_offset: usize,
    ) -> Vec<CharToken> {
        line.char_indices()
            .map(|(start_byte_index, ch)| {
                self.new_char_token(ch, start_byte_index + byte_offset)
            })
            .collect()
    }
}

/// Character group types.
#[derive(Debug, PartialEq, Eq)]
enum CharGroupType {
    /// A sequence of [`CharType::Space`] characters.
    Space,
    /// A sequence of [`CharType::Word`] characters.
    Word(WordCharGroupType),
    /// A sequence of [`CharType::NonWord`] characters.
    NonWord(NonWordCharGroupType),
    /// A sequence of [`CharType::CombiningDiacriticalMark`]. We have to make
    /// room dedicated for this type (abbr. CDM), since in terms of major
    /// class, CDM is compatible with non-汉字 word, non-汉字 word is compatible
    /// with 汉字 word, but CDM is *not* compatible with 汉字 word.
    CombiningDiacriticalMark,
    /// A sequence of [`CharType::Emoji`].
    Emoji,
}

/// Word character group types.
#[derive(Debug, PartialEq, Eq)]
enum WordCharGroupType {
    /// A sequence of [`CharType::Word`] that contains [`WordCharType::Hanzi`].
    Hanzi,
    /// A sequence of [`CharType::Word`] that doesn't contain
    /// [`WordCharType::Hanzi`].
    Other,
}

/// Non-word character group types.
#[derive(Debug, PartialEq, Eq)]
enum NonWordCharGroupType {
    /// A sequence of [`CharType::Word`] that contains [`WordCharType::Hanzi`].
    /// This happens when '@' is not in `'iskeyword'`.
    Hanzi,
    /// A sequence of [`CharType::NonWord`] that ends with a
    /// [`NonWordCharType::RightPunc`].
    RightPuncEnding,
    /// A sequence of [`CharType::NonWord`] that does not end with a
    /// [`NonWordCharType::RightPunc`].
    Other,
}

/// A non-empty group of [`CharToken`]s of compatible types.
#[derive(Debug, PartialEq, Eq)]
struct CharTokenGroup {
    col: Col,
    ty: CharGroupType,
}

impl_token_like_from_col!(CharTokenGroup);

impl From<CharToken> for CharTokenGroup {
    /// Construct a [`CharTokenGroup`] from a [`CharToken`] token.
    fn from(c: CharToken) -> Self {
        Self {
            col: c.col,
            ty: match c.ty {
                CharType::Space => CharGroupType::Space,
                CharType::Word(WordCharType::Hanzi) => {
                    CharGroupType::Word(WordCharGroupType::Hanzi)
                }
                CharType::Word(WordCharType::Other) => {
                    CharGroupType::Word(WordCharGroupType::Other)
                }
                CharType::CombiningDiacriticalMark => {
                    CharGroupType::CombiningDiacriticalMark
                }
                CharType::NonWord(NonWordCharType::RightPunc) => {
                    CharGroupType::NonWord(
                        NonWordCharGroupType::RightPuncEnding,
                    )
                }
                CharType::NonWord(NonWordCharType::Hanzi) => {
                    CharGroupType::NonWord(NonWordCharGroupType::Hanzi)
                }
                CharType::NonWord(NonWordCharType::Other) => {
                    CharGroupType::NonWord(NonWordCharGroupType::Other)
                }
                CharType::Emoji => CharGroupType::Emoji,
            },
        }
    }
}

/// An implicit whitespace.
#[derive(Debug)]
struct ImplicitWhitespace {
    start_byte_index: usize,
}

impl TokenLike for ImplicitWhitespace {
    fn first_char(&self) -> usize {
        self.start_byte_index
    }

    fn last_char(&self) -> usize {
        self.start_byte_index
    }

    fn last_char1(&self) -> usize {
        self.start_byte_index
    }
}

impl ImplicitWhitespace {
    fn new(start_byte_index: usize) -> Self {
        Self { start_byte_index }
    }
}

/// Either a [`CharTokenGroup`] or an [`ImplicitWhitespace`].
enum MaybeImplicitCharTokenGroup {
    CharTokenGroup(CharTokenGroup),
    ImplicitWhitespace(ImplicitWhitespace),
}

impl TokenLike for MaybeImplicitCharTokenGroup {
    fn first_char(&self) -> usize {
        match self {
            Self::CharTokenGroup(cg) => cg.first_char(),
            Self::ImplicitWhitespace(iw) => iw.first_char(),
        }
    }

    fn last_char(&self) -> usize {
        match self {
            Self::CharTokenGroup(cg) => cg.last_char(),
            Self::ImplicitWhitespace(iw) => iw.last_char(),
        }
    }

    fn last_char1(&self) -> usize {
        match self {
            Self::CharTokenGroup(cg) => cg.last_char1(),
            Self::ImplicitWhitespace(iw) => iw.last_char1(),
        }
    }
}

impl From<CharTokenGroup> for MaybeImplicitCharTokenGroup {
    fn from(value: CharTokenGroup) -> Self {
        Self::CharTokenGroup(value)
    }
}

impl From<ImplicitWhitespace> for MaybeImplicitCharTokenGroup {
    fn from(value: ImplicitWhitespace) -> Self {
        Self::ImplicitWhitespace(value)
    }
}

impl MaybeImplicitCharTokenGroup {
    fn to_char_token_group_mut(
        &mut self,
    ) -> Result<&mut CharTokenGroup, &ImplicitWhitespace> {
        match self {
            Self::CharTokenGroup(cg) => Ok(cg),
            Self::ImplicitWhitespace(iw) => Err(iw),
        }
    }
}

/// The `Err` value of [`CharGroup::push`].
#[derive(Debug)]
enum CharTokenGroupPushError {
    /// A [`CharTokenGroup`] consisting of a single char.
    Singleton(CharTokenGroup),
    /// An implicit whitespace followed by a singleton [`CharTokenGroup`].
    WithImplicitSpace(ImplicitWhitespace, CharTokenGroup),
}

impl CharTokenGroupPushError {
    fn into_vec(self) -> Vec<MaybeImplicitCharTokenGroup> {
        match self {
            Self::Singleton(cg) => vec![cg.into()],
            Self::WithImplicitSpace(iw, cg) => vec![iw.into(), cg.into()],
        }
    }
}

impl CharTokenGroup {
    /// Try to push a [`CharToken`]. If `self` and `c` are compatible in
    /// major class (space, word, nonword, emoji). The type of `self` may be
    /// modified accordingly, but it's guaranteed that the major class will not
    /// be changed. If `c` is of type [`CharType::CombiningDiacriticalMark`],
    /// it's guaranteed to be compatible with `self`. Otherwise, return either
    /// a singleton [`CharTokenGroup`] comprised of `c` only, or an implicit
    /// whitespace followed by such `CharGroup`. In this case, `self` will not
    /// be modified. Panics if there's gap between `self` and `c`.
    fn push(&mut self, c: CharToken) -> Result<(), CharTokenGroupPushError> {
        assert_eq!(self.col.excl_end_byte_index, c.col.start_byte_index);

        /// Push `c` into `group` without checking compatibility. Always return
        /// `Ok(())`.
        fn do_push(
            group: &mut CharTokenGroup,
            c: CharToken,
        ) -> Result<(), CharTokenGroupPushError> {
            // Combining diacritical marks modify previous character only, and
            // does not take space.
            match &c.ty {
                CombiningDiacriticalMark => (),
                _ => group.col.incl_end_byte_index = c.col.incl_end_byte_index,
            }
            group.col.excl_end_byte_index = c.col.excl_end_byte_index;
            Ok(())
        }

        use CharGroupType as G;
        use CharType::*;
        use NonWordCharGroupType as NG;
        use NonWordCharType as N;
        use WordCharGroupType as WG;
        use WordCharType as W;
        match (&self.ty, &c.ty) {
            // === Compatible cases ===
            (G::CombiningDiacriticalMark, CombiningDiacriticalMark) => {
                do_push(self, c)
            }
            (G::CombiningDiacriticalMark, Word(W::Other)) => {
                self.ty = G::Word(WG::Other);
                do_push(self, c)
            }

            (G::Space, Space) | (G::Space, CombiningDiacriticalMark) => {
                do_push(self, c)
            }

            (G::Word(WG::Hanzi), Word(_))
            | (G::Word(WG::Hanzi), CombiningDiacriticalMark) => {
                do_push(self, c)
            }

            (G::Word(WG::Other), Word(W::Hanzi)) => {
                self.ty = G::Word(WG::Hanzi);
                do_push(self, c)
            }
            (G::Word(WG::Other), Word(W::Other))
            | (G::Word(WG::Other), CombiningDiacriticalMark) => {
                do_push(self, c)
            }

            (G::NonWord(NG::RightPuncEnding), NonWord(N::Other)) => {
                self.ty = G::NonWord(NG::Other);
                do_push(self, c)
            }
            (G::NonWord(NG::RightPuncEnding), NonWord(N::RightPunc))
            | (G::NonWord(NG::RightPuncEnding), CombiningDiacriticalMark) => {
                do_push(self, c)
            }
            (G::NonWord(NG::Hanzi), NonWord(N::Hanzi))
            | (G::NonWord(NG::Hanzi), CombiningDiacriticalMark) => {
                do_push(self, c)
            }

            (G::NonWord(NG::Other), NonWord(N::Other))
            | (G::NonWord(NG::Other), CombiningDiacriticalMark) => {
                do_push(self, c)
            }
            (G::NonWord(NG::Other), NonWord(N::RightPunc)) => {
                self.ty = G::NonWord(NG::RightPuncEnding);
                do_push(self, c)
            }

            (G::Emoji, Emoji) | (G::Emoji, CombiningDiacriticalMark) => {
                do_push(self, c)
            }

            // === Not compatible cases ===
            (G::CombiningDiacriticalMark, Word(W::Hanzi))
            | (G::CombiningDiacriticalMark, NonWord(_))
            | (G::CombiningDiacriticalMark, Space)
            | (G::CombiningDiacriticalMark, Emoji) => {
                Err(CharTokenGroupPushError::Singleton(c.into()))
            }

            // We never need to insert implicit space after a space.
            (G::Space, Word(_))
            | (G::Space, NonWord(_))
            | (G::Space, Emoji) => {
                Err(CharTokenGroupPushError::Singleton(c.into()))
            }

            (G::Word(_), Space)
            | (G::Word(_), NonWord(_))
            | (G::Word(_), Emoji) => {
                Err(CharTokenGroupPushError::Singleton(c.into()))
            }

            (G::NonWord(NG::Hanzi), NonWord(_)) => {
                Err(CharTokenGroupPushError::Singleton(c.into()))
            }
            (G::NonWord(NG::RightPuncEnding), Word(_)) => {
                let c = CharTokenGroup::from(c);
                let ispace = ImplicitWhitespace::new(c.col.start_byte_index);
                Err(CharTokenGroupPushError::WithImplicitSpace(ispace, c))
            }
            (G::NonWord(_), Space)
            | (G::NonWord(_), Word(_))
            | (G::NonWord(_), NonWord(N::Hanzi))
            | (G::NonWord(_), Emoji) => {
                Err(CharTokenGroupPushError::Singleton(c.into()))
            }

            (G::Emoji, Space)
            | (G::Emoji, Word(_))
            | (G::Emoji, NonWord(_)) => {
                Err(CharTokenGroupPushError::Singleton(c.into()))
            }
        }
    }

    /// Append `group` after `self`. The type of `self` won't be changed.
    /// Panics if there's gap between `self` and `other`. Note that we are
    /// not handling the case where `other` is a combining diacritical mark
    /// as in [`CharTokenGroup::push`]. This is because whether or not `self`
    /// is [`CharGroupType::CombiningDiacriticalMark`],  `other` won't be a
    /// combining diacritical mark too, as it would have been pushed to `self`.
    /// Therefore, we don't need to worry about `other` being such type as long
    /// as this method is called after [`CharTokenGroup::push`].
    fn append(&mut self, other: CharTokenGroup) {
        assert_eq!(self.col.excl_end_byte_index, other.col.start_byte_index);
        self.col.incl_end_byte_index = other.col.incl_end_byte_index;
        self.col.excl_end_byte_index = other.col.excl_end_byte_index;
    }
}

impl<C> Tokenizer<C> {
    /// Construct a [`CharTokenGroup`] from string `s` by first constructing
    /// a `CharGroup` from the first char, and then iteratively pushing
    /// subsequent chars. Return None if `s` is empty. Panics if any
    /// [`CharTokenGroup::push`] fails. Used as a convenient function in tests
    /// only.
    #[cfg(test)]
    fn new_char_group_from_str(
        &self,
        s: &str,
        byte_offset: usize,
    ) -> Option<CharTokenGroup> {
        let mut it = self.parse_str_into_chars(s, byte_offset).into_iter();
        it.next().map(|ch| {
            let mut cg = CharTokenGroup::from(ch);
            for ch in it {
                cg.push(ch).unwrap();
            }
            cg
        })
    }
}

/// Group contiguous [`CharToken`]s of compatible major class into
/// [`CharTokenGroup`]s, and insert implicit whitespaces in between as needed.
/// It's guaranteed that the last item of the returned Vec is a
/// [`CharTokenGroup`].
fn group_chars_rule(
    group: Option<MaybeImplicitCharTokenGroup>,
    c: CharToken,
) -> Vec<MaybeImplicitCharTokenGroup> {
    match group {
        None => vec![CharTokenGroup::from(c).into()],
        Some(mut group) => match &mut group {
            MaybeImplicitCharTokenGroup::CharTokenGroup(cg) => {
                match cg.push(c) {
                    Err(joined) => {
                        utils::chain_into_vec([group], joined.into_vec())
                    }
                    Ok(()) => vec![group],
                }
            }
            // This is unreachable because in all cases, the top element of the
            // underlying stack is never an `ImplicitWhitespace`.
            _ => unreachable!(),
        },
    }
}

/// See [`group_chars_rule`] for details. It's guaranteed that the returned Vec
/// never starts or ends with an [`ImplicitWhitespace`].
fn group_chars(chars: Vec<CharToken>) -> Vec<MaybeImplicitCharTokenGroup> {
    utils::stack_merge(chars, group_chars_rule)
}

/// If the first [`CharTokenGroup`] is of type
/// [`CharGroupType::CombiningDiacriticalMark`], convert it to
/// [`WordCharGroupType::Other`]. Panics if it's an implicit whitespace
/// instead.
fn convert_first_cdm_group_rule(
    prev_group: Option<MaybeImplicitCharTokenGroup>,
    mut group: MaybeImplicitCharTokenGroup,
) -> Vec<MaybeImplicitCharTokenGroup> {
    match prev_group {
        None => {
            // If it panics, it should happen here.
            let cg = group.to_char_token_group_mut().unwrap();
            if let CharGroupType::CombiningDiacriticalMark = cg.ty {
                cg.ty = CharGroupType::Word(WordCharGroupType::Other);
            }
            vec![group]
        }
        Some(prev_group) => vec![prev_group, group],
    }
}

/// See [`convert_first_cdm_group_rule`] for details. Panics if the first token
/// group is an implicit whitespace. If this is run after [`group_chars`], the
/// first token group should never be an implicit whitespace, and thus, panic
/// should not happen.
fn convert_first_cdm_group(
    groups: Vec<MaybeImplicitCharTokenGroup>,
) -> Vec<MaybeImplicitCharTokenGroup> {
    utils::stack_merge(groups, convert_first_cdm_group_rule)
}

/// If the first [`CharTokenGroup`] is of type
/// [`CharGroupType::CombiningDiacriticalMark`], convert it to
/// [`WordCharGroupType::Other`]. Contrary to [`convert_first_cdm_group_rule`],
/// this one never panics.
fn convert_first_cdm_group_rule2(
    prev_group: Option<CharTokenGroup>,
    mut group: CharTokenGroup,
) -> Vec<CharTokenGroup> {
    match prev_group {
        None => {
            if let CharGroupType::CombiningDiacriticalMark = group.ty {
                group.ty = CharGroupType::Word(WordCharGroupType::Other);
            }
            vec![group]
        }
        Some(prev_group) => vec![prev_group, group],
    }
}

/// See [`convert_first_cdm_group_rule2`] for details.
fn convert_first_cdm_group2(
    groups: Vec<CharTokenGroup>,
) -> Vec<CharTokenGroup> {
    utils::stack_merge(groups, convert_first_cdm_group_rule2)
}

impl<C> Tokenizer<C> {
    /// Split `self` into subgroups, whose types will be recategorized. Panics
    /// if `self.chars.len() != sizes.sum()`.
    fn split_into_subgroups(
        &self,
        line: &str,
        char_group: CharTokenGroup,
        sizes: Vec<usize>,
    ) -> Vec<CharTokenGroup> {
        let token = get_token(line, &char_group);
        assert_eq!(token.chars().count(), sizes.iter().sum::<usize>());
        let mut sub_groups = Vec::with_capacity(sizes.len());
        let mut chars = token.chars();
        let mut start = char_group.col.start_byte_index;
        for sz in sizes {
            let mut sub_chars = (0..sz).map(|_| {
                // Calling `unwrap()` won't panic, because it has been ensured
                // that `self.chars.len() == sizes.sum()`.
                let ch = chars.next().unwrap();
                let ch = self.new_char_token(ch, start);
                start = ch.col.excl_end_byte_index;
                ch
            });
            if let Some(ch) = sub_chars.next() {
                let mut sub_group = CharTokenGroup::from(ch);
                for ch in sub_chars {
                    // Calling `unwrap()` won't panic, because `char_group` is
                    // already a `CharTokenGroup`, which is made up of Chars of
                    // compatible major class.
                    sub_group.push(ch).unwrap();
                }
                sub_groups.push(sub_group);
            }
        }
        sub_groups
    }
}

/// Concatenate contiguous [`WordCharGroupType::Other`] groups, and insert
/// implict whitespace in between otherwise. We assume that both `prev_group`
/// and `group` are of major class [`CharGroupType::Word`]. Panics if they
/// aren't.
fn insert_implicit_whitespace_in_cut_result_rule(
    prev_group: Option<MaybeImplicitCharTokenGroup>,
    group: CharTokenGroup,
) -> Vec<MaybeImplicitCharTokenGroup> {
    match prev_group {
        None => vec![group.into()],
        Some(mut prev_group) => {
            use CharGroupType::*;
            use WordCharGroupType as W;
            // Calling `unwrap()` won't panic, because in all cases, the top
            // element of the underlying stack is always `CharTokenGroup`.
            let prev_group_inner =
                prev_group.to_char_token_group_mut().unwrap();
            match (&prev_group_inner.ty, &group.ty) {
                (Word(W::Other), Word(W::Other)) => {
                    // Concatenate `group` after `prev_group`.
                    prev_group_inner.append(group);
                    vec![prev_group]
                }
                (Word(_), Word(_)) => {
                    let ispace =
                        ImplicitWhitespace::new(group.col.start_byte_index);
                    vec![prev_group, ispace.into(), group.into()]
                }
                _ => panic!("prev_group or group is not word"),
            }
        }
    }
}

/// See [`insert_implicit_whitespace_in_cut_result_rule`] for details.
fn insert_implicit_whitespace_in_cut_result(
    groups: Vec<CharTokenGroup>,
) -> Vec<MaybeImplicitCharTokenGroup> {
    utils::stack_merge(groups, insert_implicit_whitespace_in_cut_result_rule)
}

/// Assuming `group.ty` is [`WordCharGroupType::Hanzi`], this function goes
/// through the following steps:
///
/// 1. Temporarily remove all combining diacritical marks from the group.
/// 2. Cut words using `jieba`.
/// 3. Revert removal of the combining marks and append combining marks to each
///    cut group.
/// 4. Count the number of chars in each cut group and return.
fn cut_hanzi_group_and_count_chars<C: JiebaPlaceholder>(
    line: &str,
    group: &CharTokenGroup,
    jieba: &C,
) -> Vec<usize> {
    let mut marks = Vec::new();
    let group_string_no_marks: String = get_token(line, group)
        .chars()
        .filter_map(|c| {
            let is_mark = char::is_combining_diacritical_mark(c);
            marks.push(is_mark);
            if is_mark { None } else { Some(c) }
        })
        .collect();
    let cut_char_counts0 = utils::chain_into_vec(
        [0],
        jieba
            .cut_hmm(&group_string_no_marks)
            .into_iter()
            .map(|part| part.chars().count()),
    );
    
    append_mark_to_cuts(&marks, &cut_char_counts0)
}

/// The step 3 in [`cut_hanzi_group_and_count_chars`].
///
/// For example, given a [`CharTokenGroup`] of type
/// [`WordCharGroupType::Hanzi`], denote 汉字 by `H`, combining marks by `m`,
/// other non-space characters by `A`, the string representation of the group
/// might be: `m H m H A m`. Clearly, `marks` will be `true false true false
/// false true`. Suppose the first `H`s make up a word, then `cut_char_counts0`
/// will be `0 2 1`, where `0` is fixed, `2` signifies the two `H`s, and `1`
/// for the `A`. The output will be `1 3 2`, corresponding to `[m] [H m H]
/// [A m]`.
///
/// Properties:
///
/// - Neither `marks` nor `cut_char_counts0` is empty.
/// - The first element of `cut_char_counts0` is zero.
/// - The `cut_char_counts0` and the output elements are guaranteed positive,
///   except for the first element.
/// - Number of false's in `marks` equals the sum of input `cut_char_counts0`.
/// - The sum of the output equals the length of `marks`.
fn append_mark_to_cuts(
    marks: &[bool],
    cut_char_counts0: &[usize],
) -> Vec<usize> {
    let mut out = vec![0; cut_char_counts0.len()];
    let mut x = 0; // The accumulator of `marks`.
    let mut y = 0; // The accumulator of `cut_char_counts0`.
    let mut cum_marks = marks
        .iter()
        .map(|m| {
            if !*m {
                x += 1;
            }
            x
        })
        .peekable();
    let mut cum_char_counts = cut_char_counts0
        .iter()
        .map(|c| {
            y += c;
            y
        })
        .peekable();
    let mut out_iter = out.iter_mut().peekable();
    while cum_marks.peek().is_some()
        && cum_char_counts.peek().is_some()
        && out_iter.peek().is_some()
    {
        let x = cum_marks.peek().unwrap();
        let y = cum_char_counts.peek().unwrap();
        if x <= y {
            **out_iter.peek_mut().unwrap() += 1;
            cum_marks.next().unwrap();
        } else {
            cum_char_counts.next().unwrap();
            out_iter.next().unwrap();
        }
    }

    out
}

/// Cut [`CharTokenGroup`]s of type [`WordCharGroupType::Hanzi`] into sub
/// groups, and insert implicit whitespaces in between. Since this merging
/// rule ought to be used after [`group_chars_rule`], we won't need to care
/// about `prev_group` -- we need only to prepend it to the returned vec. One
/// caveat of `jieba` is that it may also cut text with no 汉字 at all. For
/// example, "B超abc_def" will become ("B超", "abc", "_", "def"). Therefore,
/// after the cut operation, we are required to concatenate contiguous
/// [`CharTokenGroup`]s of type [`WordCharGroupType::Other`] before inserting
/// the implicit whitespaces. [`insert_implicit_whitespace_in_cut_result_rule`]
/// achieves the concatenation and insertion implicit whitespaces work.
fn cut_hanzi_rule<C: JiebaPlaceholder>(
    prev_group: Option<MaybeImplicitCharTokenGroup>,
    group: MaybeImplicitCharTokenGroup,
    line: &str,
    tokenizer: &Tokenizer<C>,
) -> Vec<MaybeImplicitCharTokenGroup> {
    use CharGroupType::*;
    use MaybeImplicitCharTokenGroup::*;
    use WordCharGroupType as W;
    match group {
        // If `group` is an implicit whitespace, return as is.
        ImplicitWhitespace(iw) => {
            utils::chain_into_vec(prev_group, [iw.into()])
        }
        CharTokenGroup(group) => match group.ty {
            Word(W::Hanzi) => {
                let n_chars =
                    cut_hanzi_group_and_count_chars(line, &group, tokenizer);
                let sub_groups =
                    tokenizer.split_into_subgroups(line, group, n_chars);
                // In the case where `group` is the first group, it's likely
                // that the first sub-group is a combining diacritical mark,
                // and we need to convert it again to a word. This happens
                // because `split_into_subgroups` recategorizes chars.
                let sub_groups = if prev_group.is_none() {
                    convert_first_cdm_group2(sub_groups)
                } else {
                    sub_groups
                };
                utils::chain_into_vec(
                    prev_group,
                    insert_implicit_whitespace_in_cut_result(sub_groups),
                )
            }

            // Otherwise, return as is.
            _ => utils::chain_into_vec(prev_group, [group.into()]),
        },
    }
}

/// See [`cut_hanzi_rule`] for details.
fn cut_hanzi<C: JiebaPlaceholder>(
    groups: Vec<MaybeImplicitCharTokenGroup>,
    line: &str,
    tokenizer: &Tokenizer<C>,
) -> Vec<MaybeImplicitCharTokenGroup> {
    utils::stack_merge(groups, |prev_group, group| {
        cut_hanzi_rule(prev_group, group, line, tokenizer)
    })
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token {
    col: Col,
    pub(crate) ty: TokenType,
}

/// Token types.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    /// Either a word or a WORD token, depending on the context.
    ///
    /// A word, by definition of Vim, is either a sequence of word characters
    /// or a sequence of non-word non-whitespace characters. A WORD, in
    /// contrast, is a sequence of either word or non-word non-whitespace
    /// characters.
    Word,
    /// Tokens that contain space and/or unicode combining characters only.
    Space,
}

impl Token {
    #[cfg(test)]
    pub(crate) fn new(
        start_byte_index: usize,
        incl_end_byte_index: usize,
        excl_end_byte_index: usize,
        ty: TokenType,
    ) -> Self {
        Self {
            col: Col {
                start_byte_index,
                incl_end_byte_index,
                excl_end_byte_index,
            },
            ty,
        }
    }
}

impl_token_like_from_col!(Token);

// `None` is used to denote the empty line.
impl TokenLike for Option<Token> {
    fn first_char(&self) -> usize {
        self.map(|t| t.first_char()).unwrap_or(0)
    }

    fn last_char(&self) -> usize {
        self.map(|t| t.last_char()).unwrap_or(0)
    }

    fn last_char1(&self) -> usize {
        self.map(|t| t.last_char1()).unwrap_or(0)
    }
}

impl From<CharTokenGroup> for Token {
    fn from(g: CharTokenGroup) -> Self {
        Self {
            col: g.col,
            ty: match g.ty {
                CharGroupType::Space => TokenType::Space,
                _ => TokenType::Word,
            },
        }
    }
}

fn remove_implicit_whitespace_rule(
    prev_group: Option<CharTokenGroup>,
    group: MaybeImplicitCharTokenGroup,
) -> Vec<CharTokenGroup> {
    use MaybeImplicitCharTokenGroup::*;
    match group {
        // Remove this implicit whitespace.
        ImplicitWhitespace(_) => prev_group.into_iter().collect(),
        // Otherwise, return as is.
        CharTokenGroup(group) => utils::chain_into_vec(prev_group, [group]),
    }
}

fn remove_implicit_whitespace(
    groups: Vec<MaybeImplicitCharTokenGroup>,
) -> Vec<CharTokenGroup> {
    utils::stack_merge(groups, remove_implicit_whitespace_rule)
}

impl<C: JiebaPlaceholder> Tokenizer<C> {
    /// Parse a vec of [`CharToken`]s into `word`s and space.
    fn parse_chars_into_words(
        &self,
        line: &str,
        chars: Vec<CharToken>,
    ) -> Vec<Token> {
        let groups = group_chars(chars);
        let groups = convert_first_cdm_group(groups);
        let groups = cut_hanzi(groups, line, self);
        let groups = remove_implicit_whitespace(groups);
        groups.into_iter().map(Token::from).collect()
    }
}

/// Concatenate contiguous non-space [`MaybeImplicitCharTokenGroup`]s.
fn concat_nonspace_groups_rule(
    prev_group: Option<MaybeImplicitCharTokenGroup>,
    group: MaybeImplicitCharTokenGroup,
) -> Vec<MaybeImplicitCharTokenGroup> {
    match prev_group {
        None => vec![group],
        Some(mut prev_group) => {
            use CharGroupType::*;
            use MaybeImplicitCharTokenGroup::*;
            match (&mut prev_group, group) {
                (
                    CharTokenGroup(prev_group_inner),
                    CharTokenGroup(group_inner),
                ) => match (&prev_group_inner.ty, &group_inner.ty) {
                    (Space, _) | (_, Space) => {
                        vec![prev_group, group_inner.into()]
                    }
                    _ => {
                        prev_group_inner.append(group_inner);
                        vec![prev_group]
                    }
                },
                (ImplicitWhitespace(_), CharTokenGroup(group_inner)) => {
                    vec![prev_group, group_inner.into()]
                }
                (_, ImplicitWhitespace(iw)) => vec![prev_group, iw.into()],
            }
        }
    }
}

/// See [`concat_nonspace_groups_rule`] for details.
fn concat_nonspace_groups(
    groups: Vec<MaybeImplicitCharTokenGroup>,
) -> Vec<MaybeImplicitCharTokenGroup> {
    utils::stack_merge(groups, concat_nonspace_groups_rule)
}

impl<C: JiebaPlaceholder> Tokenizer<C> {
    /// Parse a vec of [`CharToken`]s into `WORD`s and space.
    #[allow(non_snake_case)]
    fn parse_chars_into_WORDs(
        &self,
        line: &str,
        chars: Vec<CharToken>,
    ) -> Vec<Token> {
        let groups = group_chars(chars);
        let groups = convert_first_cdm_group(groups);
        let groups = cut_hanzi(groups, line, self);
        let groups = concat_nonspace_groups(groups);
        let groups = remove_implicit_whitespace(groups);
        groups.into_iter().map(Token::from).collect()
    }
}

impl<C: JiebaPlaceholder> Tokenizer<C> {
    /// Parse `line` into tokens. If `into_word` is `true`, the non-space
    /// tokens will be interpretable as `word`s; otherwise, they will be
    /// `WORD`s.
    pub fn parse_str(&self, line: &str, into_word: bool) -> Vec<Token> {
        let chars = self.parse_str_into_chars(line, 0);
        if into_word {
            self.parse_chars_into_words(line, chars)
        } else {
            self.parse_chars_into_WORDs(line, chars)
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::token::JiebaPlaceholder;
    use crate::token::jieba::KeywordCutter;

    use super::{
        CharGroupType, CharTokenGroup, Col, Token, TokenType, Tokenizer,
        WordCharGroupType, append_mark_to_cuts,
    };

    #[test]
    fn test_append_mark_to_cuts() {
        let counts = vec![0, 2, 1];
        let marks = vec![true, false, true, false, false, true];
        assert_eq!(append_mark_to_cuts(&marks, &counts), vec![1, 3, 2]);

        let counts = vec![0, 2, 1];
        let marks = vec![true, true, false, true, false, false, true];
        assert_eq!(append_mark_to_cuts(&marks, &counts), vec![2, 3, 2]);

        let counts = vec![0, 1, 1, 1];
        let marks = vec![true, false, true, false, false, true];
        assert_eq!(append_mark_to_cuts(&marks, &counts), vec![1, 2, 1, 2]);

        let counts = vec![0, 2, 2];
        let marks = vec![false, true, false, true, false, false];
        assert_eq!(append_mark_to_cuts(&marks, &counts), vec![0, 4, 2]);

        let counts = vec![0, 2];
        let marks = vec![false, false];
        assert_eq!(append_mark_to_cuts(&marks, &counts), vec![0, 2]);
    }

    #[test]
    fn test_char_group_split_into_subgroups_sanity_check() {
        let jieba = KeywordCutter::new(["你好".into()]);
        let tokenizer = Tokenizer::new(jieba, "a-z");
        let cg = tokenizer.new_char_group_from_str("hello", 0).unwrap();
        assert_eq!(
            cg,
            CharTokenGroup {
                col: Col {
                    start_byte_index: 0,
                    incl_end_byte_index: 4,
                    excl_end_byte_index: 5,
                },
                ty: CharGroupType::Word(WordCharGroupType::Other),
            }
        );
        let groups = tokenizer.split_into_subgroups("hello", cg, vec![2, 2, 1]);
        assert_eq!(
            groups,
            vec![
                CharTokenGroup {
                    col: Col {
                        start_byte_index: 0,
                        incl_end_byte_index: 1,
                        excl_end_byte_index: 2,
                    },
                    ty: CharGroupType::Word(WordCharGroupType::Other),
                },
                CharTokenGroup {
                    col: Col {
                        start_byte_index: 2,
                        incl_end_byte_index: 3,
                        excl_end_byte_index: 4,
                    },
                    ty: CharGroupType::Word(WordCharGroupType::Other),
                },
                CharTokenGroup {
                    col: Col {
                        start_byte_index: 4,
                        incl_end_byte_index: 4,
                        excl_end_byte_index: 5,
                    },
                    ty: CharGroupType::Word(WordCharGroupType::Other),
                },
            ]
        );
    }

    fn parse_str_test<C: JiebaPlaceholder>(
        tokenizer: &Tokenizer<C>,
        s: &str,
        into_word: bool,
    ) -> Vec<Token> {
        let output = tokenizer.parse_str(s, into_word);
        output
    }

    /// Test whether for any input string, the tokenization outputs are
    /// contiguous non-empty tokens. Note that we didn't use `jieba_rs::Jieba`.
    /// This is because it's very rare to get 汉字 words from randomly generated
    /// string. Having a Jieba instance here won't really change anything
    /// except for making tests slow.
    macro_rules! def_parse_str_tests {
        ($($test_name:ident: $isk:literal, $into_word:literal);*$(;)?) => {
            $(
                proptest! {
                    #![proptest_config(ProptestConfig::with_cases(10000))]
                    #[test]
                    #[allow(non_snake_case)]
                    fn $test_name(s in "\\PC*") {
                        let tokenizer = Tokenizer::new(KeywordCutter::new([]), $isk);
                        let tokens = parse_str_test(&tokenizer, &s, $into_word);
                        let mut start = 0;
                        for tok in tokens {
                            assert_eq!(tok.col.start_byte_index, start);
                            assert!(tok.col.start_byte_index < tok.col.excl_end_byte_index);
                            assert!(tok.col.start_byte_index <= tok.col.incl_end_byte_index);
                            assert!(tok.col.incl_end_byte_index < tok.col.excl_end_byte_index);
                            start = tok.col.excl_end_byte_index;
                        }
                    }
                }
            )*
        };
    }

    def_parse_str_tests!(
        parse_str_tokens_are_nonempty_contiguous_word_default_isk:
            "@,48-57,_,192-255",
            true;
        parse_str_tokens_are_nonempty_contiguous_WORD_default_isk:
            "@,48-57,_,192-255",
            false;
        parse_str_tokens_are_nonempty_contiguous_word_digit_punc_isk:
            r#"48-57,!,",#,$,%,&,',(,),*,+,,,-,.,/,:,;,<,=,>,?,@-@,[,\,],_,`,{,|,},~,^"#,
            true;
        parse_str_tokens_are_nonempty_contiguous_WORD_digit_punc_isk:
            r#"48-57,!,",#,$,%,&,',(,),*,+,,,-,.,/,:,;,<,=,>,?,@-@,[,\,],_,`,{,|,},~,^"#,
            false;
        parse_str_tokens_are_nonempty_contiguous_word_empty_isk:
            "", true;
        parse_str_tokens_are_nonempty_contiguous_WORD_empty_isk:
            "", false;
    );

    #[test]
    fn test_parse_str_tokens_are_nonempty_contiguous_word_default_isk_failed_1()
    {
        let s = "\u{300}A⼀";
        // Mimic `jieba_rs::Jieba`'s property to cut at the boundary of
        // different character classes.
        let kc = KeywordCutter::new(["⼀".into()]);
        let tokenizer = Tokenizer::new(kc, "@,48-57,_,192-255");
        let tokens = parse_str_test(&tokenizer, s, true);
        let mut start = 0;
        for tok in tokens {
            assert_eq!(tok.col.start_byte_index, start);
            assert!(tok.col.start_byte_index < tok.col.excl_end_byte_index);
            assert!(tok.col.start_byte_index <= tok.col.incl_end_byte_index);
            assert!(tok.col.incl_end_byte_index < tok.col.excl_end_byte_index);
            start = tok.col.excl_end_byte_index;
        }
    }

    /// Convenient function to build a vec of tokens used as groundtruth in
    /// tests. There should be no special characters like combining diacritical
    /// marks in the tokens. Usage:
    ///
    /// ```ignore
    /// use TokenType::*;
    /// assert_eq!(
    ///     build_simple_tokens(vec![("foo", Word), (" ", Space), ("bar", Word)]),
    ///     vec![
    ///         Token::new(0, 2, 3, Word),
    ///         Token::new(3, 3, 4, Space),
    ///         Token::new(4, 6, 7, Word)
    ///     ]);
    /// ```
    fn build_simple_tokens(
        token_data: Vec<(&'static str, TokenType)>,
    ) -> Vec<Token> {
        let mut start = 0;
        let mut result = Vec::with_capacity(token_data.len());
        for (s, ty) in token_data {
            let (last_char, _) = s.char_indices().last().unwrap();
            let last_char1 = s.len();
            result.push(Token::new(
                start,
                start + last_char,
                start + last_char1,
                ty,
            ));
            start += last_char1;
        }
        result
    }

    #[test]
    fn test_build_simple_tokens() {
        use TokenType::*;
        assert_eq!(
            build_simple_tokens(vec![
                ("foo", Word),
                (" ", Space),
                ("bar", Word)
            ]),
            vec![
                Token::new(0, 2, 3, Word),
                Token::new(3, 3, 4, Space),
                Token::new(4, 6, 7, Word)
            ]
        );
        assert_eq!(
            build_simple_tokens(vec![("\u{1f596}", Word), ("bar", Word)]),
            vec![Token::new(0, 0, 4, Word), Token::new(4, 6, 7, Word)]
        );
    }

    #[test]
    fn test_parse_empty() {
        let tokenizer =
            Tokenizer::new(KeywordCutter::new(["你好".into()]), "a-z");
        let tokens = parse_str_test(&tokenizer, "", true);
        assert!(tokens.is_empty());
        let tokens = parse_str_test(&tokenizer, "", false);
        assert!(tokens.is_empty());
    }

    mod test_parse_en_only {
        use super::*;

        const SENT: &str = "Hello, World";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("Hello", Word),
                    (",", Word),
                    (" ", Space),
                    ("World", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("Hello,", Word),
                    (" ", Space),
                    ("World", Word)
                ])
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("Hello,", Word),
                    (" ", Space),
                    ("World", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("Hello,", Word),
                    (" ", Space),
                    ("World", Word)
                ])
            );
        }

        #[test]
        fn c_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("H", Word),
                    ("ello", Word),
                    (",", Word),
                    (" ", Space),
                    ("W", Word),
                    ("orld", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn c_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("Hello,", Word),
                    (" ", Space),
                    ("World", Word)
                ])
            );
        }
    }

    mod test_parse_hanzi_and_en_1 {
        use super::*;

        const SENT: &str = "B超foo_bar";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new([
                    "B超".into(),
                    "foo".into(),
                    "_".into(),
                    "bar".into(),
                ]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![("B超", Word), ("foo_bar", Word)])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new([
                    "B超".into(),
                    "foo".into(),
                    "_".into(),
                    "bar".into(),
                ]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![("B超", Word), ("foo_bar", Word)])
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new([
                    "B超".into(),
                    "foo".into(),
                    "_".into(),
                    "bar".into(),
                ]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("B", Word),
                    ("超", Word),
                    ("foo_bar", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new([
                    "B超".into(),
                    "foo".into(),
                    "_".into(),
                    "bar".into(),
                ]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(tokens, build_simple_tokens(vec![("B超foo_bar", Word)]));
        }

        #[test]
        fn c_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new([
                    "B超".into(),
                    "foo".into(),
                    "_".into(),
                    "bar".into(),
                ]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    // "B" and "超" are two tokens because they are of
                    // different character classes.
                    ("B", Word),
                    ("超", Word),
                    // "foo" is a word.
                    ("foo", Word),
                    // "_" is not a word.
                    ("_", Word),
                    // "bar" is a word.
                    ("bar", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn c_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new([
                    "B超".into(),
                    "foo".into(),
                    "_".into(),
                    "bar".into(),
                ]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(tokens, build_simple_tokens(vec![("B超foo_bar", Word)]));
        }
    }

    mod test_parse_hanzi_and_en_2 {
        use super::*;

        const SENT: &str = "B超，foo。。。";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["B超".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("B超", Word),
                    ("，", Word),
                    ("foo", Word),
                    ("。。。", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["B超".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![("B超，", Word), ("foo。。。", Word)])
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["B超".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("B", Word),
                    ("超", Word),
                    ("，foo。。。", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["B超".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![("B超，foo。。。", Word)])
            );
        }

        #[test]
        fn c_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["B超".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("B", Word),
                    ("超", Word),
                    ("，", Word),
                    ("foo", Word),
                    ("。。。", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn c_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["B超".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![("B超，", Word), ("foo。。。", Word)])
            );
        }
    }

    mod test_parse_hanzi_1 {
        use super::*;

        const SENT: &str = "（你好世界——世界）。";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("（", Word),
                    ("你好", Word),
                    ("世界", Word),
                    ("——", Word),
                    ("世界", Word),
                    ("）。", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("（你好", Word),
                    ("世界——世界）。", Word)
                ])
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("（", Word),
                    ("你好世界", Word),
                    ("——", Word),
                    ("世界", Word),
                    ("）。", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![("（你好世界——世界）。", Word)])
            );
        }

        #[test]
        fn c_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("（", Word),
                    ("你好世界", Word),
                    ("——", Word),
                    ("世界", Word),
                    ("）。", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn c_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![("（你好世界——世界）。", Word)])
            );
        }
    }

    mod test_parse_spacing_modifiers_1 {
        use super::*;

        const SENT: &str = "Abc  ʰDef g˦hi jkl";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("Abc", Word),
                    ("  ", Space),
                    ("ʰDef", Word),
                    (" ", Space),
                    ("g˦hi", Word),
                    (" ", Space),
                    ("jkl", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("Abc", Word),
                    ("  ", Space),
                    ("ʰDef", Word),
                    (" ", Space),
                    ("g˦hi", Word),
                    (" ", Space),
                    ("jkl", Word)
                ])
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("Abc", Word),
                    ("  ", Space),
                    ("ʰ", Word),
                    ("Def", Word),
                    (" ", Space),
                    ("g", Word),
                    ("˦", Word),
                    ("hi", Word),
                    (" ", Space),
                    ("jkl", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("Abc", Word),
                    ("  ", Space),
                    ("ʰDef", Word),
                    (" ", Space),
                    ("g˦hi", Word),
                    (" ", Space),
                    ("jkl", Word)
                ])
            );
        }

        #[test]
        fn c_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("A", Word),
                    ("bc", Word),
                    ("  ", Space),
                    ("ʰ", Word),
                    ("D", Word),
                    ("ef", Word),
                    (" ", Space),
                    ("g˦hi", Word),
                    (" ", Space),
                    ("jkl", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn c_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("Abc", Word),
                    ("  ", Space),
                    ("ʰDef", Word),
                    (" ", Space),
                    ("g˦hi", Word),
                    (" ", Space),
                    ("jkl", Word)
                ])
            );
        }
    }

    mod test_parse_combining_chars_modifying_space {
        use super::*;

        // i.e. "xx ̆cab  ̂de".
        const SENT: &str = "xx \u{0306}cab  \u{0302}de";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 1, 2, Word),
                    Token::new(2, 2, 5, Space),
                    Token::new(5, 7, 8, Word),
                    Token::new(8, 9, 12, Space),
                    Token::new(12, 13, 14, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 1, 2, Word),
                    Token::new(2, 2, 5, Space),
                    Token::new(5, 7, 8, Word),
                    Token::new(8, 9, 12, Space),
                    Token::new(12, 13, 14, Word),
                ]
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 1, 2, Word),
                    Token::new(2, 2, 5, Space),
                    Token::new(5, 7, 8, Word),
                    Token::new(8, 9, 12, Space),
                    Token::new(12, 13, 14, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 1, 2, Word),
                    Token::new(2, 2, 5, Space),
                    Token::new(5, 7, 8, Word),
                    Token::new(8, 9, 12, Space),
                    Token::new(12, 13, 14, Word),
                ]
            );
        }

        #[test]
        fn c_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 1, 2, Word),
                    Token::new(2, 2, 5, Space),
                    Token::new(5, 7, 8, Word),
                    Token::new(8, 9, 12, Space),
                    Token::new(12, 13, 14, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn c_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 1, 2, Word),
                    Token::new(2, 2, 5, Space),
                    Token::new(5, 7, 8, Word),
                    Token::new(8, 9, 12, Space),
                    Token::new(12, 13, 14, Word),
                ]
            );
        }
    }

    mod test_parse_combining_chars_modifying_letter_1 {
        use super::*;

        // i.e. xy a͡Bc D̂ef.
        const SENT: &str = "xy a\u{0361}Bc D\u{0302}ef";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 1, 2, Word),
                    Token::new(2, 2, 3, Space),
                    Token::new(3, 7, 8, Word),
                    Token::new(8, 8, 9, Space),
                    Token::new(9, 13, 14, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 1, 2, Word),
                    Token::new(2, 2, 3, Space),
                    Token::new(3, 7, 8, Word),
                    Token::new(8, 8, 9, Space),
                    Token::new(9, 13, 14, Word),
                ]
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 1, 2, Word),
                    Token::new(2, 2, 3, Space),
                    Token::new(3, 7, 8, Word),
                    Token::new(8, 8, 9, Space),
                    Token::new(9, 13, 14, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 1, 2, Word),
                    Token::new(2, 2, 3, Space),
                    Token::new(3, 7, 8, Word),
                    Token::new(8, 8, 9, Space),
                    Token::new(9, 13, 14, Word),
                ]
            );
        }

        #[test]
        fn c_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 1, 2, Word),
                    Token::new(2, 2, 3, Space),
                    Token::new(3, 3, 6, Word),
                    Token::new(6, 6, 7, Word),
                    Token::new(7, 7, 8, Word),
                    Token::new(8, 8, 9, Space),
                    Token::new(9, 9, 12, Word),
                    Token::new(12, 13, 14, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn c_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 1, 2, Word),
                    Token::new(2, 2, 3, Space),
                    Token::new(3, 7, 8, Word),
                    Token::new(8, 8, 9, Space),
                    Token::new(9, 13, 14, Word),
                ]
            );
        }
    }

    mod test_parse_combining_chars_modifying_letter_2 {
        use super::*;

        // Example from http://demo.danielmclaren.com/2015/diacriticism/.
        // i.e. "f̸̰̻̯̙̳́̍͗̕o͕̟̫ͮ͆̉̾̍̉̏o̵͖̪͇̪̥͗̈ͭ̕ B̶̬̣̜̱̜͉̾ͩ͌a͚̯̮͒ͬ̆̊̍͂̕r̹̥̟̘̱͙͊͗̀̓".
        const SENT: &str = "f\u{0330}\u{0338}\u{0315}\u{033b}\u{0301}\u{032f}\u{0319}\
            \u{030d}\u{0357}\u{0333}o\u{036e}\u{0355}\u{0346}\u{031f}\u{0309}\
            \u{033e}\u{032b}\u{030d}\u{0309}\u{030f}o\u{0357}\u{0356}\u{032a}\
            \u{0308}\u{0347}\u{032a}\u{0315}\u{036d}\u{0325}\u{0335} B\u{032c}\
            \u{0323}\u{0336}\u{031c}\u{033e}\u{0331}\u{0369}\u{031c}\u{0349}\
            \u{034c}a\u{035a}\u{0352}\u{036c}\u{0306}\u{0315}\u{030a}\u{030d}\
            \u{032f}\u{032e}\u{0342}r\u{0339}\u{034a}\u{0357}\u{0325}\u{031f}\
            \u{0318}\u{0331}\u{0340}\u{0359}\u{0343}";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 42, 63, Word),
                    Token::new(63, 63, 64, Space),
                    Token::new(64, 106, 127, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 42, 63, Word),
                    Token::new(63, 63, 64, Space),
                    Token::new(64, 106, 127, Word),
                ]
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 42, 63, Word),
                    Token::new(63, 63, 64, Space),
                    Token::new(64, 106, 127, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 42, 63, Word),
                    Token::new(63, 63, 64, Space),
                    Token::new(64, 106, 127, Word),
                ]
            );
        }

        #[test]
        fn c_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 42, 63, Word),
                    Token::new(63, 63, 64, Space),
                    Token::new(64, 64, 85, Word),
                    Token::new(85, 106, 127, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn c_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 42, 63, Word),
                    Token::new(63, 63, 64, Space),
                    Token::new(64, 106, 127, Word),
                ]
            );
        }
    }

    mod test_parse_combining_chars_modifying_hanzi_1 {
        use super::*;

        const SENT: &str = "你好\u{0302}世界";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![Token::new(0, 3, 8, Word), Token::new(8, 11, 14, Word)]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![Token::new(0, 3, 8, Word), Token::new(8, 11, 14, Word)]
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(tokens, vec![Token::new(0, 11, 14, Word)]);
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(tokens, vec![Token::new(0, 11, 14, Word)]);
        }
    }

    mod test_parse_combining_chars_modifying_hanzi_2 {
        use super::*;

        const SENT: &str = "你\u{0302}好世界";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![Token::new(0, 5, 8, Word), Token::new(8, 11, 14, Word)]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![Token::new(0, 5, 8, Word), Token::new(8, 11, 14, Word)]
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(tokens, vec![Token::new(0, 11, 14, Word)]);
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(tokens, vec![Token::new(0, 11, 14, Word)]);
        }
    }

    mod test_parse_combining_chars_modifying_hanzi_3 {
        use super::*;

        const SENT: &str = "你好世界\u{0302}";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![Token::new(0, 3, 6, Word), Token::new(6, 9, 14, Word)]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![Token::new(0, 3, 6, Word), Token::new(6, 9, 14, Word)]
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(tokens, vec![Token::new(0, 9, 14, Word)]);
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(tokens, vec![Token::new(0, 9, 14, Word)]);
        }
    }

    mod test_parse_starts_with_combining_chars_1 {
        use super::*;

        const SENT: &str = "\u{0302}\u{0302}\u{0302}\u{0302}\u{0302} abc";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 0, 10, Word),
                    Token::new(10, 10, 11, Space),
                    Token::new(11, 13, 14, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 0, 10, Word),
                    Token::new(10, 10, 11, Space),
                    Token::new(11, 13, 14, Word),
                ]
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 0, 10, Word),
                    Token::new(10, 10, 11, Space),
                    Token::new(11, 13, 14, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 0, 10, Word),
                    Token::new(10, 10, 11, Space),
                    Token::new(11, 13, 14, Word),
                ]
            );
        }
    }

    mod test_parse_starts_with_combining_chars_2 {
        use super::*;

        const SENT: &str = "\u{0302}\u{0302}\u{0302}\u{0302}\u{0302}Abc";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(tokens, vec![Token::new(0, 12, 13, Word)]);
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(tokens, vec![Token::new(0, 12, 13, Word)]);
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![Token::new(0, 0, 10, Word), Token::new(10, 12, 13, Word)]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(tokens, vec![Token::new(0, 12, 13, Word)]);
        }

        #[test]
        fn c_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 0, 10, Word),
                    Token::new(10, 10, 11, Word),
                    Token::new(11, 12, 13, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn c_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(tokens, vec![Token::new(0, 12, 13, Word)]);
        }
    }

    mod test_parse_starts_with_combining_chars_3 {
        use super::*;

        const SENT: &str = "\u{0302}你好世界";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 0, 2, Word),
                    Token::new(2, 5, 8, Word),
                    Token::new(8, 11, 14, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                vec![Token::new(0, 5, 8, Word), Token::new(8, 11, 14, Word)]
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![Token::new(0, 0, 2, Word), Token::new(2, 11, 14, Word)]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(tokens, vec![Token::new(0, 11, 14, Word)]);
        }

        #[test]
        fn c_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![Token::new(0, 0, 2, Word), Token::new(2, 11, 14, Word)]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn c_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(tokens, vec![Token::new(0, 11, 14, Word)]);
        }
    }

    mod test_parse_combining_chars_only {
        use super::*;

        const SENT: &str = "\u{0302}\u{0302}\u{0302}\u{0302}\u{0302}";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(tokens, vec![Token::new(0, 0, 10, Word)]);
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(tokens, vec![Token::new(0, 0, 10, Word)]);
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(tokens, vec![Token::new(0, 0, 10, Word)]);
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(tokens, vec![Token::new(0, 0, 10, Word)]);
        }
    }

    mod test_emoji_zwj {
        use super::*;

        // i.e. 👨‍👩‍👧‍👦.
        const SENT: &str =
            "\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}\u{200d}\u{1f466}";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 0, 4, Word),
                    Token::new(4, 4, 7, Word),
                    Token::new(7, 7, 11, Word),
                    Token::new(11, 11, 14, Word),
                    Token::new(14, 14, 18, Word),
                    Token::new(18, 18, 21, Word),
                    Token::new(21, 21, 25, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(tokens, vec![Token::new(0, 21, 25, Word)]);
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                vec![
                    Token::new(0, 0, 4, Word),
                    Token::new(4, 4, 7, Word),
                    Token::new(7, 7, 11, Word),
                    Token::new(11, 11, 14, Word),
                    Token::new(14, 14, 18, Word),
                    Token::new(18, 18, 21, Word),
                    Token::new(21, 21, 25, Word),
                ]
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(tokens, vec![Token::new(0, 21, 25, Word)]);
        }
    }

    mod test_emoji_surrounded_by_nonword {
        use super::*;

        // i.e. ",🖖,.abc".
        const SENT: &str = ",\u{1f596},.abc";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    (",", Word),
                    ("\u{1f596}", Word),
                    (",.", Word),
                    ("abc", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![(",\u{1f596},.abc", Word)])
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    (",", Word),
                    ("\u{1f596}", Word),
                    (",.abc", Word),
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer =
                Tokenizer::new(KeywordCutter::new(["你好".into()]), "");
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![(",\u{1f596},.abc", Word)])
            );
        }

        #[test]
        fn c_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    (",", Word),
                    ("\u{1f596}", Word),
                    (",", Word),
                    (".abc", Word),
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn c_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![(",\u{1f596},.abc", Word)])
            );
        }
    }

    mod test_emoji_surrounded_by_hanzi {
        use super::*;

        // i.e. "你好你好🖖世界".
        const SENT: &str = "你好你好\u{1f596}世界";

        #[test]
        fn default_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("你好", Word),
                    ("你好", Word),
                    ("\u{1f596}", Word),
                    ("世界", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn default_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "@,48-57,_,192-255",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("你好", Word),
                    ("你好\u{1f596}世界", Word)
                ])
            );
        }

        #[test]
        fn empty_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("你好你好", Word),
                    ("\u{1f596}", Word),
                    ("世界", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn empty_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![("你好你好\u{1f596}世界", Word)])
            );
        }

        #[test]
        fn c_isk_word() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, true);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![
                    ("你好你好", Word),
                    ("\u{1f596}", Word),
                    ("世界", Word)
                ])
            );
        }

        #[test]
        #[allow(non_snake_case)]
        fn c_isk_WORD() {
            use TokenType::*;
            let tokenizer = Tokenizer::new(
                KeywordCutter::new(["你好".into(), "世界".into()]),
                "a-z,48-57,.,-,>",
            );
            let tokens = parse_str_test(&tokenizer, SENT, false);
            assert_eq!(
                tokens,
                build_simple_tokens(vec![("你好你好\u{1f596}世界", Word)])
            );
        }
    }
}
