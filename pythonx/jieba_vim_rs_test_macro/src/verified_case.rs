// Copyright 2024 Kaiwen Wu. All Rights Reserved.
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

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use jieba_vim_rs_test::verified_case::cases::{
    NmapBCase, NmapECase, NmapGeCase, NmapWCase, OmapCBCase, OmapCECase,
    OmapCGeCase, OmapCWCase, OmapDBCase, OmapDECase, OmapDGeCase, OmapDWCase,
    OmapYBCase, OmapYECase, OmapYGeCase, OmapYWCase, XmapBCase, XmapECase,
    XmapGeCase, XmapWCase,
};
use jieba_vim_rs_test::verified_case::{
    verify_cases, Count, Mode, Motion, Operator,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, ExprArray, ExprLit, ItemMod, Lit, Meta, Token};

/// The data for attribute `verified_case`.
pub struct VerifiedCase {
    buffer: Vec<String>,
    count: Count,
    d_special: bool,
    prevent_change: bool,
}

struct NamedVerifiedCase {
    case: VerifiedCase,
    name: String,
}

fn parse_str_value(value: &Expr) -> Option<String> {
    match value {
        Expr::Lit(ExprLit {
            lit: Lit::Str(lit_str),
            ..
        }) => Some(lit_str.value()),
        _ => None,
    }
}

fn parse_str_array_value(value: &Expr) -> Option<Vec<String>> {
    match value {
        Expr::Array(ExprArray { elems, .. }) => Some(
            elems
                .iter()
                .filter_map(|el| {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = el
                    {
                        Some(lit_str.value())
                    } else {
                        None
                    }
                })
                .collect(),
        ),
        _ => None,
    }
}

fn parse_int_value<N>(value: &Expr) -> Option<N>
where
    N: FromStr,
    N::Err: fmt::Display,
{
    match value {
        Expr::Lit(ExprLit {
            lit: Lit::Int(lit_int),
            ..
        }) => Some(lit_int.base10_parse().unwrap()),
        _ => None,
    }
}

impl Parse for NamedVerifiedCase {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name: Option<String> = None;
        let mut buffer: Option<Vec<String>> = None;
        let mut count: Option<u64> = None;
        let mut d_special = false;
        let mut prevent_change = false;

        let pairs = input.parse_terminated(Meta::parse, Token![,])?;
        for pair in pairs {
            match pair {
                Meta::NameValue(name_value) => {
                    if let Some(ident) = name_value.path.get_ident() {
                        match ident.to_string().as_str() {
                            "name" => {
                                name = Some(
                                    parse_str_value(&name_value.value).unwrap(),
                                )
                            }
                            "buffer" => {
                                buffer = Some(
                                    parse_str_array_value(&name_value.value)
                                        .unwrap(),
                                )
                            }
                            "count" => {
                                count = Some(
                                    parse_int_value(&name_value.value).unwrap(),
                                )
                            }
                            _ => (),
                        }
                    }
                }
                Meta::Path(path) => {
                    if let Some(ident) = path.get_ident() {
                        match ident.to_string().as_str() {
                            "d_special" => d_special = true,
                            "prevent_change" => prevent_change = true,
                            _ => (),
                        }
                    }
                }
                _ => (),
            }
        }
        Ok(NamedVerifiedCase {
            name: name
                .ok_or(syn::Error::new(Span::call_site(), "Missing `name`"))?,
            case: VerifiedCase {
                buffer: buffer.ok_or(syn::Error::new(
                    Span::call_site(),
                    "Missing `buffer`",
                ))?,
                count: count.into(),
                d_special,
                prevent_change,
            },
        })
    }
}

pub struct NamedVerifiedCasesAndMod {
    cases: Vec<NamedVerifiedCase>,
    mod_name: String,
}

impl Parse for NamedVerifiedCasesAndMod {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item_mod: ItemMod = input.parse()?;
        let cases: Vec<_> = item_mod
            .attrs
            .iter()
            .filter_map(|a| {
                if a.path().is_ident("vcase") {
                    let case: NamedVerifiedCase = a.parse_args().unwrap();
                    Some(case)
                } else {
                    None
                }
            })
            .collect();
        Ok(NamedVerifiedCasesAndMod {
            cases,
            mod_name: item_mod.ident.to_string(),
        })
    }
}

/// The data for attribute `verified_cases` itself.
pub struct VerifiedCasesHeader {
    mode: Mode,
    operator: Operator,
    motion: Motion,
    backend_path: String,
    buffer_type: String,
}

fn parse_str_value_into<T: FromStr>(
    value: &Expr,
    span: Span,
) -> Option<syn::Result<T>>
where
    T::Err: fmt::Display,
{
    let value = parse_str_value(value)?;
    Some(value.parse().map_err(|err| syn::Error::new(span, err)))
}

impl Parse for VerifiedCasesHeader {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut mode = None;
        let mut operator = None;
        let mut motion = None;
        let mut backend_path = None;
        let mut buffer_type = None;

        let pairs = input.parse_terminated(Meta::parse, Token![,])?;
        for pair in pairs {
            match pair {
                Meta::NameValue(name_value) => {
                    if let Some(ident) = name_value.path.get_ident() {
                        match ident.to_string().as_str() {
                            "mode" => {
                                let parsed: Mode = parse_str_value_into(
                                    &name_value.value,
                                    Span::call_site(),
                                )
                                .unwrap()?;
                                mode = Some(parsed);
                            }
                            "operator" => {
                                let parsed: Operator = parse_str_value_into(
                                    &name_value.value,
                                    Span::call_site(),
                                )
                                .unwrap()?;
                                operator = Some(parsed);
                            }
                            "motion" => {
                                let parsed: Motion = parse_str_value_into(
                                    &name_value.value,
                                    Span::call_site(),
                                )
                                .unwrap()?;
                                motion = Some(parsed);
                            }
                            "backend_path" => {
                                let parsed =
                                    parse_str_value(&name_value.value).unwrap();
                                backend_path = Some(parsed);
                            }
                            "buffer_type" => {
                                let parsed =
                                    parse_str_value(&name_value.value).unwrap();
                                buffer_type = Some(parsed);
                            }
                            _ => (),
                        }
                    }
                }
                _ => (),
            }
        }

        Ok(VerifiedCasesHeader {
            mode: mode
                .ok_or(syn::Error::new(Span::call_site(), "Missing `mode`"))?,
            operator: operator.unwrap_or(Operator::NoOp),
            motion: motion.ok_or(syn::Error::new(
                Span::call_site(),
                "Missing `motion`",
            ))?,
            backend_path: backend_path.ok_or(syn::Error::new(
                Span::call_site(),
                "Missing `backend_path`",
            ))?,
            buffer_type: buffer_type.unwrap_or("Vec<String>".into()),
        })
    }
}

pub struct VerifiedCases {
    mode: Mode,
    operator: Operator,
    motion: Motion,
    backend_path: syn::Path,
    buffer_type: syn::Type,
    group_name: String,
    cases: HashMap<String, Vec<VerifiedCase>>,
}

fn clone_cases_as<T, F>(
    cases: &HashMap<String, Vec<VerifiedCase>>,
    clone_func: F,
) -> HashMap<String, Vec<T>>
where
    F: Fn(&VerifiedCase) -> T,
{
    let mut new_map = HashMap::new();
    for (key, value) in cases.iter() {
        new_map
            .entry(key.clone())
            .or_insert_with(|| Vec::new())
            .extend(value.iter().map(|c| clone_func(c)));
    }
    new_map
}

impl VerifiedCases {
    pub fn new(
        header: VerifiedCasesHeader,
        flat_cases: NamedVerifiedCasesAndMod,
    ) -> Self {
        let mut cases = HashMap::new();
        for case in flat_cases.cases {
            cases
                .entry(case.name)
                .or_insert_with(|| Vec::new())
                .push(case.case);
        }
        Self {
            mode: header.mode,
            operator: header.operator,
            motion: header.motion,
            backend_path: syn::parse_str(&header.backend_path).unwrap(),
            buffer_type: syn::parse_str(&header.buffer_type).unwrap(),
            group_name: flat_cases.mod_name,
            cases,
        }
    }

    pub fn verify_and_write_tests(
        &self,
        skip_verify: bool,
    ) -> Result<TokenStream, String> {
        macro_rules! def_common_match_arm {
            ( xmap; $case_typ:ident, $write_fun_name:ident, $visual_kind_arg:ident, $word_arg:ident ) => {{
                let cases = clone_cases_as(&self.cases, |c| {
                    $case_typ::new(
                        c.buffer.clone(),
                        c.count,
                        *$word_arg,
                        *$visual_kind_arg,
                    )
                    .unwrap()
                });
                if !skip_verify {
                    verify_cases(&self.group_name, &cases)?;
                }
                Ok(self.write_all_tests(&cases, |case_name, case_id, case| {
                    self.$write_fun_name(case_name, case_id, case, *$word_arg)
                }))
            }};
            ( $case_typ:ident, $write_fun_name:ident, $word_arg:ident ) => {{
                let cases = clone_cases_as(&self.cases, |c| {
                    $case_typ::new(c.buffer.clone(), c.count, *$word_arg)
                        .unwrap()
                });
                if !skip_verify {
                    verify_cases(&self.group_name, &cases)?;
                }
                Ok(self.write_all_tests(&cases, |case_name, case_id, case| {
                    self.$write_fun_name(case_name, case_id, case, *$word_arg)
                }))
            }};
        }

        match (&self.mode, &self.operator, &self.motion) {
            (Mode::Normal, Operator::NoOp, Motion::W(word)) => {
                def_common_match_arm!(NmapWCase, write_nmap_w_assertion, word)
            }
            (Mode::Normal, Operator::NoOp, Motion::E(word)) => {
                def_common_match_arm!(NmapECase, write_nmap_e_assertion, word)
            }
            (Mode::Operator, Operator::Change, Motion::W(word)) => {
                def_common_match_arm!(
                    OmapCWCase,
                    write_omap_c_w_assertion,
                    word
                )
            }
            (Mode::Operator, Operator::Delete, Motion::W(word)) => {
                def_common_match_arm!(
                    OmapDWCase,
                    write_omap_d_w_assertion,
                    word
                )
            }
            (Mode::Operator, Operator::Yank, Motion::W(word)) => {
                def_common_match_arm!(
                    OmapYWCase,
                    write_omap_y_w_assertion,
                    word
                )
            }
            (Mode::Operator, Operator::Delete, Motion::E(word)) => {
                let cases = clone_cases_as(&self.cases, |c| {
                    OmapDECase::new(
                        c.buffer.clone(),
                        c.count,
                        *word,
                        c.d_special,
                    )
                    .unwrap()
                });
                if !skip_verify {
                    verify_cases(&self.group_name, &cases)?;
                }
                Ok(self.write_all_tests(&cases, |case_name, case_id, case| {
                    self.write_omap_d_e_assertion(
                        case_name, case_id, case, *word,
                    )
                }))
            }
            (Mode::Operator, Operator::Change, Motion::E(word)) => {
                def_common_match_arm!(
                    OmapCECase,
                    write_omap_c_e_assertion,
                    word
                )
            }
            (Mode::Operator, Operator::Yank, Motion::E(word)) => {
                def_common_match_arm!(
                    OmapYECase,
                    write_omap_y_e_assertion,
                    word
                )
            }
            (Mode::Visual(kind), Operator::NoOp, Motion::W(word)) => {
                def_common_match_arm!(xmap; XmapWCase, write_xmap_w_assertion, kind, word)
            }
            (Mode::Visual(kind), Operator::NoOp, Motion::E(word)) => {
                def_common_match_arm!(xmap; XmapECase, write_xmap_e_assertion, kind, word)
            }
            (Mode::Normal, Operator::NoOp, Motion::B(word)) => {
                def_common_match_arm!(NmapBCase, write_nmap_b_assertion, word)
            }
            (Mode::Operator, Operator::Change, Motion::B(word)) => {
                let cases = clone_cases_as(&self.cases, |c| {
                    OmapCBCase::new(
                        c.buffer.clone(),
                        c.count,
                        *word,
                        c.prevent_change,
                    )
                    .unwrap()
                });
                if !skip_verify {
                    verify_cases(&self.group_name, &cases)?;
                }
                Ok(self.write_all_tests(&cases, |case_name, case_id, case| {
                    self.write_omap_c_b_assertion(
                        case_name, case_id, case, *word,
                    )
                }))
            }
            (Mode::Operator, Operator::Delete, Motion::B(word)) => {
                let cases = clone_cases_as(&self.cases, |c| {
                    OmapDBCase::new(
                        c.buffer.clone(),
                        c.count,
                        *word,
                        c.prevent_change,
                    )
                    .unwrap()
                });
                if !skip_verify {
                    verify_cases(&self.group_name, &cases)?;
                }
                Ok(self.write_all_tests(&cases, |case_name, case_id, case| {
                    self.write_omap_d_b_assertion(
                        case_name, case_id, case, *word,
                    )
                }))
            }
            (Mode::Operator, Operator::Yank, Motion::B(word)) => {
                let cases = clone_cases_as(&self.cases, |c| {
                    OmapYBCase::new(
                        c.buffer.clone(),
                        c.count,
                        *word,
                        c.prevent_change,
                    )
                    .unwrap()
                });
                if !skip_verify {
                    verify_cases(&self.group_name, &cases)?;
                }
                Ok(self.write_all_tests(&cases, |case_name, case_id, case| {
                    self.write_omap_y_b_assertion(
                        case_name, case_id, case, *word,
                    )
                }))
            }
            (Mode::Visual(kind), Operator::NoOp, Motion::B(word)) => {
                def_common_match_arm!(xmap; XmapBCase, write_xmap_b_assertion, kind, word)
            }
            (Mode::Normal, Operator::NoOp, Motion::Ge(word)) => {
                def_common_match_arm!(NmapGeCase, write_nmap_ge_assertion, word)
            }
            (Mode::Visual(kind), Operator::NoOp, Motion::Ge(word)) => {
                def_common_match_arm!(xmap; XmapGeCase, write_xmap_ge_assertion, kind, word)
            }
            (Mode::Operator, Operator::Delete, Motion::Ge(word)) => {
                let cases = clone_cases_as(&self.cases, |c| {
                    OmapDGeCase::new(
                        c.buffer.clone(),
                        c.count,
                        *word,
                        c.d_special,
                        c.prevent_change,
                    )
                    .unwrap()
                });
                if !skip_verify {
                    verify_cases(&self.group_name, &cases)?;
                }
                Ok(self.write_all_tests(&cases, |case_name, case_id, case| {
                    self.write_omap_d_ge_assertion(
                        case_name, case_id, case, *word,
                    )
                }))
            }
            (Mode::Operator, Operator::Change, Motion::Ge(word)) => {
                let cases = clone_cases_as(&self.cases, |c| {
                    OmapCGeCase::new(
                        c.buffer.clone(),
                        c.count,
                        *word,
                        c.prevent_change,
                    )
                    .unwrap()
                });
                if !skip_verify {
                    verify_cases(&self.group_name, &cases)?;
                }
                Ok(self.write_all_tests(&cases, |case_name, case_id, case| {
                    self.write_omap_c_ge_assertion(
                        case_name, case_id, case, *word,
                    )
                }))
            }
            (Mode::Operator, Operator::Yank, Motion::Ge(word)) => {
                let cases = clone_cases_as(&self.cases, |c| {
                    OmapYGeCase::new(
                        c.buffer.clone(),
                        c.count,
                        *word,
                        c.prevent_change,
                    )
                    .unwrap()
                });
                if !skip_verify {
                    verify_cases(&self.group_name, &cases)?;
                }
                Ok(self.write_all_tests(&cases, |case_name, case_id, case| {
                    self.write_omap_y_ge_assertion(
                        case_name, case_id, case, *word,
                    )
                }))
            }
            _ => Err("Unsupported mode/operator/motion combination".into()),
        }
    }

    fn write_all_tests<T, F>(
        &self,
        cases: &HashMap<String, Vec<T>>,
        mut write_assertion_func: F,
    ) -> TokenStream
    where
        F: FnMut(&str, usize, &T) -> TokenStream,
    {
        let mut test_func_codes = Vec::new();
        for (case_name, sub_cases) in cases.iter() {
            for (i, case) in sub_cases.iter().enumerate() {
                let case_id = i + 1;
                test_func_codes
                    .push(write_assertion_func(case_name, case_id, case));
            }
        }
        let group_name: Ident = syn::parse_str(&self.group_name).unwrap();
        quote! {
            mod #group_name {
                #(#test_func_codes)*
            }
        }
    }
}

macro_rules! def_assertion {
    ( $fun_name:ident, $typ:ty, $fun_name_to_test:ident ) => {
        impl VerifiedCases {
            fn $fun_name(
                &self,
                case_name: &str,
                case_id: usize,
                case: $typ,
                word: bool,
            ) -> TokenStream {
                use jieba_vim_rs_test::verified_case::cases::MotionOutput as TestMotionOutput;

                let test_name: Ident =
                    syn::parse_str(&format!("{}_{}", case_name, case_id)).unwrap();
                let backend_path = &self.backend_path;
                let buffer_type = &self.buffer_type;

                let lnum_before = case.lnum_before;
                let col_before = case.col_before;
                let buffer = &case.buffer;
                let count = case.count.explicit();
                let case_desc = case.to_string();

                // We can't pass `true_output` directly to quote! because
                // `TestMotionOutput` does not implement `ToToken` trait.
                let true_output: TestMotionOutput = case.clone().into();
                let (true_lnum_after, true_col_after) = true_output.new_cursor_pos;
                let true_d_special = true_output.d_special;
                let true_prevent_change = true_output.prevent_change;

                quote! {
                    #[test]
                    fn #test_name() {
                        use jieba_vim_rs_test::verified_case::cases::MotionOutput as TestMotionOutput;

                        let buffer: #buffer_type = vec![#(#buffer.to_string()),*].into();
                        let pred_output = #backend_path.$fun_name_to_test(&buffer, (#lnum_before, #col_before), #count, #word).unwrap();
                        let true_output = TestMotionOutput {
                            new_cursor_pos: (#true_lnum_after, #true_col_after),
                            d_special: #true_d_special,
                            prevent_change: #true_prevent_change,
                        };
                        assert_eq!(pred_output, true_output, "\n{}", #case_desc);
                    }
                }
            }
        }
    };
}

def_assertion!(write_nmap_w_assertion, &NmapWCase, nmap_w);
def_assertion!(write_nmap_e_assertion, &NmapECase, nmap_e);
def_assertion!(write_omap_c_w_assertion, &OmapCWCase, omap_c_w);
def_assertion!(write_omap_d_w_assertion, &OmapDWCase, omap_w);
def_assertion!(write_omap_y_w_assertion, &OmapYWCase, omap_w);
def_assertion!(write_omap_c_e_assertion, &OmapCECase, omap_e);
def_assertion!(write_omap_y_e_assertion, &OmapYECase, omap_e);
def_assertion!(write_xmap_w_assertion, &XmapWCase, xmap_w);
def_assertion!(write_xmap_e_assertion, &XmapECase, xmap_e);
def_assertion!(write_nmap_b_assertion, &NmapBCase, nmap_b);
def_assertion!(write_omap_c_b_assertion, &OmapCBCase, omap_b);
def_assertion!(write_omap_d_b_assertion, &OmapDBCase, omap_b);
def_assertion!(write_omap_y_b_assertion, &OmapYBCase, omap_b);
def_assertion!(write_xmap_b_assertion, &XmapBCase, xmap_b);
def_assertion!(write_nmap_ge_assertion, &NmapGeCase, nmap_ge);
def_assertion!(write_xmap_ge_assertion, &XmapGeCase, xmap_ge);
def_assertion!(write_omap_d_e_assertion, &OmapDECase, omap_d_e);
def_assertion!(write_omap_d_ge_assertion, &OmapDGeCase, omap_d_ge);
def_assertion!(write_omap_c_ge_assertion, &OmapCGeCase, omap_ge);
def_assertion!(write_omap_y_ge_assertion, &OmapYGeCase, omap_ge);
